# Real-Photo Pass (EXIF + JPEG Output) — Implementation Plan

> **For agentic workers:** test-first; every task ends green.

**Goal:** Decode images upright from EXIF, add JPEG output (PNG stays the default) as a
parameter in the CLI, WASM and web demo, and release 0.6.0.

**Architecture:** One orientation-aware decode helper shared by both core loaders; a
validated JPEG encoder that flattens onto white; thin adapters that pass a format string
(`png`|`jpeg`) and quality through.

**Tech Stack:** Rust, `image 0.25` (already enables jpeg+png), clap 4, wasm-bindgen,
vanilla ES modules, Playwright.

**Spec:** `docs/superpowers/specs/2026-11-26-real-photo-pass-design.md`

## Global Constraints

- No new dependencies; MSRV 1.88; existing 112 Rust tests and 20 node tests stay green.
- PNG remains the default at every boundary; existing PNG output is byte-identical.
- Quality is `1..=100`, never clamped; invalid values are errors.
- JPEG transparency is flattened onto white with integer rounding.
- No EXIF metadata is written to output; orientation is applied physically.

## Task 1 — Orientation-aware loading

**Files:** `crates/stitch-core/src/image_io.rs`, `crates/stitch-core/tests/io_tests.rs`

Build a small 2×2/2×3 JPEG in-test with `JpegEncoder`, call
`set_exif_metadata(tiff)` where `tiff` is a minimal little-endian IFD with one entry
(`0x0112`, SHORT, count 1, value 6 = Rotate90); the encoder prepends `Exif\0\0` itself.
Tests: `load_image_from_bytes_applies_exif_orientation` (dimensions swap 2×3 → 3×2 and
corner colours match a `rotate90` of the source within JPEG tolerance),
`load_image_from_path_applies_exif_orientation`, and an unwrapped JPEG keeps its
dimensions. Implement a shared helper that does
`ImageReader::new(..).with_guessed_format()?.into_decoder()?`, reads `orientation()`,
builds `DynamicImage::from_decoder`, then `apply_orientation`.

## Task 2 — JPEG encoding

**Files:** `crates/stitch-core/src/image_io.rs`, `error.rs`, `crates/stitch-core/tests/io_tests.rs`

Add `ComposeError::InvalidJpegQuality { quality }` and
`encode_jpeg(&ComposedImage, quality)`, `save_jpeg(path, &ComposedImage, quality)`.
Validate quality then the buffer (reuse `validate_output_buffer`), flatten
`(c*a + 255*(255-a) + 127)/255`, encode `Rgb8` via `JpegEncoder::new_with_quality`.
Tests: `encode_jpeg_rejects_quality_zero_and_101`, `encode_jpeg_roundtrips_dimensions`,
`encode_jpeg_flattens_transparency_onto_white` (background `#ffffff00` → decoded pixels
≥ 250 in every channel), `encode_jpeg_rejects_invalid_buffer`, and `FF D8` magic.

## Task 3 — CLI format parameter

**Files:** `crates/stitch-cli/src/lib.rs`

Add `OutputFormatArg { Png, Jpeg }` (`ValueEnum`) with `--format` default `png`, and
`--quality <1..=100>` as `Option<u8>` parsed with a range. `--quality` with `--format png`
is an error: `--quality only applies to --format jpeg`. Write via `save_png`/`save_jpeg`
and keep `Wrote <path>`. Tests: default invocation still parses as PNG; `--format jpeg
--quality 80` parses; quality-with-png error text; an end-to-end run writes `FF D8`.

## Task 4 — WASM format parameter

**Files:** `crates/stitch-wasm/src/lib.rs`

Add `compose_with_format(images, format, quality)`; `format` is case-insensitive `png`
or `jpeg`; `compose` remains PNG. Unknown format throws
`unknown output format "{format}"; expected png or jpeg`; quality is validated only for
jpeg. Factor the shared decode-and-compose path out of `compose` so behaviour cannot
drift. Native tests cover format parsing, the unknown-format text, and quality errors.

## Task 5 — Web output switch

**Files:** `examples/web/index.html`, `app.js`, `lib.js`, `styles.css`,
`examples/web/tests/lib.test.js`, `web-tests/browser.spec.js`

Add `#format` (`png` default, `jpeg`) and `#quality` (1–100, default 90, disabled for
PNG). Compose calls `compose` or `compose_with_format`; blob MIME, `downloadName(date,
ext)`, preview and meta follow the format. `lib.js`: `downloadName(date, ext = "png")`
and `isValidQuality(value)`; keep the existing filename test and add extension/quality
cases. Playwright: selecting JPEG yields an `image/jpeg` preview and a `.jpg` download
name; PNG remains the default.

## Task 6 — Docs, version and release (separate gated dispatch)

`docs/api.md`, `docs/cli.md`, `docs/wasm.md`, `README.md`, `ROADMAP.md` 0.6 section,
`CHANGELOG.md` 0.6.0. Then full gates, fast-forward merge, push, watch CI/Pages, verify
the live JPEG switch, tag `v0.6.0`, release with six assets, publish core → wasm → CLI,
install check.

## Acceptance criteria

- EXIF-tagged JPEGs load upright through both loaders; untagged images are unchanged.
- PNG default everywhere; `--format jpeg` / `compose_with_format(..., "jpeg", q)` /
  the demo switch produce valid JPEGs; invalid quality is an error, never a clamp.
- Transparent canvases become white in JPEG, never black or raw RGB.
- Existing tests, node tests and the Playwright suite stay green; new cases cover
  orientation, quality bounds, flattening, CLI parsing, and the demo switch.
