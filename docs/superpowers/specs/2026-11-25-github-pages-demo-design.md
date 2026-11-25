# Public Web Demo on GitHub Pages — Design

**Date:** 2026-11-25
**Status:** awaiting approval
**Live URL:** https://arthurianx.github.io/stitch-me-up/ (Pages already enabled, `build_type: workflow`)

## Intent

Publish the WebAssembly adapter as a real browser tool: a visitor uploads a few
images, arranges and decorates them, and downloads a stitched PNG. Nothing leaves
the browser. The page doubles as the project's showcase, replacing the bare
`examples/web` demo that requires `wasm-pack` and a local server.

## Goals

- Upload or drop 2+ PNG/JPEG files, reorder and remove them, see a live preview,
  download the full-resolution PNG.
- Expose every core option the current example does, with layout-aware controls.
- Keyboard-operable reordering and labelled controls; responsive layout;
  light/dark via `prefers-color-scheme`.
- Deploy automatically from `main`; show the engine version and deployed commit.
- Pure JS logic covered by `node --test`; the wasm build and tests run in PR CI.

## Non-goals

- Accounts, server-side processing, persistence, analytics, cookies.
- PWA/offline caching, per-PR preview deployments, custom domain.
- WebP/AVIF transcoding (core decodes PNG/JPEG only).
- Per-image `set_pane_style` editor (deferred), npm or a front-end framework.
- A marketing landing page; this is one screen.

## Constraints

- Vanilla HTML/CSS/ES modules; the only build step is `wasm-pack build --target web`.
- Pages serves the project at `/stitch-me-up/`, so every asset URL is relative
  (`./pkg/stitch_wasm.js`, `./version.json`).
- `compose` is synchronous; previews are debounced on the main thread.
- Limits: 24 files, 25 MP per image, 100 MP total input; warn above 60 MP.
- `pkg/` and `version.json` stay out of git; both are produced by the build.

## Approaches considered

1. **Deploy the current example unchanged.** Cheapest, but no drag/drop,
   reordering, live preview, or limits; not a tool.
2. **Upgrade `examples/web` in place with vanilla modules and node tests.** One
   source of truth, no npm, deployable subdirectory, testable pure logic. Chosen.
3. **New `site/` app with Vite + React/Svelte.** Better component structure for a
   screen this size, but adds npm, a bundler, and a second toolchain to a
   zero-dependency repo. Rejected.

## Design

### Files

```text
examples/web/
├── index.html          # shell, form controls, preview, footer
├── styles.css          # tokens, layout, light/dark
├── app.js              # DOM wiring, compose orchestration, blob URLs
├── lib.js              # pure logic (importable by node --test)
├── tests/lib.test.js   # limit checks, ordering, validation, filenames
├── version.json        # generated at deploy; gitignored
└── pkg/                # wasm-pack output; gitignored
```

### Screen

```text
┌ 🩹 Stitch Me Up                                v0.5.0 · 87cc4f0 ┐
│ Compose images in your browser. Nothing is uploaded.              │
│ ┌ Drop PNG/JPEG here ─────────────────────────── [Choose files] ┐ │
│ └───────────────────────────────────────────────────────────────┘ │
│ [1 ▸ ↑ ↓ ✕] [2 ▸ ↑ ↓ ✕] [3 ▸ ↑ ↓ ✕]                               │
│ Layout [grid ▾] Columns [2] Weights [1,2]   Spacing x [ ] y [ ]   │
│ Padding T R B L   Align [center ▾] Resize [none ▾] Bg [■]         │
│ Frame [0] [■]   Radius [0]   Divider [0] [■] (before-after only)  │
│ ┌ preview, scaled to fit ───────────────────────────────────────┐ │
│ └───────────────────────────────────────────────────────────────┘ │
│ 1240×860 · 482 KB · [Download PNG]        status / error (alert)  │
└──────────────────────────────────────────────────────────────────┘
```

### Behaviour

- Files append in selection order. Thumbnails are draggable and each has labelled
  “move up” / “move down” / “remove” buttons, so reordering works without a mouse.
- Any change to files or controls schedules a compose 250 ms later, cancelling the
  pending timer. A busy indicator shows during composition; the last successful
  preview is kept if a compose fails, and the error is announced in the alert box.
- Layout selection enables only the controls that layout uses: columns/weights for
  grid, divider for before-after. Before-after with a count other than two shows an
  inline message and skips composition instead of surfacing a raw core error.
- Adding files validates type (PNG/JPEG), per-file and total limits, with a
  per-file message and a non-blocking warning above 60 MP.
- Composition constructs a fresh `Composer`, applies every setter, and calls
  `compose(buffers)`. The PNG bytes become a blob URL; the previous URL is revoked.
  Output dimensions come from decoding the blob; size from `blob.size`.
- Download is disabled until a result exists. Filename is
  `stitch-me-up-YYYYMMDD-HHMMSS.png`.
- Footer fetches `version.json` (`{version, commit}`). Missing file (local dev,
  `file://`) hides the footer version instead of erroring.

### Error handling

- Setter and compose errors are shown verbatim (`error.message`), prefixed with
  “Could not compose:”.
- `init()` failure shows build instructions and disables the tool, as today.
- A 404 for `version.json` is silent.

### Deployment

`.github/workflows/pages.yml`:

- Triggers: `push` on `main`, `workflow_dispatch`.
- Permissions: `contents: read`, `pages: write`, `id-token: write`.
- Concurrency: group `pages`, cancel in progress.
- Environment: `github-pages`.
- Steps: checkout → `jetli/wasm-pack-action` with wasm-pack `v0.13.1` → build
  `--target web --out-dir examples/web/pkg` in `crates/stitch-wasm` → `node --test
  examples/web/tests` → write `examples/web/version.json` from `Cargo.toml` and
  `GITHUB_SHA` → `actions/upload-pages-artifact` (path `examples/web`) →
  `actions/deploy-pages`.
- Every push to `main` deploys; no path filter, because core changes alter output.

`.github/workflows/ci.yml` gains a `web` job with the same wasm-pack build and
`node --test`, without deploy, so PRs fail before merge.

### Testing

`node --test examples/web/tests` covers the pure logic in `lib.js`:

- limit and type validation (24 / 25 MP / 100 MP, warning threshold)
- ordering (move up/down bounds, remove, stable order)
- layout-specific validation (before-after count, grid columns/weights)
- option assembly and colour/format guards
- download filename formatting and human-readable sizes

The integration gate is the wasm build plus the Pages artifact upload. No browser
automation in this pass (no npm).

### Documentation

- `examples/web/README.md`: live URL, local build/serve, deployment notes.
- `docs/wasm.md`: link the live demo, note that `version.json`/`pkg/` are build output.
- `README.md`: add the demo link under Examples.
- `ROADMAP.md`: add a “Public demo” item; no crate version bump, no tag, since the
  site deploys from `main` and the engine is unchanged.

## Calls made without asking

- Reordering is drag-and-drop *plus* buttons, not drag-only.
- Failed composes keep the last good preview; the error is announced.
- The site shows the engine version (`0.5.0`), not a site version.
- No service worker, no offline mode, no share links, no image history.
