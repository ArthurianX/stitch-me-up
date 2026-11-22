# Per-image Pane Styles + 0.4.0 Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `0.4.0` with per-image frames/corner radii, adapter parity, refreshed docs, and a `v0.4.0` tag.

**Architecture:** Add `style: Option<PaneStyle>` to `InputImage`; rendering resolves it per pane and falls back to `ComposeOptions::pane_style`. The CLI and WASM adapters build complete per-pane styles from partial user input, so the core keeps whole-replacement semantics. Then bump the workspace to `0.4.0`, refresh docs, and tag.

**Tech Stack:** Rust 2021 (MSRV 1.75), `image`, `clap`, `wasm-bindgen`, Cargo workspace.

**Spec:** `docs/superpowers/specs/2026-11-22-per-image-pane-styles-design.md`

## Global Constraints

- No new dependencies; no float arithmetic in the engine; overflow-checked integer math everywhere.
- Decoration stays hard-edged (no antialiasing) and is drawn inside the placed rectangle; `CanvasPlan` and layout math do not change.
- Core per-image style is all-or-nothing: `Some(style)` replaces the global style for that pane.
- CLI `--pane-style` indices are **1-based**; WASM indices are **0-based**.
- Workspace version for this release is exactly `0.4.0`; no crates.io publish.
- Every task ends green: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`.
- Work happens on branch `release/0.4.0` created from clean `main`; tag only after the final verification.

## Review Focus

- `--pane-style` values contain `#`; docs must show quoted usage and the parser must accept `#` inside values.
- Index handling: `0` rejected, `1` is the first image, values beyond the image count rejected, repeated indices merge.
- Global frame off (`--frame-width 0`) but `--frame-color` set, then a pane enables only `frame-width`: the frame must use the global color.
- WASM style indices pointing past the input list must throw at compose, not be silently ignored.
- `Some(PaneStyle::default())` must deliberately strip global decoration from that pane.

---

### Task 1: Core per-image style

**Files:**
- Modify: `crates/stitch-core/src/types.rs:172-190`
- Modify: `crates/stitch-core/src/image_io.rs:8-27`
- Modify: `crates/stitch-core/src/render.rs:20-37`
- Modify: `crates/stitch-core/tests/helpers.rs:5-18`
- Test: `crates/stitch-core/tests/render_tests.rs`
- Test: `crates/stitch-core/tests/io_tests.rs`

**Interfaces:**
- Consumes: existing `PaneStyle`, `Frame`, `RgbaColor`, `blit_image(..., style: &PaneStyle)`.
- Produces:
  - `InputImage { pub width: u32, pub height: u32, pub pixels: Vec<u8>, pub style: Option<PaneStyle> }`
  - `impl InputImage { pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self }`
  - Rendering uses `image.style.as_ref().unwrap_or(&options.pane_style)` per placed pane.

- [ ] **Step 1: Create the release branch**

```bash
git switch --create release/0.4.0
```

- [ ] **Step 2: Add the breaking field and constructor**

In `types.rs`, append `pub style: Option<PaneStyle>` to `InputImage` and add:

```rust
impl InputImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Self {
            width,
            height,
            pixels,
            style: None,
        }
    }

    pub fn expected_len(&self) -> Option<usize> { /* unchanged */ }
}
```

Update both loaders in `image_io.rs` to return `InputImage::new(width, height, pixels)`. Update `tests/helpers.rs` `solid_image` to return `InputImage::new(width, height, pixels)`. Update the literal in `crates/stitch-core/README.md` in Task 5, not here.

- [ ] **Step 3: Write the failing render tests**

Add to `render_tests.rs`:

