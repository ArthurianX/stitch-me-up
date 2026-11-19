# Layout Cookbook

All layouts: panes sit inside a content band, the band sits inside `padding`
(`--padding`, optionally overridden per side), gaps between panes are `spacing`.
`alignment` (`start`, `center`, `end`) shifts each pane within its cross-axis track;
canvas size never depends on alignment. Sizes are integers; scaling rounds
(`num * target / den` plus half the denominator).

## Choosing a layout

| Want | Layout |
| --- | --- |
| Images stacked top-to-bottom, original sizes | `vertical` |
| Images side by side, original sizes | `horizontal` |
| A mosaic with a fixed number of columns | `grid` |
| Exactly two images, same height, side by side | `before-after` |

## `vertical`

Canvas width = widest image; height = sum of heights + `spacing` between each pair.
Shorter panes slide horizontally according to `alignment`.

```text
padding ┌──────────────────────────────┐
        │  ┌───────────────┐           │   content width = widest pane
        │  │    image 1    │           │
        │  └───────────────┘           │
        │     spacing ┆               │
        │    ┌───────────────┐        │   narrower pane, aligned center
        │    │   image 2     │        │
        └──────────────────────────────┘
```

```bash
stitch-me-up combine --layout vertical --spacing 16 --padding 24 \
  --alignment center --output out.png a.png b.png c.png
```

## `horizontal`

Canvas height = tallest image; width = sum of widths + `spacing`. Shorter panes slide
vertically according to `alignment`.

```text
┌──────────────────────────────────────────┐
│ ┌─────────┐  ┌──────────┐  ┌─────────┐ │
│ │         │  │          │  │         │ │   tallest pane sets
│ │ image 1 │  │  image 2 │  │ image 3 │ │   the canvas height
│ │         │  │          │  │         │ │
│ └─────────┘  └──────────┘  │         │ │   shorter pane, aligned end
│                            └─────────┘ │
└──────────────────────────────────────────┘
```

```bash
stitch-me-up combine --layout horizontal --spacing 8 --padding 12 \
  --output out.png a.png b.png c.png
```

## `grid`

Images fill cells row-major (image `i` lands in row `i / columns`, column
`i % columns`). Each column is as wide as its widest member; each row is as tall as
its tallest member. `--columns` is clamped to the image count; a partial last row
leaves background-colored cells. Panes align within their column.

```text
columns = 2, three images
┌──────────────────┬───────────────┐
│                  │               │   row 0 height = max(h1, h2)
│     image 1      │    image 2    │   col 0 width  = max(w1)
│                  │               │   col 1 width  = max(w2)
├──────────────────┼───────────────┤
│     image 3      │   (background)│   row 1 height = max(h3)
│                  │               │   image 3 wraps into column 0
└──────────────────┴───────────────┘
```

```bash
stitch-me-up combine --layout grid --columns 2 --spacing 8 --padding 16 \
  --output out.png a.png b.png c.png
```

Track sizing example: images 40x30, 20x60, 30x30 with `--columns 2 --spacing 8
--padding 16` fill `(0,0)`, `(0,1)`, `(1,0)`, so column widths are `max(40) = 40` and
`max(20) = 20` — the third image wraps into column 0, it does *not* widen column 1 —
and row heights are `max(30, 60) = 60` and `max(30) = 30`. The canvas is
`40 + 8 + 20 + 32 = 100` wide and `60 + 8 + 30 + 32 = 130` tall.

## `before-after`

Exactly two images. Both are scaled to the shared tallest height, preserving aspect
ratio — `--resize-mode` is ignored. They are then placed like a horizontal stack.

```text
┌───────────────────┬────────────────────────┐
│                   │                        │
│   image 1 scaled  │     image 2 scaled     │   equal heights = max(h1, h2)
│   to shared H     │     to shared H        │   widths scale with aspect ratio
│                   │                        │
└───────────────────┴────────────────────────┘
```

```bash
stitch-me-up combine --layout before-after --spacing 10 --padding 5 \
  --divider-width 4 --divider-color "#ff0000" \
  --output pair.png before.png after.png
```

A 40x30 and a 20x60 image become 40x30 and 20x60 → both scaled to height 60: widths
80 and 20; canvas `80 + 10 + 20 + 10 = 120` wide, `60 + 10 = 70` tall.

## How decoration interacts

- `--frame-width` / `--frame-color` and `--corner-radius` apply to **every** layout:
  they are overlays drawn inside each pane's rectangle. A frame covers the outer
  pixels of the image; a corner radius exposes the background outside the rounded
  mask (hard-edged, clamped to half the smaller pane dimension).
- `--divider-width` / `--divider-color` are valid for **`before-after` only**: the
  divider is centered in the `spacing` gap and spans the shared pane height. It must
  fit inside the gap — `--spacing 2 --divider-width 4` fails with
  `divider width 4 exceeds the 2px gap between panes; increase spacing` — and using a
  non-zero divider with any other layout fails with
  `a divider is only supported by the before-after layout`.
- All decoration composites source-over against the background, so `#RRGGBBAA` colors
  blend rather than replace.

See [API reference](api.md) for the exact semantics and
[CLI reference](cli.md) for flag defaults.
