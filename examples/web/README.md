# Web example

A small browser UI that composes PNG/JPEG images entirely client-side, using the
[`stitch-wasm`](../../crates/stitch-wasm) adapter. No CDN, no framework, no build step
for the page itself — but the WebAssembly module must be generated first.

## Prerequisites

- Rust (stable) installed
- `wasm-pack`:

```bash
cargo install wasm-pack
```

## Build the module

```bash
cd crates/stitch-wasm
wasm-pack build --target web --out-dir ../../examples/web/pkg
```

This writes generated files (JS glue plus the `.wasm` binary) into `examples/web/pkg/`.
That directory is generated output, committed nowhere: it is listed in `.gitignore`.

## Serve the page

ES modules do not load from `file://`, so serve the directory over HTTP:

```bash
python3 -m http.server 8080 --directory examples/web
```

Open <http://localhost:8080/>, pick two or more images, adjust the controls, and click
Compose. The result is shown as a preview and can be downloaded as `stitch-result.png`.

## Notes

- Inputs must be PNG or JPEG bytes; the composed output is always PNG.
- The divider controls are only enabled for the `before-after` layout, because the core
  rejects dividers in every other layout.
- CI does not run `wasm-pack`; it only checks that the crate compiles for
  `wasm32-unknown-unknown`. See [docs/wasm.md](../../docs/wasm.md) for the full JS API
  reference and limitations.
