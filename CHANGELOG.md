# Changelog

## Unreleased

- Added extension-aware JavaScript, TypeScript, JSX, and TSX parser selection.
- Added a public `Language` API, path detection, and versioned structured CLI JSON.
- Made serialization optional through a dedicated `serde` feature.
- Fixed parser hangs and common syntax handling across Go, Python, Rust, C, C++, and JavaScript.
- Added generator-reuse tests and per-language corpus diagnostic budgets.
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
