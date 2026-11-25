#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: examples/cli/compose.sh [options] <input image> <input image> [more images...]

Combines 2 or more PNG or JPEG images into a single decorated grid PNG using the
stitch-me-up `combine` command.

Options:
  -h, --help   Show this message

Environment variables:
  OUT          Output PNG path (default: stitch-example.png in the current directory)
  LAYOUT       One of vertical, horizontal, grid, before-after (default: grid)

The composition runs from the repository root via cargo. Inside the repository the root
comes from `git rev-parse --show-toplevel`; otherwise it is derived from this script's
location, so the script works from anywhere.
EOF
}

for arg in "$@"; do
  case "$arg" in
    -h | --help)
      usage
      exit 0
      ;;
  esac
done

if [ "$#" -lt 2 ]; then
  usage >&2
  exit 1
fi

OUT="${OUT:-stitch-example.png}"
LAYOUT="${LAYOUT:-grid}"

root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$root" ]; then
  # Not inside a git repository: derive the repo root from this script's own location
  # (examples/cli/compose.sh -> repo root).
  root="$(cd "$(dirname "$0")/../.." && pwd)"
fi

cargo run --quiet --manifest-path "$root/Cargo.toml" -p stitch-me-up -- combine \
  --layout "$LAYOUT" \
  --output "$OUT" \
  --columns 2 \
  --spacing 8 \
  --padding 16 \
  --corner-radius 8 \
  --frame-width 2 \
  --frame-color "#1f2937" \
  --background "#f8fafc" \
  "$@"

echo "Output written to $OUT"
