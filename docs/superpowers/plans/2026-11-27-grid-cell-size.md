# Uniform Grid Tiles Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a grid-only `cell_size` option that gives every tile explicit dimensions and center-crops each source to fill it, across core, CLI, WASM, web demo, and docs.

**Architecture:** The plan stays the source of truth: `plan_grid` sizes every track to the cell and computes a source-space `SourceCrop` per placed image. `render` crops the source, premultiplies, resizes once to the tile, and leaves decoration untouched. Adapters only set the option.

**Tech Stack:** Rust 2021, `image` 0.25 (Triangle filter), `clap` 4, `wasm-bindgen`, plain ES modules + `node --test`.

**Spec:** `docs/superpowers/specs/2026-11-27-grid-cell-size-design.md`

## Global Constraints

- MSRV 1.88; no new dependencies; no new Cargo features.
- Deterministic integer math only (no floats); all new dimensions checked against overflow.
- Decoration is an overlay inside the placed rect and runs after pixel preparation.
- The plan is the source of truth for geometry; rendering executes it.
- `0.x` breaking changes are allowed; `ComposeOptions` and `PlacedImage` gain fields.
- Core error strings and CLI messages are exactly as written in the spec and this plan.
- Spec acceptance: six images, 3 columns, 320×320, spacing 8, padding 0 → **976×648**.

## Review Focus

- **Extreme aspect ratio** (e.g. 4000×10 into 320×320): a reasonable user expects the tile filled without a panic, overflow, or a huge intermediate buffer. Covered in Task 2.
- **Source smaller than the tile**: expect an upscale that fills the tile, not background showing through. Covered in Task 2.
- **Odd crop excess**: expect the same pixels on every run, with the extra row/column dropped from the right/bottom. Covered in Task 1 (plan) and Task 2 (pixels).
- **Transparent or semi-transparent source**: expect no color tint at cropped/resized edges and preserved alpha. Covered in Task 2.
- **Contradictory options** (non-grid cell, zero size, weights, resize mode): expect a clear typed error, never silently ignored geometry. Covered in Task 1 (core) and Task 3 (CLI).

---

## File Structure

- `crates/stitch-core/src/types.rs` — `CellSize`, `SourceCrop`, `PlacedImage.crop`, `ComposeOptions.cell_size`.
- `crates/stitch-core/src/error.rs` — four new variants.
- `crates/stitch-core/src/layout.rs` — validation, cell geometry, `cover_crop`.
- `crates/stitch-core/src/render.rs` — crop-aware `prepare_pixels`; premultiply/unpremultiply helpers.
- `crates/stitch-core/src/lib.rs` — exports.
- `crates/stitch-core/tests/{layout_tests,render_tests}.rs`, `tests/helpers.rs` — behavior tests + fixtures.
- `crates/stitch-cli/src/lib.rs` — flag, parser, pre-checks, mapping, tests.
- `crates/stitch-wasm/src/lib.rs` — setters and native tests.
- `examples/web/{index.html,app.js,lib.js,tests/lib.test.js,README.md}` — preset control.
- `README.md`, `docs/{api,cli,layouts,wasm}.md`, `docs/README.md`, `ROADMAP.md`, `CHANGELOG.md` — docs.

Each task ends with a commit. Run `cargo fmt` before committing.

---

### Task 1: Core cell types, validation, and grid geometry

**Files:**
- Modify: `crates/stitch-core/src/types.rs`
- Modify: `crates/stitch-core/src/error.rs`
- Modify: `crates/stitch-core/src/layout.rs`
- Modify: `crates/stitch-core/src/lib.rs`
- Modify: `crates/stitch-core/tests/layout_tests.rs`
- Modify: `crates/stitch-core/tests/render_tests.rs` (literals only)
- Modify: `crates/stitch-core/tests/io_tests.rs` (literal only)

**Interfaces:**
- Produces:
  - `pub struct CellSize { pub width: u32, pub height: u32 }` (Debug, Clone, Copy, PartialEq, Eq)
  - `pub struct SourceCrop { pub x: u32, pub y: u32, pub width: u32, pub height: u32 }` (same derives)
  - `PlacedImage.crop: Option<SourceCrop>`
  - `ComposeOptions.cell_size: Option<CellSize>` (default `None`)
  - `ComposeError::{CellSizeRequiresGrid, InvalidCellSize { width, height }, CellSizeConflictsWithColumnWeights, CellSizeConflictsWithResizeMode}`
  - `fn cover_crop(sw: u32, sh: u32, cw: u32, ch: u32) -> Result<Option<SourceCrop>, ComposeError>` (private)
  - `fn plan_grid_with_cells(images: &[InputImage], options: &ComposeOptions, columns: u32, cell: CellSize) -> Result<CanvasPlan, ComposeError>` (private)
