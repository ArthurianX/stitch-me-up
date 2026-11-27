# Uniform grid tiles: `--cell-size` (0.7)

## Intent

`stitch-me-up` can already stitch images, but every pane keeps its own aspect ratio.
A grid of phone photos therefore comes out ragged: row heights vary, cells hold
background slack, and there is no way to ask for a contact sheet. This adds one
grid-only option, `cell_size`, that gives every tile an explicit pixel size and
center-crops the source to fill it. The output geometry becomes predictable, which is
the point: a caller should be able to compute the canvas size before running anything.

The browser demo exposes the same behavior through two presets, Square (320×320) and
Portrait 4:5 (320×400).

## Goals

- Grid-only `CellSize { width, height }`: every placed tile is exactly that size.
- Center-crop (`cover`) to fill; no letterboxing, no distortion.
- One resample per image; deterministic integer math; no new dependencies.
- Frames, corner radius, alpha compositing, spacing, padding and background behave
  exactly as they do now, applied at the final tile boundary.
- CLI flag, WASM setters, web-demo presets, and documentation in the same change.
- Acceptance case: six photos, three columns, 320×320 tiles, 8 px spacing, zero
  padding → **976×648** canvas with six 320×320 tiles.

## Non-goals

- A general `ResizeMode::Fill`. Fill only has a clear meaning once a rectangle exists;
  grids with an explicit cell are the first such rectangle. Other layouts keep deriving
  size from content.
- Per-pane cell sizes, non-center anchors, focal points, or manual crop offsets.
- Cropping when no target rectangle is given.
- Antialiasing, captions/text, new formats, new dependencies.
- A custom numeric cell size in the browser; presets only for 0.7 (revisit on demand).
- Release mechanics (tagging/publishing). This spec lands the code, docs, and roadmap
  entry for 0.7; cutting the release stays a separate step.

## Constraints

- The plan is the source of truth for geometry; rendering only executes it.
- Decoration stays an overlay inside the placed rectangle (existing rule).
- `CanvasPlan`/`PlacedImage` are public; 0.x allows the additive breaking change
  (a new field on `PlacedImage` and on `ComposeOptions`).
- MSRV and the dependency graph must not change.

## Approaches

**A — chosen: cell size in the plan, crop rect per placed image.** `ComposeOptions`
gains `cell_size: Option<CellSize>`. `plan_grid` sizes every track to the cell and
computes a `SourceCrop` (source-space rectangle) per image; `PlacedImage` carries it.
`render` crops the source, then resizes the crop to the placed rect. Plan tests can
assert the crop; render stays a mechanical executor; crop bounds the work to the part
of the source that is actually used.

**B — derive the crop in `render`.** Keep the plan unchanged and recompute the crop
from source + placed size + `cell_size`. Smallest diff, but the crop rule disappears
from the plan, cannot be asserted at plan level, and render starts owning layout
semantics.

**C — resize-to-cover, then crop in `render`.** Exact aspect at the cell, but it
resamples pixels that are immediately thrown away, can allocate a huge intermediate
for extreme aspect ratios (e.g. a 4000×10 source into a 320×320 cell), and pushes the
odd-excess rule into scaled space instead of source pixels.

A is chosen because the crop is geometry, the plan already owns sizing, and it keeps
a single resample of the pixels that survive.

## The design

### Core types

```rust
pub struct CellSize {
    pub width: u32,
    pub height: u32,
}

/// A source-space rectangle consumed by rendering when a pane is filled.
pub struct SourceCrop {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct PlacedImage {
    pub index: usize,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    /// `Some` when the source must be cropped before it is resized to this rect.
    pub crop: Option<SourceCrop>,
}

pub struct ComposeOptions {
    // existing fields …
    pub cell_size: Option<CellSize>,
}
```

`CellSize` and `SourceCrop` are re-exported from `stitch_core`. `cell_size` stays flat
on `ComposeOptions` rather than inside `LayoutKind::Grid`: the WASM adapter lets
callers set layout and cell size in any order, and a flat option keeps
`set_cell_size` independent of `set_layout`.

### Validation (in `plan_layout`)

