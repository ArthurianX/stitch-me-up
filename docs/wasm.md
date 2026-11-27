# WASM Reference (`stitch-me-up-wasm`, lib `stitch_wasm`)

The `Composer` class exposes the `stitch-me-up-core` engine to JavaScript. Everything runs
client-side: inputs are PNG/JPEG **bytes**; output is PNG bytes by default.

## Live demo

The web example is deployed to GitHub Pages at
<https://arthurianx.github.io/stitch-me-up/>. The footer shows the engine version and
deployed commit from `version.json`, which is written at deploy time (`{version, commit}`
from the workspace `Cargo.toml` and `GITHUB_SHA`) and is gitignored. A missing file
(local dev, `file://`) hides the footer version instead of erroring.

## Build and serve

`wasm-pack` generates the JS glue; it is not installed in CI and not required for the
Rust test suite.

```bash
cargo install wasm-pack
cd crates/stitch-wasm
wasm-pack build --target web --out-dir ../../examples/web/pkg
python3 -m http.server 8080 --directory examples/web
```

Open `http://localhost:8080/` (ES modules do not load from `file://`). CI only runs
`cargo check -p stitch-me-up-wasm --target wasm32-unknown-unknown`; the generated `pkg/`
directory is gitignored.

## API

```js
import init, { Composer } from "./pkg/stitch_wasm.js";

await init();                 // must resolve before constructing a Composer
const composer = new Composer();
```

Constructor: `new Composer()` starts from the core defaults (`vertical` layout, no
spacing/padding, white background, `start` alignment, `none` resize mode, no frame,
radius 0, no divider).

### Setters

| Method | Accepted values |
| --- | --- |
| `set_layout(layout, columns)` | `"vertical"`, `"horizontal"`, `"grid"`, `"before-after"` (case-insensitive, surrounding whitespace trimmed). `columns` is only used by `"grid"`; pass `0` and `compose` fails with `grid layouts require at least one column` |
| `set_spacing(x, y)` | horizontal and vertical gap in pixels; each layout uses its own axis (see [layouts.md](layouts.md)) |
| `set_padding(top, right, bottom, left)` | numbers of pixels, in that order |
| `set_background(hex)` | `#RRGGBB` or `#RRGGBBAA` (`#` optional) |
| `set_alignment(alignment)` | `"start"`, `"center"`, `"end"` (case-insensitive) |
| `set_resize_mode(mode)` | `"none"`, `"fit-to-same-width"`, `"fit-to-same-height"` (case-insensitive) |
| `set_frame(width, hex)` | width `0` disables the frame but still validates the color |
| `set_corner_radius(radius)` | pixels; clamped to half the smaller pane dimension |
| `set_column_weights(weights)` | `Uint32Array` of relative grid column widths; count must match the layout's column count, empty array keeps auto sizing |
| `set_cell_size(width, height)` | Fixed tile size for the grid layout; each image is center-cropped to the tile. Grid-only; non-zero values fail at compose time |
| `clear_cell_size()` | Restores automatic grid sizing (`cell_size` cleared) |
| `set_divider(width, hex)` | width `0` disables the divider but still validates the color. Setting a non-zero divider and then composing a non-`before-after` layout throws |
| `set_pane_style(index, frameWidth, frameColorHex, cornerRadius)` | 0-based input index; stores a complete per-pane style override. `frameWidth == 0` disables that pane's frame while still validating the color |
| `clear_pane_style(index)` | Removes the override; the pane inherits the global style again |

### `compose(images)`

```js
const png = composer.compose([
  new Uint8Array(await beforeFile.arrayBuffer()),
  new Uint8Array(await afterFile.arrayBuffer()),
]);
const url = URL.createObjectURL(new Blob([png], { type: "image/png" }));
```

Every element must be a `Uint8Array` of PNG or JPEG bytes; the return value is a
`Uint8Array` of PNG bytes. `compose` is always PNG — it ignores any format selection.

### `compose_with_format(images, format, quality)`

```js
const jpg = composer.compose_with_format(images, "jpeg", 85);
const url = URL.createObjectURL(new Blob([jpg], { type: "image/jpeg" }));
```

`format` is `"png"` or `"jpeg"` (case-insensitive, surrounding whitespace trimmed);
`quality` is only used for JPEG and is ignored for PNG. Every element of `images`
must be a `Uint8Array` of PNG or JPEG bytes; the return value is a `Uint8Array`
matching the requested format.

Errors:

- unknown format: `unknown output format "<value>"; expected png or jpeg`
- JPEG quality outside 1–100: `jpeg quality <q> is outside 1..=100`

Stored per-pane styles are applied to the decoded images after decoding. An index with
no matching input throws, so a stale `set_pane_style(2, ...)` with only one input fails
that compose call.

## Error behavior

Every failure throws a JavaScript `Error` whose message is the Rust error text. Setter
typos surface immediately; core and index problems surface at `compose`.

Setter errors:

- unknown strings: `unknown layout "panorama"; expected vertical, horizontal, grid, or before-after`
- bad colors: `expected color in #RRGGBB or #RRGGBBAA format`, `invalid hex color component`
- non-`Uint8Array` input: `each image must be a Uint8Array`

Compose errors:

- core errors propagate verbatim, e.g. `at least two images are required`,
  `before-after layout requires exactly two images`,
  `a divider is only supported by the before-after layout`,
  `divider width 4 exceeds the 2px gap between panes; increase spacing`,
  `grid layouts require at least one column`, `image I/O error: ...`
- `pane style index N has no matching image`: a stored `set_pane_style` index has no
  corresponding input image

## Tile presets

The web demo maps its grid `Tile shape` select to these setters: `off` (the default)
keeps automatic sizing and leaves `set_cell_size` untouched; `square` sets
`set_cell_size(320, 320)`; `portrait` sets `set_cell_size(320, 400)`. Applying a preset
clears and disables the Weights input, so a preset and `set_column_weights` never
combine.

## Limitations

- Only PNG and JPEG inputs are decoded; output is PNG via `compose`, or PNG/JPEG via `compose_with_format`.
- `wasm-pack` is not part of CI, so the JS bindings are checked only by the wasm target
  compile; the browser behavior is verified manually via the web example.
- The generated `pkg/` directory and `version.json` are local build output and are
  gitignored.

See the [web example](../examples/web/README.md) for a complete UI and
[API reference](api.md) for the underlying layout semantics.