- Consumes: existing `track_offsets`, `padded_dimension`, `scaled_dimension`, `sum_dimensions`.

- [ ] **Step 1: Add the types and fields, then repair every literal**

Add the structs, the two fields, the error variants, and re-export `CellSize`/`SourceCrop` from `lib.rs` in alphabetical position. Set `cell_size: None` in `ComposeOptions::default` and `crop: None` in the three `PlacedImage` literals in `layout.rs`, and add `cell_size: None` to the `ComposeOptions` literal in `crates/stitch-cli/src/lib.rs:243`.

Then run:

```bash
cargo check --all-targets
```

Add `cell_size: None,` to every remaining `ComposeOptions { … }` literal and `crop: None,` to the two `PlacedImage` literals in `render_tests.rs` that the compiler reports. Repeat until the workspace compiles. Behavior is unchanged; `cargo test` must pass before continuing.

- [ ] **Step 2: Write the failing geometry and validation tests**

Append to `crates/stitch-core/tests/layout_tests.rs` (imports: add `CellSize`, `SourceCrop`):

```rust
fn cell_options(columns: u32, width: u32, height: u32) -> ComposeOptions {
    ComposeOptions {
        layout: LayoutKind::Grid { columns },
        spacing: Spacing::all(8),
        padding: Insets::default(),
        alignment: Alignment::Center,
        cell_size: Some(CellSize { width, height }),
        ..ComposeOptions::default()
    }
}

#[test]
fn plans_uniform_grid_cells_for_six_images() {
    let images = (0..6u32)
        .map(|i| solid_image(40 + i * 10, 30 + i * 5, [i as u8, 0, 0, 255]))
        .collect::<Vec<_>>();
    let plan = plan_layout(&images, &cell_options(3, 320, 320)).expect("cell grid");

    assert_eq!((plan.width, plan.height), (976, 648));
    assert_eq!(plan.placed_images.len(), 6);
    for (index, placed) in plan.placed_images.iter().enumerate() {
        assert_eq!((placed.width, placed.height), (320, 320));
        assert_eq!(placed.x, (index as u32 % 3) * 328);
        assert_eq!(placed.y, (index as u32 / 3) * 328);
    }
}

#[test]
fn cell_mode_keeps_padding_spacing_and_partial_rows() {
    let images = (0..4u32)
        .map(|_| solid_image(8, 8, [0, 255, 0, 255]))
        .collect::<Vec<_>>();
    let options = ComposeOptions {
        spacing: Spacing { x: 2, y: 3 },
        padding: Insets::all(5),
        ..cell_options(3, 10, 20)
    };
    let plan = plan_layout(&images, &options).expect("cell grid");

    assert_eq!((plan.width, plan.height), (44, 53));
    let last = &plan.placed_images[3];
    assert_eq!((last.x, last.y), (5, 28));
}

#[test]
fn cell_mode_computes_centered_source_crops() {
    let wide = vec![solid_image(200, 50, [0; 4]), solid_image(200, 50, [0; 4])];
    let plan = plan_layout(&wide, &cell_options(2, 100, 100)).expect("wide");
    assert_eq!(
        plan.placed_images[0].crop,
        Some(SourceCrop { x: 75, y: 0, width: 50, height: 50 })
    );

    let tall = vec![solid_image(50, 200, [0; 4]), solid_image(50, 200, [0; 4])];
    let plan = plan_layout(&tall, &cell_options(2, 100, 100)).expect("tall");
    assert_eq!(
        plan.placed_images[0].crop,
        Some(SourceCrop { x: 0, y: 75, width: 50, height: 50 })
    );

    let same = vec![solid_image(200, 100, [0; 4]), solid_image(200, 100, [0; 4])];
    let plan = plan_layout(&same, &cell_options(2, 100, 50)).expect("same aspect");
    assert_eq!(plan.placed_images[0].crop, None);

    let exact = vec![solid_image(100, 100, [0; 4]), solid_image(100, 100, [0; 4])];
    let plan = plan_layout(&exact, &cell_options(2, 100, 100)).expect("exact");
    assert_eq!(plan.placed_images[0].crop, None);

    // Odd excess: 7 wide → 4 wide leaves 3 columns; floor offset 1 drops one from
    // the left and two from the right (the extra pixel goes to the right edge).
    let odd = vec![solid_image(7, 4, [0; 4]), solid_image(7, 4, [0; 4])];
    let plan = plan_layout(&odd, &cell_options(2, 3, 3)).expect("odd");
    assert_eq!(
        plan.placed_images[0].crop,
        Some(SourceCrop { x: 1, y: 0, width: 4, height: 4 })
    );
}

#[test]
fn cell_mode_clamps_columns_and_rejects_zero() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];

    let plan = plan_layout(&images, &cell_options(5, 10, 10)).expect("clamped");
    assert_eq!(plan.placed_images.len(), 2);
    // Requested 5 columns clamp to 2 (image count), so one row: 2*10 + one 8px gap.
    assert_eq!((plan.width, plan.height), (28, 10));
    assert_eq!(plan.placed_images[1].x, 18);
    assert_eq!(plan.placed_images[1].y, 0);

    assert!(matches!(
        plan_layout(&images, &cell_options(0, 10, 10)).unwrap_err(),
        ComposeError::InvalidGridColumns
    ));
}

#[test]
fn rejects_cell_size_outside_grid() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];
    let options = ComposeOptions {
        layout: LayoutKind::VerticalStack,
        cell_size: Some(CellSize { width: 4, height: 4 }),
        ..ComposeOptions::default()
    };
    assert!(matches!(
        plan_layout(&images, &options).unwrap_err(),
        ComposeError::CellSizeRequiresGrid
    ));
}

#[test]
fn rejects_zero_cell_dimensions() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];
    let options = cell_options(2, 0, 10);
    assert!(matches!(
        plan_layout(&images, &options).unwrap_err(),
        ComposeError::InvalidCellSize { width: 0, height: 10 }
    ));
}

#[test]
fn rejects_cell_size_conflicts() {
    let images = vec![solid_image(4, 4, [0; 4]), solid_image(4, 4, [0; 4])];

    let weights = ComposeOptions {
        column_weights: vec![1, 1],
        ..cell_options(2, 10, 10)
    };
    assert!(matches!(
        plan_layout(&images, &weights).unwrap_err(),
        ComposeError::CellSizeConflictsWithColumnWeights
    ));

    let resize = ComposeOptions {
        resize_mode: ResizeMode::FitToSameWidth,
        ..cell_options(2, 10, 10)
    };
    assert!(matches!(
        plan_layout(&images, &resize).unwrap_err(),
        ComposeError::CellSizeConflictsWithResizeMode
    ));
}
```

