use crate::error::ComposeError;
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
