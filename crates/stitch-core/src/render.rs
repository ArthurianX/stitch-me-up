use image::imageops::FilterType;

use crate::error::ComposeError;
use crate::types::{
    CanvasPlan, ComposeOptions, ComposedImage, Divider, InputImage, PaneStyle, PlacedImage,
};

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
        let style = source.style.as_ref().unwrap_or(&options.pane_style);
        blit_image(
            &mut pixels,
            plan.width,
            plan.height,
            placed,
            &source_pixels,
            style,
        )?;
    }

    if let Some(divider) = options.divider {
        if divider.width > 0 && plan.placed_images.len() >= 2 {
            let first = &plan.placed_images[0];
            let second = &plan.placed_images[1];
            let gap_start = first
                .x
                .checked_add(first.width)
                .ok_or(ComposeError::CoordinateOverflow)?;
            let gap = second.x.saturating_sub(gap_start);

            draw_divider(&mut pixels, plan.width, plan.height, first, gap, &divider)?;
        }
    }

    Ok(ComposedImage {
        width: plan.width,
        height: plan.height,
        pixels,
    })
}

/// Centers the divider in the gap between the two panes and composites it pixel by pixel.
fn draw_divider(
    canvas: &mut [u8],
    canvas_width: u32,
    canvas_height: u32,
    first: &PlacedImage,
    gap: u32,
    divider: &Divider,
) -> Result<(), ComposeError> {
    let offset = gap.saturating_sub(divider.width) / 2;
    let start_x = first
        .x
        .checked_add(first.width)
        .and_then(|x| x.checked_add(offset))
        .ok_or(ComposeError::CoordinateOverflow)?;
    let divider_pixel = divider.color.to_rgba();

    for row in 0..first.height {
        let canvas_row = first
            .y
            .checked_add(row)
            .ok_or(ComposeError::CoordinateOverflow)?;
        if canvas_row >= canvas_height {
            return Err(ComposeError::CoordinateOverflow);
        }

        for column in 0..divider.width {
            let x = start_x
                .checked_add(column)
                .ok_or(ComposeError::CoordinateOverflow)?;
            if x >= canvas_width {
                return Err(ComposeError::CoordinateOverflow);
            }

            let index = linear_rgba_index(canvas_width, x, canvas_row)?;
            let end = index
                .checked_add(4)
                .ok_or(ComposeError::CoordinateOverflow)?;
            let pixel = canvas
                .get_mut(index..end)
                .ok_or(ComposeError::CoordinateOverflow)?;

            blend_pixel(pixel, &divider_pixel);
        }
    }

    Ok(())
}

