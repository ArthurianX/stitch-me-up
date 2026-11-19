# 🩹 `stitch-cli`

Command-line wrapper for the 🩹 `stitch-me-up` image composition engine.

## Example

```bash
stitch-me-up combine \
  --layout vertical \
  --spacing 16 \
  --background "#ffffff" \
  --output out.png \
  a.png b.png c.png
```

## Grid Example

```bash
stitch-me-up combine --layout grid --columns 2 --spacing 8 --output grid.png a.png b.png c.png d.png
```

## Before-After Example

```bash
stitch-me-up combine --layout before-after --spacing 12 --output pair.png before.png after.png
```

## Per-Side Padding Example

```bash
stitch-me-up combine --layout vertical --padding 8 --padding-left 32 --output out.png a.png b.png
```

## Decoration Example

```bash
stitch-me-up combine --layout horizontal --frame-width 4 --frame-color "#000000" --corner-radius 12 --output framed.png a.png b.png
```

## Notes

- The CLI expects decoded-image support from the sibling `stitch-core` crate.
- Output is always written as PNG in this first pass.
