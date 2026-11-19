# CLI Reference

```text
stitch-me-up combine [OPTIONS] --output <FILE> <INPUT> <INPUT>...
```

`combine` reads 2 or more PNG or JPEG files and writes one PNG. This matches
`stitch-me-up combine --help`.

## Flags

| Flag | Default | Meaning |
| --- | --- | --- |
| `<INPUT>...` | required, 2+ | Input PNG or JPEG files |
| `-o, --output <FILE>` | required | Output PNG file |
| `--layout <LAYOUT>` | `vertical` | `vertical`, `horizontal`, `grid`, `before-after` |
| `--columns <COLUMNS>` | `2` | Grid column count when layout is `grid` |
| `--spacing <SPACING>` | `0` | Space between images in pixels |
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
| `-h, --help` | — | Print help |

Value rules:

- Sizes (`--spacing`, `--padding*`, `--columns`, `--frame-width`, `--corner-radius`,
  `--divider-width`) are non-negative integers (`u32`).
- Colors accept `#RRGGBB` or `#RRGGBBAA`; the `#` prefix is optional but shells usually
  need quoting to keep it from being treated as a comment. A malformed color fails at
  argument-parsing time with `expected color in #RRGGBB or #RRGGBBAA format` or
  `invalid hex color component`.
- Enum values are the lowercase kebab-case strings listed above, matched
  case-sensitively by clap.
- `--alignment` defaults to `center` in the CLI although the core default is `start`.

## Examples

Plain vertical stack:

```bash
stitch-me-up combine --output out.png a.png b.png c.png
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

Malformed arguments (unknown flag, unparsable color, missing `--output`) exit `2` with
clap's usage message. Successful runs print `Wrote <output path>` and exit `0`.
