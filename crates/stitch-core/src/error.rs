use thiserror::Error;

#[derive(Debug, Error)]
pub enum ComposeError {
    #[error("at least two images are required")]
    NotEnoughImages,
    #[error("image {index} has invalid RGBA buffer length: expected {expected}, got {actual}")]
    InvalidPixelBuffer {
        index: usize,
        expected: usize,
        actual: usize,
    },
    #[error("layout {0} is not implemented yet")]
    UnsupportedLayout(&'static str),
    #[error("grid layouts require at least one column")]
    InvalidGridColumns,
    #[error("before-after layout requires exactly two images")]
    BeforeAfterRequiresTwo,
    #[error("a divider is only supported by the before-after layout")]
    DividerRequiresBeforeAfter,
    #[error("divider width {width} exceeds the {gap}px gap between panes; increase spacing")]
    DividerTooWide { width: u32, gap: u32 },
    #[error("image dimension overflow while calculating layout")]
    DimensionOverflow,
    #[error("output coordinates overflowed during rendering")]
    CoordinateOverflow,
    #[error("image I/O error: {0}")]
    ImageIo(#[from] image::ImageError),
}
