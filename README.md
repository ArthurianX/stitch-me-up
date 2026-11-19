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
├── examples/
├── docs/
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
- Alpha compositing: source-over blending (opaque pixels are copied, fully transparent pixels are skipped)
- Input formats: PNG and JPEG
- Output format: PNG

## Decoration

Pane decoration is drawn as an overlay inside each pane's placed rectangle, so layout
math and `CanvasPlan` stay untouched.

- Frames: `Frame { width, color }` paints an inset ring over the outer pixels of a pane
  instead of shrinking its content, and its alpha composites with the background exactly
  like image pixels.
- Corner radius: a hard-edged rounded mask, computed with integer squared distances, no
  antialiasing. The radius is clamped to half the smaller pane dimension; larger values
  are not an error. Pixels outside the mask leave whatever was underneath (usually the
  background) visible.
- Divider: an optional stroke for `BeforeAfter` only, centered inside the `spacing` gap
  between the two panes and spanning their shared height. It must fit inside the gap,
  otherwise planning reports the gap width, and its alpha composites like everything else.
- CLI flags: `--frame-width`, `--frame-color`, `--corner-radius`, `--divider-width`,
  `--divider-color`.

```bash
stitch-me-up combine --layout before-after --spacing 12 --frame-width 4 --frame-color "#000000" \
  --corner-radius 12 --divider-width 4 --divider-color "#ffffff" --output pair.png before.png after.png
```

## CLI Example

```bash
stitch-me-up combine \
  --layout vertical \
  --spacing 16 \
  --padding 24 \
  --padding-left 32 \
  --background "#ffffff" \
  --output out.png \
  a.png b.png c.png
```

## Examples

- WebAssembly browser demo: [examples/web](examples/web/README.md) — compose images in
  the page itself; requires `wasm-pack` to build its `pkg/` directory first.
- Command-line demo: [examples/cli](examples/cli/compose.sh) — a decorated grid in a
  few lines of bash, runnable from any directory in the repository.

## Documentation

- [docs/api.md](docs/api.md) — core API reference: types, functions, errors, behavior
- [docs/cli.md](docs/cli.md) — every CLI flag, default, and error format
- [docs/layouts.md](docs/layouts.md) — layout cookbook with diagrams and sizing rules
- [docs/wasm.md](docs/wasm.md) — the `Composer` JavaScript API and build workflow

## Development

Once Rust is installed locally:

```bash
cargo fmt
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidance and [SECURITY.md](SECURITY.md) for vulnerability reporting expectations.