```rust
#[test]
fn global_style_applies_when_all_styles_are_none() {
    let images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame { width: 1, color: RgbaColor::WHITE }),
            corner_radius: 0,
        },
        RgbaColor { r: 0, g: 0, b: 0, a: 255 },
    );

    let composed = compose(&images, &options).expect("compose global frames");

    assert_eq!(pixel_at(&composed.pixels, composed.width, 0, 0), [255, 255, 255, 255]);
    assert_eq!(pixel_at(&composed.pixels, composed.width, 0, 4), [255, 255, 255, 255]);
}

#[test]
fn per_image_style_replaces_global_style() {
    let mut images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    images[0].style = Some(PaneStyle::default());
    let options = pane_options(
        PaneStyle {
            frame: Some(Frame { width: 1, color: RgbaColor::WHITE }),
            corner_radius: 0,
        },
        RgbaColor { r: 0, g: 0, b: 0, a: 255 },
    );

    let composed = compose(&images, &options).expect("compose mixed styles");

    assert_eq!(pixel_at(&composed.pixels, composed.width, 0, 0), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&composed.pixels, composed.width, 0, 4), [255, 255, 255, 255]);
}

#[test]
fn per_image_style_adds_decoration_to_one_pane() {
    let mut images = vec![
        solid_image(4, 4, [255, 0, 0, 255]),
        solid_image(4, 4, [0, 255, 0, 255]),
    ];
    images[1].style = Some(PaneStyle {
        frame: Some(Frame { width: 1, color: RgbaColor::WHITE }),
        corner_radius: 0,
    });
    let options = pane_options(
        PaneStyle::default(),
        RgbaColor { r: 0, g: 0, b: 0, a: 255 },
    );

    let composed = compose(&images, &options).expect("compose one decorated pane");

    assert_eq!(pixel_at(&composed.pixels, composed.width, 0, 0), [255, 0, 0, 255]);
    assert_eq!(pixel_at(&composed.pixels, composed.width, 0, 4), [255, 255, 255, 255]);
}
```

- [ ] **Step 4: Run the new tests and verify they fail**

Run: `cargo test -p stitch-core --test render_tests per_image`
Expected: `per_image_style_replaces_global_style` and `per_image_style_adds_decoration_to_one_pane` FAIL because rendering still applies the global style to every pane. `global_style_applies_when_all_styles_are_none` passes both before and after.

- [ ] **Step 5: Resolve the style per pane in `render.rs`**

Inside the `for placed in &plan.placed_images` loop, replace `&options.pane_style` with:

```rust
let style = source
    .style
    .as_ref()
    .unwrap_or(&options.pane_style);
blit_image(..., style)?;
```

No other render or layout code changes.

- [ ] **Step 6: Run the render tests**

Run: `cargo test -p stitch-core --test render_tests`
Expected: PASS, including the three new tests.

- [ ] **Step 7: Write the loader test**

Add to `io_tests.rs`:

```rust
#[test]
fn loaders_leave_style_unset() {
    let source = solid_image(2, 1, [255, 0, 0, 255]);
    let bytes = png_bytes(&source);

    let from_bytes = load_image_from_bytes(&bytes).expect("load bytes");
    assert_eq!(from_bytes.style, None);

    let path = temp_path("style.png");
    fs::write(&path, &bytes).expect("write png");
    let from_path = load_image_from_path(&path).expect("load path");
    assert_eq!(from_path.style, None);

    let _ = fs::remove_file(path);
}
```

- [ ] **Step 8: Run the full core suite**

Run: `cargo test -p stitch-core`
Expected: PASS (all unit, layout, render, I/O tests).

- [ ] **Step 9: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/stitch-core/src/types.rs crates/stitch-core/src/image_io.rs crates/stitch-core/src/render.rs crates/stitch-core/tests/helpers.rs crates/stitch-core/tests/render_tests.rs crates/stitch-core/tests/io_tests.rs
git commit -m "feat(core)!: per-image pane styles"
```

---

### Task 2: CLI `--pane-style`

**Files:**
- Modify: `crates/stitch-cli/src/lib.rs` (imports, `CombineArgs`, `combine`, helpers, tests)
- Test: `crates/stitch-cli/src/lib.rs` `#[cfg(test)]` module

