# Roadmap

`stitch-me-up` is a composition engine, not a panorama or CV tool. The core stays
pure Rust, deterministic, and free of new dependencies unless a milestone says
otherwise. Breaking API changes are acceptable while the version is `0.x`.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the principles this roadmap follows.

## 0.1 — Foundations (done)

- [x] Cargo workspace: `stitch-core` (pure engine) and `stitch-cli` (thin adapter)
- [x] Layouts: `VerticalStack`, `HorizontalStack`
- [x] Options: spacing, insets, background, alignment, resize modes
- [x] PNG/JPEG input, PNG output
- [x] Grid layout with auto-sized row and column tracks
- [x] `BeforeAfter` as a two-pane, equal-height layout
- [x] Source-over alpha compositing
- [x] Per-side padding flags in the CLI
- [x] CI: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`

## 0.2 — Visual polish (in progress)

All items are core-only, add no dependencies, and must be covered by
pixel-level tests. Decoration is drawn *inside* each pane's placed rectangle, so
layout math and `CanvasPlan` stay unchanged.

- [x] Per-pane frame/border: inset ring drawn over the image, with its own color
      and alpha
- [x] Per-pane corner radius: hard-edged rounded mask, composited over the
      background
- [x] `BeforeAfter` divider: a stroke centered in the gap between panes

Decision log:

- **Hard edges, no antialiasing.** Antialiasing needs supersampling to stay
  deterministic across platforms; deferred until there is demand.
- **Decoration is an overlay.** A frame covers the outer pixels of the image
  instead of shrinking its content, which keeps every layout test valid.
- **Radius is clamped** to half the smaller pane dimension; oversized values are
  not an error.

## 0.3 — Reach

- [x] `stitch-wasm` adapter exposing `compose` to JavaScript
- [x] `examples/` with runnable samples used by the README
- [x] `docs/` with the API reference and a layout cookbook
- [ ] Published crate metadata (`repository`, `readme`, keywords, categories)

## 0.4 — Layout expressiveness

- [ ] Per-axis spacing (`spacing_x` / `spacing_y`)
- [ ] Grid column weights for uneven columns
- [ ] Per-image style overrides (new input shape; breaking change)

## Out of scope

- Panorama stitching, computer vision, and any AI features
- Text labels and captions: a font rasterizer conflicts with the
  zero-dependency, deterministic core. Revisit only with a bitmap-font strategy.
- Runtime plugin systems and non-Rust adapters inside this repository
