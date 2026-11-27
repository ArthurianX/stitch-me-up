use crate::error::ComposeError;
use crate::types::{
    Alignment, CanvasPlan, CellSize, ComposeOptions, InputImage, LayoutKind, PlacedImage,
    ResizeMode, SourceCrop,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResolvedSize {
    width: u32,
    height: u32,
}

pub fn plan_layout(
    images: &[InputImage],
    options: &ComposeOptions,
) -> Result<CanvasPlan, ComposeError> {
    validate_images(images)?;

    if options.divider.is_some() && !matches!(options.layout, LayoutKind::BeforeAfter) {
        return Err(ComposeError::DividerRequiresBeforeAfter);
    }

    if !options.column_weights.is_empty() && !matches!(options.layout, LayoutKind::Grid { .. }) {
        return Err(ComposeError::ColumnWeightsRequireGrid);
    }

    if let Some(cell) = options.cell_size {
        if !matches!(options.layout, LayoutKind::Grid { .. }) {
            return Err(ComposeError::CellSizeRequiresGrid);
        }
        if cell.width == 0 || cell.height == 0 {
            return Err(ComposeError::InvalidCellSize {
                width: cell.width,
                height: cell.height,
            });
        }
        if !options.column_weights.is_empty() {
            return Err(ComposeError::CellSizeConflictsWithColumnWeights);
        }
        if options.resize_mode != ResizeMode::None {
            return Err(ComposeError::CellSizeConflictsWithResizeMode);
        }
    }

    match options.layout {
        LayoutKind::VerticalStack => {
            plan_vertical(&resolve_sizes(images, options.resize_mode)?, options)
        }
        LayoutKind::HorizontalStack => {
            plan_horizontal(&resolve_sizes(images, options.resize_mode)?, options)
        }
        LayoutKind::BeforeAfter => plan_before_after(images, options),
        LayoutKind::Grid { columns } => match options.cell_size {
            Some(cell) => plan_grid_with_cells(images, options, columns, cell),
            None => plan_grid(&resolve_sizes(images, options.resize_mode)?, options, columns),
        },
    }
}

fn validate_images(images: &[InputImage]) -> Result<(), ComposeError> {
    if images.len() < 2 {
        return Err(ComposeError::NotEnoughImages);
    }

    for (index, image) in images.iter().enumerate() {
        let expected = image
            .expected_len()
            .ok_or(ComposeError::DimensionOverflow)?;
        let actual = image.pixels.len();
        if expected != actual {
            return Err(ComposeError::InvalidPixelBuffer {
                index,
                expected,
                actual,
            });
        }
    }

    Ok(())
}

fn resolve_sizes(
    images: &[InputImage],
    resize_mode: ResizeMode,
) -> Result<Vec<ResolvedSize>, ComposeError> {
    match resize_mode {
        ResizeMode::None => Ok(images
            .iter()
            .map(|image| ResolvedSize {
                width: image.width,
                height: image.height,
            })
            .collect()),
        ResizeMode::FitToSameWidth => {
            let target_width = images.iter().map(|image| image.width).max().unwrap_or(0);
            images
                .iter()
                .map(|image| scale_to_width(image.width, image.height, target_width))
                .collect()
        }
        ResizeMode::FitToSameHeight => {
            let target_height = images.iter().map(|image| image.height).max().unwrap_or(0);
            images
                .iter()
                .map(|image| scale_to_height(image.width, image.height, target_height))
                .collect()
        }
    }
}

fn scale_to_width(
    width: u32,
    height: u32,
    target_width: u32,
) -> Result<ResolvedSize, ComposeError> {
    if width == 0 || height == 0 || target_width == 0 {
        return Err(ComposeError::DimensionOverflow);
    }

    let scaled_height = scaled_dimension(height, target_width, width)?;

    Ok(ResolvedSize {
        width: target_width,
        height: scaled_height,
    })
}

fn scale_to_height(
    width: u32,
    height: u32,
    target_height: u32,
) -> Result<ResolvedSize, ComposeError> {
    if width == 0 || height == 0 || target_height == 0 {
        return Err(ComposeError::DimensionOverflow);
    }

    let scaled_width = scaled_dimension(width, target_height, height)?;

    Ok(ResolvedSize {
        width: scaled_width,
        height: target_height,
    })
}

fn scaled_dimension(numerator: u32, target: u32, denominator: u32) -> Result<u32, ComposeError> {
    let numerator = u64::from(numerator);
    let target = u64::from(target);
    let denominator = u64::from(denominator);

    let value = numerator
        .checked_mul(target)
        .ok_or(ComposeError::DimensionOverflow)?
        .checked_add(denominator / 2)
        .ok_or(ComposeError::DimensionOverflow)?
        / denominator;

    u32::try_from(value).map_err(|_| ComposeError::DimensionOverflow)
}

fn plan_vertical(
    sizes: &[ResolvedSize],
    options: &ComposeOptions,
) -> Result<CanvasPlan, ComposeError> {
    let content_width = sizes.iter().map(|size| size.width).max().unwrap_or(0);
    let content_height = sum_dimensions(sizes.iter().map(|size| size.height), options.spacing.y)?;

    let canvas_width =
        padded_dimension(content_width, options.padding.left, options.padding.right)?;
    let canvas_height =
        padded_dimension(content_height, options.padding.top, options.padding.bottom)?;

    let mut y = options.padding.top;
    let mut placed_images = Vec::with_capacity(sizes.len());

    for (index, size) in sizes.iter().enumerate() {
        let x = options
            .padding
            .left
            .checked_add(aligned_offset(
                content_width,
                size.width,
                options.alignment,
            )?)
            .ok_or(ComposeError::DimensionOverflow)?;

        placed_images.push(PlacedImage {
            index,
            x,
            y,
            width: size.width,
            height: size.height,
            crop: None,
        });

        y = y
            .checked_add(size.height)
            .ok_or(ComposeError::DimensionOverflow)?;
        if index + 1 < sizes.len() {
            y = y
                .checked_add(options.spacing.y)
                .ok_or(ComposeError::DimensionOverflow)?;
        }
    }

    Ok(CanvasPlan {
        width: canvas_width,
        height: canvas_height,
        placed_images,
    })
}

fn plan_horizontal(
    sizes: &[ResolvedSize],
    options: &ComposeOptions,
) -> Result<CanvasPlan, ComposeError> {
    let content_width = sum_dimensions(sizes.iter().map(|size| size.width), options.spacing.x)?;
    let content_height = sizes.iter().map(|size| size.height).max().unwrap_or(0);

    let canvas_width =
        padded_dimension(content_width, options.padding.left, options.padding.right)?;
    let canvas_height =
        padded_dimension(content_height, options.padding.top, options.padding.bottom)?;

    let mut x = options.padding.left;
    let mut placed_images = Vec::with_capacity(sizes.len());

    for (index, size) in sizes.iter().enumerate() {
        let y = options
            .padding
            .top
            .checked_add(aligned_offset(
                content_height,
                size.height,
                options.alignment,
            )?)
            .ok_or(ComposeError::DimensionOverflow)?;

        placed_images.push(PlacedImage {
            index,
            x,
            y,
            width: size.width,
            height: size.height,
            crop: None,
        });

        x = x
            .checked_add(size.width)
            .ok_or(ComposeError::DimensionOverflow)?;
        if index + 1 < sizes.len() {
            x = x
                .checked_add(options.spacing.x)
                .ok_or(ComposeError::DimensionOverflow)?;
        }
    }

    Ok(CanvasPlan {
        width: canvas_width,
        height: canvas_height,
        placed_images,
    })
}

/// Scales both panes to the shared tallest height, ignoring `options.resize_mode`.
fn plan_before_after(
    images: &[InputImage],
    options: &ComposeOptions,
) -> Result<CanvasPlan, ComposeError> {
    if images.len() != 2 {
        return Err(ComposeError::BeforeAfterRequiresTwo);
    }

    let target_height = images.iter().map(|image| image.height).max().unwrap_or(0);

    let sizes = images
        .iter()
        .map(|image| scale_to_height(image.width, image.height, target_height))
        .collect::<Result<Vec<ResolvedSize>, ComposeError>>()?;

    let plan = plan_horizontal(&sizes, options)?;

    if let Some(divider) = options.divider {
        let gap_start = plan.placed_images[0]
            .x
            .checked_add(plan.placed_images[0].width)
            .ok_or(ComposeError::DimensionOverflow)?;
        let gap = plan.placed_images[1].x.saturating_sub(gap_start);

        if divider.width > gap {
            return Err(ComposeError::DividerTooWide {
                width: divider.width,
                gap,
            });
        }
    }

    Ok(plan)
}

fn plan_grid(
    sizes: &[ResolvedSize],
    options: &ComposeOptions,
    columns: u32,
) -> Result<CanvasPlan, ComposeError> {
    if columns == 0 {
        return Err(ComposeError::InvalidGridColumns);
    }

    let requested = usize::try_from(columns).map_err(|_| ComposeError::InvalidGridColumns)?;

    if !options.column_weights.is_empty() {
        if options.column_weights.len() != requested {
            return Err(ComposeError::ColumnWeightCountMismatch {
                expected: requested,
                actual: options.column_weights.len(),
            });
        }

        for (index, weight) in options.column_weights.iter().enumerate() {
            if *weight == 0 {
                return Err(ComposeError::ZeroColumnWeight { index });
            }
        }
    }

    let columns = requested.min(sizes.len()).max(1);
    let rows = sizes.len().div_ceil(columns);

    let mut column_widths = vec![0u32; columns];
    let mut row_heights = vec![0u32; rows];

    for (index, size) in sizes.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;

        column_widths[column] = column_widths[column].max(size.width);
        row_heights[row] = row_heights[row].max(size.height);
    }

    if !options.column_weights.is_empty() {
        // `unit` is the smallest track unit that keeps every weighted track at or
        // above its natural column width; weights beyond the clamped column count
        // are unused, so zip stops with the natural widths.
        let mut unit = 0u64;
        for (natural, weight) in column_widths.iter().zip(options.column_weights.iter()) {
            let natural = u64::from(*natural);
            let weight = u64::from(*weight);

            let quotient = if natural == 0 {
                0
            } else {
                natural
                    .checked_add(weight)
                    .and_then(|sum| sum.checked_sub(1))
                    .ok_or(ComposeError::DimensionOverflow)?
                    / weight
            };

            unit = unit.max(quotient);
        }

        let unit = u32::try_from(unit).map_err(|_| ComposeError::DimensionOverflow)?;

        column_widths = options
            .column_weights
            .iter()
            .take(columns)
            .map(|weight| {
                unit.checked_mul(*weight)
                    .ok_or(ComposeError::DimensionOverflow)
            })
            .collect::<Result<Vec<u32>, ComposeError>>()?;
    }

    let content_width = sum_dimensions(column_widths.iter().copied(), options.spacing.x)?;
    let content_height = sum_dimensions(row_heights.iter().copied(), options.spacing.y)?;

    let canvas_width =
        padded_dimension(content_width, options.padding.left, options.padding.right)?;
    let canvas_height =
        padded_dimension(content_height, options.padding.top, options.padding.bottom)?;

    let column_offsets = track_offsets(&column_widths, options.padding.left, options.spacing.x)?;
    let row_offsets = track_offsets(&row_heights, options.padding.top, options.spacing.y)?;

    let mut placed_images = Vec::with_capacity(sizes.len());

    for (index, size) in sizes.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;

        let x = column_offsets[column]
            .checked_add(aligned_offset(
                column_widths[column],
                size.width,
                options.alignment,
            )?)
            .ok_or(ComposeError::DimensionOverflow)?;

        placed_images.push(PlacedImage {
            index,
            x,
            y: row_offsets[row],
            width: size.width,
            height: size.height,
            crop: None,
        });
    }

    Ok(CanvasPlan {
        width: canvas_width,
        height: canvas_height,
        placed_images,
    })
}


