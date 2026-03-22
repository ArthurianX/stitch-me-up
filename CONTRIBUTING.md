# Contributing

Thanks for helping build `stitch-me-up`.

## Project Direction

This project is a Rust image composition engine. Please keep contributions aligned with these principles:

- composition and layout first
- pure Rust core with thin adapters
- deterministic behavior and testable layout math
- no panorama stitching, CV pipelines, or AI features unless explicitly planned

## Development Setup

1. Install a current stable Rust toolchain.
2. Clone the repository.
3. Run `cargo test` from the repository root.
4. Run `cargo fmt` before submitting changes.

## Preferred Contribution Shape

- Keep the core API target-agnostic.
- Add tests with feature work and bug fixes.
- Favor small, reviewable pull requests.
- Update docs when behavior or public APIs change.

## Testing Expectations

- Add or update unit tests for layout math and option handling.
- Add rendering or integration-style tests when behavior crosses module boundaries.
- Prefer deterministic assertions over ad hoc visual checks.

## Pull Requests

- Explain the user-facing motivation.
- Summarize the technical approach.
- Call out tradeoffs, limitations, and follow-up work.
- Confirm the work stays within the project scope.

## Communication

Questions, ideas, and scoped design discussions are welcome through issues and pull requests.
