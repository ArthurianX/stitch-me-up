# stitch-me-up-wasm

WebAssembly adapter for the 🩹 Stitch Me Up image composition engine. It exposes the
`stitch-me-up-core` engine to JavaScript through a single `Composer` object: configure it with
setters, hand it encoded images, get PNG bytes back. All decode, compose and encode work
lives in `stitch-me-up-core`; this crate only bridges types.

## Build

`wasm-pack` is required to generate the JavaScript glue. Point it at the web example:

```bash
wasm-pack build crates/stitch-wasm --target web --out-dir ../../examples/web/pkg
```

The generated `pkg/` directory backs the [web example](../../examples/web/README.md).
To check the crate compiles for the web target without wasm-pack:

```bash
rustup target add wasm32-unknown-unknown
cargo check -p stitch-me-up-wasm --target wasm32-unknown-unknown
```

## Usage

Inputs are `Uint8Array`s holding **PNG or JPEG bytes**; the output is always **PNG bytes**.

```js
import init, { Composer } from './pkg/stitch_wasm.js';
await init();

const composer = new Composer();
composer.set_layout('before-after', 0);
composer.set_spacing(12, 12);
composer.set_background('#ffffff');
composer.set_corner_radius(8);
// Per-image style override (0-based); remove with clear_pane_style.
composer.set_pane_style(1, 4, '#f59e0b', 20);

const png = composer.compose([new Uint8Array(await beforeFile.arrayBuffer()), new Uint8Array(await afterFile.arrayBuffer())]);
const url = URL.createObjectURL(new Blob([png], { type: 'image/png' }));
```

Every setter throws a JS `Error` whose message is the core error text, so typos in hex
colors or layout names surface immediately rather than at compose time.
