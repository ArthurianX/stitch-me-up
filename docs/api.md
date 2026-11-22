# Core API Reference (`stitch-core`)

Everything below is re-exported at the crate root (`use stitch_core::…`). The core
operates on decoded RGBA8 buffers: 4 bytes per pixel, row-major, `width * height * 4`
bytes. All arithmetic is integer; no floats are involved.

## Types

### `InputImage`

```rust
pub struct InputImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,   // RGBA8, row-major
}

impl InputImage {
    pub fn expected_len(&self) -> Option<usize>;  // width * height * 4, overflow-checked
}
```

### `ComposedImage`

```rust
pub struct ComposedImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,   // RGBA8, row-major
}
```

### `CanvasPlan` and `PlacedImage`

```rust
pub struct CanvasPlan {
    pub width: u32,
    pub height: u32,
    pub placed_images: Vec<PlacedImage>,
}

pub struct PlacedImage {
    pub index: usize,      // position in the input slice
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
```

A plan is pure geometry: `(x, y)` is the top-left of the placed rectangle in canvas
coordinates, `width`/`height` the (possibly scaled) pane size. Rendering never changes
the plan.

### `Insets`

```rust
pub struct Insets {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Insets {
    pub const fn all(value: u32) -> Self;
}
```

`Default` is `Insets::all(0)`. Padding surrounds the whole canvas, not each pane.

### `RgbaColor`

```rust
pub struct RgbaColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RgbaColor {
    pub const TRANSPARENT: Self;   // (0, 0, 0, 0)
    pub const WHITE: Self;         // (255, 255, 255, 255)
    pub const fn to_rgba(self) -> [u8; 4];
    /// Parses `#RRGGBB`, `RRGGBB`, `#RRGGBBAA`, or `RRGGBBAA`.
    pub fn parse_hex(input: &str) -> Result<Self, HexColorError>;
}

pub enum HexColorError {
    InvalidLength,    // "expected color in #RRGGBB or #RRGGBBAA format"
    InvalidComponent, // "invalid hex color component"
}
```

A 6-digit value gets `a = 255`. The `#` prefix is optional.

### `LayoutKind`

```rust
pub enum LayoutKind {
    VerticalStack,
    HorizontalStack,
    Grid { columns: u32 },
    BeforeAfter,   // exactly two images, equal heights; ignores resize_mode
}
```

### `Alignment`

```rust
pub enum Alignment { Start, Center, End }
```

`Default` (in `ComposeOptions`) is `Start`. Alignment shifts each pane within its
cross-axis track (or its grid column); it never changes canvas size.

### `ResizeMode`

```rust
pub enum ResizeMode { None, FitToSameWidth, FitToSameHeight }
```

Images are scaled preserving aspect ratio toward the largest shared width/height, with
rounding (`num * target / den` plus half-denominator). `BeforeAfter` ignores this
option entirely.

### `Frame` and `PaneStyle`

```rust
pub struct Frame {
    pub width: u32,
    pub color: RgbaColor,
}

pub struct PaneStyle {
    pub frame: Option<Frame>,
    pub corner_radius: u32,
}
```

`PaneStyle::default()` is `frame: None`, `corner_radius: 0`.

### `Divider`

```rust
pub struct Divider {
    pub width: u32,
    pub color: RgbaColor,
}
```

### `Spacing`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Spacing {
    pub x: u32,
    pub y: u32,
}

impl Spacing {
    pub const fn all(value: u32) -> Self {
        Self { x: value, y: value }
    }
}
```

`Spacing::default()` is `x: 0, y: 0`; `Spacing::all(n)` sets both axes to `n`. Each
layout reads only the axis it needs: `VerticalStack` uses `y` for gaps between panes,
`HorizontalStack` and `BeforeAfter` use `x`, and `Grid` uses `x` for column gaps and
`y` for row gaps. The unused axis never affects placement or canvas size.

**Breaking change (0.4):** `ComposeOptions.spacing` was a single `u32` applied to both
axes; it is now `Spacing`. `Spacing::all(n)` reproduces the old behavior exactly. At 0.x
this ships without a compatibility shim.

### `ComposeOptions`

```rust
pub struct ComposeOptions {
    pub layout: LayoutKind,
    pub spacing: Spacing,
    pub padding: Insets,
    pub background: RgbaColor,
    pub alignment: Alignment,
    pub resize_mode: ResizeMode,
    pub pane_style: PaneStyle,
    pub divider: Option<Divider>,
    pub column_weights: Vec<u32>,
}
```

Defaults: `layout: VerticalStack`, `spacing: Spacing::default()` (both axes 0),
`padding: Insets::default()` (all 0),
`background: RgbaColor::WHITE`, `alignment: Start`, `resize_mode: None`,
`pane_style: PaneStyle::default()`, `divider: None`, `column_weights: Vec::new()`.
Note that the CLI uses different defaults for some options — see the
[CLI reference](cli.md).

### Grid column weights

`column_weights` sizes grid tracks. It is empty by default, which keeps auto sizing:
each column is as wide as its widest image. When non-empty it must contain one weight
per requested column, every weight at least `1`, and the layout must be `Grid`.

With `natural[i]` = widest image in column `i`:

```text
unit   = max_i ceil(natural[i] / weights[i])
track[i] = unit * weights[i]
```

Tracks never shrink below their content and images are never stretched; extra width is
background and images are aligned inside the wider track as usual. Because weights are
relative, `[1, 1]` makes all used columns equally wide (the widest natural width), and
`[1, 2]` makes the second twice the first.

Example: images 40x30, 20x60, 30x30 with `columns: 2` and `column_weights: vec![1, 2]`
→ natural `[40, 20]`, `unit = max(40, 10) = 40`, tracks `[40, 80]`, canvas 120x90.

## Functions

```rust
pub fn plan_layout(images: &[InputImage], options: &ComposeOptions)
    -> Result<CanvasPlan, ComposeError>;

