use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutKind {
    VerticalStack,
    HorizontalStack,
    Grid {
        columns: u32,
    },
    /// Exactly two images, each scaled to the shared tallest height and placed side by side.
    /// `ComposeOptions::resize_mode` is ignored for this layout.
    BeforeAfter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceCrop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeMode {
    None,
    FitToSameWidth,
    FitToSameHeight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Insets {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Insets {
    pub const fn all(value: u32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

impl Default for Insets {
    fn default() -> Self {
        Self::all(0)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HexColorError {
    #[error("expected color in #RRGGBB or #RRGGBBAA format")]
    InvalidLength,
    #[error("invalid hex color component")]
    InvalidComponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub const fn to_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Parses `#RRGGBB`, `RRGGBB`, `#RRGGBBAA`, or `RRGGBBAA`.
    pub fn parse_hex(input: &str) -> Result<Self, HexColorError> {
        let value = input.strip_prefix('#').unwrap_or(input);

        let (rgb, alpha) = match value.len() {
            6 => (&value[0..6], None),
            8 => (&value[0..6], Some(&value[6..8])),
            _ => return Err(HexColorError::InvalidLength),
        };

        Ok(RgbaColor {
            r: parse_byte(&rgb[0..2])?,
            g: parse_byte(&rgb[2..4])?,
            b: parse_byte(&rgb[4..6])?,
            a: alpha.map(parse_byte).transpose()?.unwrap_or(255),
        })
    }
}

fn parse_byte(input: &str) -> Result<u8, HexColorError> {
    u8::from_str_radix(input, 16).map_err(|_| HexColorError::InvalidComponent)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub width: u32,
    pub color: RgbaColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaneStyle {
    pub frame: Option<Frame>,
    pub corner_radius: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Divider {
    pub width: u32,
    pub color: RgbaColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Spacing {
    pub x: u32,
    pub y: u32,
}

impl Spacing {
    pub const fn all(value: u32) -> Self {
        Self { x: value, y: value }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeOptions {
    pub layout: LayoutKind,
    pub spacing: Spacing,
    pub padding: Insets,
    pub background: RgbaColor,
    pub alignment: Alignment,
    pub resize_mode: ResizeMode,
    pub pane_style: PaneStyle,
    pub divider: Option<Divider>,
    pub column_weights: Vec<u32>,
    pub cell_size: Option<CellSize>,
}

impl Default for ComposeOptions {
    fn default() -> Self {
        Self {
            layout: LayoutKind::VerticalStack,
            spacing: Spacing::default(),
            padding: Insets::default(),
            background: RgbaColor::WHITE,
            alignment: Alignment::Start,
            resize_mode: ResizeMode::None,
            pane_style: PaneStyle::default(),
            divider: None,
            column_weights: Vec::new(),
            cell_size: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    /// Overrides `ComposeOptions::pane_style` for this pane; `None` inherits the global style.
    pub style: Option<PaneStyle>,
}

impl InputImage {
    /// An undecorated image (`style: None`).
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            width,
            height,
            pixels,
            style: None,
        }
    }

    pub fn expected_len(&self) -> Option<usize> {
        let pixels = u64::from(self.width).checked_mul(u64::from(self.height))?;
        let bytes = pixels.checked_mul(4)?;
        usize::try_from(bytes).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedImage {
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub crop: Option<SourceCrop>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasPlan {
    pub width: u32,
    pub height: u32,
    pub placed_images: Vec<PlacedImage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
