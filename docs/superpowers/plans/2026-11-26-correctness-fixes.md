# Correctness Fixes (0.5.1) — Implementation Plan

> **For agentic workers:** fix test-first; every task ends green.

**Goal:** Fix six confirmed defects from the review — transparent-edge tinting, the WASM
pane-style trap, PNG encoder panics, revoked preview URLs, lost upload messages, and
incomplete startup handling — add a browser test gate, and release 0.5.1.

**Architecture:** Each fix addresses the root cause found in investigation: resize math on
straight alpha, dense pane-style storage, unchecked encoder buffers, early URL revocation,
a shared error region, and a static wasm import. Regression tests are added at the layer
that owns each bug; the browser layer gets a Playwright suite in CI.

**Tech Stack:** Rust (core + wasm-bindgen), vanilla ES modules, Node `node --test`,
Playwright (dev-only), GitHub Actions.

**Spec:** the 2026-11-25 Pages design and 2026-11-22 pane-style design, plus the review in
this conversation.

## Global Constraints

- No new Rust dependencies; integer math for premultiply/unpremultiply; MSRV stays 1.88.
- WASM/API behavior must remain source-compatible apart from the new error variant.
- 104 Rust tests must keep passing; add tests, do not weaken existing ones.
- The site keeps zero runtime dependencies; npm is dev-only under `web-tests/`.
- No tag or publish in this phase; 0.5.1 release is a separate gated step.

## Confirmed root causes (from reproduction)

1. `prepare_pixels` resizes straight RGBA → invisible RGB bleeds (evidence: edge pixels
   `(207,64,112)`, `(207,191,239)` on a red/transparent-blue boundary).
2. `apply_pane_style` does `Vec::resize(index + 1, None)` → `RuntimeError: unreachable` at
   `4294967294`, large allocation at `1_000_000`.
3. `encode_png`/`save_png` pass unchecked buffers to the image crate → panic
   (`expected 4 got 0 for 1x1`).
4. `compose()` revokes the old blob URL before decoding the replacement; a failed decode
   leaves preview/download on a revoked URL.
5. Compose calls `clearError()` on the shared `#error`, erasing upload rejections once a
   compose succeeds; batch-limit errors overwrite file messages.
6. Static `import` of `pkg/stitch_wasm.js` prevents any guidance when the module is
   missing; `addFiles` accepts drops after init failure.

---

### Task 1 — Premultiplied resize

- [x]

**Files:** `crates/stitch-core/src/render.rs` (`prepare_pixels`), `crates/stitch-core/tests/render_tests.rs`

**Tests (write first):**
`resize_does_not_bleed_transparent_color_into_visible_edges`: image A 5×4, columns 0–3 opaque
red, column 4 fully transparent blue; image B 5×8 opaque green; `BeforeAfter`, white
background. Assert for every pixel in the left pane: `r == 255` and `g == b`; and at least
one pixel has `0 < g < 255` (the filter actually ran).

**Fix:** in `prepare_pixels`, premultiply each pixel before `imageops::resize` and
unpremultiply after:
`pm(c,a) = (c*a + 127)/255`; `unpm(pm,a) = if a == 0 { 0 } else { ((pm*255 + a/2)/a).min(255) }`.
Lengths stay `w*h*4`; alpha is unchanged by both steps.

---

### Task 2 — Output buffer validation

- [x]

**Files:** `crates/stitch-core/src/error.rs`, `crates/stitch-core/src/image_io.rs`, `crates/stitch-core/tests/io_tests.rs`

**Tests (write first):** `encode_png` rejects `1×1` with 0 bytes and `2×1` with 4 bytes as
`ComposeError::InvalidOutputBuffer { .. }` (no panic); `save_png` does the same and leaves
no file; a valid `1×1` still produces bytes starting with `\x89PNG`.

**Fix:** add
`InvalidOutputBuffer { width, height, expected, actual }` with message
`output image {width}x{height} has invalid buffer length: expected {expected}, got {actual}`,
validate `checked(width*height*4)` in both entry points before touching the encoder.

---

### Task 3 — Sparse pane styles in WASM

- [x]

**Files:** `crates/stitch-wasm/src/lib.rs`

**Tests (write first):** `huge_pane_style_index_does_not_allocate_or_panic`
(`apply_pane_style(usize::MAX, 1, "#000000", 0)` is `Ok`, then `apply_pane_styles` on one
image returns the `pane style index … has no matching image` error);
`unmatched_pane_style_index_errors_at_compose` for index 5 with two images.