**Interfaces:**
- Consumes: `InputImage.style`, `PaneStyle`, `Frame`, `RgbaColor::parse_hex`, `pane_style(...)`.
- Produces:
  - `CombineArgs::pane_styles: Vec<String>` (repeatable `--pane-style <SPEC>`)
  - `struct GlobalPaneStyle { pub frame_width: u32, pub frame_color: RgbaColor, pub corner_radius: u32 }`
  - `fn pane_style_overrides(specs: &[String], image_count: usize, global: GlobalPaneStyle) -> Result<Vec<Option<PaneStyle>>>`

- [ ] **Step 1: Write the failing parser and resolver tests**

Add to the CLI test module:

```rust
#[test]
fn parses_repeatable_pane_style_flags() {
    let cli = Cli::parse_from([
        "stitch-me-up", "combine",
        "--pane-style", "1:frame-width=4,frame-color=#11223344,corner-radius=7",
        "--pane-style", "3:corner-radius=2",
        "--output", "out.png", "a.png", "b.png", "c.png",
    ]);

    match cli.command {
        Commands::Combine(args) => assert_eq!(
            args.pane_styles,
            vec![
                "1:frame-width=4,frame-color=#11223344,corner-radius=7".to_string(),
                "3:corner-radius=2".to_string(),
            ]
        ),
    }
}

fn global_style() -> GlobalPaneStyle {
    GlobalPaneStyle {
        frame_width: 2,
        frame_color: RgbaColor::WHITE,
        corner_radius: 6,
    }
}

#[test]
fn pane_style_overrides_merge_with_global_fields() {
    let overrides =
        pane_style_overrides(&["1:frame-color=#11223344".into()], 2, global_style()).unwrap();

    assert_eq!(
        overrides[0],
        Some(PaneStyle {
            frame: Some(Frame {
                width: 2,
                color: RgbaColor { r: 0x11, g: 0x22, b: 0x33, a: 0x44 },
            }),
            corner_radius: 6,
        })
    );
    assert_eq!(overrides[1], None);
}

#[test]
fn pane_style_override_enables_frame_with_global_color() {
    let global = GlobalPaneStyle {
        frame_width: 0,
        frame_color: RgbaColor { r: 255, g: 0, b: 0, a: 255 },
        corner_radius: 0,
    };
    let overrides = pane_style_overrides(&["1:frame-width=3".into()], 2, global).unwrap();

    assert_eq!(
        overrides[0],
        Some(PaneStyle {
            frame: Some(Frame {
                width: 3,
                color: RgbaColor { r: 255, g: 0, b: 0, a: 255 },
            }),
            corner_radius: 0,
        })
    );
}

#[test]
fn pane_style_frame_width_zero_disables_global_frame() {
    let overrides =
        pane_style_overrides(&["1:frame-width=0".into()], 2, global_style()).unwrap();

    assert_eq!(
        overrides[0],
        Some(PaneStyle { frame: None, corner_radius: 6 })
    );
}

#[test]
fn pane_style_indices_are_one_based_and_range_checked() {
    let error = pane_style_overrides(&["0:corner-radius=1".into()], 2, global_style()).unwrap_err();
    assert_eq!(error.to_string(), "pane style index 0 is out of range for 2 images");

    let error = pane_style_overrides(&["3:corner-radius=1".into()], 2, global_style()).unwrap_err();
    assert_eq!(error.to_string(), "pane style index 3 is out of range for 2 images");
}

#[test]
fn pane_style_rejects_malformed_specs() {
    let cases = [
        ("1", "invalid pane style \"1\""),
        ("1:radius=2", "unknown pane style key \"radius\""),
        ("1:frame-width=x", "invalid frame-width value \"x\""),
        ("1:frame-color=zzzzzz", "invalid hex color component"),
    ];

    for (spec, expected) in cases {
        let error = pane_style_overrides(&[spec.into()], 2, global_style()).unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{spec}: {error:#}");
    }
}

#[test]
fn pane_style_last_repeated_key_wins() {
    let overrides = pane_style_overrides(
        &["1:corner-radius=2,corner-radius=9".into()],
        2,
        global_style(),
    )
    .unwrap();

    assert_eq!(
        overrides[0],
        Some(PaneStyle {
            frame: Some(Frame { width: 2, color: RgbaColor::WHITE }),
            corner_radius: 9,
        })
    );
}

#[test]
fn pane_style_repeated_index_merges() {
    let overrides = pane_style_overrides(
        &["1:frame-width=9".into(), "1:corner-radius=3".into()],
        2,
        global_style(),
    )
    .unwrap();

    assert_eq!(
        overrides[0],
        Some(PaneStyle {
            frame: Some(Frame { width: 9, color: RgbaColor::WHITE }),
            corner_radius: 3,
        })
    );
}

#[test]
fn pane_style_color_without_width_is_accepted_and_inert() {
    let global = GlobalPaneStyle {
        frame_width: 0,
        frame_color: RgbaColor::WHITE,
        corner_radius: 0,
    };
    let overrides =
        pane_style_overrides(&["1:frame-color=#ff0000".into()], 2, global).unwrap();

    assert_eq!(
        overrides[0],
        Some(PaneStyle { frame: None, corner_radius: 0 })
    );
}
```

