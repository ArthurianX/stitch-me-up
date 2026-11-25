# Public Web Demo on GitHub Pages — Implementation Plan

> **For agentic workers:** execute tasks in order. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Ship the upgraded `examples/web` page as the live WASM demo at
https://arthurianx.github.io/stitch-me-up/, deployed from `main` by GitHub Actions.

**Architecture:** Pure logic lives in `examples/web/lib.js` (ES module, no DOM) and is
unit-tested with `node --test`. `app.js` owns DOM wiring, file decoding, debounced
composition, and blob URLs. A new `pages.yml` builds wasm with wasm-pack, runs the node
tests, writes `version.json`, and deploys the directory. `ci.yml` gains the same
build+test as a non-deploying `web` job.

**Tech Stack:** vanilla ES modules, HTML/CSS, `wasm-pack` 0.13.1 (`--target web`),
Node 24 `node --test`, GitHub Pages Actions (`upload-pages-artifact` / `deploy-pages`).

**Spec:** `docs/superpowers/specs/2026-11-25-github-pages-demo-design.md`

## Global Constraints

- PNG/JPEG inputs only; output PNG. No npm dependencies, no bundler, no framework.
- Limits: 24 files, 25 MP per image (25_000_000), 100 MP total (100_000_000), warn above
  60 MP (60_000_000).
- Pages serves under `/stitch-me-up/`: all asset URLs relative (`./pkg/...`, `./version.json`).
- `examples/web/pkg/` and `examples/web/version.json` are generated, gitignored, never committed.
- No crate version bump, no tag; engine behavior unchanged; 104 Rust tests still pass.
- Deploy happens only from `main` and only after the site is verified locally.

## Review Focus

- **before-after with ≠ 2 images:** inline message, no raw core error, composition skipped.
- **GIF/WebP renamed to .png, or corrupt bytes:** a clear per-file or compose error, never a silent no-op.
- **Total pixels at/over limits:** rejected files named; warning between 60 and 100 MP.
- **Removing all images:** preview and download cleared, controls still usable.
- **Rapid control changes:** debounce cancels stale work; a slow decode never overwrites a newer preview (generation counter).
- **Missing `version.json` (local dev):** footer hides, no console error surfaced to the user.

---

### Task 1: Pure logic and its tests

**Files:**
- Create: `examples/web/package.json`, `examples/web/lib.js`, `examples/web/tests/lib.test.js`

**Interfaces:**
- Produces (named exports from `lib.js`):
  `MAX_FILES`, `MAX_IMAGE_PIXELS`, `MAX_TOTAL_PIXELS`, `WARN_TOTAL_PIXELS`,
  `pixelCount(width, height) -> number`,
  `validateFile({name, type}, pixels) -> string | null`,
  `canAddFiles(currentCount, incomingCount) -> {ok: boolean, message: string}`,
  `totalPixels(items) -> number`,
  `totalWarning(total) -> string | null`,
  `moveItem(items, from, to) -> Array`, `moveUp(items, index) -> Array`,
  `moveDown(items, index) -> Array`, `removeAt(items, index) -> Array`,
  `isLayoutValid(layout, imageCount, columns) -> {ok: boolean, message: string}`,
  `parseWeights(text) -> number[]`, `formatBytes(bytes) -> string`,
  `downloadName(date) -> string`.

- [ ] **Step 1: Add the ESM marker**

`examples/web/package.json`: `{"name": "stitch-me-up-web", "private": true, "type": "module"}`
(no dependencies; it exists so Node treats `.js` as ESM).

- [ ] **Step 2: Write failing tests**

`examples/web/tests/lib.test.js`, `node:test` + `node:assert/strict`, covering at least:
limits and messages (24-file cap, per-file 25 MP, total 100 MP, warning at 60 MP), type
rejection (`.gif`, `image/webp`, PNG/JPEG accepted), `canAddFiles` boundary values,
`totalPixels`/`totalWarning` thresholds, move/remove bounds (no mutation of the input),
`isLayoutValid` for before-after counts 0..3 and grid columns 0/1, `parseWeights`
("1,2", " 3 , 4 ", "", "a,b"), `formatBytes` (999, 1024, 1.5e6), and `downloadName`
producing `stitch-me-up-YYYYMMDD-HHMMSS.png` from a fixed `Date`.

- [ ] **Step 3: Run tests to verify they fail**

Run: `node --test examples/web/tests/*.test.js`
Expected: FAIL, `Cannot find module .../lib.js` (or missing exports).

- [ ] **Step 4: Implement `lib.js`**

