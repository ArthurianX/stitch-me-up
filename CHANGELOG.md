# Changelog

All notable changes to this project are documented here.

The format follows Keep a Changelog; version numbers follow Semantic Versioning.

## 0.5.1 — 2026-11-26

Correctness release: six defects fixed across the render, IO, WASM, and web layers,
plus a Playwright browser suite in CI. No engine behavior change beyond the fixes.

### Fixed

- Premultiplied resize: `prepare_pixels` now premultiplies straight RGBA before
  `imageops::resize` and unpremultiplies after, so transparent edges no longer tint
  visible pixels (`BeforeAfter` included).
- `InvalidOutputBuffer` instead of encoder panics: `encode_png`/`save_png` validate
  the buffer length (`width*height*4`) before touching the encoder and return
  `ComposeError::InvalidOutputBuffer` instead of panicking.
- Sparse `set_pane_style`: pane styles are stored in a `BTreeMap`, so setting a style
  at a large index no longer traps (`RuntimeError: unreachable`) or allocates a huge
  vector; unmatched indices error at compose time.
- Preview URL kept until replacement decodes: the new blob URL is decoded before the
  old preview is swapped and revoked, so a failed decode leaves the previous preview
  and download link intact.
- Upload notices survive composition: upload rejections are listed in a dedicated
  `#upload-errors` region that compose no longer clears, so batch-limit and per-file
  messages persist across a successful compose.
- Dynamic wasm import + readiness guard: the demo loads `pkg/stitch_wasm.js` via
  `await import(...)` and refuses file drops until the module is ready, showing
  guidance instead of failing silently.

### Added

- `ComposeError::InvalidOutputBuffer` variant (`width`, `height`, `expected`, `actual`).
- Playwright CI suite (`web-tests/`) covering the demo's failure paths, run in the
  `web` job alongside the node tests.

## 0.5.0 — 2026-11-25

Distribution release: package rename, prebuilt binaries, and the first crates.io publish.
No engine, CLI, or WASM behavior change.

### Changed (breaking)

- Package renames: `stitch-core` → `stitch-me-up-core`, `stitch-cli` → `stitch-me-up`,
  `stitch-wasm` → `stitch-me-up-wasm`. Lib targets are unchanged (`stitch_core`,
  `stitch_wasm`), so `use stitch_core::…` and the generated `pkg/stitch_wasm.js` path
  are unaffected. Directory names stay `crates/stitch-*`.

### Added

- Prebuilt binaries for Linux (x86_64), macOS (aarch64), and Windows (x86_64), attached
  to GitHub Releases with `.sha256` checksums, built by `release.yml` on tag push.
- `release.yml` workflow: native builds on stable with `--locked`, `.tar.gz`/`.zip`
  archives containing the binary, README, and LICENSE.

### Fixed

- Clippy 1.100 `chunks_exact_to_as_chunks` lint in RGBA pixel iteration (no behavior
  change).
- Manifest warnings removed: `homepage` dropped from the workspace and every crate,
  `readme` dropped from `[workspace.package]` (each crate sets its own), and `js-sys`
  moved to wasm32-only dependencies.

## 0.4.2 — 2026-11-25

Release hygiene; no engine, CLI, or WASM behavior change.

### Changed

- MSRV corrected from 1.75 to 1.88: the locked dependency graph requires `image 0.25.10`
  (1.88), `clap` (1.85), and `wasm-bindgen` (1.81). CI now enforces the MSRV with a
  dedicated `msrv` job pinned to 1.88.0.
- Dependency bumps: clap 4.6.1 → 4.6.9, anyhow 1.0.102 → 1.0.104, thiserror 2.0.18 → 2.0.21.

### Documentation

- GitHub Releases backfilled for the existing `v0.4.0` and `v0.4.1` tags.
- crates.io distribution reality documented: `stitch-core` and `stitch-wasm` are held by
  unrelated projects, so nothing is published; distribution is source-only.
- The shipped 0.4 plan is retro-ticked and indexed; the release-hygiene plan is indexed.

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