Also add `assert!(args.pane_styles.is_empty());` to `parses_minimal_combine_command`, and add `pane_styles: Vec::new(),` to the two literal `CombineArgs { ... }` values in `rejects_before_after_with_more_than_two_inputs` and `rejects_grid_with_zero_columns`.

- [ ] **Step 2: Run the tests and verify they fail**

Run: `cargo test -p stitch-cli pane_style`
Expected: FAIL to compile — `pane_styles`, `GlobalPaneStyle`, and `pane_style_overrides` do not exist.

- [ ] **Step 3: Add the flag and helper types**

In `CombineArgs`, after `column_weights`:

```rust
#[arg(
    long = "pane-style",
    value_name = "INDEX:KEY=VALUE[,...]",
    help = "Per-image decoration override (1-based, repeatable), e.g. 1:frame-width=4,frame-color=#ff0000"
)]
pub pane_styles: Vec<String>,
```

In the module (private):

```rust
#[derive(Debug, Clone, Copy)]
pub struct GlobalPaneStyle {
    pub frame_width: u32,
    pub frame_color: RgbaColor,
    pub corner_radius: u32,
}
```

- [ ] **Step 4: Implement `pane_style_overrides`**

Signature exactly:

```rust
pub fn pane_style_overrides(
    specs: &[String],
    image_count: usize,
    global: GlobalPaneStyle,
) -> Result<Vec<Option<PaneStyle>>>
```

Algorithm:
1. `let mut fields = vec![StyleFields::default(); image_count];` where `StyleFields` is a private `Default` struct with `frame_width: Option<u32>`, `frame_color: Option<RgbaColor>`, `corner_radius: Option<u32>` (derive `Clone` to fill the vec).
2. For each spec, `split_once(':')`; on failure `bail!("invalid pane style \"{spec}\": expected <index>:<key>=<value>[,<key>=<value>...]")`.
3. Parse the index as `usize`; on failure use the same malformed message. Reject `index == 0 || index > image_count` with `bail!("pane style index {index} is out of range for {image_count} images")`.
4. For each comma-separated assignment, `split_once('=')`; on failure use the malformed message. Match trimmed keys:
   - `frame-width` → `u32::from_str(value).with_context(|| format!("invalid frame-width value \"{value}\""))?`
   - `corner-radius` → same with `corner-radius`
   - `frame-color` → `RgbaColor::parse_hex(value).with_context(|| format!("invalid frame-color in pane style \"{spec}\""))?`
   - other → `bail!("unknown pane style key \"{key}\"; expected frame-width, frame-color, or corner-radius")`
   Repeated keys overwrite, so later wins.
5. Resolve each touched index: effective width = override or `global.frame_width`; effective color = override or `global.frame_color`; effective radius = override or `global.corner_radius`; produce `Some(PaneStyle { frame: (width > 0).then_some(Frame { width, color }), corner_radius })`. Untouched indices stay `None`.

- [ ] **Step 5: Wire overrides into `combine`**

Before loading images:

```rust
let overrides = pane_style_overrides(
    &args.pane_styles,
    args.inputs.len(),
    GlobalPaneStyle {
        frame_width: args.frame_width,
        frame_color: args.frame_color,
        corner_radius: args.corner_radius,
    },
)?;
```

