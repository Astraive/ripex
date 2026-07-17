# Testing

Four layers: unit/integration tests, the structural corpus gate, production
compiler conformance, and fuzzing.

## 1. Unit + integration tests

```bash
cargo test --release
```

## 2. End-to-end corpus gate

`tests/ripex_lang_test_repos.rs` walks **every** file in `tests/lang-test/`, parsing + extracting
each inside a per-file **5-second watchdog thread** (so an infinite loop is a reported HANG, not a
wedged run). It prints a per-language table (files / ok / errors / panics / hangs / facts) and
asserts zero panics, zero hangs, zero parser diagnostics, and nonempty symbol,
import, call, and variable facts for every enabled language.

```bash
cargo test --release --test ripex_lang_test_repos -- --nocapture --test-threads=1
```

Corpus layout (`tests/lang-test/`): one dir per language
(`javascript`, `python`, `go`, `rust`, `c`, `cpp`, `csharp`). TS/TSX fixtures
live under `javascript/src/`.

## 3. Compiler conformance

The `compiler-conformance` CI job installs the production toolchains and runs
`ripex check` over every fixture project. Locally, run the checks documented in
the workflow after installing GCC/Clang, Go, .NET 8, Node, TypeScript, Python,
mypy, and Rust. Missing tools are failures, not skips.

Native projects with a `compile_commands.json` file should be included in this
gate. The planner tests assert that recorded compilation flags are preserved
while object and dependency output flags are removed for no-output checking.

## 4. Fuzzing

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
