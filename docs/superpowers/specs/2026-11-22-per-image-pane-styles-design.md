# Per-image pane styles (0.4) — design

## Intent

Finish the last 0.4 roadmap item: let each input image carry its own decoration
(frame and corner radius) while the existing `ComposeOptions::pane_style` remains the
global default. Then cut the first tagged release, `0.4.0`.

Why: a collage often wants one pane highlighted, or a logo left square inside a grid of
rounded photos. Today decoration is all-or-nothing.

## Goals

- `stitch-core`: per-image style, no change to layout math or `CanvasPlan`.
- CLI: per-pane override flags that merge over the global style flags.
- WASM: per-index style setters with the same semantics.
- Docs and READMEs updated; stale wasm README note removed.
- Release `0.4.0`: workspace version, `CHANGELOG.md`, annotated `v0.4.0` tag.

## Non-goals

- Partial merge inside the core: a per-image style is a complete `PaneStyle`.
- Per-pane divider, background, spacing, padding, or alignment.
- Antialiasing; decoration stays hard-edged.
- Publishing to crates.io (explicitly out of this round).
- New dependencies.

## Anti-goals

- Parallel style arrays positionally matched to images.
- Silently ignoring a style that points at a missing pane.
- Touching `plan_layout` output for existing options.

## Chosen approach

**A: style on the input.** Add `style: Option<PaneStyle>` to `InputImage`:

```rust
pub struct InputImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    /// Overrides `ComposeOptions::pane_style` for this pane; `None` inherits the global style.
    pub style: Option<PaneStyle>,
}

impl InputImage {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self;
}
```

`new` sets `style: None`. The PNG/JPEG loaders do the same. Rendering resolves
`image.style.as_ref().unwrap_or(&options.pane_style)` per placed pane and passes the
result to the existing `blit_image` style parameter. Planning is untouched.

### Rejected alternatives

- **B: `StyledImage { image, style }` wrapper.** Cleaner separation, but breaks every
  call site, adds a second layer in every adapter, and makes the common undecorated
  case noisier. No mismatch protection beyond what A already gives.
- **C: `ComposeOptions::pane_styles: Vec<Option<PaneStyle>>`.** Smallest diff, but
  positional, needs count validation, and does not match the roadmap's "new input
  shape" decision.

## Semantics

- `InputImage::style = None` → use `ComposeOptions::pane_style`.
- `InputImage::style = Some(style)` → that pane uses exactly `style`; global decoration
  is not merged in. `Some(PaneStyle::default())` therefore means "plain pane".
- Two images may share or differ; the divider is global and unaffected.
- Existing output is byte-identical when every input has `style: None`.

## CLI

Repeatable flag:

```text
--pane-style <INDEX>:<KEY>=<VALUE>[,<KEY>=<VALUE>...]
```

- `INDEX` is **1-based**, matching "first image".
- Keys: `frame-width` (u32), `frame-color` (`#RRGGBB`/`#RRGGBBAA`), `corner-radius` (u32).
- Keys are parsed into independent fields (`frame_width`, `frame_color`,
  `corner_radius`), then resolved against the global style; **key order never matters**.
  Effective width/color come from the override when present, otherwise from the global
  flags. A frame is emitted only when the effective width is greater than `0`.
- `frame-width=0` disables the frame for that pane. `frame-color` alone updates the
  color but keeps the global width, so with the default `--frame-width 0` it is accepted,
  validated, and unused — exactly like global `--frame-color` without `--frame-width`.
  `frame-width=4` alone creates a frame using the global `--frame-color` (default
  `#000000`).
- This is field-level inheritance at the CLI boundary; core still receives a complete
  `PaneStyle`.
- Repeating the same index merges, later keys win.
- Validation runs after inputs are parsed and before loading:
  - index `0` or greater than the input count → `pane style index N is out of range for M images`
  - malformed spec, unknown key, bad number, or bad color → one `bail!` message naming
    the expected grammar or the offending key. Repeated keys are allowed; later wins.
- Panes without `--pane-style` keep `style: None`.

Example:

```bash
stitch-me-up combine --layout grid --columns 2 \
  --frame-width 4 --corner-radius 12 \
  --pane-style 1:frame-color=#f59e0b,corner-radius=20 \
  --pane-style 4:frame-width=0 \
  --output grid.png a.png b.png c.png d.png
```

## WASM

```js
composer.set_pane_style(index, frameWidth, frameColorHex, cornerRadius); // 0-based
composer.clear_pane_style(index);
```

- `set_pane_style` validates the color immediately, stores a complete `PaneStyle`
  override at `index` (0-based, matching JS arrays); `frameWidth == 0` disables the frame.
- `clear_pane_style` removes an override so the pane inherits the global style again.
- `compose` applies stored overrides to the decoded images by index. If any stored
  index has no matching image it throws
  `pane style index N has no matching image`.
- Shared logic lives in a target-agnostic method compiled for wasm and native tests.

## Release plan (0.4.0)

1. Workspace version `0.4.0`; update `stitch-core` dependency pins in `stitch-cli` and
   `stitch-wasm`; let Cargo refresh `Cargo.lock`.
2. `CHANGELOG.md` with the 0.4.0 entry: per-image styles, per-axis spacing, grid column
   weights, decorations, WASM adapter, examples/docs; call out breaking changes
   (`Spacing`, `InputImage.style`).
3. README: per-image overrides in features, current CLI example.
4. Docs: `api.md` (InputImage + semantics), `cli.md` (flag grammar), `wasm.md` (setters),
   `layouts.md` (decoration note), `ROADMAP.md` (tick 0.4).
5. Crate READMEs: core example uses `InputImage::new`; wasm drops the stale
   "web example arrives next task" paragraph and documents the new setters; cli adds a
   per-pane example.
6. Cleanup justified by the breaking release: remove the one-line `src/options.rs`
   re-export shim and the duplicate `ComposeOptions`/`PaneStyle` re-export in `lib.rs`;
   remove the dead `ComposeError::UnsupportedLayout` variant and its docs row.
7. Verify `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, `cargo check -p stitch-wasm --target wasm32-unknown-unknown`.
8. Commit, annotated tag `v0.4.0`, push `main` and the tag.

## Test plan

Core (`render_tests.rs`, `io_tests.rs`):

- `None` on every image reproduces the global-style output byte for byte.
- A global frame/radius applies to all panes when styles are `None`.
- `Some` on one pane replaces the global style wholesale (global frame disappears).
- `Some(PaneStyle::default())` renders a plain pane while siblings stay decorated.
- Loaders return `style: None`.

CLI (`stitch-cli/src/lib.rs` tests):

- Parse one and several `--pane-style` flags.
- Merge: color-only keeps global width; width 0 disables; later duplicate wins.
- Reject out-of-range (0 and too large), unknown key, malformed spec.
- Inputs indexed 1-based.

WASM (native tests):

- Set/clear stores the expected override.
- Applying overrides to a sample image list sets the right styles.
- Out-of-range index is reported at apply time.

## Risks

- The CLI grammar is the fiddliest part; strict parsing plus unit tests contain it.
- `--pane-style` values contain `#`, so the docs must show quoting.
- `InputImage` equality now includes style; only test helpers compare whole inputs.

## Left to the builder

- Exact parser structure and error wording beyond the strings above.
- Commit split and message bodies.
- Changelog prose once the final feature list is confirmed.
