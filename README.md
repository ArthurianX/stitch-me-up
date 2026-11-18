# 🩹 Stitch Me Up / `stitch-me-up`

`stitch-me-up` is the 🩹 Stitch Me Up image composition engine for combining multiple images into one final canvas using deterministic layout rules.

This project is intentionally about composition, not panorama stitching or computer vision. The core library operates on decoded RGBA buffers and is meant to stay portable across CLI, WASM, backend, and future native/mobile adapters.

## Status

The repository now contains:

- a Cargo workspace
- a pure Rust core crate for layout planning and rendering
- a thin CLI wrapper for local file input and PNG output
- tests covering layout, rendering, image I/O, and CLI argument handling
- baseline community and GitHub repository files

`Grid` is implemented with row-major placement and auto-sized column and row tracks. `BeforeAfter` takes exactly two images, scales both to their shared tallest height while preserving aspect ratio, and places them side by side, ignoring `resize_mode`.

## Workspace

```text
stitch-me-up/
├── Cargo.toml
├── crates/
│   ├── stitch-core/
│   └── stitch-cli/
├── .github/
└── README.md
```

## Core API

The core is centered on decoded pixel buffers:

```rust
pub fn plan_layout(
    images: &[InputImage],
    options: &ComposeOptions,
) -> Result<CanvasPlan, ComposeError>;

pub fn compose(
    images: &[InputImage],
    options: &ComposeOptions,
) -> Result<ComposedImage, ComposeError>;
```

Boundary helpers currently available:

```rust
pub fn load_image_from_path(path: &Path) -> Result<InputImage, ComposeError>;
pub fn save_png(path: &Path, image: &ComposedImage) -> Result<(), ComposeError>;
```

## Current Features

- Layouts: `VerticalStack`, `HorizontalStack`, `Grid`, `BeforeAfter`
- Options: spacing, padding, background color, alignment. `resize_mode` is ignored by `BeforeAfter`, which always equalizes pane heights.
- Resize modes: `None`, `FitToSameWidth`, `FitToSameHeight`
- Input formats: PNG and JPEG
- Output format: PNG

## CLI Example

```bash
stitch-me-up combine \
  --layout vertical \
  --spacing 16 \
  --padding 24 \
  --background "#ffffff" \
  --output out.png \
  a.png b.png c.png
```

## Development

Once Rust is installed locally:

```bash
cargo fmt
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidance and [SECURITY.md](SECURITY.md) for vulnerability reporting expectations.
