#!/usr/bin/env bash
# Run a ripex fuzz target (requires nightly + cargo-fuzz).
# Usage: ./scripts/run-fuzz.sh [target] [seconds]
#   target   fuzz_all | fuzz_js | fuzz_python | fuzz_go   (default: fuzz_all)
#   seconds  max run time in seconds                       (default: 60)
set -euo pipefail

# cargo-fuzz must run from inside the fuzz crate directory.
cd "$(dirname "$0")/../tests/fuzz"

TARGET="${1:-fuzz_all}"
DURATION="${2:-60}"

case "$TARGET" in
    fuzz_all|fuzz_js|fuzz_python|fuzz_go)
        ;;
    *)
        echo "invalid fuzz target '$TARGET'; expected fuzz_all, fuzz_js, fuzz_python, or fuzz_go" >&2
        exit 2
        ;;
esac

case "$DURATION" in
    ''|*[!0-9]*)
        echo "invalid fuzz duration '$DURATION'; expected a positive integer number of seconds" >&2
        exit 2
        ;;
esac
case "$DURATION" in
    *[1-9]*)
        ;;
    *)
        echo "invalid fuzz duration '$DURATION'; expected a positive integer number of seconds" >&2
        exit 2
        ;;
esac

if ! command -v cargo-fuzz >/dev/null 2>&1; then
    echo "cargo-fuzz not found. Install it with: cargo +stable install --locked cargo-fuzz" >&2
    exit 1
fi

# cargo-fuzz does not expose Cargo's --locked flag. CARGOFLAGS is inherited by
# each Cargo subprocess it starts, keeping fuzz builds on tests/fuzz/Cargo.lock.
CARGOFLAGS="${CARGOFLAGS:-}"
case " $CARGOFLAGS " in
    *" --locked "*)
        ;;
    *)
        CARGOFLAGS="${CARGOFLAGS:+$CARGOFLAGS }--locked"
        ;;
esac
export CARGOFLAGS

CORPUS_DIR="corpus/$TARGET"
if [ ! -d "$CORPUS_DIR" ]; then
    echo "committed corpus missing for '$TARGET': $CORPUS_DIR" >&2
    exit 1
fi

seed_found=0
for corpus_file in "$CORPUS_DIR"/*; do
    if [ -f "$corpus_file" ]; then
        if [ ! -r "$corpus_file" ]; then
            echo "committed corpus file is unreadable: $corpus_file" >&2
            exit 1
        fi
        seed_found=1
    fi
done
if [ "$seed_found" -eq 0 ]; then
    echo "committed corpus is empty for '$TARGET': $CORPUS_DIR" >&2
    exit 1
fi

echo "== replaying committed $TARGET corpus =="
for corpus_file in "$CORPUS_DIR"/*; do
    if [ -f "$corpus_file" ]; then
        echo "replay: $corpus_file"
        cargo +nightly fuzz run "$TARGET" "$corpus_file" -- -runs=1
    fi
done

echo "== fuzzing $TARGET for ${DURATION}s =="
cargo +nightly fuzz run "$TARGET" "$CORPUS_DIR" -- -max_total_time="$DURATION"
