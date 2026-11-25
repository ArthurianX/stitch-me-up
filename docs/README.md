# Documentation

- [API reference](api.md) — the public surface of `stitch-me-up-core` (lib `stitch_core`): types, functions,
  errors, and the behavior notes that decide pixel placement.
- [CLI reference](cli.md) — every `stitch-me-up combine` flag, its default, accepted
  values, examples, and how errors are reported.
- [Layout cookbook](layouts.md) — one diagram per layout, the sizing and alignment
  rules that matter, and how decoration interacts with each layout.
- [WASM reference](wasm.md) — the `Composer` JavaScript API, error behavior, and the
  build/serve workflow for the web example.
- [Design notes and plans](superpowers/) — the 0.4 per-image style design and the
  implementation plan that shipped it, plus the release-hygiene plan for v0.4.2 and the 0.5.0 release plan (package rename, prebuilt binaries, first publish), and the GitHub Pages demo design and plan (live WASM demo, deployment, node tests).

## Source of truth

These docs describe the code, not the other way around. When they disagree, the code
wins: `crates/stitch-core/src/` for engine behavior, `crates/stitch-cli/src/lib.rs` for
CLI flags (cross-check with `stitch-me-up combine --help`), and
`crates/stitch-wasm/src/lib.rs` for the JS surface.
