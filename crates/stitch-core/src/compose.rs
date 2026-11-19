use crate::error::ComposeError;
use crate::image_io::{encode_png, load_image_from_bytes};
use crate::layout;
use crate::render;
use crate::types::{CanvasPlan, ComposeOptions, ComposedImage, InputImage};

pub fn plan_layout(
    images: &[InputImage],
    options: &ComposeOptions,
) -> Result<CanvasPlan, ComposeError> {
    layout::plan_layout(images, options)
}

pub fn compose(
    images: &[InputImage],
    options: &ComposeOptions,
) -> Result<ComposedImage, ComposeError> {
    let plan = plan_layout(images, options)?;
    render::render(images, &plan, options)
}

/// Decodes encoded PNG or JPEG buffers, composes them, and re-encodes the result as PNG.
pub fn compose_to_png(
    images: &[Vec<u8>],
    options: &ComposeOptions,
) -> Result<Vec<u8>, ComposeError> {
    let decoded = images
        .iter()
        .map(|bytes| load_image_from_bytes(bytes))
        .collect::<Result<Vec<InputImage>, ComposeError>>()?;

    encode_png(&compose(&decoded, options)?)
}