- [ ] **Step 3: Run the tests and confirm they fail**

```bash
cargo test --test layout_tests cell
```

Expected: FAIL — `CellSizeRequiresGrid` never returned; cells still auto-sized. (If the tests do not compile because a variant is missing, that is the expected red state too.)

- [ ] **Step 4: Implement validation, cell geometry, and `cover_crop`**

In `plan_layout`, after the existing `ColumnWeightsRequireGrid` check, add in this order:

```rust
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
```

Dispatch the grid arm as:

```rust
LayoutKind::Grid { columns } => match options.cell_size {
    Some(cell) => plan_grid_with_cells(images, options, columns, cell),
    None => plan_grid(&resolve_sizes(images, options.resize_mode)?, options, columns),
},
```

Cell mode must not call `resolve_sizes`.

`plan_grid_with_cells`: reject `columns == 0` with `InvalidGridColumns`; compute `columns = requested.min(images.len()).max(1)` and `rows = images.len().div_ceil(columns)`; build `column_widths = vec![cell.width; columns]` and `row_heights = vec![cell.height; rows]`; compute canvas size with `padded_dimension` over `sum_dimensions(...)` exactly as `plan_grid` does; compute offsets with existing `track_offsets`; place each image with `width/height` from the cell and `crop = cover_crop(image.width, image.height, cell.width, cell.height)?`.

`cover_crop` (guard `0` on any dimension → `DimensionOverflow`; use `u64` products):

- `sw·ch == sh·cw` → `Ok(None)`.
- `sw·ch > sh·cw` → `height = sh`, `width = scaled_dimension(sh, cw, ch)?.min(sw).max(1)`, `x = (sw - width) / 2`, `y = 0`.
- otherwise → `width = sw`, `height = scaled_dimension(sw, ch, cw)?.min(sh).max(1)`, `x = 0`, `y = (sh - height) / 2`.

