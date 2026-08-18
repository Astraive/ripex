# Changelog

## v0.3.1 (2026-08-19)

Ripex v0.3.1 introduces native Python bindings via PyO3, stabilizes Node.js bindings on Rust 1.85.0 MSRV with full multi-language coverage, and restores resilient GitHub Actions CI and release workflows.

### Python Bindings (`ripex` on PyPI)

- Added official Python extension crate (`bindings/python/`) built with PyO3 0.23 and Maturin with `abi3-py38` compatibility.
- Exposed synchronous (`parse_sync`) and asynchronous (`parse` via thread pool) entry points for high-performance non-blocking parsing.
- Full parity across all 8 supported languages (JavaScript, TypeScript, Python, Go, Rust, C, C++, C#).
- Rich typed Python objects and dataclass representations for `ParseOutput`, `Facts`, `ParsedSymbol`, `ParsedImport`, `ParsedCall`, `ParsedVariable`, `ParsedParam`, `ImportSpecifier`, `UsageSite`, `TypeKind`, `Span`, `Pos`, `Diagnostic`, `Comment`, and `AstSummary`.
- Added helper query methods on `Facts`: `find_symbols_by_name`, `find_calls_by_callee`, and `find_imports_by_source`.
- Full PEP 561 compliance with `py.typed` and `_ripex.pyi` typing stubs, passing `mypy --strict`.
- Added comprehensive pytest test suite covering all supported languages, error recovery, and safety limits.

### Node.js Bindings (`@astraive/ripex` on npm)

- Aligned MSRV to stable Rust 1.85.0 using napi-rs 2.16 with Node `>=18.0.0` engine compatibility.
- Unified Dual ESM (`index.js`) and CommonJS (`index.cjs`) loaders with comprehensive TypeScript definitions (`index.d.ts`).
- Expanded test suite to cover all 8 supported languages across Node.js and Bun runtimes.

### CI / CD & Toolchain

- Replaced invalid Action commits and unpinned actions across all workflows with verified immutable releases.
- Replaced deprecated runner references with official `macos-13` (x86_64) and `macos-14` (arm64) runners.
- Added Python bindings matrix build and release workflows supporting Linux, macOS, and Windows wheels.

## v0.3.0 (2026-08-01)

Ripex v0.3.0 is published: [crates.io](https://crates.io/crates/ripex/0.3.0) ·
[docs.rs](https://docs.rs/ripex/0.3.0) ·
[GitHub release](https://github.com/Astraive/ripex/releases/tag/v0.3.0).

### Parser hardening and evidence

- Hardened recursive-descent progress and error-recovery paths across the
  eight language modes, including malformed-input watchdog coverage.
- Added the per-language corpus gate: every checked-in mode must parse without
  diagnostics, panics, or hangs and must emit symbols, imports, calls, and
  variables.
- Added the curated evidence harness and report for exact gold-fact matching,
  malformed-input behavior, throughput, and allocator observations. The report
  is structural evidence for the checked-in corpus and curated cases, not a
  compiler-level semantic-equivalence claim.

### Distribution, security, and CI

- Set the default feature set to `lang-all` for language-only library builds;
  the `cli` feature remains opt-in.
- Documented the 0.3.0 dependency and explicit CLI installation/feature usage,
  including the always-on `serde_json` dependency used by the public compiler
  module for `compile_commands.json` support.
- Added explicit Cargo security-contact metadata and updated supported versions
  and vulnerability-reporting contact guidance.
- Expanded locked CI coverage for all targets and features, isolated feature
  builds, documentation, evidence generation, and production-toolchain
  conformance.
- Both `.github/workflows/ci.yml` and `.github/workflows/release.yml` passed
  after the Graxus dependency migration, covering locked builds, tests,
  evidence, conformance, and release artifact validation.
- Recorded the downstream Graxus migration to consume the published
  `ripex` 0.3.0 package; this entry does not assert a separate Graxus release.
- Published deterministic Linux, macOS, and Windows CLI archives with
  per-archive SHA-256 files and an aggregate `SHA256SUMS` artifact.

## v0.2.0 (2026-07-28)

- Clarified Ripex's parser-and-facts focus: external toolchain validation is
  optional and remains separate from the in-process structural parsers.
- Added JavaScript/TypeScript comment retention to `ParseResult`, including
  hashbang and end-of-file comments with source spans and CLI JSON output.
- Added JavaScript/TypeScript facts for optional and awaited generic calls,
  dynamic imports, destructured bindings, and correct `let` mutability.
- Added TypeScript type-only import/re-export support, standard `with` import
  attributes, and correct `export { local as exported }` alias ordering.
- Hardened corpus walkers to ignore generated and dependency directories such
  as `node_modules`, `target`, and `dist`.
- Added production-toolchain compiler/type checking through the public
  `ripex::compiler` API and `ripex check` CLI command.
- Added strict compiler-conformance CI for C, C++, C#, Go, JavaScript,
  TypeScript/TSX, Python, and Rust fixture projects.
- Added extension-aware JavaScript, TypeScript, JSX, and TSX parser selection.
- Added a public `Language` API, path detection, and versioned structured CLI JSON.
- Made serialization optional through a dedicated `serde` feature.
- Fixed parser hangs and common syntax handling across Go, Python, Rust, C, C++, and JavaScript.
- Added generator-reuse tests and zero-diagnostic per-language corpus gates.
- Added isolated feature, minimal-library, documentation, and all-target CI checks.

## v0.1.0

### Initial Release

- Hand-written recursive-descent parsers for 7 parser families covering 8
  language modes: JavaScript and TypeScript, Python, Go, Rust, C, C++, C#
- Common pipeline: Lexer → Parser → AST → Facts extraction
- Per-language feature flags for minimal builds
- Fact extraction: symbols, imports, calls, variables with full type/visibility/modifier metadata
- Safety limits: 1 MB input, 200K tokens, 512 recursion depth
- Error recovery in all parsers
- Semantic analysis: scope tracking, symbol tables, binding resolution
- Code generation: JS printer with sourcemaps + minification
- AST transformation: JS pipeline with TypeScript/JSX/module-rewrite plugins
- Visitor/walker/fold patterns for AST traversal
- CLI: `ripex parse <file> [--json|--ast|--facts]`, `ripex ls`
- 145+ tests across all languages
