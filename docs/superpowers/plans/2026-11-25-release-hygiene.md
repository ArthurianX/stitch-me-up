# Release Hygiene + 0.4.2 Implementation Plan

> **For agentic workers:** execute task-by-task, ticking each checkbox. Every task ends
> green: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
> `cargo test --workspace`, `cargo check -p stitch-wasm --target wasm32-unknown-unknown`.

**Goal:** Make the repository's release story truthful and reproducible: real MSRV, real
GitHub releases for the existing tags, documented crates.io status, clean branches and
dependency PRs, and a `0.4.2` patch release prepared locally.

**Non-goals:** no engine, layout, rendering, CLI, or WASM behavior change; no new
dependencies; no crate renames; no binary release artifacts; no push of `main` or of the
`v0.4.2` tag (prepared locally only, pending human approval).

## Findings this plan is based on

- `Cargo.toml` declares `rust-version = "1.75"`, but the locked graph requires more:
  `image 0.25.10` → 1.88, `clap 4.6.x` → 1.85, `wasm-bindgen 0.2.130` → 1.81.
- Tags `v0.4.0` and `v0.4.1` are pushed, but `gh release list` is empty: no GitHub Releases.
- `stitch-core` and `stitch-wasm` on crates.io belong to **unrelated projects**
  (`mlb2251/stitch`, `LabOverWire/stitch-rs`); `stitch-cli` is unclaimed. Publishing under
  the current names is impossible, so distribution is source-only for now.
- `docs/superpowers/plans/2026-11-22-per-image-pane-styles.md` has 42 unchecked boxes even
  though the work shipped.
- Stale local branches: `release/0.4.0` (merged), `te2-pr` (merged), `verify-te2`
  (1 commit not on main — leave it alone and report).
- Open Dependabot PRs: #3 clap 4.6.8, #4 anyhow 1.0.104, #5 thiserror 2.0.21.

## Global constraints

- Work on branch `release/0.4.2` cut from an up-to-date `main`; fast-forward merge at the end.
- Do **not** push `main`, and do **not** push the `v0.4.2` tag. Creating GitHub Releases for
  the already-pushed `v0.4.0` / `v0.4.1` tags is in scope.
- Do not delete `verify-te2` or any unmerged branch.
- Conventional commits, one logical commit per task where practical.

## Task 1 — Merge dependency PRs

- [x] Confirm each PR is mergeable and its checks are green (`gh pr view <n> --json
      mergeable,mergeStateStatus,statusCheckRollup`). Wait/retry rather than merging on red.
- [x] Merge #3, #4, #5 with `gh pr merge <n> --squash --delete-branch`.
- [x] `git switch main && git pull --ff-only`, then verify `cargo test --workspace` and
      `cargo clippy --workspace --all-targets -- -D warnings` are still green and the lockfile
      contains the bumped versions.
- [x] Record the resulting `main` commit in the report.

## Task 2 — Truthful MSRV

- [x] Determine the real MSRV: take the maximum `rust_version` across the resolved dependency
      graph (`cargo metadata --format-version 1`), which is currently `1.88.0` via `image`.
- [x] Set `rust-version = "1.88"` in `[workspace.package]`.
- [x] Add a dedicated `msrv` job to `.github/workflows/ci.yml`:
      checkout, `dtolnay/rust-toolchain@1.88.0`, `Swatinem/rust-cache@v2`,
      `cargo check --workspace --all-targets`, `cargo test --workspace`.
- [x] Verify locally with `rustup toolchain install 1.88.0` then
      `cargo +1.88.0 test --workspace` and `cargo +1.88.0 check -p stitch-wasm --target
      wasm32-unknown-unknown`. If the toolchain cannot be installed, say so in the report and
      do not claim it passed.
- [x] Validate the workflow YAML parses (e.g. `python3 -c "import yaml; yaml.safe_load(...)"`).
- [x] Document the requirement: README `Development` section states Rust 1.88+.

## Task 3 — Document crates.io distribution reality

- [x] Add a short subsection to README (`Status` or a new `Distribution` heading): the crate
      names are held by unrelated projects, so nothing is published to crates.io; build from
      source or use a git dependency. Name the blocking crates.
- [x] In `ROADMAP.md`, add a decision-log bullet under `0.3 — Reach` clarifying that "published
      crate metadata" meant metadata only, and that crates.io distribution is blocked by the
      name collision (rename decision deferred).
- [x] Do not rename any package.

## Task 4 — GitHub Releases for existing tags

- [x] Create release `v0.4.0` from the tag with the body of the `0.4.0` CHANGELOG section:
      `gh release create v0.4.0 --verify-tag --title "v0.4.0" --notes-file <tmp>`.
- [x] Create release `v0.4.1` the same way and let it be the latest release.
- [x] `gh release list` shows both; record URLs in the report.

## Task 5 — Retro-tick the shipped 0.4 plan

- [x] Add a status banner under the title of
      `docs/superpowers/plans/2026-11-22-per-image-pane-styles.md`: executed, shipped in
      v0.4.0, review fixes in v0.4.1, boxes ticked retrospectively.
- [x] Convert every `- [x]` to `- [x]` in that file; leave content otherwise untouched.
- [x] Mention the new plan in `docs/README.md`.

## Task 6 — Prepare 0.4.2

- [x] Bump the workspace version to `0.4.2` and the `stitch-core` path-dependency version
      pins in `crates/stitch-cli/Cargo.toml` and `crates/stitch-wasm/Cargo.toml`.
- [x] Update `README.md` status line to `v0.4.2`.
- [x] Add a `## 0.4.2 — 2026-11-25` CHANGELOG section covering: MSRV correction to 1.88,
      dependency bumps (clap 4.6.8, anyhow 1.0.104, thiserror 2.0.21), CI MSRV job, GitHub
      release backfill, crates.io status documentation, plan hygiene.
- [x] Refresh `Cargo.lock` (`cargo check --workspace`) and confirm it lists `0.4.2` for the
      three workspace crates.
- [x] Full verification, then `chore(release): v0.4.2` commit, `git switch main`,
      `git merge --ff-only release/0.4.2`, `git tag -a v0.4.2 -m "v0.4.2"`.
- [x] **Stop.** Do not push.

## Task 7 — Branch cleanup

- [x] Delete only branches that are fully merged into `main` (`git branch --merged main`):
      currently `release/0.4.0` and `te2-pr`. Use `git branch -d` (never `-D`).
- [x] Leave `verify-te2` in place; state its divergence in the report.

## Task 8 — Report

Report: commits created, files changed, gate outputs verbatim (test counts, clippy, fmt, wasm
check, 1.88 run), release URLs, PR merge SHAs, branches deleted/kept, whether anything was
pushed to origin, and every deviation from this plan.

## Acceptance criteria

- `cargo metadata` reports `rust-version` 1.88 for all three packages; CI has an `msrv` job
  pinned to 1.88.0 and the YAML parses.
- `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace` (>= 104 tests) and the
  wasm check pass on stable; the 1.88 run passes or its absence is explicitly reported.
- GitHub Releases `v0.4.0` and `v0.4.1` exist, with bodies matching their CHANGELOG sections.
- PRs #3/#4/#5 are merged with their branches deleted; `main` still builds.
- The 2026-11-22 plan has zero unchecked boxes; `docs/README.md` indexes the new plan.
- Version strings agree on 0.4.2; CHANGELOG describes only changes actually made.
- Local `main` is one release commit ahead of `origin/main` plus the merged PRs; the `v0.4.2`
  tag exists only locally.
- `verify-te2` untouched; no unmerged branch deleted.
