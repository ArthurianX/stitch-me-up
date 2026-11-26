use std::io::Cursor;
use std::path::Path;

use image::{DynamicImage, ImageDecoder, ImageEncoder, ImageError, ImageReader};

use crate::error::ComposeError;
use crate::types::{ComposedImage, InputImage};

fn load_image_from_reader<R: std::io::BufRead + std::io::Seek>(
    reader: ImageReader<R>,
) -> Result<InputImage, ComposeError> {
    let mut decoder = reader
        .with_guessed_format()
        .map_err(ImageError::IoError)?
        .into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    let rgba = image.to_rgba8();

    Ok(InputImage::new(
        rgba.width(),
        rgba.height(),
        rgba.into_raw(),
    ))
}

pub fn load_image_from_path(path: &Path) -> Result<InputImage, ComposeError> {
    let reader = ImageReader::open(path).map_err(ImageError::IoError)?;
    load_image_from_reader(reader)
}

pub fn load_image_from_bytes(bytes: &[u8]) -> Result<InputImage, ComposeError> {
    load_image_from_reader(ImageReader::new(Cursor::new(bytes)))
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

pub fn encode_jpeg(image: &ComposedImage, quality: u8) -> Result<Vec<u8>, ComposeError> {
    if !(1..=100).contains(&quality) {
        return Err(ComposeError::InvalidJpegQuality { quality });
    }
    validate_output_buffer(image)?;

    let mut rgb = vec![0u8; image.width as usize * image.height as usize * 3];
    for (i, px) in image.pixels.as_chunks::<4>().0.iter().enumerate() {
        let a = px[3] as u32;
        let inv = 255 - a;
        for (c, ch) in px[..3].iter().enumerate() {
            let v = (*ch as u32 * a + 255 * inv + 127) / 255;
            rgb[i * 3 + c] = v as u8;
        }
    }

    let mut buffer = Vec::new();
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality).write_image(
        &rgb,
        image.width,
        image.height,
        image::ExtendedColorType::Rgb8,
    )?;

    Ok(buffer)
}

pub fn save_jpeg(path: &Path, image: &ComposedImage, quality: u8) -> Result<(), ComposeError> {
    let bytes = encode_jpeg(image, quality)?;
    std::fs::write(path, bytes).map_err(ImageError::IoError)?;
    Ok(())
}