- [ ] **Step 5: Run the tests and the whole suite**

```bash
cargo test --test layout_tests
cargo test
```

Expected: PASS, including all pre-existing layout/render/IO tests.

- [ ] **Step 6: Commit**

```bash
git add crates/stitch-core
git commit -m "feat(core): add grid cell sizing with planned source crops"
```

---

### Task 2: Crop-aware rendering

**Files:**
- Modify: `crates/stitch-core/src/render.rs:113-158` (`prepare_pixels`)
- Modify: `crates/stitch-core/tests/helpers.rs`
- Modify: `crates/stitch-core/tests/render_tests.rs`

**Interfaces:**
- Consumes: `PlacedImage { crop: Option<SourceCrop>, … }` and `SourceCrop` from Task 1.
- Produces: `fn prepare_pixels(image: &InputImage, placed: &PlacedImage) -> Result<Vec<u8>, ComposeError>` (private), plus `fn premultiply(pixels: &mut [u8])` and `fn unpremultiply(pixels: &mut [u8])` (private) shared by both paths.

- [ ] **Step 1: Add the striped fixture**

Append to `crates/stitch-core/tests/helpers.rs`:

```rust
pub fn striped_image(width: u32, height: u32, colors: &[[u8; 4]]) -> InputImage {
    let mut pixels = Vec::new();
    for _ in 0..height {
        for x in 0..width {
            pixels.extend_from_slice(&colors[(x as usize) % colors.len()]);
        }
    }
    InputImage::new(width, height, pixels)
}
```

- [ ] **Step 2: Write the failing render tests**

Append to `crates/stitch-core/tests/render_tests.rs` (imports: add `CellSize, SourceCrop`):

```rust
fn cell_grid(columns: u32, width: u32, height: u32) -> ComposeOptions {
    ComposeOptions {
        layout: LayoutKind::Grid { columns },
        cell_size: Some(CellSize { width, height }),
        ..ComposeOptions::default()
    }
}

#[test]
fn crops_odd_excess_from_the_right_edge() {
    let stripes = [
        [255, 0, 0, 255],
        [0, 255, 0, 255],
        [0, 0, 255, 255],
    ];
    let images = vec![
        striped_image(6, 2, &stripes),
        striped_image(6, 2, &stripes),
    ];

    let composed = compose(&images, &cell_grid(2, 3, 2)).expect("compose");

    assert_eq!((composed.width, composed.height), (6, 2));
    // Source columns 0..5 are R G B R G B; crop x=1, width=3 keeps G B R.
    assert_eq!(pixel_at(&composed.pixels, 6, 0, 0), [0, 255, 0, 255]);
    assert_eq!(pixel_at(&composed.pixels, 6, 1, 0), [0, 0, 255, 255]);
    assert_eq!(pixel_at(&composed.pixels, 6, 2, 0), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&composed.pixels, 6, 3, 0), [0, 255, 0, 255]);
}

#[test]
fn fills_tiles_from_wide_and_tall_sources() {
    let images = vec![
        solid_image(8, 2, [255, 0, 0, 255]),
        solid_image(2, 8, [0, 0, 255, 255]),
    ];

    let composed = compose(&images, &cell_grid(2, 4, 4)).expect("compose");

    for y in 0..4 {
        for x in 0..4 {
            assert_eq!(pixel_at(&composed.pixels, 8, x, y), [255, 0, 0, 255]);
            assert_eq!(pixel_at(&composed.pixels, 8, x + 4, y), [0, 0, 255, 255]);
        }
    }
}

#[test]
fn fills_a_tile_from_an_extreme_aspect_source() {
    let images = vec![
        solid_image(4000, 10, [10, 20, 30, 255]),
        solid_image(10, 4000, [40, 50, 60, 255]),
    ];

    let composed = compose(&images, &cell_grid(2, 32, 32)).expect("compose");

    assert_eq!(pixel_at(&composed.pixels, 64, 0, 0), [10, 20, 30, 255]);
    assert_eq!(pixel_at(&composed.pixels, 64, 63, 31), [40, 50, 60, 255]);
}

#[test]
fn upscales_small_sources_to_fill_the_tile() {
    let images = vec![
        solid_image(2, 1, [255, 0, 0, 255]),
        solid_image(1, 2, [0, 255, 0, 255]),
    ];

    let composed = compose(&images, &cell_grid(2, 4, 4)).expect("compose");

    for y in 0..4 {
        for x in 0..4 {
            assert_eq!(pixel_at(&composed.pixels, 8, x, y), [255, 0, 0, 255]);
            assert_eq!(pixel_at(&composed.pixels, 8, x + 4, y), [0, 255, 0, 255]);
        }
    }
}

#[test]
fn frame_and_radius_follow_the_cell_boundary() {
    let style = PaneStyle {
        frame: Some(Frame {
            width: 2,
            color: RgbaColor::WHITE,
        }),
        corner_radius: 3,
    };
    let images = vec![
        solid_image(20, 10, [255, 0, 0, 255]),
        solid_image(20, 10, [255, 0, 0, 255]),
    ];
    let mut options = cell_grid(2, 10, 10);
    options.pane_style = style;
    options.background = RgbaColor::TRANSPARENT;

    let composed = compose(&images, &options).expect("compose");

    assert_eq!(pixel_at(&composed.pixels, 20, 0, 0), [0, 0, 0, 0]);
    assert_eq!(pixel_at(&composed.pixels, 20, 2, 2), [255, 255, 255, 255]);
    assert_eq!(pixel_at(&composed.pixels, 20, 5, 5), [255, 0, 0, 255]);
}

#[test]
fn crop_preserves_alpha_without_tinting() {
    let pixels = vec![
        255, 0, 0, 128, // kept by the crop
        0, 0, 255, 255, // kept by the crop
        0, 0, 255, 255, // dropped from the right
    ];
    let source = stitch_core::InputImage::new(3, 1, pixels);
    let images = vec![source.clone(), source];
    let mut options = cell_grid(2, 2, 1);
    // Composite over transparency so the alpha pixel is observed, not blended into white.
    options.background = RgbaColor::TRANSPARENT;

    let composed = compose(&images, &options).expect("compose");

    // Crop keeps source columns 0..1 (excess 1, offset 0) with no resize.
    assert_eq!(pixel_at(&composed.pixels, 4, 0, 0), [255, 0, 0, 128]);
}
```