fn cover_crop(
    sw: u32,
    sh: u32,
    cw: u32,
    ch: u32,
) -> Result<Option<SourceCrop>, ComposeError> {
    if sw == 0 || sh == 0 || cw == 0 || ch == 0 {
        return Err(ComposeError::DimensionOverflow);
    }

    let sw64 = u64::from(sw);
    let sh64 = u64::from(sh);
    let cw64 = u64::from(cw);
    let ch64 = u64::from(ch);

    if sw64 * ch64 == sh64 * cw64 {
        return Ok(None);
    }

    if sw64 * ch64 > sh64 * cw64 {
        let height = sh;
        let width = scaled_dimension(sh, cw, ch)?.min(sw).max(1);
        let x = (sw - width) / 2;
        return Ok(Some(SourceCrop { x, y: 0, width, height }));
    }

    let width = sw;
    let height = scaled_dimension(sw, ch, cw)?.min(sh).max(1);
    let y = (sh - height) / 2;
    Ok(Some(SourceCrop { x: 0, y, width, height }))
}

fn plan_grid_with_cells(
    images: &[InputImage],
    options: &ComposeOptions,
    columns: u32,
    cell: CellSize,
) -> Result<CanvasPlan, ComposeError> {
    if columns == 0 {
        return Err(ComposeError::InvalidGridColumns);
    }

    let requested = usize::try_from(columns).map_err(|_| ComposeError::InvalidGridColumns)?;
    let columns = requested.min(images.len()).max(1);
    let rows = images.len().div_ceil(columns);

    let column_widths = vec![cell.width; columns];
    let row_heights = vec![cell.height; rows];

    let content_width = sum_dimensions(column_widths.iter().copied(), options.spacing.x)?;
    let content_height = sum_dimensions(row_heights.iter().copied(), options.spacing.y)?;

    let canvas_width =
        padded_dimension(content_width, options.padding.left, options.padding.right)?;
    let canvas_height =
        padded_dimension(content_height, options.padding.top, options.padding.bottom)?;

    let column_offsets = track_offsets(&column_widths, options.padding.left, options.spacing.x)?;
    let row_offsets = track_offsets(&row_heights, options.padding.top, options.spacing.y)?;

    let mut placed_images = Vec::with_capacity(images.len());

    for (index, image) in images.iter().enumerate() {
        let column = index % columns;
        let row = index / columns;

        let x = column_offsets[column]
            .checked_add(aligned_offset(
                column_widths[column],
                cell.width,
                options.alignment,
            )?)
            .ok_or(ComposeError::DimensionOverflow)?;

        placed_images.push(PlacedImage {
            index,
            x,
            y: row_offsets[row],
            width: cell.width,
            height: cell.height,
            crop: cover_crop(image.width, image.height, cell.width, cell.height)?,
        });
    }

    Ok(CanvasPlan {
        width: canvas_width,
        height: canvas_height,
        placed_images,
    })
}

