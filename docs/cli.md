# CLI Reference

```text
stitch-me-up combine [OPTIONS] --output <FILE> <INPUT> <INPUT>...
```

`combine` reads 2 or more PNG or JPEG files and writes one PNG or JPEG (default PNG).
This matches `stitch-me-up combine --help`.

## Flags

| Flag | Default | Meaning |
| --- | --- | --- |
| `<INPUT>...` | required, 2+ | Input PNG or JPEG files |
| `-o, --output <FILE>` | required | Output PNG file |
| `--layout <LAYOUT>` | `vertical` | `vertical`, `horizontal`, `grid`, `before-after` |
| `--columns <COLUMNS>` | `2` | Grid column count when layout is `grid` |
| `--column-weights <W1,W2,...>` | unset | Comma-separated relative grid column widths; the count must match `--columns`, and an empty value keeps auto sizing |
| `--spacing <SPACING>` | `0` | Space between images in pixels, overridden per axis by `--spacing-x`/`--spacing-y` |
| `--spacing-x <SPACING_X>` | unset | Horizontal gap between images in pixels, overriding `--spacing` |
| `--spacing-y <SPACING_Y>` | unset | Vertical gap between images in pixels, overriding `--spacing` |
| `--padding <PADDING>` | `0` | Uniform padding around the canvas |
| `--padding-top <N>` | unset | Padding above the canvas, overriding `--padding` |
| `--padding-right <N>` | unset | Padding to the right of the canvas, overriding `--padding` |
| `--padding-bottom <N>` | unset | Padding below the canvas, overriding `--padding` |
| `--padding-left <N>` | unset | Padding to the left of the canvas, overriding `--padding` |
| `--background <COLOR>` | `#ffffff` | Background color as `#RRGGBB` or `#RRGGBBAA` |
| `--alignment <ALIGNMENT>` | `center` | `start`, `center`, `end` |
| `--resize-mode <MODE>` | `none` | `none`, `fit-to-same-width`, `fit-to-same-height` |
| `--frame-width <N>` | `0` | Frame width in pixels drawn inside each image (`0` disables the frame) |
| `--frame-color <COLOR>` | `#000000` | Frame color as `#RRGGBB` or `#RRGGBBAA` |
| `--corner-radius <N>` | `0` | Corner radius in pixels applied to each image |
| `--divider-width <N>` | `0` | Divider width in pixels between before-after panes (`0` disables the divider) |
| `--divider-color <COLOR>` | `#000000` | Divider color as `#RRGGBB` or `#RRGGBBAA` |
| `--pane-style <SPEC>` | unset, repeatable | Per-image decoration override. `SPEC` is `INDEX:KEY=VALUE[,KEY=VALUE...]`; `INDEX` is `1`-based; keys are `frame-width` (u32), `frame-color` (`#RRGGBB`/`#RRGGBBAA`), `corner-radius` (u32) |
| `--format <FORMAT>` | `png` | Output format: `png` or `jpeg` |
| `--quality <1-100>` | `90` | JPEG quality (only with `--format jpeg`; default 90) |
| `-h, --help` | — | Print help |

Value rules:

- Sizes (`--spacing`, `--spacing-x`, `--spacing-y`, `--padding*`, `--columns`,
  `--frame-width`, `--corner-radius`, `--divider-width`) are non-negative integers
  (`u32`). `--spacing` sets both axes; `--spacing-x`/`--spacing-y` override one axis
  each (vertical stacks use the vertical gap, horizontal stacks and before-after use
  the horizontal gap, grids use both).
- Colors accept `#RRGGBB` or `#RRGGBBAA`; the `#` prefix is optional but shells usually
  need quoting to keep it from being treated as a comment. A malformed color fails at
  argument-parsing time with `expected color in #RRGGBB or #RRGGBBAA format` or
  `invalid hex color component`.
- Enum values are the lowercase kebab-case strings listed above, matched
  case-sensitively by clap.
- `--column-weights` is only valid with `--layout grid`. It must list exactly
  `--columns` weights, each at least `1`; otherwise the run fails before rendering.
