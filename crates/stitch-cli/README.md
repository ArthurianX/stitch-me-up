# 🩹 `stitch-me-up`

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
stitch-me-up combine --layout before-after --spacing 12 --frame-width 4 --frame-color "#000000" --corner-radius 12 --divider-width 4 --divider-color "#ffffff" --output pair.png before.png after.png
```

## Per-Image Overrides

```bash
stitch-me-up combine --layout grid --columns 2 --frame-width 4 --corner-radius 12 \
  --pane-style "2:frame-color=#f59e0b,corner-radius=20" \
  --pane-style 4:frame-width=0 \
  --output grid.png a.png b.png c.png d.png
```

`--pane-style` is repeatable; `INDEX` is 1-based. Keys override the global frame and
radius per pane only; `frame-width=0` disables the frame.

## Notes

- The CLI expects decoded-image support from the sibling `stitch-me-up-core` crate.
- Output is always written as PNG in this first pass.
