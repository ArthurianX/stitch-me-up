use image::imageops::FilterType;

use crate::error::ComposeError;
use crate::types::{CanvasPlan, ComposeOptions, ComposedImage, InputImage, PlacedImage};

pub fn render(
    images: &[InputImage],
    plan: &CanvasPlan,
    options: &ComposeOptions,
) -> Result<ComposedImage, ComposeError> {
    let len = u64::from(plan.width)
        .checked_mul(u64::from(plan.height))
        .and_then(|value| value.checked_mul(4))
        .ok_or(ComposeError::DimensionOverflow)?;
    let len = usize::try_from(len).map_err(|_| ComposeError::DimensionOverflow)?;

    let mut pixels = vec![0u8; len];
    fill_background(&mut pixels, options.background.to_rgba());

    for placed in &plan.placed_images {
        let source = &images[placed.index];
        let source_pixels = prepare_pixels(source, placed.width, placed.height)?;
        blit_image(&mut pixels, plan.width, plan.height, placed, &source_pixels)?;
    }

    Ok(ComposedImage {
        width: plan.width,
        height: plan.height,
        pixels,
    })
}

fn fill_background(pixels: &mut [u8], background: [u8; 4]) {
    for chunk in pixels.chunks_exact_mut(4) {
        chunk.copy_from_slice(&background);
    }
}

fn prepare_pixels(
    image: &InputImage,
    target_width: u32,
    target_height: u32,
) -> Result<Vec<u8>, ComposeError> {
    if image.width == target_width && image.height == target_height {
        return Ok(image.pixels.clone());
    }

    let rgba = image::RgbaImage::from_raw(image.width, image.height, image.pixels.clone())
        .ok_or(ComposeError::DimensionOverflow)?;
    let resized = image::imageops::resize(&rgba, target_width, target_height, FilterType::Triangle);

    Ok(resized.into_raw())
}

fn blit_image(
    canvas: &mut [u8],
    canvas_width: u32,
    canvas_height: u32,
    placed: &PlacedImage,
    image_pixels: &[u8],
) -> Result<(), ComposeError> {
    for row in 0..placed.height {
        let canvas_row = placed
            .y
            .checked_add(row)
            .ok_or(ComposeError::CoordinateOverflow)?;
        if canvas_row >= canvas_height {
            return Err(ComposeError::CoordinateOverflow);
        }

        let dest_index = linear_rgba_index(canvas_width, placed.x, canvas_row)?;
        let row_len = usize::try_from(u64::from(placed.width) * 4)
            .map_err(|_| ComposeError::CoordinateOverflow)?;

        let src_index = usize::try_from(u64::from(row) * u64::from(placed.width) * 4)
            .map_err(|_| ComposeError::CoordinateOverflow)?;

        let end_x = placed
            .x
            .checked_add(placed.width)
            .ok_or(ComposeError::CoordinateOverflow)?;
        if end_x > canvas_width {
            return Err(ComposeError::CoordinateOverflow);
        }

        let dest_end = dest_index
            .checked_add(row_len)
            .ok_or(ComposeError::CoordinateOverflow)?;
        let src_end = src_index
            .checked_add(row_len)
            .ok_or(ComposeError::CoordinateOverflow)?;

        let source = image_pixels
            .get(src_index..src_end)
            .ok_or(ComposeError::CoordinateOverflow)?;
        let destination = canvas
            .get_mut(dest_index..dest_end)
            .ok_or(ComposeError::CoordinateOverflow)?;

        if source.chunks_exact(4).all(|pixel| pixel[3] == 255) {
            destination.copy_from_slice(source);
            continue;
        }

        for (destination_pixel, source_pixel) in
            destination.chunks_exact_mut(4).zip(source.chunks_exact(4))
        {
            blend_pixel(destination_pixel, source_pixel);
        }
    }

    Ok(())
}

/// Source-over compositing of straight (non-premultiplied) RGBA, rounded with integer math only.
fn blend_pixel(destination: &mut [u8], source: &[u8]) {
    let source_alpha = u32::from(source[3]);

    if source_alpha == 0 {
        return;
    }

    if source_alpha == 255 {
        destination.copy_from_slice(source);
        return;
    }

    let destination_alpha = u32::from(destination[3]);
    let inverse = 255 - source_alpha;

    // `255 * out_a` before rounding, so each channel is an exact weighted average;
    // source_alpha >= 1 keeps the denominator at least 255.
    let denominator = 255 * source_alpha + destination_alpha * inverse;
    let blended_alpha = (denominator + 127) / 255;

    for channel in 0..3 {
        let source_channel = u32::from(source[channel]);
        let destination_channel = u32::from(destination[channel]);
        let numerator =
            source_channel * source_alpha * 255 + destination_channel * destination_alpha * inverse;

        destination[channel] = clamp_channel((numerator + denominator / 2) / denominator);
    }

    destination[3] = clamp_channel(blended_alpha);
}

fn clamp_channel(value: u32) -> u8 {
    u8::try_from(value).unwrap_or(u8::MAX)
}

fn linear_rgba_index(width: u32, x: u32, y: u32) -> Result<usize, ComposeError> {
    let pixel_index = u64::from(y)
        .checked_mul(u64::from(width))
        .and_then(|value| value.checked_add(u64::from(x)))
        .ok_or(ComposeError::CoordinateOverflow)?;
    let byte_index = pixel_index
        .checked_mul(4)
        .ok_or(ComposeError::CoordinateOverflow)?;

    usize::try_from(byte_index).map_err(|_| ComposeError::CoordinateOverflow)
}
