#!/usr/bin/env bash
# Run the full ripex test suite from the repo root.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "== build =="
cargo build --release --locked

echo "== unit + integration tests =="
cargo test --release --locked

echo "== end-to-end corpus gate (tests/lang-test) =="
cargo test --release --locked --test ripex_lang_test_repos -- --nocapture --test-threads=1

echo "== done =="
