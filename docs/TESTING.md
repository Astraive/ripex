# Testing

Three layers: unit/integration tests, the `lang-test` corpus gate, and fuzzing.

## 1. Unit + integration tests

```bash
cargo test --release
```

## 2. End-to-end corpus gate

`tests/ripex_lang_test_repos.rs` walks **every** file in `tests/lang-test/`, parsing + extracting
each inside a per-file **5-second watchdog thread** (so an infinite loop is a reported HANG, not a
wedged run). It prints a per-language table (files / ok / errors / panics / hangs / facts) and
asserts zero panics and zero hangs. It also requires zero diagnostics for Go,
JavaScript, Python, and Rust and enforces non-increasing diagnostic budgets elsewhere.

```bash
cargo test --release --test ripex_lang_test_repos -- --nocapture --test-threads=1
```

Corpus layout (`tests/lang-test/`): one dir per language
(`javascript`, `python`, `go`, `rust`, `c`, `cpp`, `csharp`). TS/TSX fixtures
live under `javascript/src/`.

## 3. Fuzzing

`tests/fuzz/` is a [`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) workspace that feeds the
parsers arbitrary bytes to hunt panics / hangs / OOM. Targets: `fuzz_all` (all languages),
`fuzz_js`, `fuzz_python`, `fuzz_go`.

cargo-fuzz discovers the fuzz crate from its own directory, so run it from `tests/fuzz`:

```bash
cd tests/fuzz
cargo +nightly fuzz run fuzz_all        # needs nightly + cargo-fuzz
```

Each target wraps random bytes as lossy UTF-8 and runs `parse` + `extract`; a crash saves the
offending input under `tests/fuzz/artifacts/`.

Run everything: `./scripts/run-tests.sh`. Run fuzzing: `./scripts/run-fuzz.sh fuzz_all 60`.
