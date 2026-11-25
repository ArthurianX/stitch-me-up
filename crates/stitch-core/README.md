# 🩹 `stitch-me-up-core`

Deterministic image composition engine: layout planning and rendering over RGBA buffers.

## Why buffers?

The engine takes decoded RGBA8 pixel buffers — 4 bytes per pixel, row-major — and hands
back a `ComposedImage` of the same shape. No filesystem, no network, no format magic in
the hot path: PNG/JPEG decoding and encoding are thin boundary helpers, so the same core
serves the CLI (`stitch-me-up`), the browser (`stitch-me-up-wasm`), and any future backend or
mobile adapter without change. All arithmetic is integer and overflow-checked.

## Example

```rust
use stitch_core::{compose, ComposeOptions, ComposedImage, InputImage};

fn solid(w: u32, h: u32, color: [u8; 4]) -> InputImage {
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..w * h {
        pixels.extend_from_slice(&color);
    }

    InputImage::new(w, h, pixels)
}

let images = vec![solid(4, 2, [255, 0, 0, 255]), solid(2, 4, [0, 255, 0, 255])];
let composed: ComposedImage = compose(&images, &ComposeOptions::default()).unwrap();

assert_eq!((composed.width, composed.height), (4, 6));
```

`InputImage::new` leaves `style` unset, so every image built this way inherits
`ComposeOptions::pane_style`. Setting `style = Some(…)` replaces the global frame and
corner radius for that pane only; geometry and canvas size are unchanged.

## Layouts

`VerticalStack`, `HorizontalStack`, `Grid { columns }` (row-major, auto-sized tracks),
and `BeforeAfter` (exactly two panes, equalized heights). `Frame`, `PaneStyle` corner
radii, and the `BeforeAfter` `Divider` are drawn as overlays inside each pane's placed
rectangle — layout math and `CanvasPlan` never change. See [the API
reference](../../docs/api.md) and the [layout cookbook](../../docs/layouts.md).

## Engine and adapters

This crate is the engine. The adapters stay thin: [`stitch-me-up`](../stitch-cli) does file
I/O and argument parsing, [`stitch-me-up-wasm`](../stitch-wasm) exposes the same composition to
JavaScript. Buffered decode/encode/composition helpers (`compose_to_png`,
`load_image_from_bytes`, `encode_png`) live here, not in the adapters.