| Condition | Error |
| --- | --- |
| `cell_size` with a non-grid layout | `CellSizeRequiresGrid` — "cell size is only supported by the grid layout" |
| `cell_size.width == 0 || height == 0` | `InvalidCellSize { width, height }` — "cell size {w}x{h} must be at least 1x1" |
| `cell_size` with non-empty `column_weights` | `CellSizeConflictsWithColumnWeights` — "cell size cannot be combined with column weights" |
| `cell_size` with `resize_mode != None` | `CellSizeConflictsWithResizeMode` — "cell size cannot be combined with resize_mode" |

All checks happen before any plan is produced, alongside the existing divider and
column-weight checks. `column_weights` and `resize_mode` are error cases rather than
silently ignored: both express sizing intent that the cell size overrides, and silent
precedence would make output surprising.

### Grid geometry with a cell size

- When `cell_size` is set, `plan_grid` does not call `resolve_sizes` at all: track
  and tile sizes come straight from the cell, so there is no unused resolve path and
  no behavior change for the non-cell path.
- `columns = min(requested_columns, image_count)`, `rows = ceil(image_count / columns)`
  — unchanged from today, including the zero-column error.
- Every used column track is exactly `cell.width`; every row track is exactly
  `cell.height`. Existing `track_offsets` places them with `spacing.x` / `spacing.y`,
  and canvas size is `padded_dimension(cols * w + gaps, padding…)` (same for height).
- Each `PlacedImage` gets `width = cell.width`, `height = cell.height` at its track
  offset.
- A partial last row keeps a full row height; missing cells are background, as today.
- `alignment` has no effect (tracks exactly equal the tiles, so there is no slack to
  align within). This is documented, not an error.
- Sources smaller than the cell are upscaled, consistent with the existing
  `FitToSameWidth` behavior.

### Crop rule

For a source `sw × sh` and a tile `cw × ch`, using `u64` products and the existing
integer rounding `(a * b + d / 2) / d`:

1. **Exact dims or equal aspect** (`sw == cw && sh == ch`, or `sw·ch == sh·cw`):
   `crop = None`; the existing prepare path resizes (or copies) unchanged.
2. **Source wider** (`sw·ch > sh·cw`): keep the full height.
   `crop.height = sh`, `crop.width = max(1, round(sh·cw / ch))`,
   `crop.x = (sw - crop.width) / 2`, `crop.y = 0`.
3. **Source taller**: keep the full width.
   `crop.width = sw`, `crop.height = max(1, round(sw·ch / cw))`,
   `crop.y = (sh - crop.height) / 2`, `crop.x = 0`.

Offsets use integer floor division, so when the excess is odd the extra pixel is
discarded from the **right/bottom** edge (left/top margin is the smaller one). This is
the rule a striped-image test pins down. Rounding can never produce a zero-sized crop
(the `max(1, …)` clamp) and every rect stays inside the source; overflow produces
`DimensionOverflow`.

### Rendering

`prepare_pixels` takes the placed rect plus its `crop`:

- `Some(crop)`: slice the source to `crop`, premultiply that slice, resize it to
  `placed.width × placed.height` with `FilterType::Triangle`, unpremultiply.
- `None`: today's path unchanged (early return on equal dims, premultiply, resize,
  unpremultiply).

Cropping happens before premultiplication and resizing, so only needed pixels are
processed and the alpha handling stays identical. `blit_image` is untouched: frames
and corner masks still run after pixel preparation and therefore follow the final tile
boundary. The divider is grid-invalid today and stays untouched.

## CLI

```
--cell-size 320x320
```

- `value_name = "WIDTHxHEIGHT"`, parsed by `parse_cell_size` into `CellSize`; accepts
  `x` or `X`, rejects whitespace, missing/extra separators, and zero dimensions with
  `invalid cell size "…"; expected WIDTHxHEIGHT with values of at least 1, e.g. 320x320`.
- Grid-only. `combine` pre-checks all four core rejections before loading any image,
  so the user sees a CLI message instead of `failed to compose images: …`. The core
  errors remain the definitive validation. Exact messages:

  | Condition | CLI message |
  | --- | --- |
  | cell size on a non-grid layout | `--cell-size is only supported by the grid layout` |
  | zero width or height | `--cell-size must be at least 1x1` |
  | cell size with `--column-weights` | `--cell-size cannot be combined with --column-weights` |
  | cell size with a non-default `--resize-mode` | `--cell-size cannot be combined with --resize-mode` |
- Example for `docs/cli.md`:

```bash
stitch-me-up combine --layout grid --columns 3 --cell-size 320x320 \
  --spacing 8 --padding 0 --output sheet.png 1.png 2.png 3.png 4.png 5.png 6.png
# → 976x648
```