After `let images = ...collect::<Result<Vec<_>>>()?;`, make it `let mut images`, then:

```rust
for (image, style) in images.iter_mut().zip(overrides) {
    image.style = style;
}
```

The existing global `pane_style(args.frame_width, args.frame_color, args.corner_radius)` stays unchanged.

- [ ] **Step 6: Run the CLI tests**

Run: `cargo test -p stitch-cli`
Expected: PASS.

- [ ] **Step 7: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/stitch-cli/src/lib.rs
git commit -m "feat(cli): per-image --pane-style overrides"
```

---

### Task 3: WASM per-index styles

**Files:**
- Modify: `crates/stitch-wasm/src/lib.rs` (imports, `Composer`, wasm impl, shared impl, `compose`, tests)

**Interfaces:**
- Consumes: `InputImage::new`, `InputImage.style`, `PaneStyle`, `compose`, `encode_png`, `load_image_from_bytes`.
- Produces:
  - `Composer::pane_styles: Vec<Option<PaneStyle>>`
  - `fn apply_pane_style(&mut self, index: usize, frame_width: u32, frame_color: &str, corner_radius: u32) -> Result<(), String>`
  - `fn apply_clear_pane_style(&mut self, index: usize)`
  - `fn apply_pane_styles(&self, images: &mut [InputImage]) -> Result<(), String>`
  - wasm exports `set_pane_style(index, frameWidth, frameColor, cornerRadius)` and `clear_pane_style(index)`.

- [ ] **Step 1: Write the failing native tests**

Add to the wasm test module (and extend `composer_defaults_match_core_defaults` with `assert!(Composer::default().pane_styles.is_empty());`):

```rust
#[test]
fn stores_and_clears_pane_styles() {
    let mut composer = Composer::default();
    composer.apply_pane_style(1, 4, "#ff0000", 8).expect("set style");

    assert_eq!(composer.pane_styles.len(), 2);
    assert_eq!(composer.pane_styles[0], None);
    assert_eq!(
        composer.pane_styles[1],
        Some(PaneStyle {
            frame: Some(Frame {
                width: 4,
                color: RgbaColor { r: 255, g: 0, b: 0, a: 255 },
            }),
            corner_radius: 8,
        })
    );

    composer.apply_clear_pane_style(1);
    assert_eq!(composer.pane_styles[1], None);
}

#[test]
fn applies_pane_styles_by_index() {
    let mut composer = Composer::default();
    composer.apply_pane_style(1, 0, "#00ff00", 5).expect("set style");

    let mut images = vec![
        InputImage::new(1, 1, vec![255, 0, 0, 255]),
        InputImage::new(1, 1, vec![0, 255, 0, 255]),
    ];
    composer.apply_pane_styles(&mut images).expect("apply styles");

    assert_eq!(images[0].style, None);
    assert_eq!(
        images[1].style,
        Some(PaneStyle { frame: None, corner_radius: 5 })
    );
}

#[test]
fn rejects_pane_style_for_missing_image() {
    let mut composer = Composer::default();
    composer.apply_pane_style(2, 1, "#ffffff", 0).expect("set style");

    let mut images = vec![
        InputImage::new(1, 1, vec![0, 0, 0, 255]),
        InputImage::new(1, 1, vec![0, 0, 0, 255]),
    ];
    let error = composer.apply_pane_styles(&mut images).unwrap_err();

    assert_eq!(error, "pane style index 2 has no matching image");
}

