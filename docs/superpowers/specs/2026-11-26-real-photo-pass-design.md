# Real-Photo Pass: EXIF Orientation + JPEG Output — Design

**Date:** 2026-11-26
**Status:** awaiting approval
**Target:** 0.6.0 (behaviour change + additions)

## Intent

Make the engine and the demo behave like a photo tool on real phone photos:
decode them upright, and offer a JPEG download that is a practical size. Today every
load ignores EXIF orientation (portrait shots come out sideways) and every output is
PNG (a 25 MP collage is routinely 20–60 MB).

## Goals

- Apply EXIF orientation on every image load, for both the path and bytes loaders, so
  decoded pixels are upright and 90°/270° orientations swap width and height.
- JPEG output: `encode_jpeg(&ComposedImage, quality)` and `save_jpeg(path, …, quality)`
  in core; `--format png|jpeg` and `--quality` in the CLI; `compose_jpeg(images, quality)`
  in WASM; a Format selector and quality field in the web demo.
- Define transparency for JPEG: flatten RGBA onto white with integer rounding.
- Document the change and add regression tests, including a JPEG with an EXIF
  orientation tag built inside the tests.

## Non-goals

- WebP/AVIF (not supported by the current encoder set without new work).
- Preserving EXIF metadata in output (orientation is applied physically; no metadata
  is written).
- Layout, decoration, per-image style, caption, or demo redesign changes.
- Any new dependency.

## Constraints

- `image 0.25` API only: `ImageReader::into_decoder`, `ImageDecoder::orientation`,
  `DynamicImage::from_decoder`, `DynamicImage::apply_orientation`, `JpegEncoder`.
- MSRV stays 1.88; no float math in our own transforms; JPEG encoding is lossy but
  deterministic for a fixed encoder version.
- PNG remains the default everywhere; existing outputs are unchanged for images
  without EXIF orientation.
- `quality` is 1..=100; invalid values are errors, not clamped.

## Approaches considered

1. **Orientation: always apply in the loaders** (chosen) vs an opt-in flag. Browsers
   show photos upright, and a flag that defaults to “wrong” is a footgun; the engine
   should match user expectation. This is the 0.6.0 breaking bit.
2. **JPEG: use the built-in encoder with an explicit flatten** (chosen) vs adding a
   `mozjpeg`/`webp` dependency (rejected: new dependency) vs keeping PNG-only
   (rejected: file sizes). `JpegEncoder` accepts `Rgba8` but silently drops alpha, which
   would turn transparent backgrounds into the raw RGB values — so we flatten first.
3. **WASM surface: `compose_jpeg` alongside `compose`** (chosen) vs a generic
   format/quality method. One extra method is a smaller, clearer API than a polymorphic
   one, and `compose` keeps returning PNG unchanged.

## Design

### Orientation-aware loading (`crates/stitch-core/src/image_io.rs`)

Shared helper used by both loaders:

```text
reader = ImageReader::new(...).with_guessed_format()?
decoder = reader.into_decoder()?
orientation = decoder.orientation()?          // NoTransforms when absent
image = DynamicImage::from_decoder(decoder)?
image.apply_orientation(orientation)
rgba = image.to_rgba8()
```

Path loads wrap `File`; byte loads wrap `Cursor<&[u8]>`. Errors stay `ImageIo`.

### JPEG output (`crates/stitch-core/src/image_io.rs`, `error.rs`)

- `validate_output_buffer` is reused for length checks.
- `flatten_to_rgb(pixels) -> Vec<u8>`: `rgb = (c*a + 255*(255-a) + 127) / 255` per
  channel, i.e. composite over white.
- `encode_jpeg(image, quality)`: validate quality (1..=100, else
  `ComposeError::InvalidJpegQuality { quality }`), validate buffer, flatten, encode
  with `JpegEncoder::new_with_quality(..).write_image(.., ExtendedColorType::Rgb8)`.
- `save_jpeg(path, image, quality)`: same validation, then write the encoded bytes.

### CLI (`crates/stitch-cli`)

- `--format <png|jpeg>` (default `png`), `--quality <1..=100>` (default `90`).
- `--quality` together with `--format png` is an error: `--quality only applies to
  --format jpeg` (track the flag as `Option<u8>` to distinguish “not given”).
- Output is written according to `--format`; the file extension is the user’s choice.

### WASM (`crates/stitch-wasm/src/lib.rs`)

- `compose_with_format(images: Array, format: &str, quality: u8) -> Result<Vec<u8>, JsError>`:
  `format` is `png` (default behaviour, quality ignored) or `jpeg`; same decoding and
  pane-style application as `compose`. `compose` stays as the PNG-only entry point.
- An unknown format throws `unknown output format "{format}"; expected png or jpeg`;
  quality outside 1..=100 throws the Rust error text when the format is jpeg.

### Web demo (`examples/web`)

- `#format` select (`png` default, `jpeg`) and `#quality` number input (1–100, default
  90, disabled unless JPEG is selected).
- Compose calls `compose` or `compose_jpeg` accordingly; blob MIME and download
  extension follow the format (`downloadName(date, ext)`), preview and meta size update.
- `lib.js` gains `downloadName(date, ext)` and `isValidQuality(value)` with node tests;
  the Playwright suite gains a JPEG download case.

### Docs

`docs/api.md` (load orientation, JPEG functions, new error), `docs/cli.md` (flags and
error text), `docs/wasm.md` (`compose_jpeg`), `README.md` features, `ROADMAP.md` 0.6
section and decision log, `CHANGELOG.md` 0.6.0.

## Calls made without asking

- White is the JPEG matte; no configurable matte in this pass.
- Quality defaults to 90 in both CLI and demo.
- Orientation is applied to every format, not just JPEG; PNG has no orientation and is
  unaffected.
- `compose` keeps its PNG contract; JPEG is additive.