- [ ] **Step 3: Run the tests and confirm they fail**

```bash
cargo test --test render_tests crop
cargo test --test render_tests fills_
```

Expected: FAIL — tiles are stretched, not cropped (`crops_odd_excess_from_the_right_edge` shows blended/red first pixel), and `crop_preserves_alpha_without_tinting` sees a full-width stretch.

- [ ] **Step 4: Implement the crop path**

Extract the premultiply and unpremultiply loops into `premultiply(pixels: &mut [u8])` and `unpremultiply(pixels: &mut [u8])` (same arithmetic as today). Change the call in `render` to `prepare_pixels(source, placed)?`.

In `prepare_pixels`:

- `None` crop: current behavior (early return when `image` already equals `placed` size; otherwise premultiply → resize → unpremultiply).
- `Some(crop)`: validate `crop.x + crop.width <= image.width` and `crop.y + crop.height <= image.height` with checked math (`CoordinateOverflow`), and `crop.width/height > 0`; copy each row range `[crop.x*4 .. (crop.x+crop.width)*4]` into one buffer. If the crop already equals `placed.width × placed.height`, return that buffer untouched. Otherwise premultiply, resize with `FilterType::Triangle`, unpremultiply.

- [ ] **Step 5: Run the tests and the whole suite**

```bash
cargo test --test render_tests
cargo test
cargo clippy --all-targets -D warnings
```

Expected: PASS, no warnings.

- [ ] **Step 6: Commit**

```bash
git add crates/stitch-core
git commit -m "feat(core): fill cell tiles by center-cropping sources"
```

---

### Task 3: CLI flag, parsing, and pre-checks

**Files:**
- Modify: `crates/stitch-cli/src/lib.rs`

**Interfaces:**
- Consumes: `CellSize`, `ComposeOptions.cell_size`, and Task 1 errors.
- Produces:
  - `CombineArgs.cell_size: Option<CellSize>`
  - `pub fn parse_cell_size(value: &str) -> Result<CellSize, String>`
  - CLI messages exactly: `--cell-size is only supported by the grid layout`, `--cell-size must be at least 1x1`, `--cell-size cannot be combined with --column-weights`, `--cell-size cannot be combined with --resize-mode`.

- [ ] **Step 1: Add the flag and parser**

Add to `CombineArgs` after `column_weights`:

