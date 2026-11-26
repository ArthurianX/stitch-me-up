//! WebAssembly adapter exposing the 🩹 Stitch Me Up composition engine to JavaScript.
//!
//! The JS-facing setters are compiled only for wasm targets: they throw `JsError`, which
//! binds the JavaScript `Error` constructor and panics when constructed on native targets.
//! The shared option logic lives in the private `apply_*` methods, which return plain
//! `String` errors and are exercised by the native unit tests.

use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use js_sys::Array;
use stitch_core::{ComposeOptions, PaneStyle};
// `Insets` is only named by the wasm-only `set_padding`; the tests import it themselves.
#[cfg(target_arch = "wasm32")]
use stitch_core::compose;
#[cfg(target_arch = "wasm32")]
use stitch_core::load_image_from_bytes;
#[cfg(target_arch = "wasm32")]
use stitch_core::Insets;
#[cfg(any(target_arch = "wasm32", test))]
use stitch_core::{encode_jpeg, encode_png, ComposedImage};
#[cfg(any(target_arch = "wasm32", test))]
use stitch_core::{
    Alignment, Divider, Frame, InputImage, LayoutKind, ResizeMode, RgbaColor, Spacing,
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
// The `#[wasm_bindgen]` attribute on the exported struct is needed on every target, so
// the macro itself is imported unconditionally.
#[cfg(not(target_arch = "wasm32"))]
use wasm_bindgen::prelude::wasm_bindgen;

/// A configurable composer. Each setter updates one option; `compose` runs the engine.
///
/// Setters that take strings throw a JS `Error` whose message is the parse or core error,
/// so typos surface immediately rather than at compose time.
#[wasm_bindgen]
#[derive(Default)]
pub struct Composer {
    // Read only by wasm's `compose` and by the native tests.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    options: ComposeOptions,
    // Same for the per-pane overrides.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pane_styles: BTreeMap<usize, PaneStyle>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Composer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Composer::default()
    }

    /// Horizontal and vertical gaps in pixels; each layout uses its own axis
    /// (vertical stack → y, horizontal stack and before-after → x, grid → both).
    pub fn set_spacing(&mut self, x: u32, y: u32) {
        self.apply_spacing(x, y);
    }

    pub fn set_padding(&mut self, top: u32, right: u32, bottom: u32, left: u32) {
        self.options.padding = Insets {
            top,
            right,
            bottom,
            left,
        };
    }

    pub fn set_corner_radius(&mut self, radius: u32) {
        self.options.pane_style.corner_radius = radius;
    }

    /// `layout` is `vertical`, `horizontal`, `grid`, or `before-after` (case-insensitive).
    /// `columns` is only used by `grid`; `0` fails at compose time with `InvalidGridColumns`.
    pub fn set_layout(&mut self, layout: &str, columns: u32) -> Result<(), JsError> {
        self.apply_layout(layout, columns).map_err(js_error)
    }

    pub fn set_background(&mut self, hex: &str) -> Result<(), JsError> {
        self.apply_background(hex).map_err(js_error)
    }

    /// `alignment` is `start`, `center`, or `end` (case-insensitive).
    pub fn set_alignment(&mut self, alignment: &str) -> Result<(), JsError> {
        self.apply_alignment(alignment).map_err(js_error)
    }

    /// `mode` is `none`, `fit-to-same-width`, or `fit-to-same-height` (case-insensitive).
    pub fn set_resize_mode(&mut self, mode: &str) -> Result<(), JsError> {
        self.apply_resize_mode(mode).map_err(js_error)
    }

    /// `width == 0` disables the frame but still validates the color.
    pub fn set_frame(&mut self, width: u32, hex: &str) -> Result<(), JsError> {
        self.apply_frame(width, hex).map_err(js_error)
    }

    /// `width == 0` disables the divider but still validates the color.
    pub fn set_divider(&mut self, width: u32, hex: &str) -> Result<(), JsError> {
        self.apply_divider(width, hex).map_err(js_error)
    }

    /// Relative column widths for the grid layout, as a `Uint32Array`.
    /// The count must match the column count; pass an empty array for auto sizing.
    pub fn set_column_weights(&mut self, weights: &[u32]) {
        self.apply_column_weights(weights);
    }

    /// 0-based input index; stores a complete style override for that pane.
    /// `frame_width == 0` disables the frame while still validating the color.
    pub fn set_pane_style(
        &mut self,
        index: usize,
        frame_width: u32,
        frame_color: &str,
        corner_radius: u32,
    ) -> Result<(), JsError> {
        self.apply_pane_style(index, frame_width, frame_color, corner_radius)
            .map_err(js_error)
    }

    /// Removes the override at `index`; the pane inherits the global style again.
    pub fn clear_pane_style(&mut self, index: usize) {
        self.apply_clear_pane_style(index);
    }

    fn compose_images(&self, images: Array) -> Result<ComposedImage, JsError> {
        let buffers = images
            .iter()
            .map(|value| {
                value
                    .dyn_into::<js_sys::Uint8Array>()
                    .map(|bytes| bytes.to_vec())
                    .map_err(|_| JsError::new("each image must be a Uint8Array"))
            })
            .collect::<Result<Vec<Vec<u8>>, JsError>>()?;

        let mut decoded = buffers
            .iter()
            .map(|bytes| load_image_from_bytes(bytes))
            .collect::<Result<Vec<InputImage>, _>>()
            .map_err(js_error)?;

        self.apply_pane_styles(&mut decoded).map_err(js_error)?;
        compose(&decoded, &self.options).map_err(js_error)
    }

    /// Each element must be a `Uint8Array` containing PNG or JPEG bytes.
    /// Returns a `Uint8Array` of PNG bytes.
    pub fn compose(&self, images: Array) -> Result<Vec<u8>, JsError> {
        let composed = self.compose_images(images)?;
        encode_png(&composed).map_err(js_error)
    }

    /// Each element must be a `Uint8Array` containing PNG or JPEG bytes.
    /// `format` is `png` or `jpeg` (case-insensitive, trimmed); quality is
    /// only used for JPEG.
    pub fn compose_with_format(
        &self,
        images: Array,
        format: &str,
        quality: u8,
    ) -> Result<Vec<u8>, JsError> {
        let composed = self.compose_images(images)?;
        encode_output(&composed, format, quality).map_err(js_error)
    }
}