Match the signatures in Interfaces. Rules: `validateFile` accepts only `image/png` and
`image/jpeg`, rejects `pixels === 0` or `pixels > MAX_IMAGE_PIXELS`, and returns a short
human message naming the file; `canAddFiles` checks `current + incoming <= MAX_FILES`;
`isLayoutValid` requires exactly two images for `before-after`, `columns >= 1` for `grid`,
and accepts every other layout with 2+ images (`{ok:false}` below 2); list helpers return
new arrays and clamp out-of-range indices to a no-op copy.

- [ ] **Step 5: Run tests to verify they pass**

Run: `node --test examples/web/tests/*.test.js`
Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add examples/web/package.json examples/web/lib.js examples/web/tests/lib.test.js
git commit -m "feat(web): add pure composition helpers with node tests"
```

---

### Task 2: Page shell and styles

**Files:**
- Create: `examples/web/index.html`, `examples/web/styles.css`

**Interfaces:**
- Produces these element ids for `app.js`: `files`, `drop`, `thumbs`, `layout`,
  `columns`, `weights`, `spacing-x`, `spacing-y`, `pad-top`, `pad-right`, `pad-bottom`,
  `pad-left`, `background`, `alignment`, `resize-mode`, `frame-width`, `frame-color`,
  `corner-radius`, `divider-width`, `divider-color`, `preview`, `meta`, `download`,
  `status`, `error`, `version`.
- Script tag: `<script type="module" src="./app.js"></script>`; stylesheet `./styles.css`.

- [ ] **Step 1: Write `index.html`**

Header (title, one-line privacy note, `<span id="version" hidden>`), uploader with
`input#files[type=file][multiple][accept="image/png,image/jpeg"]` and a labelled drop zone
`#drop` containing a “Choose files” button that clicks the input; ordered `ol#thumbs`;
fieldsets for Layout / Geometry / Decoration with every control named above and sensible
defaults (vertical, 2 columns, spacing 0, padding 0, white, center, none, frame 0/black,
radius 0, divider 0/black); `<img id="preview" hidden>` inside a preview frame;
`<p id="meta">`; `<a id="download" hidden download>Download PNG</a>`;
`<p id="status" role="status" aria-live="polite">`; `<div id="error" role="alert" hidden>`;
footer link to the repo and `#version`.

- [ ] **Step 2: Write `styles.css`**

Token-based palette with `@media (prefers-color-scheme: dark)`, max-width ~1100px, a
two-column layout on wide screens (controls left, preview right) collapsing to one column
under 860px, focus-visible outlines, `.thumb` grid rows with 64px thumbnails, `.busy` and
`[hidden]` rules, and no external fonts.

- [ ] **Step 3: Commit**

```bash
git add examples/web/index.html examples/web/styles.css
git commit -m "feat(web): rebuild demo page shell and styles"
```

---

### Task 3: Application wiring

**Files:**
- Create: `examples/web/app.js`
- Modify: `examples/web/lib.js` only if a missing pure helper is discovered (add it with a test)

**Interfaces:**
- Consumes: every export from Task 1 and the ids from Task 2.
- Produces: no exports; runs on module load.

- [ ] **Step 1: Implement state and rendering**

State: `{ items: [{id, file, name, url}], generation, busy, lastUrl, lastBlob, composerReady }`.
Render thumbnails as `<li class="thumb" draggable="true">` with a thumbnail `<img>`, the
name, and three labelled buttons (`Move up`, `Move down`, `Remove`) calling the lib list
helpers. Render from state after every mutation; revoke object URLs of removed items.

- [ ] **Step 2: Implement adding files**

Picker/drop handler collects `File`s in order, applies `validateFile` once dimensions are
known (decode via `createImageBitmap`), applies `canAddFiles`, appends accepted items, and
surfaces rejections in `#error` as a short list. Show `totalWarning(...)` in `#status`
without blocking. Drag-and-drop uses `dragover` (preventDefault) and `drop`, filtered to
images.

- [ ] **Step 3: Implement composition**

`schedule()` debounces 250 ms and calls `compose()`. `compose()` increments `generation`,
sets busy, reads `arrayBuffer()` for each item, builds a fresh `Composer`, applies:
layout+columns, grid weights (only when non-empty), spacing, padding, background,
alignment, resize mode, frame, radius, and divider only for `before-after`. Validate with
`isLayoutValid` first; on failure show the inline message and skip. After `compose`, drop
the result if `generation` changed; otherwise create a blob URL, revoke the previous one,
update `#preview`, `#meta` (`W×H · size` from decoding the blob), enable `#download` with
`downloadName(new Date())`, and clear busy. On error: keep the previous preview, show
`Could not compose: <message>`. Empty list clears preview/download and shows status.

- [ ] **Step 4: Implement startup and footer**

`init()` failure: show build guidance, disable controls. `fetch("./version.json",
{cache: "no-store"})` success fills `#version` with `v<version> · <commit>` and unhides it;
404/failure leaves it hidden. `setLayoutControls()` disables columns/weights unless grid and
divider controls unless before-after; call on change and once at startup.

