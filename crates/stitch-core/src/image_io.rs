use std::path::Path;

use image::ImageEncoder;

use crate::error::ComposeError;
use crate::types::{ComposedImage, InputImage};

pub fn load_image_from_path(path: &Path) -> Result<InputImage, ComposeError> {
    let image = image::open(path)?;
    let rgba = image.to_rgba8();

    Ok(InputImage::new(
        rgba.width(),
        rgba.height(),
        rgba.into_raw(),
    ))
}

pub fn load_image_from_bytes(bytes: &[u8]) -> Result<InputImage, ComposeError> {
    let image = image::load_from_memory(bytes)?;
    let rgba = image.to_rgba8();

    Ok(InputImage::new(
        rgba.width(),
        rgba.height(),
        rgba.into_raw(),
    ))
}

fn validate_output_buffer(image: &ComposedImage) -> Result<(), ComposeError> {
    let expected = u64::from(image.width)
        .checked_mul(u64::from(image.height))
        .and_then(|pixels| pixels.checked_mul(4))
        .and_then(|bytes| usize::try_from(bytes).ok())
        .ok_or(ComposeError::DimensionOverflow)?;
    let actual = image.pixels.len();
    if expected != actual {
        return Err(ComposeError::InvalidOutputBuffer {
            width: image.width,
            height: image.height,
            expected,
            actual,
        });
    }
    Ok(())
}

pub fn encode_png(image: &ComposedImage) -> Result<Vec<u8>, ComposeError> {
    validate_output_buffer(image)?;

    let mut buffer = Vec::new();
    image::codecs::png::PngEncoder::new(&mut buffer).write_image(
        &image.pixels,
        image.width,
        image.height,
        image::ExtendedColorType::Rgba8,
    )?;

    Ok(buffer)
}

pub fn save_png(path: &Path, image: &ComposedImage) -> Result<(), ComposeError> {
    validate_output_buffer(image)?;

    image::save_buffer_with_format(
        path,
        &image.pixels,
        image.width,
        image.height,
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;

    Ok(())
}
