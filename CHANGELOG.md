# Changelog

## v0.3.0

- Set the default feature set to `lang-all` for language-only library builds;
  the `cli` feature remains opt-in.
- Documented the 0.3.0 dependency and explicit CLI installation/feature usage,
  including the always-on `serde_json` dependency used by the public compiler
  module for `compile_commands.json` support.
- Added explicit Cargo security-contact metadata and updated supported versions
  and vulnerability-reporting contact guidance.

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

- Hand-written recursive-descent parsers for 7 languages: JavaScript/TypeScript, Python, Go, Rust, C, C++, C#
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
