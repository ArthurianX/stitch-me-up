# Changelog

All notable changes to this project are documented here.

The format follows Keep a Changelog; version numbers follow Semantic Versioning.

## 0.4.0 — 2026-11-22

First tagged release; the 0.1–0.4 milestones were developed before this tag.

### Added

- Per-image pane styles: `InputImage::style` overrides the global `ComposeOptions::pane_style`.
- CLI `--pane-style <INDEX>:<KEY=VALUE>[,...]` for 1-based, repeatable per-image overrides.
- WASM `set_pane_style` / `clear_pane_style` for 0-based per-input overrides.
- Per-axis spacing (`Spacing { x, y }`, `--spacing-x` / `--spacing-y`).
- Grid column weights (`column_weights`, `--column-weights`).
- Per-pane frames, corner radii, and the before-after divider.
- `stitch-wasm` WebAssembly adapter and browser example.
- API, CLI, layout, and WASM documentation.

### Changed (breaking)

- `ComposeOptions::spacing` is `Spacing`; `Spacing::all(n)` reproduces the old uniform value.
- `InputImage` gained `style: Option<PaneStyle>`; construct it with `InputImage::new`.
- Removed the dead `stitch_core::options` re-export shim and `ComposeError::UnsupportedLayout`.

### Removed

- `ComposeError::UnsupportedLayout` (unused since all layouts were implemented).

## 0.1.0 — 2026-11-19

- Cargo workspace with `stitch-core`, `stitch-cli`, and `stitch-wasm`.
- Layouts: vertical stack, horizontal stack, grid, before-after.
- Resize modes, per-axis padding, RGBA compositing, PNG/JPEG I/O, CLI, WASM adapter.