```rust
#[arg(
    long,
    value_name = "WIDTHxHEIGHT",
    value_parser = parse_cell_size,
    help = "Fixed tile size for the grid layout, center-cropping each image (e.g. 320x320)"
)]
pub cell_size: Option<CellSize>,
```

Implement:

```rust
pub fn parse_cell_size(value: &str) -> Result<CellSize, String> {
    let invalid = || {
        format!(
            "invalid cell size \"{value}\"; expected WIDTHxHEIGHT with values of at least 1, e.g. 320x320"
        )
    };
    let (width, height) = value
        .split_once(['x', 'X'])
        .ok_or_else(invalid)?;
    if height.contains(['x', 'X']) {
        return Err(invalid());
    }
    let width: u32 = width.parse().map_err(|_| invalid())?;
    let height: u32 = height.parse().map_err(|_| invalid())?;
    if width == 0 || height == 0 {
        return Err(invalid());
    }
    Ok(CellSize { width, height })
}
```

Add `cell_size: args.cell_size` to the `ComposeOptions` literal. Add `cell_size: None` to every `CombineArgs` literal in the tests.

- [ ] **Step 2: Write the failing CLI tests**

Append inside the existing `mod tests`:

```rust
#[test]
fn parses_cell_size_flag() {
    let cli = Cli::parse_from([
        "stitch-me-up", "combine", "--layout", "grid", "--columns", "3",
        "--cell-size", "320x320", "--output", "out.png", "a.png", "b.png",
    ]);
    match cli.command {
        Commands::Combine(args) => {
            assert_eq!(args.cell_size, Some(CellSize { width: 320, height: 320 }));
        }
    }
}

#[test]
fn accepts_uppercase_cell_separator() {
    assert_eq!(
        parse_cell_size("320X240"),
        Ok(CellSize { width: 320, height: 240 })
    );
}

#[test]
fn rejects_malformed_cell_sizes() {
    for value in ["320", "320x", "x320", "0x10", "10x0", "a x b", "320x10x2"] {
        let error = parse_cell_size(value).unwrap_err();
        assert!(error.contains("expected WIDTHxHEIGHT"), "{value}: {error}");
    }
}

fn cell_args(cell_size: CellSize) -> CombineArgs {
    CombineArgs {
        inputs: vec!["a.png".into(), "b.png".into()],
        output: "out.png".into(),
        layout: LayoutArg::Grid,
        format: OutputFormatArg::Png,
        quality: None,
        spacing: 0,
        spacing_x: None,
        spacing_y: None,
        padding: 0,
        padding_top: None,
        padding_right: None,
        padding_bottom: None,
        padding_left: None,
        background: RgbaColor::WHITE,
        alignment: AlignmentArg::Center,
        resize_mode: ResizeModeArg::None,
        columns: 2,
        column_weights: Vec::new(),
        pane_styles: Vec::new(),
        frame_width: 0,
        frame_color: RgbaColor::WHITE,
        corner_radius: 0,
        divider_width: 0,
        divider_color: RgbaColor::WHITE,
        cell_size: Some(cell_size),
    }
}

#[test]
fn rejects_cell_size_outside_grid_before_io() {
    let mut args = cell_args(CellSize { width: 10, height: 10 });
    args.layout = LayoutArg::Vertical;
    assert_eq!(
        combine(args).unwrap_err().to_string(),
        "--cell-size is only supported by the grid layout"
    );
}

#[test]
fn rejects_cell_size_conflicts_before_io() {
    let mut args = cell_args(CellSize { width: 10, height: 10 });
    args.column_weights = vec![1, 1];
    assert_eq!(
        combine(args).unwrap_err().to_string(),
        "--cell-size cannot be combined with --column-weights"
    );

    let mut args = cell_args(CellSize { width: 10, height: 10 });
    args.resize_mode = ResizeModeArg::FitToSameHeight;
    assert_eq!(
        combine(args).unwrap_err().to_string(),
        "--cell-size cannot be combined with --resize-mode"
    );
}

#[test]
fn rejects_zero_cell_size_before_io() {
    let args = cell_args(CellSize { width: 0, height: 10 });
    assert_eq!(
        combine(args).unwrap_err().to_string(),
        "--cell-size must be at least 1x1"
    );
}
```

- [ ] **Step 3: Run and confirm failure**

```bash
cargo test -p stitch-me-up cell
```

Expected: FAIL to compile (field/parser missing), then the behavior tests fail.

- [ ] **Step 4: Implement the pre-checks**