- `--pane-style` overrides start from the global style fields (`--frame-width`,
  `--frame-color`, `--corner-radius`) and merge per key; key order never matters and a
  repeated `INDEX` or `KEY` keeps the later value. `frame-width=0` disables the frame for
  that pane; `frame-color` alone with the default global `frame-width 0` is accepted and
  validated but unused, exactly like the global flag. `INDEX` must point at an existing
  input (`1` is the first image). Quote the whole spec when it contains `#`
  (`--pane-style "1:frame-color=#f59e0b"`), so the shell does not treat the rest as a
  comment.
- `--alignment` defaults to `center` in the CLI although the core default is `start`.
- `--quality` is only valid with `--format jpeg`; passing it with the default `png` format
  fails with `--quality only applies to --format jpeg`. The value must be `1..=100`
  (clap rejects out-of-range values before rendering).

## Examples

Plain vertical stack:

```bash
stitch-me-up combine --output out.png a.png b.png c.png
```

JPEG output:

```bash
stitch-me-up combine --format jpeg --quality 85 --output out.jpg a.png b.png
```

Before-after pair with a divider and rounded corners:

```bash
stitch-me-up combine --layout before-after --spacing 12 \
  --frame-width 4 --frame-color "#000000" --corner-radius 12 \
  --divider-width 4 --divider-color "#ffffff" \
  --output pair.png before.png after.png
```

Decorated grid with asymmetric padding:

```bash
stitch-me-up combine --layout grid --columns 3 --spacing 8 \
  --padding 16 --padding-left 32 --background "#f8fafc" \
  --output grid.png a.png b.png c.png d.png
```

Weighted grid columns (the second column is twice the first):

```bash
stitch-me-up combine --layout grid --columns 2 --column-weights 1,2 \
  --output weighted.png a.png b.png c.png
```

Per-image overrides (second pane gets an amber frame, fourth gets no frame):

```bash
stitch-me-up combine --layout grid --columns 2 \
  --frame-width 4 --corner-radius 12 \
  --pane-style "2:frame-color=#f59e0b,corner-radius=20" \
  --pane-style 4:frame-width=0 \
  --output grid.png a.png b.png c.png d.png
```

The runnable [`examples/cli/compose.sh`](../examples/cli/compose.sh) wraps a similar
grid invocation.

## Error behavior

Failures exit with a non-zero status and print an `Error:` line; when the failure came
from the engine or I/O, `Caused by:` lines show the underlying message:

```text
$ stitch-me-up combine --layout before-after --spacing 2 --divider-width 4 \
    --output out.png a.png b.png
Error: failed to compose images

Caused by:
    divider width 4 exceeds the 2px gap between panes; increase spacing
$ echo $?
1
```

```text
$ stitch-me-up combine --layout vertical --divider-width 4 --output out.png a.png b.png
Error: failed to compose images

Caused by:
    a divider is only supported by the before-after layout
$ echo $?
1
```

```text
$ stitch-me-up combine --output out.png missing.png b.png
Error: failed to load input image missing.png

Caused by:
    0: image I/O error: No such file or directory (os error 2)
    1: No such file or directory (os error 2)
$ echo $?
1
```

The CLI also rejects two combinations before loading anything, as a single-line message
without a `Caused by:` chain (these are CLI-level checks, not core errors):

- `before-after layout requires exactly 2 input images`
- `grid layout requires at least 1 column`
- `--quality only applies to --format jpeg`

The CLI also rejects malformed `--pane-style` specs after parsing, before loading any
input (single-line errors, no `Caused by:` chain):

- `pane style index N is out of range for M images` (N is 1-based; `0` and too large both fail)
- `invalid pane style "…"` for a spec without `<index>:<key>=<value>`
- `unknown pane style key "…"` for a key outside `frame-width`, `frame-color`, `corner-radius`
- `invalid frame-width value "…"`, `invalid corner-radius value "…"`, or a bad color message
  (`expected color in #RRGGBB or #RRGGBBAA format`, `invalid hex color component`)

Malformed arguments (unknown flag, unparsable color, missing `--output`) exit `2` with
clap's usage message. Successful runs print `Wrote <output path>` and exit `0`.
