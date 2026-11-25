# Web demo — Stitch Me Up

**Live demo:** <https://arthurianx.github.io/stitch-me-up/>

Everything runs in your browser: pick PNG/JPEG images, arrange and decorate them, and
download a stitched PNG. Nothing is uploaded — the page is a static site that loads the
WebAssembly module and composes locally.

## What it does

- Upload or drag-and-drop 2+ PNG/JPEG files (up to 24 files, 25 MP per image, 100 MP total)
- Reorder and remove images (drag-and-drop plus labelled buttons)
- Live preview with debounced composition (250 ms) and a generation counter, so a slow
  decode never overwrites a newer preview
- Layout-aware controls: columns/weights for grid, divider for before-after
- Download the full-resolution PNG as `stitch-me-up-YYYYMMDD-HHMMSS.png`

## Local build and serve

Prerequisites: Rust (stable) and `wasm-pack`:

```bash
cargo install wasm-pack
```

Build the module:

```bash
cd crates/stitch-wasm
wasm-pack build --target web --out-dir ../../examples/web/pkg
```

This writes generated files (JS glue plus the `.wasm` binary) into `examples/web/pkg/`.
That directory is generated output, committed nowhere: it is listed in `.gitignore`.

Serve the directory over HTTP (ES modules do not load from `file://`):

```bash
python3 -m http.server 8080 --directory examples/web
```

Open <http://localhost:8080/>.

## Tests

The pure logic in `lib.js` is covered by `node --test`:

```bash
node --test examples/web/tests/*.test.js
```

## Generated files

- `pkg/` — wasm-pack output (JS glue + `.wasm`), gitignored
- `version.json` — written at deploy time with `{version, commit}` from the workspace
  `Cargo.toml` and `GITHUB_SHA`, gitignored

## Deployment

Every push to `main` builds the wasm package, runs the node tests, writes `version.json`,
and deploys `examples/web` to GitHub Pages via `.github/workflows/pages.yml`. The footer
shows the engine version and deployed commit from `version.json`; a missing file (local
dev, `file://`) hides the footer version instead of erroring.

## Notes

- Inputs must be PNG or JPEG bytes; the composed output is always PNG.
- The divider controls are only enabled for the `before-after` layout, because the core
  rejects dividers in every other layout.
- See [docs/wasm.md](../../docs/wasm.md) for the full JS API reference and limitations.