**Fix:** replace `Vec<Option<PaneStyle>>` with `BTreeMap<usize, PaneStyle>`; setter inserts,
clear removes, `apply_pane_styles` iterates entries in ascending order and errors when
`images.get(index)` is `None`. Update `stores_and_clears_pane_styles` accordingly.

---

### Task 4 — Preview URL lifecycle

- [x]

**Files:** `examples/web/app.js`

**Behaviour:** create the new blob URL, decode it, and only then swap `#preview.src`/`#download`
and revoke the previous URL. If decoding fails or the change token moved on, revoke the new
URL and leave the old preview intact. No other compose semantics change.

**Test:** covered by the browser suite (Task 7), which stubs `createImageBitmap` to fail once
and asserts the old URL is still fetchable and the preview unchanged.

---

### Task 5 — Upload messages survive composition

- [x]

**Files:** `examples/web/index.html`, `examples/web/app.js`, `examples/web/styles.css`

**Behaviour:** add `<ul id="upload-errors" role="alert" hidden>`; uploads own it and clear it
only on the next upload; append batch-limit/per-file messages as list items instead of
overwriting. `#error` remains compose-only. Style the list like the existing error box.

**Test:** browser suite: 2 valid + 1 GIF → list visible with the GIF named; wait for a
successful compose → list still visible, `#error` hidden.

---

### Task 6 — Startup failure handling

- [x]

**Files:** `examples/web/app.js`

**Behaviour:** remove the static wasm import; `startWasm()` uses
`await import("./pkg/stitch_wasm.js")`, calls its default init, and stores `Composer` in
state. Any failure reaches the existing guidance branch. `addFiles` returns early with an
upload notice when `state.composerReady` is false, so drops after a failed init do not add
thumbnails.

**Tests:** browser suite: blocking `pkg/stitch_wasm.js` shows guidance in `#error`;
blocking the `.wasm` then dropping a file yields zero thumbnails and an upload notice.

---

### Task 7 — Browser test gate

- [x]

**Files (new):** `web-tests/package.json`, `web-tests/package-lock.json`,
`web-tests/playwright.config.js`, `web-tests/serve.mjs`, `web-tests/browser.spec.js`,
`web-tests/fixtures/{a,b,c}.png`; modify `.github/workflows/ci.yml`, `.gitignore`.

- `package.json`: `{"name":"stitch-me-up-web-tests","private":true,"devDependencies":{"@playwright/test":"1.65.1"}}`; script `test: "playwright test"`.
- `serve.mjs`: dependency-free static server on `127.0.0.1:8123` serving `../examples/web`
  with correct MIME types (`.wasm → application/wasm`, `.js → text/javascript`), so the
  served page matches Pages.
- `playwright.config.js`: `testDir: "."`, `use.baseURL` from `BASE_URL` (default local),
  `webServer` running `node serve.mjs`, chromium; allow `channel: process.env.PW_CHANNEL`
  for local runs against system Chrome.
- Spec coverage: load→Ready; two uploads→preview/meta/download name; grid then before-after;
  third-image rejection keeps preview; failed decode keeps the old URL fetchable; mixed
  upload keeps its notice through a compose; missing `pkg` shows guidance; failed init
  refuses drops.
- CI: extend the `web` job after `node --test`: `cd web-tests && npm ci`,
  `npx playwright install --with-deps chromium`, `npx playwright test`; cache
  `~/.cache/ms-playwright` keyed on `web-tests/package-lock.json`.
- `.gitignore`: add `web-tests/node_modules/`, `web-tests/test-results/`.

---

### Task 8 — 0.5.1 release (separate dispatch, after review)

- Bump workspace + path deps to `0.5.1`; CHANGELOG `0.5.1` with the six fixes and the new
  error variant; full gates on stable and 1.88 plus browser suite.
- Fast-forward merge, push, watch CI (test/msrv/web incl. browser) and Pages; verify live
  footer says `0.5.1` and re-run the browser suite against the live URL.
- Tag `v0.5.1`, release notes from CHANGELOG, verify six assets, publish
  `stitch-me-up-core` → `stitch-me-up-wasm` → `stitch-me-up`, `cargo install` check.

## Acceptance criteria

- Each root cause has a regression test that fails before the fix and passes after.
- `node --check` clean; `node --test` 20/20; Playwright suite green locally and in CI.
- 104+ Rust tests green on stable and 1.88; clippy/fmt clean; wasm target check green.
- Re-running the original repros shows no tint (`g == b`), no trap at `usize::MAX`, an `Err`
  instead of a panic, a live preview URL after a failed decode, retained upload notices, and
  guidance when the module is missing.
