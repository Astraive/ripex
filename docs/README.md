# ripex

ripex is a fast, dependency-light **multi-language parser and fact extractor** — tree-sitter-like in
scope, but hand-written (no grammar-codegen step), so it compiles in seconds and is easy to extend.

It parses 8 languages and extracts structural "facts" (definitions, imports, calls, variables) that
downstream code-intelligence tooling (e.g. the Graxus engine) consumes.

- **Languages:** JavaScript / TypeScript / JSX / TSX, Python, Go, Rust, C, C++, C#
- **CLI:** `ripex parse <file>` parses + extracts facts from one file
- **Library:** `use ripex::registry;` then `parser.parse()` + `parser.extract()`

## Layout

```
ripex/
├── src/          library code (per-language modules under src/<lang>/)
├── tests/        integration tests + lang-test/ corpus + fuzz/ harness
├── examples/     runnable demos (scan, probe_file, probe)
├── benches/      criterion benchmarks
├── docs/         this documentation
├── scripts/      dev helper scripts
└── .github/      CI workflows
```

## Further reading

- [Architecture](ARCHITECTURE.md) — parser design and the `Fact` / `ExtractionResult` model
- [Testing](TESTING.md) — the `lang-test` corpus gate and fuzzing
