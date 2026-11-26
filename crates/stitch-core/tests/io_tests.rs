mod helpers;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use image::ImageEncoder;

use helpers::solid_image;
use stitch_core::{
    compose_to_png, encode_jpeg, encode_png, load_image_from_bytes, load_image_from_path,
    save_jpeg, save_png, Alignment, ComposeError, ComposeOptions, ComposedImage, HexColorError,
    InputImage, Insets, LayoutKind, PaneStyle, ResizeMode, RgbaColor, Spacing,
};

fn temp_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("stitch-me-up-{nanos}-{name}"))
}

#[test]
fn saves_png_and_loads_it_back() {
    let path = temp_path("roundtrip.png");
    let image = ComposedImage {
        width: 2,
        height: 1,
        pixels: vec![255, 0, 0, 255, 0, 255, 0, 255],
    };

    save_png(&path, &image).expect("save png");
    let loaded = load_image_from_path(&path).expect("load png");

    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 1);
    assert_eq!(loaded.pixels, image.pixels);

    let _ = fs::remove_file(path);
}

#[test]
fn loads_jpeg_input() {
    let path = temp_path("source.jpg");
    let buffer = image::RgbImage::from_raw(2, 1, vec![200, 100, 50, 25, 75, 125]).unwrap();
    buffer
        .save_with_format(&path, image::ImageFormat::Jpeg)
        .expect("write jpeg");

    let loaded = load_image_from_path(&path).expect("load jpeg");

    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 1);
    assert_eq!(loaded.pixels.len(), 8);

    let _ = fs::remove_file(path);
}

fn png_bytes(image: &InputImage) -> Vec<u8> {
    let composed = ComposedImage {
        width: image.width,
        height: image.height,
        pixels: image.pixels.clone(),
    };

    encode_png(&composed).expect("encode png")
}

fn vertical_options() -> ComposeOptions {
    ComposeOptions {
        layout: LayoutKind::VerticalStack,
        spacing: Spacing::all(1),
        padding: Insets::default(),
        background: RgbaColor::WHITE,
        alignment: Alignment::Start,
        resize_mode: ResizeMode::None,
        pane_style: PaneStyle::default(),
        divider: None,
        column_weights: Vec::new(),
    }
}

#[test]
fn encodes_png_in_memory_and_loads_it_back() {
    let image = ComposedImage {
        width: 2,
        height: 1,
        pixels: vec![255, 0, 0, 255, 0, 255, 0, 255],
    };

    let encoded = encode_png(&image).expect("encode png");
    let loaded = load_image_from_bytes(&encoded).expect("load encoded png");

    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 1);
    assert_eq!(loaded.pixels, image.pixels);
}

#[test]
fn compose_to_png_round_trips() {
    let red = solid_image(2, 1, [255, 0, 0, 255]);
    let green = solid_image(2, 1, [0, 255, 0, 255]);

    let output = compose_to_png(&[png_bytes(&red), png_bytes(&green)], &vertical_options())
        .expect("compose to png");

    let composed = load_image_from_bytes(&output).expect("load composed png");

    assert_eq!(composed.width, 2);
    assert_eq!(composed.height, 3);
    assert_eq!(&composed.pixels[0..4], &[255, 0, 0, 255]);
    assert_eq!(&composed.pixels[4..8], &[255, 0, 0, 255]);
    assert_eq!(&composed.pixels[8..12], &[255, 255, 255, 255]);
    assert_eq!(&composed.pixels[16..20], &[0, 255, 0, 255]);
}

#[test]
fn rejects_invalid_image_bytes() {
    let error = load_image_from_bytes(b"not an image").unwrap_err();
    assert!(matches!(error, ComposeError::ImageIo(_)));

    let good = png_bytes(&solid_image(2, 1, [255, 0, 0, 255]));
    let error = compose_to_png(&[good, b"not an image".to_vec()], &vertical_options()).unwrap_err();
    assert!(matches!(error, ComposeError::ImageIo(_)));
}

#[test]
fn parse_hex_color_variants() {
    let expected_rgb = RgbaColor {
        r: 0xaa,
        g: 0xbb,
        b: 0xcc,
        a: 0xff,
    };

    assert_eq!(RgbaColor::parse_hex("#aabbcc").unwrap(), expected_rgb);
    assert_eq!(RgbaColor::parse_hex("aabbcc").unwrap(), expected_rgb);

    let expected_rgba = RgbaColor {
        r: 0x01,
        g: 0x02,
        b: 0x03,
        a: 0x04,
    };

    assert_eq!(RgbaColor::parse_hex("#01020304").unwrap(), expected_rgba);
    assert_eq!(RgbaColor::parse_hex("01020304").unwrap(), expected_rgba);

    assert_eq!(
        RgbaColor::parse_hex("#12345").unwrap_err(),
        HexColorError::InvalidLength
    );
    assert_eq!(
        HexColorError::InvalidLength.to_string(),
        "expected color in #RRGGBB or #RRGGBBAA format"
    );
    assert_eq!(
        RgbaColor::parse_hex("#gg0000").unwrap_err(),
        HexColorError::InvalidComponent
    );
    assert_eq!(
        HexColorError::InvalidComponent.to_string(),
        "invalid hex color component"
    );
}