#[test]
fn rejects_invalid_pane_style_color() {
    let mut composer = Composer::default();
    let error = composer
        .apply_pane_style(0, 2, "zzzzzz", 0)
        .unwrap_err();

    assert!(error.contains("invalid hex color component"));
    assert!(composer.pane_styles.is_empty());
}
```

- [ ] **Step 2: Run the wasm tests and verify they fail**

Run: `cargo test -p stitch-wasm`
Expected: FAIL to compile — `pane_styles`, `apply_pane_style`, `apply_clear_pane_style`, `apply_pane_styles` do not exist.

- [ ] **Step 3: Add state, setters, and helpers**

- Import `PaneStyle` **unconditionally** (the `Composer` field is compiled on every target), e.g. change the existing `use stitch_core::ComposeOptions;` to `use stitch_core::{ComposeOptions, PaneStyle};`.
- Add `InputImage` to the `#[cfg(any(target_arch = "wasm32", test))]` import block, and add `InputImage` and `PaneStyle` to the test module's `use stitch_core::{...}` list.
- Add to `Composer`:

```rust
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pane_styles: Vec<Option<PaneStyle>>,
```

- In the wasm-only `#[wasm_bindgen] impl Composer`, add:

```rust
/// 0-based; stores a complete override for that input. `frame_width == 0` disables the frame.
pub fn set_pane_style(
    &mut self,
    index: usize,
    frame_width: u32,
    frame_color: &str,
    corner_radius: u32,
) -> Result<(), JsError> {
    self.apply_pane_style(index, frame_width, frame_color, corner_radius)
        .map_err(js_error)
}

/// Removes the override at `index`; the pane inherits the global style again.
pub fn clear_pane_style(&mut self, index: usize) {
    self.apply_clear_pane_style(index);
}
```

- In the shared impl, implement:

```rust
fn apply_pane_style(
    &mut self,
    index: usize,
    frame_width: u32,
    frame_color: &str,
    corner_radius: u32,
) -> Result<(), String> {
    let color = parse_color_hex(frame_color)?;
    if self.pane_styles.len() <= index {
        self.pane_styles.resize(index + 1, None);
    }
    self.pane_styles[index] = Some(PaneStyle {
        frame: (frame_width > 0).then_some(Frame { width: frame_width, color }),
        corner_radius,
    });
    Ok(())
}

fn apply_clear_pane_style(&mut self, index: usize) {
    if let Some(slot) = self.pane_styles.get_mut(index) {
        *slot = None;
    }
}

fn apply_pane_styles(&self, images: &mut [InputImage]) -> Result<(), String> {
    for (index, style) in self.pane_styles.iter().enumerate() {
        if let Some(style) = style {
            let image = images
                .get_mut(index)
                .ok_or_else(|| format!("pane style index {index} has no matching image"))?;
            image.style = Some(style.clone());
        }
    }
    Ok(())
}
```

- [ ] **Step 4: Apply styles during compose**

Replace the wasm `compose` body's `compose_to_png(&buffers, &self.options)` path. Under the wasm import block use `compose`, `encode_png`, and `load_image_from_bytes` instead of `compose_to_png`:

```rust
let mut decoded = buffers
    .iter()
    .map(|bytes| load_image_from_bytes(bytes))
    .collect::<Result<Vec<_>, _>>()
    .map_err(js_error)?;

self.apply_pane_styles(&mut decoded).map_err(js_error)?;
let composed = compose(&decoded, &self.options).map_err(js_error)?;
encode_png(&composed).map_err(js_error)
```

Keep the existing `Uint8Array` validation and return type.

- [ ] **Step 5: Run the wasm tests and wasm target check**

```bash
cargo test -p stitch-wasm
cargo check -p stitch-wasm --target wasm32-unknown-unknown
```

Expected: both PASS.