/// Shared option logic, compiled for wasm and for native test builds; plain native
/// builds would warn about unused methods here.
#[cfg(any(target_arch = "wasm32", test))]
impl Composer {
    /// Horizontal and vertical gaps in pixels; each layout uses its own axis
    /// (vertical stack → y, horizontal stack and before-after → x, grid → both).
    fn apply_spacing(&mut self, x: u32, y: u32) {
        self.options.spacing = Spacing { x, y };
    }

    fn apply_layout(&mut self, layout: &str, columns: u32) -> Result<(), String> {
        self.options.layout = parse_layout(layout, columns)?;
        Ok(())
    }

    fn apply_background(&mut self, hex: &str) -> Result<(), String> {
        self.options.background = parse_color_hex(hex)?;
        Ok(())
    }

    fn apply_alignment(&mut self, alignment: &str) -> Result<(), String> {
        self.options.alignment = parse_alignment(alignment)?;
        Ok(())
    }

    fn apply_resize_mode(&mut self, mode: &str) -> Result<(), String> {
        self.options.resize_mode = parse_resize_mode(mode)?;
        Ok(())
    }

    /// `width == 0` disables the frame but still validates the color.
    fn apply_frame(&mut self, width: u32, hex: &str) -> Result<(), String> {
        let color = parse_color_hex(hex)?;
        self.options.pane_style.frame = (width > 0).then_some(Frame { width, color });
        Ok(())
    }

    /// `width == 0` disables the divider but still validates the color.
    fn apply_divider(&mut self, width: u32, hex: &str) -> Result<(), String> {
        let color = parse_color_hex(hex)?;
        self.options.divider = (width > 0).then_some(Divider { width, color });
        Ok(())
    }

    /// Relative column widths for the grid layout; empty keeps auto sizing.
    fn apply_column_weights(&mut self, weights: &[u32]) {
        self.options.column_weights = weights.to_vec();
    }

    /// 0-based input index; complete override, replacing the global style for the pane.
    /// `frame_width == 0` disables the frame but still validates the color.
    fn apply_pane_style(
        &mut self,
        index: usize,
        frame_width: u32,
        frame_color: &str,
        corner_radius: u32,
    ) -> Result<(), String> {
        let color = parse_color_hex(frame_color)?;
        self.pane_styles.insert(
            index,
            PaneStyle {
                frame: (frame_width > 0).then_some(Frame {
                    width: frame_width,
                    color,
                }),
                corner_radius,
            },
        );
        Ok(())
    }