- [ ] **Step 5: Manual local check**

Build the package locally if wasm-pack is available, serve `examples/web` over HTTP, and
verify: adding 3 images, reordering, before-after rejection, removing all, download
enabled only with a result. Record what was checked; if wasm-pack is unavailable, say so
and rely on the CI build.

- [ ] **Step 6: Commit**

```bash
git add examples/web/app.js examples/web/lib.js examples/web/tests
git commit -m "feat(web): add uploader, reordering, live preview and download"
```

---

### Task 4: Deployment and CI

**Files:**
- Create: `.github/workflows/pages.yml`
- Modify: `.github/workflows/ci.yml` (new `web` job), `.gitignore`

- [ ] **Step 1: Add `pages.yml`**

Triggers `push` on `main` and `workflow_dispatch`; `permissions: contents: read, pages:
write, id-token: write`; `concurrency: {group: pages, cancel-in-progress: true}`;
`environment: github-pages`. Steps: checkout → `jetli/wasm-pack-action@v0.4.0` with
`version: v0.13.1` → `wasm-pack build --target web --out-dir ../../examples/web/pkg`
(working-directory `crates/stitch-wasm`) → `node --test examples/web/tests/*.test.js` → write
`examples/web/version.json` (`version` from workspace `Cargo.toml`, `commit` from
`${GITHUB_SHA:0:7}`) → `actions/configure-pages@v5` →
`actions/upload-pages-artifact@v3` (`path: examples/web`) → `actions/deploy-pages@v4`.

- [ ] **Step 2: Add the `web` job to `ci.yml`**

Same checkout/wasm-pack/build/`node --test`, no Pages permissions and no deploy.

- [ ] **Step 3: Ignore generated files**

Append `examples/web/version.json` to `.gitignore` (`pkg/` is already ignored).

- [ ] **Step 4: Validate YAML and commit**

Parse both workflows with Ruby's YAML (`ruby -ryaml -e 'YAML.load_file(...)'`) and commit:

```bash
git add .github/workflows/pages.yml .github/workflows/ci.yml .gitignore
git commit -m "ci: build and deploy the wasm demo to GitHub Pages"
```

---

### Task 5: Documentation

**Files:**
- Modify: `examples/web/README.md`, `README.md`, `docs/wasm.md`, `ROADMAP.md`, `docs/README.md`
- Add: `docs/superpowers/specs/2026-11-25-github-pages-demo-design.md` (commit the spec), this plan

- [ ] **Step 1: Rewrite `examples/web/README.md`**

Lead with the live URL and “everything runs in your browser”. Keep local build/serve
instructions, document `node --test examples/web/tests/*.test.js`, the generated `pkg/` and
`version.json`, and that deploys come from `main` via `pages.yml`.

- [ ] **Step 2: Update cross-references**

`README.md` Examples: add the live demo first. `docs/wasm.md`: link the live demo and note
the footer version file. `ROADMAP.md`: add a “Public demo” item marked done. `docs/README.md`:
index the design + plan alongside the others.

- [ ] **Step 3: Commit**

```bash
git add README.md ROADMAP.md docs examples/web/README.md
git commit -m "docs: document the live wasm demo and its deployment"
```

---

### Task 6: Gates, push, deploy, verify (external effect — only after review)

- [ ] **Step 1: Full gates**

`node --test examples/web/tests/*.test.js`; `cargo fmt --all -- --check`; `cargo clippy --workspace
--all-targets -- -D warnings`; `cargo test --workspace` (104); `cargo +1.88.0 test
--workspace` (104). All must pass.

- [ ] **Step 2: Merge and push**

`git switch main && git merge --ff-only <branch> && git push origin main`.

- [ ] **Step 3: Watch both workflows**

CI (incl. the new `web` job) and Pages must go green.

- [ ] **Step 4: Verify live**

`curl -sI https://arthurianx.github.io/stitch-me-up/` → 200; the HTML references
`./app.js`, `./styles.css`; `./pkg/stitch_wasm.js`, `./pkg/stitch_wasm_bg.wasm`, and
`./version.json` each return 200; `version.json` commit matches `git rev-parse --short
main`. Fetch the page HTML and confirm the control ids exist.

## Acceptance criteria

- `node --test examples/web/tests/*.test.js` passes locally and in the `web` CI job.
- CI green on `main`; Pages run green and the live URL serves HTML, JS, CSS, wasm, and
  `version.json` for the deployed commit.
- The page exposes every control in the spec, rejects non-PNG/JPEG, enforces the limits,
  refuses before-after with ≠ 2 images with an inline message, keeps the last preview on
  error, and enables download only after a result.
- No generated files committed; no Rust behavior change; no version bump or tag.