fn track_offsets(sizes: &[u32], leading: u32, spacing: u32) -> Result<Vec<u32>, ComposeError> {
    let mut offsets = Vec::with_capacity(sizes.len());
    let mut offset = leading;

    for (index, size) in sizes.iter().copied().enumerate() {
        offsets.push(offset);

        offset = offset
            .checked_add(size)
            .ok_or(ComposeError::DimensionOverflow)?;
        if index + 1 < sizes.len() {
            offset = offset
                .checked_add(spacing)
                .ok_or(ComposeError::DimensionOverflow)?;
        }
    }

    Ok(offsets)
}

fn padded_dimension(value: u32, leading: u32, trailing: u32) -> Result<u32, ComposeError> {
    value
        .checked_add(leading)
        .and_then(|result| result.checked_add(trailing))
        .ok_or(ComposeError::DimensionOverflow)
}

fn sum_dimensions(values: impl Iterator<Item = u32>, spacing: u32) -> Result<u32, ComposeError> {
    let mut total = 0u32;
    let mut count = 0u32;

    for value in values {
        total = total
            .checked_add(value)
            .ok_or(ComposeError::DimensionOverflow)?;
        count = count
            .checked_add(1)
            .ok_or(ComposeError::DimensionOverflow)?;
    }

    let gaps = count.saturating_sub(1);
    total
        .checked_add(
            gaps.checked_mul(spacing)
                .ok_or(ComposeError::DimensionOverflow)?,
        )
        .ok_or(ComposeError::DimensionOverflow)
}

fn aligned_offset(
    available: u32,
    occupied: u32,
    alignment: Alignment,
) -> Result<u32, ComposeError> {
    let free = available
        .checked_sub(occupied)
        .ok_or(ComposeError::DimensionOverflow)?;

    Ok(match alignment {
        Alignment::Start => 0,
        Alignment::Center => free / 2,
        Alignment::End => free,
    })
}
