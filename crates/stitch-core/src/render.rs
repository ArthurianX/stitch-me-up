use image::imageops::FilterType;

use crate::error::ComposeError;
use crate::types::{CanvasPlan, ComposeOptions, ComposedImage, InputImage};

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
        blit_image(
            &mut pixels,
            plan.width,
            plan.height,
            placed.x,
            placed.y,
            placed.width,
            placed.height,
            &source_pixels,
        )?;
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
    dest_x: u32,
    dest_y: u32,
    image_width: u32,
    image_height: u32,
    image_pixels: &[u8],
) -> Result<(), ComposeError> {
    for row in 0..image_height {
        let canvas_row = dest_y
            .checked_add(row)
            .ok_or(ComposeError::CoordinateOverflow)?;
        if canvas_row >= canvas_height {
            return Err(ComposeError::CoordinateOverflow);
        }

        let dest_index = linear_rgba_index(canvas_width, dest_x, canvas_row)?;
        let row_len = usize::try_from(u64::from(image_width) * 4)
            .map_err(|_| ComposeError::CoordinateOverflow)?;

        let src_index = usize::try_from(u64::from(row) * u64::from(image_width) * 4)
            .map_err(|_| ComposeError::CoordinateOverflow)?;

        let end_x = dest_x
            .checked_add(image_width)
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

        canvas[dest_index..dest_end].copy_from_slice(&image_pixels[src_index..src_end]);
    }

    Ok(())
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
