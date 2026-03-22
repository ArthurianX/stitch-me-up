# `stitch-cli`

Command-line wrapper for the `stitch-me-up` image composition engine.

## Example

```bash
stitch-me-up combine \
  --layout vertical \
  --spacing 16 \
  --background "#ffffff" \
  --output out.png \
  a.png b.png c.png
```

## Notes

- The CLI expects decoded-image support from the sibling `stitch-core` crate.
- Output is always written as PNG in this first pass.