## WASM adapter

- `Composer.set_cell_size(width: u32, height: u32)` — stores
  `Some(CellSize { width, height })`; invalid values surface at compose time, matching
  `set_layout("grid", 0)`.
- `Composer.clear_cell_size()` — restores auto sizing.
- Documented in `docs/wasm.md` next to `set_column_weights`.

## Web demo

- A grid-only "Tile shape" control with **Off**, **Square (320×320)**, and
  **Portrait 4:5 (320×400)**.
- Selecting a preset calls `set_cell_size`; Off calls `clear_cell_size`. Changing it
  triggers the existing debounced re-compose.
- The control is hidden or disabled for non-grid layouts.
- Because cell size and column weights conflict, choosing a preset clears the weights
  field and disables it (with a short hint); leaving preset mode re-enables it.
- Pure logic lives in `examples/web/lib.js` as `cellPresetToSize(preset)` returning
  `null` or `{ width, height }`, covered by `node --test`. DOM wiring stays in
  `app.js`.

## Documentation and metadata

- `README.md`: feature bullet + one short example.
- `docs/api.md`: new types, `ComposeOptions.cell_size`, `PlacedImage.crop`, the crop
  rule, and the four new error variants.
- `docs/cli.md`: `--cell-size`, conflicts, the 976×648 example.
- `docs/layouts.md`: grid subsection with the tile diagram and the acceptance
  arithmetic; note that alignment has no effect in cell mode.
- `docs/wasm.md`, `examples/web/README.md`: new setters and presets.
- `ROADMAP.md`: 0.7 "Uniform tiles" entry with a short decision log (why grid-only,
  why explicit dimensions, why the crop lives in the plan).
- `CHANGELOG.md`: Unreleased/0.7.0 "Added" entries.

## Tests

Core layout (`crates/stitch-core/tests/layout_tests.rs`):

- Six images, three columns, 320×320, spacing 8, padding 0 → canvas 976×648; six
  placed rects, each 320×320 at the expected offsets.
- Padding and per-axis spacing arithmetic in cell mode, including a partial last row.
- Crop rects: wide source crops horizontally, tall source crops vertically, equal
  aspect yields `None`, exact dims yield `None`, odd excess floors the offset.
- Validation: zero dimension, non-grid layout, weights conflict, resize-mode conflict.
- Column clamping and zero-column error still hold with a cell size.

Core render (`crates/stitch-core/tests/render_tests.rs`):

- A striped source with an odd excess pixel proves the right/bottom edge is the one
  dropped (pixel-level assertion, not just geometry).
- A wide and a tall source fill the tile with no background showing inside it.
- Frame and corner radius follow the 320×320 tile boundary in cell mode.
- Transparency: cropped edges composite correctly (premultiplied resize path).
- Upscaling a smaller source fills the tile.

CLI (`crates/stitch-cli/src/lib.rs` tests):

- `--cell-size 320x320` parses; `320X240` parses; `320`, `320x`, `0x10`, `a x b`
  fail with the documented message.
- `CombineArgs` default is `None`; the flag reaches `ComposeOptions.cell_size`.
- All four pre-checks reject before I/O with the exact messages above (asserted).

Web (`examples/web/tests/lib.test.js`):

- `cellPresetToSize` maps off/square/portrait/unknown correctly.

Full gate: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`,
`cargo test`, `node --test examples/web/tests/*.test.js`, plus a `wasm-pack` build if
the adapter changes. Playwright coverage is optional; the preset logic is covered by
node tests.

## Acceptance

```bash
stitch-me-up combine --layout grid --columns 3 --cell-size 320x320 \
  --spacing 8 --padding 0 --background "#000000" \
  --output sheet.png 1.png 2.png 3.png 4.png 5.png 6.png
```

produces a 976×648 PNG: `3·320 + 2·8 = 976` wide, `2·320 + 1·8 = 648` tall, six
320×320 tiles with 8 px gaps and no padding, each filled edge to edge (center-cropped,
upscaled if the source is smaller).

## Left to the builder

- Where the crop helper and `CellSize` parsing live inside the existing modules.
- Exact fixture colors/patterns for the striped and transparency tests.
- Wording of docs and changelog entries.
- Whether to add a Playwright case for the preset select.
- Implementation order and commit slicing.
