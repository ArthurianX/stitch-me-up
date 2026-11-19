use std::path::Path;

use image::ImageEncoder;

use crate::error::ComposeError;
use crate::types::{ComposedImage, InputImage};

pub fn load_image_from_path(path: &Path) -> Result<InputImage, ComposeError> {
    let image = image::open(path)?;
    let rgba = image.to_rgba8();

    Ok(InputImage {
        width: rgba.width(),
        height: rgba.height(),
        pixels: rgba.into_raw(),
    })
}

pub fn load_image_from_bytes(bytes: &[u8]) -> Result<InputImage, ComposeError> {
    let image = image::load_from_memory(bytes)?;
    let rgba = image.to_rgba8();

    Ok(InputImage {
        width: rgba.width(),
        height: rgba.height(),
        pixels: rgba.into_raw(),
    })
}

pub fn encode_png(image: &ComposedImage) -> Result<Vec<u8>, ComposeError> {
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
