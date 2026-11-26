# 🩹 Stitch Me Up / `stitch-me-up`

`stitch-me-up` is the 🩹 Stitch Me Up image composition engine for combining multiple images into one final canvas using deterministic layout rules.

This project is intentionally about composition, not panorama stitching or computer vision. The core library operates on decoded RGBA buffers and is meant to stay portable across CLI, WASM, backend, and future native/mobile adapters.

## Status

The repository contains a `v0.6.0` workspace:

- a Cargo workspace
- a pure Rust core crate for layout planning and rendering
- a thin CLI wrapper for local file input and PNG output
- tests covering layout, rendering, image I/O, and CLI argument handling
- baseline community and GitHub repository files

`Grid` is implemented with row-major placement and auto-sized column and row tracks. `BeforeAfter` takes exactly two images, scales both to their shared tallest height while preserving aspect ratio, and places them side by side, ignoring `resize_mode`.

## Distribution

The workspace publishes three crates to crates.io:

- `stitch-me-up-core` — the pure engine (lib `stitch_core`)
- `stitch-me-up` — the CLI (binary `stitch-me-up`)
- `stitch-me-up-wasm` — the WebAssembly adapter (lib `stitch_wasm`)

Install the CLI with:

```bash
cargo install stitch-me-up
```

Prebuilt binaries for Linux (x86_64), macOS (aarch64), and Windows (x86_64) are attached
to each GitHub Release, with `.sha256` checksums alongside the archives.

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
- Options: per-axis spacing (`Spacing { x, y }` — uniform via `--spacing`, overridden per axis by `--spacing-x`/`--spacing-y`), grid column weights, padding, background color, alignment. `resize_mode` is ignored by `BeforeAfter`, which always equalizes pane heights.
- Per-image decoration overrides: `InputImage::style` (core) replaces the global frame/corner radius for one pane; the CLI gets `--pane-style <INDEX>:<KEY=VALUE>[,...]` (1-based, repeatable) and the WASM adapter gets `set_pane_style` / `clear_pane_style` (0-based).
- Resize modes: `None`, `FitToSameWidth`, `FitToSameHeight`
- Alpha compositing: source-over blending (opaque pixels are copied, fully transparent pixels are skipped)
- Input formats: PNG and JPEG
- Output format: PNG and JPEG (PNG default)
- EXIF orientation: images are decoded upright via `ImageDecoder::orientation` (no manual rotation needed)

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
- Divider: an optional stroke for `BeforeAfter` only, centered inside the horizontal
  (`Spacing::x`) gap between the two panes and spanning their shared height. It must fit
  inside the gap, otherwise planning reports the gap width, and its alpha composites like
  everything else.
- CLI flags: `--frame-width`, `--frame-color`, `--corner-radius`, `--divider-width`,
  `--divider-color`, plus `--pane-style INDEX:key=value,...` per-image overrides.

```bash
stitch-me-up combine --layout before-after --spacing 12 --frame-width 4 --frame-color "#000000" \
  --corner-radius 12 --divider-width 4 --divider-color "#ffffff" --output pair.png before.png after.png
```

Per-image overrides merge per key over the global flags (second pane gets an amber frame,
fourth gets no frame):

```bash
stitch-me-up combine --layout grid --columns 2 --frame-width 4 --corner-radius 12 \
  --pane-style "2:frame-color=#f59e0b,corner-radius=20" --pane-style 4:frame-width=0 \
  --output grid.png a.png b.png c.png d.png
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

JPEG output (`--quality` defaults to 90 and only applies with `--format jpeg`):

```bash
stitch-me-up combine --format jpeg --quality 85 --output out.jpg a.png b.png
```

`--spacing` sets both axes; `--spacing-x` and `--spacing-y` override one axis each:

```bash
stitch-me-up combine --layout grid --columns 2 --spacing 8 --spacing-x 20 --spacing-y 4 \
  --output grid.png a.png b.png c.png d.png
```

## Examples

- Live WebAssembly demo: <https://arthurianx.github.io/stitch-me-up/> — compose images in
  the browser; nothing is uploaded. Source: [examples/web](examples/web/README.md).
- Command-line demo: [examples/cli](examples/cli/compose.sh) — a decorated grid in a
  few lines of bash, runnable from any directory in the repository.

## Documentation

- [docs/api.md](docs/api.md) — core API reference: types, functions, errors, behavior
- [docs/cli.md](docs/cli.md) — every CLI flag, default, and error format
- [docs/layouts.md](docs/layouts.md) — layout cookbook with diagrams and sizing rules
- [docs/wasm.md](docs/wasm.md) — the `Composer` JavaScript API and build workflow

## Development

Requires Rust 1.88 or newer (the workspace MSRV; the locked dependency graph needs
`image 0.25.10` → 1.88, `clap` → 1.85, `wasm-bindgen` → 1.81). CI enforces this with a
dedicated `msrv` job pinned to 1.88.0.

Once Rust is installed locally:

```bash
cargo fmt
cargo test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for contribution guidance and [SECURITY.md](SECURITY.md) for vulnerability reporting expectations.
