#!/usr/bin/env bash
# Run a ripex fuzz target (requires nightly + cargo-fuzz).
# Usage: ./scripts/run-fuzz.sh [target] [seconds]
#   target   fuzz_all | fuzz_js | fuzz_python | fuzz_go   (default: fuzz_all)
#   seconds  max run time in seconds                       (default: 60)
set -euo pipefail

# cargo-fuzz must run from inside the fuzz crate directory.
cd "$(dirname "$0")/../tests/fuzz"

TARGET="${1:-fuzz_all}"
SECONDS="${2:-60}"

if ! command -v cargo-fuzz >/dev/null 2>&1; then
  echo "cargo-fuzz not found. Install: cargo +nightly install cargo-fuzz" >&2
  exit 1
fi

echo "== fuzzing $TARGET for ${SECONDS}s =="
cargo +nightly fuzz run "$TARGET" -- -max_total_time="$SECONDS"
