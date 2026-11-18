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
    #[error("image dimension overflow while calculating layout")]
    DimensionOverflow,
    #[error("output coordinates overflowed during rendering")]
    CoordinateOverflow,
    #[error("image I/O error: {0}")]
    ImageIo(#[from] image::ImageError),
}