#[test]
fn loaders_leave_style_unset() {
    let source = solid_image(2, 1, [255, 0, 0, 255]);
    let bytes = png_bytes(&source);

    let from_bytes = load_image_from_bytes(&bytes).expect("load bytes");
    assert_eq!(from_bytes.style, None);

    let path = temp_path("style.png");
    fs::write(&path, &bytes).expect("write png");
    let from_path = load_image_from_path(&path).expect("load path");
    assert_eq!(from_path.style, None);

    let _ = fs::remove_file(path);
}

#[test]
fn encode_png_rejects_undersized_buffer() {
    let image = ComposedImage {
        width: 1,
        height: 1,
        pixels: vec![],
    };

    let error = encode_png(&image).unwrap_err();
    assert!(matches!(
        error,
        ComposeError::InvalidOutputBuffer {
            width: 1,
            height: 1,
            expected: 4,
            actual: 0,
        }
    ));
    assert_eq!(
        error.to_string(),
        "output image 1x1 has invalid buffer length: expected 4, got 0"
    );
}

#[test]
fn encode_png_rejects_wrong_length_buffer() {
    let image = ComposedImage {
        width: 2,
        height: 1,
        pixels: vec![255, 0, 0, 255],
    };

    let error = encode_png(&image).unwrap_err();
    assert!(matches!(
        error,
        ComposeError::InvalidOutputBuffer {
            width: 2,
            height: 1,
            expected: 8,
            actual: 4,
        }
    ));
}

#[test]
fn save_png_rejects_undersized_buffer_and_writes_nothing() {
    let path = temp_path("undersized.png");
    let image = ComposedImage {
        width: 1,
        height: 1,
        pixels: vec![],
    };

    let error = save_png(&path, &image).unwrap_err();
    assert!(matches!(
        error,
        ComposeError::InvalidOutputBuffer {
            width: 1,
            height: 1,
            expected: 4,
            actual: 0,
        }
    ));
    assert!(!path.exists(), "no file should be written");
}

#[test]
fn save_png_rejects_wrong_length_buffer_and_writes_nothing() {
    let path = temp_path("wrong-length.png");
    let image = ComposedImage {
        width: 2,
        height: 1,
        pixels: vec![255, 0, 0, 255],
    };

    let error = save_png(&path, &image).unwrap_err();
    assert!(matches!(
        error,
        ComposeError::InvalidOutputBuffer {
            width: 2,
            height: 1,
            expected: 8,
            actual: 4,
        }
    ));
    assert!(!path.exists(), "no file should be written");
}

#[test]
fn encode_png_still_encodes_valid_1x1() {
    let image = ComposedImage {
        width: 1,
        height: 1,
        pixels: vec![255, 0, 0, 255],
    };

    let encoded = encode_png(&image).expect("encode valid 1x1");
    assert!(encoded.starts_with(b"\x89PNG"));
}

fn minimal_tiff_orientation_6() -> Vec<u8> {
    let mut tiff = Vec::new();
    tiff.extend_from_slice(b"II*\0");
    tiff.extend_from_slice(&8u32.to_le_bytes()); // IFD offset
    tiff.extend_from_slice(&1u16.to_le_bytes()); // entry count
    tiff.extend_from_slice(&0x0112u16.to_le_bytes()); // tag: Orientation
    tiff.extend_from_slice(&3u16.to_le_bytes()); // type: SHORT
    tiff.extend_from_slice(&1u32.to_le_bytes()); // count
    tiff.extend_from_slice(&6u32.to_le_bytes()); // value
    tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD
    tiff
}

fn colored_jpeg(width: u32, height: u32, exif: Option<Vec<u8>>) -> Vec<u8> {
    let mut pixels = Vec::new();
    for y in 0..height {
        for x in 0..width {
            let (r, g, b) = if y == 0 && x == 0 {
                (255, 0, 0)
            } else if y == 0 {
                (0, 255, 0)
            } else if x == 0 {
                (0, 0, 255)
            } else {
                (255, 255, 0)
            };
            pixels.extend_from_slice(&[r, g, b]);
        }
    }
    let mut out = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, 100);
    if let Some(metadata) = exif {
        encoder.set_exif_metadata(metadata).expect("set exif");
    }
    encoder
        .write_image(&pixels, width, height, image::ExtendedColorType::Rgb8)
        .expect("encode jpeg");
    out
}