In `combine`, after the existing grid-column check:

```rust
if let Some(cell) = args.cell_size {
    if !matches!(args.layout, LayoutArg::Grid) {
        bail!("--cell-size is only supported by the grid layout");
    }
    if cell.width == 0 || cell.height == 0 {
        bail!("--cell-size must be at least 1x1");
    }
    if !args.column_weights.is_empty() {
        bail!("--cell-size cannot be combined with --column-weights");
    }
    if !matches!(args.resize_mode, ResizeModeArg::None) {
        bail!("--cell-size cannot be combined with --resize-mode");
    }
}
```

The checks run before any path is loaded, so the tests never touch the filesystem.

- [ ] **Step 5: Run and verify**

```bash
cargo test -p stitch-me-up
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/stitch-cli
git commit -m "feat(cli): add --cell-size with grid pre-checks"
```

---

### Task 4: WASM adapter

**Files:**
- Modify: `crates/stitch-wasm/src/lib.rs`

**Interfaces:**
- Consumes: `CellSize`, `ComposeOptions.cell_size`.
- Produces: `Composer.set_cell_size(width: u32, height: u32)`, `Composer.clear_cell_size()`, private `apply_cell_size` / `apply_clear_cell_size`.

- [ ] **Step 1: Write the failing native test**

Append inside the existing native `mod tests`:

```rust
#[test]
fn stores_and_clears_cell_size() {
    let mut composer = Composer::default();
    assert_eq!(composer.options.cell_size, None);

    composer.apply_cell_size(320, 400);
    assert_eq!(
        composer.options.cell_size,
        Some(CellSize { width: 320, height: 400 })
    );

    composer.apply_clear_cell_size();
    assert_eq!(composer.options.cell_size, None);
}
```

Add `CellSize` to the test module's `use stitch_core::{…}` list.

- [ ] **Step 2: Run and confirm failure**

```bash
cargo test -p stitch-me-up-wasm stores_and_clears_cell_size
```

Expected: FAIL to compile — `apply_cell_size` missing.

- [ ] **Step 3: Implement the setters**

In the shared `#[cfg(any(target_arch = "wasm32", test))] impl Composer`:

```rust
fn apply_cell_size(&mut self, width: u32, height: u32) {
    self.options.cell_size = Some(CellSize { width, height });
}

fn apply_clear_cell_size(&mut self) {
    self.options.cell_size = None;
}
```

In the `#[cfg(target_arch = "wasm32")] impl Composer`, add:

```rust
/// Fixed tile size for the grid layout, center-cropping each image.
/// Invalid values (grid-only, non-zero) fail at compose time.
pub fn set_cell_size(&mut self, width: u32, height: u32) {
    self.apply_cell_size(width, height);
}

/// Restore automatic grid sizing.
pub fn clear_cell_size(&mut self) {
    self.apply_clear_cell_size();
}
```

Import `CellSize` in the top-level `use stitch_core::{…}`.

- [ ] **Step 4: Run and verify**

```bash
cargo test -p stitch-me-up-wasm
cargo clippy --all-targets -D warnings
```

Expected: PASS, no warnings (the native-only `allow(dead_code)` pattern already covers unused wasm methods).

- [ ] **Step 5: Commit**

```bash
git add crates/stitch-wasm
git commit -m "feat(wasm): add set_cell_size and clear_cell_size"
```

---

### Task 5: Web demo tile presets

**Files:**
- Modify: `examples/web/lib.js`
- Modify: `examples/web/tests/lib.test.js`
- Modify: `examples/web/index.html`
- Modify: `examples/web/app.js`

**Interfaces:**
- Produces: `export const CELL_PRESETS = { off: null, square: { width: 320, height: 320 }, portrait: { width: 320, height: 400 } }` and `export function cellPresetToSize(preset) → { width, height } | null`.

- [ ] **Step 1: Write the failing node test**

In `examples/web/tests/lib.test.js`, add `cellPresetToSize` to the import list and append:

```js
test("cellPresetToSize maps presets and falls back to off", () => {
  assert.deepEqual(cellPresetToSize("square"), { width: 320, height: 320 });
  assert.deepEqual(cellPresetToSize("portrait"), { width: 320, height: 400 });
  assert.equal(cellPresetToSize("off"), null);
  assert.equal(cellPresetToSize("nonsense"), null);
});
```

- [ ] **Step 2: Run and confirm failure**