pub fn compose(images: &[InputImage], options: &ComposeOptions)
    -> Result<ComposedImage, ComposeError>;

/// Decodes PNG/JPEG buffers, composes them, and re-encodes the result as PNG.
pub fn compose_to_png(images: &[Vec<u8>], options: &ComposeOptions)
    -> Result<Vec<u8>, ComposeError>;

pub fn load_image_from_path(path: &Path) -> Result<InputImage, ComposeError>;
pub fn load_image_from_bytes(bytes: &[u8]) -> Result<InputImage, ComposeError>;

pub fn save_png(path: &Path, image: &ComposedImage) -> Result<(), ComposeError>;
pub fn encode_png(image: &ComposedImage) -> Result<Vec<u8>, ComposeError>;
```

`compose` is `plan_layout` followed by rendering. Input formats are PNG and JPEG;
output is always RGBA8 PNG. `encode_png` and `load_image_from_bytes` perform no
filesystem access.

## `ComposeError` variants

| Variant | Message |
| --- | --- |
| `NotEnoughImages` | `at least two images are required` |
| `InvalidPixelBuffer { index, expected, actual }` | `image {index} has invalid RGBA buffer length: expected {expected}, got {actual}` |
| `InvalidGridColumns` | `grid layouts require at least one column` |
| `BeforeAfterRequiresTwo` | `before-after layout requires exactly two images` |
| `DividerRequiresBeforeAfter` | `a divider is only supported by the before-after layout` |
| `DividerTooWide { width, gap }` | `divider width {width} exceeds the {gap}px gap between panes; increase spacing` |
| `ColumnWeightsRequireGrid` | `column weights are only supported by the grid layout` |
| `ColumnWeightCountMismatch { expected, actual }` | `grid received {actual} column weights for {expected} columns` |
| `ZeroColumnWeight { index }` | `column weight {index} must be at least 1` |
| `DimensionOverflow` | `image dimension overflow while calculating layout` |
| `CoordinateOverflow` | `output coordinates overflowed during rendering` |
| `ImageIo(image::ImageError)` | `image I/O error: {0}` |

## Behavior notes

- **Validation order.** `plan_layout` first requires at least two images, then rejects
  any `divider: Some(_)` on a non-`BeforeAfter` layout (even a zero-width one) before
  resolving sizes; `BeforeAfter` then checks the two-image count.
- **BeforeAfter** scales both panes to the shared tallest height with aspect ratio
  preserved and ignores `resize_mode`; the divider (if any) must fit inside the
  `spacing.x` gap and is centered within it.
- **Grid** fills row-major and auto-sizes each column to its widest pane and each row
  to its tallest pane; `columns` is clamped to the image count, and a partial last row
  leaves background-colored cells.
- **Frames** are overlays drawn inside the placed rectangle: they cover outer pixels
  instead of shrinking the content, so the plan is unchanged.
- **Corner radius** is a hard-edged mask computed with integer squared distances, no
  antialiasing, clamped to half the smaller pane dimension; larger values are not an
  error and outside pixels show the background.
- **Compositing is source-over**: fully transparent source pixels are skipped, opaque
  ones copied, and semi-transparent ones blended with an exact integer denominator.
  Frames, dividers, and images all blend the same way against the background.
- **Overflow**, not wrapping: canvas math uses checked arithmetic and reports
  `DimensionOverflow` / `CoordinateOverflow` rather than wrapping.
