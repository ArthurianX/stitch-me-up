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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeOptions {
    pub layout: LayoutKind,
    pub spacing: u32,
    pub padding: Insets,
    pub background: RgbaColor,
    pub alignment: Alignment,
    pub resize_mode: ResizeMode,
}

impl Default for ComposeOptions {
    fn default() -> Self {
        Self {
            layout: LayoutKind::VerticalStack,
            spacing: 0,
            padding: Insets::default(),
            background: RgbaColor::WHITE,
            alignment: Alignment::Start,
            resize_mode: ResizeMode::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl InputImage {
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
