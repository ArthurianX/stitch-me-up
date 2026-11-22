# Changelog

All notable changes to this project are documented here.

The format follows Keep a Changelog; version numbers follow Semantic Versioning.

## 0.4.1 — 2026-11-22

### Fixed

- `--pane-style` validation errors are single-line (no `Caused by:` chain), matching
  the CLI reference.
- WASM docs place `pane style index N has no matching image` under compose-time, where
  it is actually thrown.

### Documentation

- All `--pane-style` examples quote specs containing `#`; the CLI reference explains why.
- Added a byte-for-byte test that `None` per-image styles match an explicit global style.
- Indexed the design and plan documents, and removed a duplicate changelog entry.

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

### Removed

- The `stitch_core::options` re-export shim and `ComposeError::UnsupportedLayout`.