fn fill_background(pixels: &mut [u8], background: [u8; 4]) {
    let (chunks, _) = pixels.as_chunks_mut::<4>();
    for chunk in chunks {
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

    // Premultiply before resampling so transparent pixels contribute no color;
    // unpremultiply afterwards. Alpha is left untouched by both steps.
    let mut pixels = image.pixels.clone();
    for chunk in pixels.as_chunks_mut::<4>().0 {
        let alpha = u32::from(chunk[3]);
        for channel in chunk.iter_mut().take(3) {
            *channel = ((u32::from(*channel) * alpha + 127) / 255) as u8;
        }
    }

    let premultiplied = image::RgbaImage::from_raw(image.width, image.height, pixels)
        .ok_or(ComposeError::DimensionOverflow)?;

    let resized = image::imageops::resize(
        &premultiplied,
        target_width,
        target_height,
        FilterType::Triangle,
    );

    let mut pixels = resized.into_raw();
    for chunk in pixels.as_chunks_mut::<4>().0 {
        let alpha = u32::from(chunk[3]);
        if alpha == 0 {
            chunk[0] = 0;
            chunk[1] = 0;
            chunk[2] = 0;
            continue;
        }
        for channel in chunk.iter_mut().take(3) {
            let value = (u32::from(*channel) * 255 + alpha / 2) / alpha;
            *channel = u8::try_from(value).unwrap_or(u8::MAX);
        }
    }

    Ok(pixels)
}

fn blit_image(
    canvas: &mut [u8],
    canvas_width: u32,
    canvas_height: u32,
    placed: &PlacedImage,
    image_pixels: &[u8],
    style: &PaneStyle,
) -> Result<(), ComposeError> {
    // Undecorated panes keep the plain copy/blend path and never evaluate corner masks.
    let decorated = style.frame.is_some() || style.corner_radius > 0;
    let radius = style.corner_radius.min(placed.width.min(placed.height) / 2);
    let frame_width = style.frame.map(|frame| frame.width).unwrap_or(0);
    let frame_pixel = style
        .frame
        .map(|frame| frame.color.to_rgba())
        .unwrap_or([0, 0, 0, 0]);

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

        if !decorated {
            let (source_pixels, _) = source.as_chunks::<4>();
            if source_pixels.iter().all(|pixel| pixel[3] == 255) {
                destination.copy_from_slice(source);
                continue;
            }

            let (destination_pixels, _) = destination.as_chunks_mut::<4>();
            for (destination_pixel, source_pixel) in
                destination_pixels.iter_mut().zip(source_pixels.iter())
            {
                blend_pixel(destination_pixel, source_pixel);
            }

            continue;
        }

        let (destination_pixels, _) = destination.as_chunks_mut::<4>();
        let (source_pixels, _) = source.as_chunks::<4>();
        for (column, (destination_pixel, source_pixel)) in destination_pixels
            .iter_mut()
            .zip(source_pixels.iter())
            .enumerate()
        {
            let x = u32::try_from(column).map_err(|_| ComposeError::CoordinateOverflow)?;

            if !inside_rounded(x, row, placed.width, placed.height, radius) {
                continue;
            }

            let source_pixel: &[u8] =
                if in_frame_ring(x, row, placed.width, placed.height, radius, frame_width) {
                    &frame_pixel[..]
                } else {
                    source_pixel
                };

            blend_pixel(destination_pixel, source_pixel);
        }
    }

    Ok(())
}

/// True when `(x, y)` is inside a hard-edged rounded rectangle, comparing the squared distance
/// to the nearest corner center against `radius^2` in integer arithmetic.
fn inside_rounded(x: u32, y: u32, width: u32, height: u32, radius: u32) -> bool {
    if radius == 0 {
        return true;
    }

    let dx = corner_distance(x, width, radius);
    let dy = corner_distance(y, height, radius);
    let radius = i64::from(radius);

    dx * dx + dy * dy <= radius * radius
}

/// Distance along one axis from `position` to the nearest corner center.
/// Callers guarantee `radius <= extent / 2`, which keeps that nearest-center distance
/// symmetric, including when the radius is exactly half the extent.
fn corner_distance(position: u32, extent: u32, radius: u32) -> i64 {
    let position = i64::from(position);
    let radius = i64::from(radius);
    let far_center = i64::from(extent) - 1 - radius;

    if position < radius {
        radius - position
    } else if position > far_center {
        position - far_center
    } else {
        0
    }
}

/// True when `(x, y)` belongs to the frame ring of `frame_width` pixels inside a rounded pane.
fn in_frame_ring(x: u32, y: u32, width: u32, height: u32, radius: u32, frame_width: u32) -> bool {
    if frame_width == 0 {
        return false;
    }

    let inner_width = width.saturating_sub(frame_width.saturating_mul(2));
    let inner_height = height.saturating_sub(frame_width.saturating_mul(2));

    if inner_width == 0 || inner_height == 0 {
        return true;
    }

    if x < frame_width || y < frame_width || x >= width - frame_width || y >= height - frame_width {
        return true;
    }

    !inside_rounded(
        x - frame_width,
        y - frame_width,
        inner_width,
        inner_height,
        radius.saturating_sub(frame_width),
    )
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