- [ ] **Step 6: Format, lint, commit**

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
git add crates/stitch-wasm/src/lib.rs
git commit -m "feat(wasm): per-index pane styles"
```

---

### Task 4: Remove dead public surface

**Files:**
- Delete: `crates/stitch-core/src/options.rs`
- Modify: `crates/stitch-core/src/lib.rs:1-14`
- Modify: `crates/stitch-core/src/error.rs:8-9`
- Modify: `docs/api.md` error table (`UnsupportedLayout` row)

**Interfaces:**
- Produces: no `stitch_core::options` module, one `ComposeOptions`/`PaneStyle` re-export path (`crate::types`), no `ComposeError::UnsupportedLayout`.

- [ ] **Step 1: Remove the shim and duplicate re-export**

Delete `crates/stitch-core/src/options.rs`. In `lib.rs`, remove `pub mod options;` and `pub use options::{ComposeOptions, PaneStyle};`. Keep the `types` re-export that already includes both names.

- [ ] **Step 2: Remove the dead error variant**

Delete the `UnsupportedLayout` variant from `ComposeError`. Confirm no Rust code references it:

```bash
rg "UnsupportedLayout"
```

Expected: only `docs/api.md` remains.

- [ ] **Step 3: Update the API docs table**

Delete the `UnsupportedLayout` row from `docs/api.md`.

- [ ] **Step 4: Verify the workspace**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Expected: all PASS.

- [ ] **Step 5: Commit**

```bash
git add -A crates/stitch-core docs/api.md
git commit -m "refactor(core)!: drop options shim and unused error"
```

---

### Task 5: Documentation and READMEs

**Files:**
- Modify: `README.md`
- Modify: `ROADMAP.md`
- Modify: `docs/api.md`
- Modify: `docs/cli.md`
- Modify: `docs/layouts.md`
- Modify: `docs/wasm.md`
- Modify: `crates/stitch-core/README.md`
- Modify: `crates/stitch-cli/README.md`
- Modify: `crates/stitch-wasm/README.md`

**Interfaces:**
- Consumes: the behavior implemented in Tasks 1–3 and the final error strings.
- Produces: docs that describe the code.

- [ ] **Step 1: Update `docs/api.md`**

- In the `InputImage` block, add `pub style: Option<PaneStyle>`, and `pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self;` (style `None`).
- Add a paragraph: `None` inherits `ComposeOptions::pane_style`; `Some(style)` replaces it wholesale for that pane, so `Some(PaneStyle::default())` renders a plain pane. The loaders always return `None`. Planning ignores style.
- In the `ComposeOptions` defaults paragraph or behavior notes, state that `pane_style` is the global fallback.
- Add a behavior bullet: "Per-image styles are resolved at render time; they never change pane geometry."

- [ ] **Step 2: Update `docs/cli.md`**

- Add the flag table row:

```markdown
| `--pane-style <SPEC>` | unset | Per-image decoration override, repeatable. `INDEX` is 1-based. `SPEC` is `INDEX:KEY=VALUE[,KEY=VALUE...]` with keys `frame-width`, `frame-color`, `corner-radius`. |
```

- In value rules, explain: overrides start from the global style fields, key order does not matter, repeated index/key means later wins, `frame-width=0` disables the frame, and `frame-color` alone with global width 0 is accepted but unused.
- Document errors: out-of-range index, unknown key, bad number/color.
- Add an example:

```bash
stitch-me-up combine --layout grid --columns 2 \
  --frame-width 4 --corner-radius 12 \
  --pane-style 1:frame-color=#f59e0b,corner-radius=20 \
  --pane-style 4:frame-width=0 \
  --output grid.png a.png b.png c.png d.png