```bash
node --test examples/web/tests/lib.test.js
```

Expected: FAIL — `cellPresetToSize` is not exported.

- [ ] **Step 3: Implement the preset mapping**

Append to `examples/web/lib.js`:

```js
export const CELL_PRESETS = {
  off: null,
  square: { width: 320, height: 320 },
  portrait: { width: 320, height: 400 },
};

export function cellPresetToSize(preset) {
  return CELL_PRESETS[preset] ?? null;
}
```

- [ ] **Step 4: Add the control and wiring**

In `examples/web/index.html`, after the Weights label in the Layout panel:

```html
<label>
  Tile shape
  <select id="cell-preset">
    <option value="off" selected>off</option>
    <option value="square">square 320×320</option>
    <option value="portrait">4:5 320×400</option>
  </select>
</label>
```

In `examples/web/app.js`:

- Import `cellPresetToSize`.
- In `compose()` after `composer.set_layout(layout, columns)`:

```js
const presetSize = layout === "grid" ? cellPresetToSize($("cell-preset").value) : null;
if (presetSize) composer.set_cell_size(presetSize.width, presetSize.height);
```

- Apply column weights only when `layout === "grid" && !presetSize`.
- Extend `setLayoutControls()`:

```js
const presetSize = cellPresetToSize($("cell-preset").value);
$("cell-preset").disabled = !isGrid;
if (isGrid && presetSize) $("weights").value = "";
$("weights").disabled = !isGrid || presetSize !== null;
```

- Register `$("cell-preset").addEventListener("change", setLayoutControls);` next to the layout listener; the existing generic change listener already schedules a recompose.

- [ ] **Step 5: Verify**

```bash
node --test examples/web/tests/lib.test.js
```

Expected: PASS. A manual browser check (build the wasm package, serve `examples/web`, confirm Square and 4:5 change the preview and disable Weights) is useful but not part of the gate.

- [ ] **Step 6: Commit**

```bash
git add examples/web
git commit -m "feat(web): add square and 4:5 tile presets"
```

---

### Task 6: Documentation and roadmap

**Files:**
- Modify: `README.md`
- Modify: `docs/api.md`, `docs/cli.md`, `docs/layouts.md`, `docs/wasm.md`, `docs/README.md`
- Modify: `examples/web/README.md`
- Modify: `ROADMAP.md`, `CHANGELOG.md`

**Interfaces:**
- Consumes: everything above; no new code.

- [ ] **Step 1: Update the API reference**

In `docs/api.md`: document `CellSize`, `SourceCrop`, `ComposeOptions.cell_size`, `PlacedImage.crop`, the crop rule (binding axis, rounding, floor offset dropping the odd pixel right/bottom, upscaling allowed), the four errors, and the fact that cell mode bypasses `resolve_sizes` and ignores `alignment`.

- [ ] **Step 2: Update the CLI and layout docs**

`docs/cli.md`: flag row with default (none), accepted `WIDTHxHEIGHT` case-insensitive separator, the four pre-check messages, and the 976×648 example. `docs/layouts.md`: a "Uniform tiles" grid subsection with a one-line diagram, the acceptance arithmetic, and notes that partial rows are background and alignment has no effect.

- [ ] **Step 3: Update README, WASM docs, demo README**

`README.md`: feature bullet and one example. `docs/wasm.md`: `set_cell_size` / `clear_cell_size` next to `set_column_weights`, plus the preset mapping. `docs/README.md`: list this spec and plan in the design-notes line. `examples/web/README.md`: mention the tile-shape presets and that a preset disables Weights.

- [ ] **Step 4: Update roadmap and changelog**

`ROADMAP.md`: add a `0.7 — Uniform tiles` section with the checked items and a decision log (grid-only because Fill needs a rectangle; explicit dimensions over an aspect ratio; crop lives in the plan). `CHANGELOG.md`: add an `## Unreleased` section with the new option across core/CLI/WASM/demo, the four errors, and the behavior note that cell-mode grids now crop.

- [ ] **Step 5: Verify the docs describe the code**

```bash
cargo test
node --test examples/web/tests/lib.test.js
cargo fmt --check
cargo clippy --all-targets -D warnings
```

Expected: all green. Re-read the docs for the exact flag names and numbers (976×648, 320×320, 320×400).

- [ ] **Step 6: Commit**

```bash
git add README.md docs ROADMAP.md CHANGELOG.md examples/web/README.md
git commit -m "docs: document grid cell sizing"
```