#[test]
fn exif_orientation_rotates_jpeg_from_bytes() {
    let bytes = colored_jpeg(2, 3, Some(minimal_tiff_orientation_6()));
    let loaded = load_image_from_bytes(&bytes).expect("load bytes");
    let source = image::RgbImage::from_raw(
        2,
        3,
        vec![
            255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 0, 0, 0, 255, 255, 255, 0,
        ],
    )
    .expect("source");
    let expected = image::DynamicImage::ImageRgb8(source).rotate90().to_rgba8();

    assert_eq!(loaded.width, 3);
    assert_eq!(loaded.height, 2);

    // JPEG is lossy: compare corner pixels with a tolerance.
    let corners = [(0usize, 0usize), (2, 0), (0, 1), (2, 1)];
    for (x, y) in corners {
        let i = (y * 3 + x) * 4;
        let actual = &loaded.pixels[i..i + 4];
        let want = &expected.as_raw()[i..i + 4];
        for (a, w) in actual.iter().zip(want.iter()) {
            assert!(
                a.abs_diff(*w) <= 24,
                "corner ({x},{y}) channel mismatch: actual {actual:?}, expected {want:?}"
            );
        }
    }
}

#[test]
fn exif_orientation_rotates_jpeg_from_path() {
    let bytes = colored_jpeg(2, 3, Some(minimal_tiff_orientation_6()));
    let path = temp_path("oriented.jpg");
    fs::write(&path, &bytes).expect("write jpeg");

    let loaded = load_image_from_path(&path).expect("load path");

    assert_eq!(loaded.width, 3);
    assert_eq!(loaded.height, 2);

    let _ = fs::remove_file(path);
}

#[test]
fn untagged_jpeg_keeps_dimensions() {
    let bytes = colored_jpeg(2, 3, None);
    let loaded = load_image_from_bytes(&bytes).expect("load bytes");
    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 3);

    let path = temp_path("untagged.jpg");
    fs::write(&path, &bytes).expect("write jpeg");
    let loaded = load_image_from_path(&path).expect("load path");
    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 3);

    let _ = fs::remove_file(path);
}

#[test]
fn encode_jpeg_rejects_quality_zero_and_101() {
    let image = ComposedImage {
        width: 1,
        height: 1,
        pixels: vec![255, 0, 0, 255],
    };

    let error = encode_jpeg(&image, 0).unwrap_err();
    assert!(matches!(
        error,
        ComposeError::InvalidJpegQuality { quality: 0 }
    ));
    assert_eq!(error.to_string(), "jpeg quality 0 is outside 1..=100");

    let error = encode_jpeg(&image, 101).unwrap_err();
    assert!(matches!(
        error,
        ComposeError::InvalidJpegQuality { quality: 101 }
    ));
    assert_eq!(error.to_string(), "jpeg quality 101 is outside 1..=100");
}

#[test]
fn encode_jpeg_roundtrips_dimensions() {
    let image = ComposedImage {
        width: 3,
        height: 2,
        pixels: vec![200, 100, 50, 255]
            .into_iter()
            .cycle()
            .take(3 * 2 * 4)
            .collect(),
    };

    let bytes = encode_jpeg(&image, 90).expect("encode jpeg");
    assert_eq!(&bytes[..2], &[0xFF, 0xD8]);

    let loaded = load_image_from_bytes(&bytes).expect("load jpeg");
    assert_eq!(loaded.width, 3);
    assert_eq!(loaded.height, 2);
}

#[test]
fn encode_jpeg_flattens_transparency_onto_white() {
    let image = ComposedImage {
        width: 1,
        height: 1,
        pixels: vec![0, 0, 0, 0],
    };

    let bytes = encode_jpeg(&image, 90).expect("encode jpeg");
    let loaded = load_image_from_bytes(&bytes).expect("load jpeg");
    for ch in loaded.pixels.as_chunks::<4>().0 {
        assert!(ch[0] >= 250, "r={}", ch[0]);
        assert!(ch[1] >= 250, "g={}", ch[1]);
        assert!(ch[2] >= 250, "b={}", ch[2]);
    }
}

#[test]
fn encode_jpeg_rejects_invalid_buffer() {
    let image = ComposedImage {
        width: 1,
        height: 1,
        pixels: vec![],
    };

    let error = encode_jpeg(&image, 90).unwrap_err();
    assert!(matches!(
        error,
        ComposeError::InvalidOutputBuffer {
            width: 1,
            height: 1,
            expected: 4,
            actual: 0,
        }
    ));
}

#[test]
fn save_jpeg_writes_decodable_file() {
    let path = temp_path("roundtrip.jpg");
    let image = ComposedImage {
        width: 2,
        height: 2,
        pixels: vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ],
    };

    save_jpeg(&path, &image, 80).expect("save jpeg");
    let bytes = fs::read(&path).expect("read jpeg");
    assert_eq!(&bytes[..2], &[0xFF, 0xD8]);

    let loaded = load_image_from_path(&path).expect("load jpeg");
    assert_eq!(loaded.width, 2);
    assert_eq!(loaded.height, 2);

    let _ = fs::remove_file(path);
}