```

- [ ] **Step 3: Update `docs/wasm.md`**

- Add setter rows:

```markdown
| `set_pane_style(index, frameWidth, frameColorHex, cornerRadius)` | 0-based input index; stores a complete override. `frameWidth == 0` disables that pane's frame while still validating the color. |
| `clear_pane_style(index)` | Removes the override so the pane inherits the global style. |
```

- Note in `compose`: stored overrides are applied by index; an index with no matching input throws `pane style index N has no matching image`.
- Add the error string to the error list.

- [ ] **Step 4: Update `docs/layouts.md`**

In "How decoration interacts", add a bullet: per-image `InputImage::style` (`--pane-style`) overrides the global frame/radius for that pane only; geometry and canvas size are unchanged.

- [ ] **Step 5: Update `README.md` and `ROADMAP.md`**

- README "Current Features": add per-image pane styles with `InputImage::style` and `--pane-style`.
- README "Decoration": add a short per-image override paragraph and one CLI line.
- README "Status": say the workspace is at `0.4.0`.
- ROADMAP: tick `Per-image style overrides (new input shape; breaking change)`.

- [ ] **Step 6: Update crate READMEs**

- `crates/stitch-core/README.md`: the example constructs `InputImage` directly; switch to `InputImage::new(w, h, pixels)` and add one sentence about `style`.
- `crates/stitch-cli/README.md`: add a short per-pane example.
- `crates/stitch-wasm/README.md`: delete the stale sentence saying the web example "arrives in the next roadmap task"; add `set_pane_style` to the JS example.

- [ ] **Step 7: Check for stale prose and broken links**

```bash
rg -n "arrives in the next|not implemented yet|UnsupportedLayout" README.md ROADMAP.md docs crates/*/README.md
```

Expected: no matches.

- [ ] **Step 8: Verify and commit**

```bash
cargo test --workspace
cargo fmt --all -- --check
git add README.md ROADMAP.md docs crates/stitch-core/README.md crates/stitch-cli/README.md crates/stitch-wasm/README.md
git commit -m "docs: document per-image styles and 0.4.0"
```

---

### Task 6: Release 0.4.0

**Files:**
- Modify: `Cargo.toml` (workspace version)
- Modify: `crates/stitch-cli/Cargo.toml` (`stitch-core` pin)
- Modify: `crates/stitch-wasm/Cargo.toml` (`stitch-core` pin)
- Modify: `Cargo.lock` (generated)
- Create: `CHANGELOG.md`
- Modify: `README.md` only if the status line still says an older version

**Interfaces:**
- Produces: workspace version `0.4.0`, annotated tag `v0.4.0`.

- [ ] **Step 1: Bump the workspace version**

In root `Cargo.toml`, set `version = "0.4.0"` under `[workspace.package]`. In `crates/stitch-cli/Cargo.toml` and `crates/stitch-wasm/Cargo.toml`, set the `stitch-core` dependency to `version = "0.4.0"`.

- [ ] **Step 2: Refresh the lockfile**

Run: `cargo check --workspace`
Expected: success; `Cargo.lock` now lists `stitch-core`, `stitch-cli`, `stitch-wasm` at `0.4.0`.

- [ ] **Step 3: Write `CHANGELOG.md`**

Use this structure and content:

```markdown
# Changelog

All notable changes to this project are documented here.

## 0.4.0 — 2026-11-22

First tagged release; the 0.1–0.4 milestones were developed before this tag.

### Added

- Per-image pane styles: `InputImage::style` overrides the global `ComposeOptions::pane_style`.
- CLI `--pane-style <INDEX>:<KEY>=<VALUE>[,...]` for 1-based per-image overrides.
- WASM `set_pane_style` / `clear_pane_style` per input index.
- Per-axis spacing (`Spacing { x, y }`, `--spacing-x` / `--spacing-y`).
- Grid column weights (`column_weights`, `--column-weights`).
- Per-pane frames, corner radii, and the before-after divider.
- `stitch-wasm` WebAssembly adapter and browser example.
- API, CLI, layout, and WASM documentation.

### Changed (breaking)

- `ComposeOptions::spacing` is `Spacing`; `Spacing::all(n)` reproduces the old uniform value.
- `InputImage` gained `style: Option<PaneStyle>`; construct with `InputImage::new`.
- Dead `options` re-export shim and `ComposeError::UnsupportedLayout` removed.
```

- [ ] **Step 4: Final full verification**

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo check -p stitch-wasm --target wasm32-unknown-unknown
```

Expected: all PASS, zero warnings.

- [ ] **Step 5: Commit the release**

```bash
git add Cargo.toml Cargo.lock crates/stitch-cli/Cargo.toml crates/stitch-wasm/Cargo.toml CHANGELOG.md README.md
git commit -m "chore(release): v0.4.0"
```

- [ ] **Step 6: Merge to main and tag**

```bash
git switch main
git merge --ff-only release/0.4.0
git tag -a v0.4.0 -m "v0.4.0"
git push origin main
git push origin v0.4.0
```

Expected: fast-forward merge, tag created and pushed; CI starts on `main`.

- [ ] **Step 7: Report**

Print `git log --oneline -6`, `git tag --points-at HEAD`, and the final test counts. The release is ready for the user to test.
