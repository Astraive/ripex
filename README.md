# ripex

A language-agnostic parser and fact-extraction library with a CLI. ripex has
seven hand-written recursive-descent parser families and eight language modes
(JavaScript and TypeScript are distinct modes), built around a common pipeline:

`Lexer → Parser → AST → Facts extraction`.

## Supported languages

| Language | Extensions | Lexer | Parser | Facts |
|----------|-----------|-------|--------|-------|
| JavaScript / TypeScript | `.js` `.jsx` `.mjs` `.cjs` `.ts` `.tsx` `.mts` `.cts` | yes | partial | yes |
| Python | `.py` `.pyi` | yes | partial | yes |
| Go | `.go` | yes | partial | yes |
| Rust | `.rs` | yes | partial | yes |
| C | `.c` `.h` | yes | partial | yes |
| C++ | `.cpp` `.hpp` `.cc` `.cxx` `.hh` `.hxx` | yes | partial | yes |
| C# | `.cs` | yes | partial | yes |

The parsers are resilient structural parsers, not compiler front ends. They
return partial ASTs plus diagnostics for unsupported or invalid syntax; they do
not perform full language type checking. Go, JavaScript, Python, and Rust currently parse
the complete checked-in corpus without diagnostics, while the other parsers
have explicit non-increasing diagnostic budgets in CI.

Each language pipeline produces four fact types:

- **Symbols** — functions, classes, structs, enums, traits, interfaces, methods, constants, fields
- **Imports** — `use`, `import`, `#include`, `using`, `from … import`, etc.
- **Calls** — function calls, method calls, constructor calls, path calls
- **Variables** — local bindings, parameters, statics, constants with type info

## Library

```rust
use ripex::{detect_language, parser_for_ext};

// Auto-detect language from extension.
let language = detect_language("src/lib.rs").unwrap();
let parser = parser_for_ext(language.id(), "rs").unwrap();
let result = parser.parse("pub fn add(a: i32, b: i32) -> i32 { a + b }");
let facts = parser.extract(&result);

println!("{} symbols", facts.symbols.len());
```

The crate exposes:
- `LanguageParser` trait — `parse()`, `extract()`, `symbols()`, `imports()`, `calls()`, `variables()`
- `parser_for(lang_id)` / `parser_for_ext(lang_id, ext)` — language registry
- `ExtractionResult` — `symbols`, `imports`, `calls`, `variables` vectors
- Fact types: `ParsedSymbol`, `ParsedImport`, `ParsedCall`, `ParsedVariable`

All fact types are serializable when the `serde` feature is enabled. The `cli`
feature enables `serde` automatically.

### Feature flags

Each language is behind a feature flag, allowing minimal builds:

```toml
[dependencies]
ripex = { version = "0.1", default-features = false, features = ["lang-rust", "lang-js"] }
```

| Feature | Languages |
|---------|-----------|
| `lang-js` (default) | JavaScript + TypeScript + JSX |
| `lang-python` (default) | Python |
| `lang-go` (default) | Go |
| `lang-rust` (default) | Rust |
| `lang-c` (default) | C |
| `lang-cpp` (default) | C++ |
| `lang-csharp` (default) | C# |
| `cli` (default) | CLI binary (`serde`, `clap`, `anyhow`) |
| `serde` | Serialization for public facts, spans, and diagnostics |
| `lang-all` (default) | All language features |

## CLI

```sh
cargo build

# Parse a file; language auto-detected from extension.
ripex parse path/to/file.rs

# Force a language and emit facts as JSON.
ripex parse src/models/product.js --lang javascript --json

# List supported parsers.
ripex ls
```

Flags: `--json` (versioned machine output), `--ast` (AST shape summary), `--facts`
(symbol/import/call/variable listing). The process exits non-zero when parse
errors are present. JSON includes `schema_version`, the selected language,
structured diagnostics, an AST summary, and all extracted facts.

## Architecture

Each language follows the same module structure:

```
src/<lang>/
  lexer/        Scanner, token definitions, keyword tables
  parser/       Recursive-descent parser with error recovery
  ast/          Typed AST nodes (statements, expressions, declarations)
  facts.rs      AST → ExtractionResult (the consumer-facing fact extractor)
  semantic/     Scope tracking, symbol tables, binding resolution
  syntax/       Language feature flags, operator precedence
  codegen/      Experimental canonical printers (not source preserving)
  transform/    AST transformation passes (JS pipeline + plugins)
  visit/        Visitor, walker, fold patterns for AST traversal
  diagnostics/  Error types and reporter
  config/       Parser options and configuration
  tests/        Per-language test suites
```

### Safety limits

Built-in guards prevent pathological inputs from causing runaway allocation:

- **MAX_INPUT_SIZE**: 1 MB
- **MAX_TOKENS**: 200,000
- **MAX_RECURSION**: 512 depth

All guards are enforced in the lexer and parser state machines.

## Testing

The test suite covers lexer tokenization, parser correctness, feature-isolated
builds, fact extraction, generator reuse, safety limits, and fixture parsing.

```sh
cargo test --all-targets --all-features
```

## License

MIT
