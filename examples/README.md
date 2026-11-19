# Examples

Two runnable examples, one per adapter:

- **CLI** — [`cli/compose.sh`](cli/compose.sh): composes 2+ images into a decorated
  grid via `cargo run`. Set `OUT` and `LAYOUT` to change the destination and layout;
  run it from any directory inside the repository.
- **Web** — [`web/README.md`](web/README.md): a browser UI on top of the WASM adapter.
  Requires `wasm-pack` to generate `web/pkg/` before serving.

The CLI example works out of the box once Rust is installed; the web example needs the
extra `wasm-pack` step described in its own README.