    /// Removes the override at `index`; the pane inherits the global style again.
    fn apply_clear_pane_style(&mut self, index: usize) {
        self.pane_styles.remove(&index);
    }

    /// Applies stored overrides to the decoded images by index; a stored index with no
    /// matching image is an error.
    fn apply_pane_styles(&self, images: &mut [InputImage]) -> Result<(), String> {
        for (index, style) in &self.pane_styles {
            let image = images
                .get_mut(*index)
                .ok_or_else(|| format!("pane style index {index} has no matching image"))?;
            image.style = Some(*style);
        }
        Ok(())
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn encode_output(composed: &ComposedImage, format: &str, quality: u8) -> Result<Vec<u8>, String> {
    let normalized = normalize(format);
    match normalized.as_str() {
        "png" => encode_png(composed).map_err(|e| e.to_string()),
        "jpeg" => encode_jpeg(composed, quality).map_err(|e| e.to_string()),
        _ => Err(format!(
            "unknown output format \"{normalized}\"; expected png or jpeg"
        )),
    }
}

/// Converts any core or parse error into a JS `Error` carrying its message.
#[cfg(target_arch = "wasm32")]
fn js_error<E: std::fmt::Display>(error: E) -> JsError {
    JsError::new(&error.to_string())
}

#[cfg(any(target_arch = "wasm32", test))]
fn parse_color_hex(hex: &str) -> Result<RgbaColor, String> {
    RgbaColor::parse_hex(hex).map_err(|error| error.to_string())
}

#[cfg(any(target_arch = "wasm32", test))]
fn parse_layout(layout: &str, columns: u32) -> Result<LayoutKind, String> {
    match normalize(layout).as_str() {
        "vertical" => Ok(LayoutKind::VerticalStack),
        "horizontal" => Ok(LayoutKind::HorizontalStack),
        "grid" => Ok(LayoutKind::Grid { columns }),
        "before-after" => Ok(LayoutKind::BeforeAfter),
        other => Err(format!(
            "unknown layout \"{other}\"; expected vertical, horizontal, grid, or before-after"
        )),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn parse_alignment(alignment: &str) -> Result<Alignment, String> {
    match normalize(alignment).as_str() {
        "start" => Ok(Alignment::Start),
        "center" => Ok(Alignment::Center),
        "end" => Ok(Alignment::End),
        other => Err(format!(
            "unknown alignment \"{other}\"; expected start, center, or end"
        )),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn parse_resize_mode(mode: &str) -> Result<ResizeMode, String> {
    match normalize(mode).as_str() {
        "none" => Ok(ResizeMode::None),
        "fit-to-same-width" => Ok(ResizeMode::FitToSameWidth),
        "fit-to-same-height" => Ok(ResizeMode::FitToSameHeight),
        other => Err(format!(
            "unknown resize mode \"{other}\"; expected none, fit-to-same-width, or fit-to-same-height"
        )),
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn normalize(value: &str) -> String {
    value.trim().to_lowercase()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use stitch_core::{
        Alignment, ComposeOptions, Divider, Frame, InputImage, Insets, LayoutKind, PaneStyle,
        ResizeMode, RgbaColor, Spacing,
    };

    use super::{parse_alignment, parse_layout, parse_resize_mode, Composer};

    #[test]
    fn composer_defaults_match_core_defaults() {
        let composer = Composer::default();
        assert_eq!(composer.options, ComposeOptions::default());
        assert!(composer.pane_styles.is_empty());
    }

    #[test]
    fn parses_every_layout() {
        assert_eq!(
            parse_layout("vertical", 0).unwrap(),
            LayoutKind::VerticalStack
        );
        assert_eq!(
            parse_layout("Horizontal", 0).unwrap(),
            LayoutKind::HorizontalStack
        );
        assert_eq!(
            parse_layout(" GRID ", 3).unwrap(),
            LayoutKind::Grid { columns: 3 }
        );
        assert_eq!(
            parse_layout("before-after", 0).unwrap(),
            LayoutKind::BeforeAfter
        );

        let error = parse_layout("panorama", 0).unwrap_err();
        assert!(error.contains("unknown layout \"panorama\""));
        assert!(error.contains("before-after"));
    }

    #[test]
    fn parses_every_alignment() {
        assert_eq!(parse_alignment("start").unwrap(), Alignment::Start);
        assert_eq!(parse_alignment("CENTER").unwrap(), Alignment::Center);
        assert_eq!(parse_alignment("end").unwrap(), Alignment::End);

        let error = parse_alignment("middle").unwrap_err();
        assert!(error.contains("unknown alignment \"middle\""));
        assert!(error.contains("start, center, or end"));
    }

    #[test]
    fn parses_every_resize_mode() {
        assert_eq!(parse_resize_mode("none").unwrap(), ResizeMode::None);
        assert_eq!(
            parse_resize_mode("Fit-To-Same-Width").unwrap(),
            ResizeMode::FitToSameWidth
        );
        assert_eq!(
            parse_resize_mode("fit-to-same-height").unwrap(),
            ResizeMode::FitToSameHeight
        );

        let error = parse_resize_mode("stretch").unwrap_err();
        assert!(error.contains("unknown resize mode \"stretch\""));
        assert!(error.contains("fit-to-same-height"));
    }

    #[test]
    fn sets_background_color_and_rejects_invalid_hex() {
        let mut composer = Composer::default();

        composer
            .apply_background("#11223344")
            .expect("valid background");
        assert_eq!(
            composer.options.background,
            RgbaColor {
                r: 0x11,
                g: 0x22,
                b: 0x33,
                a: 0x44,
            }
        );

        let error = composer.apply_background("# nope").unwrap_err();
        assert!(error.contains("#RRGGBB"));
    }

    #[test]
    fn frame_is_disabled_at_zero_width_but_color_is_still_checked() {
        let mut composer = Composer::default();

        composer
            .apply_frame(0, "#ff0000")
            .expect("valid frame color");
        assert_eq!(composer.options.pane_style.frame, None);

        composer.apply_frame(4, "#0000ff").expect("valid frame");
        assert_eq!(
            composer.options.pane_style.frame,
            Some(Frame {
                width: 4,
                color: RgbaColor {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                },
            })
        );

        let error = composer.apply_frame(2, "zzzzzz").unwrap_err();
        assert!(error.contains("invalid hex color component"));
    }

    #[test]
    fn divider_is_disabled_at_zero_width_but_color_is_still_checked() {
        let mut composer = Composer::default();

        composer
            .apply_divider(0, "#ffffff")
            .expect("valid divider color");
        assert_eq!(composer.options.divider, None);

        composer.apply_divider(3, "#ffffff").expect("valid divider");
        assert_eq!(
            composer.options.divider,
            Some(Divider {
                width: 3,
                color: RgbaColor::WHITE,
            })
        );

        assert!(composer.apply_divider(3, "nope").is_err());
    }

    #[test]
    fn stores_column_weights() {
        let mut composer = Composer::default();

        composer.apply_column_weights(&[1, 2]);
        assert_eq!(composer.options.column_weights, vec![1, 2]);

        composer.apply_column_weights(&[]);
        assert!(composer.options.column_weights.is_empty());
    }

    #[test]
    fn updates_radius_padding_spacing_and_layout() {
        let mut composer = Composer::default();

        composer.apply_spacing(12, 3);
        composer.options.padding = Insets {
            top: 1,
            right: 2,
            bottom: 3,
            left: 4,
        };
        composer.options.pane_style.corner_radius = 8;
        composer
            .apply_layout("before-after", 0)
            .expect("before-after layout");
        composer.apply_alignment("end").expect("end alignment");
        composer
            .apply_resize_mode("fit-to-same-width")
            .expect("same width resize mode");

        assert_eq!(composer.options.spacing, Spacing { x: 12, y: 3 });
        assert_eq!(composer.options.pane_style.corner_radius, 8);
        assert_eq!(composer.options.layout, LayoutKind::BeforeAfter);
        assert_eq!(composer.options.alignment, Alignment::End);
        assert_eq!(composer.options.resize_mode, ResizeMode::FitToSameWidth);
        assert_eq!(
            composer.options.padding,
            Insets {
                top: 1,
                right: 2,
                bottom: 3,
                left: 4,
            }
        );
    }

    #[test]
    fn stores_and_clears_pane_styles() {
        let mut composer = Composer::default();
        composer
            .apply_pane_style(1, 4, "#ff0000", 8)
            .expect("set style");

        assert_eq!(composer.pane_styles.len(), 1);
        assert_eq!(
            composer.pane_styles.get(&1),
            Some(&PaneStyle {
                frame: Some(Frame {
                    width: 4,
                    color: RgbaColor {
                        r: 255,
                        g: 0,
                        b: 0,
                        a: 255,
                    },
                }),
                corner_radius: 8,
            })
        );

        composer.apply_clear_pane_style(1);
        assert!(composer.pane_styles.is_empty());
    }

    #[test]
    fn huge_pane_style_index_does_not_allocate_or_panic() {
        let mut composer = Composer::default();
        composer
            .apply_pane_style(usize::MAX, 1, "#000000", 0)
            .expect("set style");

        let mut images = vec![InputImage::new(1, 1, vec![0, 0, 0, 255])];
        let error = composer.apply_pane_styles(&mut images).unwrap_err();

        assert_eq!(
            error,
            format!("pane style index {} has no matching image", usize::MAX)
        );
    }

    #[test]
    fn unmatched_pane_style_index_errors_at_compose() {
        let mut composer = Composer::default();
        composer
            .apply_pane_style(5, 1, "#000000", 0)
            .expect("set style");

        let mut images = vec![
            InputImage::new(1, 1, vec![0, 0, 0, 255]),
            InputImage::new(1, 1, vec![0, 0, 0, 255]),
        ];
        let error = composer.apply_pane_styles(&mut images).unwrap_err();

        assert_eq!(error, "pane style index 5 has no matching image");
    }

    #[test]
    fn applies_pane_styles_by_index() {
        let mut composer = Composer::default();
        composer
            .apply_pane_style(1, 0, "#00ff00", 5)
            .expect("set style");

        let mut images = vec![
            InputImage::new(1, 1, vec![255, 0, 0, 255]),
            InputImage::new(1, 1, vec![0, 255, 0, 255]),
        ];
        composer
            .apply_pane_styles(&mut images)
            .expect("apply styles");

        assert_eq!(images[0].style, None);
        assert_eq!(
            images[1].style,
            Some(PaneStyle {
                frame: None,
                corner_radius: 5,
            })
        );
    }

    #[test]
    fn rejects_pane_style_for_missing_image() {
        let mut composer = Composer::default();
        composer
            .apply_pane_style(2, 1, "#ffffff", 0)
            .expect("set style");

        let mut images = vec![
            InputImage::new(1, 1, vec![0, 0, 0, 255]),
            InputImage::new(1, 1, vec![0, 0, 0, 255]),
        ];
        let error = composer.apply_pane_styles(&mut images).unwrap_err();

        assert_eq!(error, "pane style index 2 has no matching image");
    }

    #[test]
    fn encode_output_returns_png_magic_for_png_even_when_uppercase_and_padded() {
        let composed = test_composed_image();
        for format in ["PNG", "  png  "] {
            let bytes = super::encode_output(&composed, format, 90).unwrap();
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }
    }

    #[test]
    fn encode_output_returns_jpeg_magic_for_jpeg() {
        let composed = test_composed_image();
        let bytes = super::encode_output(&composed, "jpeg", 90).unwrap();
        assert!(bytes.starts_with(b"\xff\xd8"));
    }

    #[test]
    fn encode_output_rejects_unknown_format() {
        let composed = test_composed_image();
        let error = super::encode_output(&composed, "webp", 90).unwrap_err();
        assert_eq!(
            error,
            "unknown output format \"webp\"; expected png or jpeg"
        );
    }

    #[test]
    fn encode_output_rejects_jpeg_quality_zero() {
        let composed = test_composed_image();
        let error = super::encode_output(&composed, "jpeg", 0).unwrap_err();
        assert!(error.contains("jpeg quality 0"));
    }

    fn test_composed_image() -> stitch_core::ComposedImage {
        stitch_core::ComposedImage {
            width: 2,
            height: 2,
            pixels: vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        }
    }

    #[test]
    fn rejects_invalid_pane_style_color() {
        let mut composer = Composer::default();
        let error = composer.apply_pane_style(0, 2, "zzzzzz", 0).unwrap_err();

        assert!(error.contains("invalid hex color component"));
        assert!(composer.pane_styles.is_empty());
    }
}
