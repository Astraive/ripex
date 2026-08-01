# ripex

A language-agnostic parser and fact-extraction library with a CLI. ripex has
seven hand-written recursive-descent parser families and eight language modes
(JavaScript and TypeScript are distinct modes), built around a common pipeline:

`Lexer → Parser → AST → Facts extraction`.

ripex is not a compiler, linker, runtime, or type checker. The optional
`check` command delegates validation to the language's existing production
toolchain; it does not reimplement compiler semantics.

## Ripex v0.3.0

Ripex v0.3.0 was published on 2026-08-01:
[crates.io package](https://crates.io/crates/ripex/0.3.0) ·
[docs.rs API documentation](https://docs.rs/ripex/0.3.0) ·
[GitHub release](https://github.com/Astraive/ripex/releases/tag/v0.3.0).

## Supported languages

| Language | Extensions | Structural parser | Optional external validation |
|----------|------------|-------------------|-----------------------|
| JavaScript | `.js` `.jsx` `.mjs` `.cjs` | yes | Node + TypeScript `checkJs` |
| TypeScript | `.ts` `.tsx` `.mts` `.cts` | yes | TypeScript `tsc --noEmit --strict` |
| Python | `.py` `.pyi` | yes | CPython compile + `mypy --strict` |
| Go | `.go` | yes | `go build` |
| Rust | `.rs` | yes | `rustc` / `cargo check` |
| C | `.c` `.h` | yes | GCC / Clang, strict C17 by default; `compile_commands.json` projects |
| C++ | `.cpp` `.hpp` `.cc` `.cxx` `.hh` `.hxx` | yes | GCC / Clang, strict C++20 by default; `compile_commands.json` projects |
| C# | `.cs` | yes | `dotnet build` / `csc` |

The in-process parsers are resilient structural parsers optimized for
high-fidelity source facts, not replacement compiler front ends. Name resolution, project
loading, linking, SDK behavior, and type semantics remain the responsibility
of each language's production toolchain. A missing toolchain is reported as
`unavailable` and never treated as success.

All eight language modes parse the complete checked-in corpus without
diagnostics. CI also compiles/type-checks the corresponding real fixture
projects and requires every language to extract symbols, imports, calls, and
variables, with no parser panics or hangs.

The checked-in corpus and curated evidence report establish structural parsing
and fact-extraction behavior only. They do not establish compiler-level
semantic equivalence; use `ripex check` and the production-toolchain
conformance gate for that separate question. See
[`docs/BENCHMARKS.md`](docs/BENCHMARKS.md) for measured limits and
[`docs/TESTING.md`](docs/TESTING.md) for the evidence and compiler gates.

Each language pipeline produces four fact types:

- **Symbols** — functions, classes, structs, enums, traits, interfaces, methods, constants, fields
- **Imports** — `use`, `import`, `#include`, `using`, `from … import`, etc.
- **Calls** — function calls, method calls, constructor calls, path calls
- **Variables** — local bindings, parameters, statics, constants with type info

JavaScript and TypeScript also retain hashbang, line, and block comments with
precise source spans through `ParseResult::comments`, including comments at
end-of-file. Type-only imports/re-exports, dynamic imports, optional calls,
awaited calls, generic call arguments, and destructured bindings are emitted
as first-class facts.

The public fact payloads retain the detail needed for lossless downstream
indexing: symbol parameters/defaults/docstrings/attributes and type metadata;
import specifiers and re-export metadata; awaited, optional, receiver, and
generic call metadata; plus variable scope, usage-site, and storage metadata.
Consumers can serialize these native facts with the `serde` feature, then build
their own cross-file relationships without discarding parser provenance.

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

External toolchain validation is a separate explicit operation:

```rust
use ripex::compiler::{check_with_compiler, CompilerCheckOptions};

let report = check_with_compiler(
    "tests/lang-test/rust/Cargo.toml",
    None,
    &CompilerCheckOptions::default(),
)?;
assert!(report.passed(), "{report:#?}");
# Ok::<(), std::io::Error>(())
```

The crate exposes:
- `LanguageParser` trait — `parse()`, `extract()`, `symbols()`, `imports()`, `calls()`, `variables()`
- `parser_for(lang_id)` / `parser_for_ext(lang_id, ext)` — language registry
- `ExtractionResult` — `symbols`, `imports`, `calls`, `variables` vectors
- Fact types: `ParsedSymbol`, `ParsedImport`, `ParsedCall`, `ParsedVariable`
- `check_with_compiler()` / `plan_compiler_check()` — optional bounded production-toolchain validation
- `CompilerCheckReport` — per-stage commands, status, exit code, output, and normalized diagnostics

All fact types are serializable when the `serde` feature is enabled. The `cli`
feature enables `serde` automatically.

`serde_json` remains a required dependency for library builds because the public
`ripex::compiler` module parses `compile_commands.json`; the `serde` feature
only controls serialization derives for public fact and diagnostic types.

### Feature flags

Each language is behind a feature flag, allowing minimal builds. Add Ripex as a
library dependency and choose only the language features you need:

```toml
[dependencies]
ripex = { version = "0.3.0", default-features = false, features = ["lang-rust", "lang-js"] }
```

| Feature | Languages |
|---------|-----------|
| `lang-js` | JavaScript + TypeScript + JSX |
| `lang-python` | Python |
| `lang-go` | Go |
| `lang-rust` | Rust |
| `lang-c` | C |
| `lang-cpp` | C++ |
| `lang-csharp` | C# |
| `cli` | CLI binary (`serde`, `clap`, `anyhow`) |
| `serde` | Serialization for public facts, spans, and diagnostics |
| `lang-all` (default) | All language features |

## CLI

To install the CLI binary, opt in explicitly:

```sh
cargo install ripex --version 0.3.0 --features cli
```

```sh
cargo build --features cli

# Parse a file; language auto-detected from extension.
cargo run --features cli -- parse path/to/file.rs

# Force a language and emit facts as JSON.
cargo run --features cli -- parse src/models/product.js --lang javascript --json

# List supported parsers.
cargo run --features cli -- ls

# Optionally validate a file with its production compiler.
cargo run --features cli -- check src/lib.rs

# Discover the nearest project manifest and check the whole project.
cargo run --features cli -- check src/lib.rs --project --json

# Select a standard and pass include/configuration flags through to the compiler.
cargo run --features cli -- check native/main.cpp --standard c++23 --arg=-Iinclude
```

Flags: `--json` (versioned machine output), `--ast` (AST shape summary), `--facts`
(symbol/import/call/variable listing). The process exits non-zero when parse
errors are present. JSON includes `schema_version`, the selected language,
structured diagnostics, an AST summary, and all extracted facts.

`check` exits `0` when every compiler stage passes, `1` when source is rejected,
and `2` when a toolchain is unavailable, times out, or cannot be invoked. Each
stage has a 120-second default timeout. Project checks can execute compiler
plugins, procedural macros, and build scripts; only check trusted projects.

### Compiler requirements

ripex discovers these commands on `PATH`: `gcc`/`clang`, `g++`/`clang++`,
`rustc`/`cargo`, `go`, `dotnet`/`csc`, `node`, `tsc`, `python`, and `mypy`.
TypeScript can also be supplied as a local project dependency, discovered via
`npx --no-install tsc`; the checked-in JavaScript fixture pins this dependency.
Use `--toolchain PATH` to override the primary compiler. C/C++ include paths,
defines, target triples, SDKs, and other build-specific inputs can be supplied
with repeated `--arg` options. Language-standard conformance is therefore the
conformance of the selected production compiler, standard, target, and project
configuration rather than a claim that ripex reimplements those compilers.

For a C/C++ directory containing `compile_commands.json`, `check` validates
each configured translation unit using its recorded compiler, include paths,
defines, target flags, and generated-header settings. Output and dependency
generation flags are removed and replaced with a no-output semantic check, so
the project is not rebuilt or modified. Pass `--project` when checking one
source file and the database is in a parent directory.

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
builds, every fact category, generator reuse, corpus parse-generate-parse round
trips, safety limits, and zero-diagnostic fixture parsing.

```sh
cargo test --all-targets --all-features
```

## Verification and Safety

### Resource Limits and Boundaries
To prevent denial of service and resource exhaustion, `ripex` enforces strict limits on all parsed inputs:
- **Maximum Input Size**: 1 MB (`MAX_INPUT_SIZE` = 1,048,576 bytes). Enforced at CLI ingestion before allocations occur.
- **Maximum Token Count**: 200,000 tokens (`MAX_TOKENS`). Enforced during lexing.
- **Maximum Recursion Depth**: 512 frames (`MAX_RECURSION`). Enforced in all recursive-descent parsers to prevent stack overflow.

Breaching any limit produces a structured diagnostic of code `limit_exceeded` and sets `status` to `LimitExceeded`.

### Trust Boundary and Compiler Safety
The `ripex check` command executes external compilers (such as `gcc`, `clang`, `rustc`/`cargo`, `go`, `dotnet`/`csc`, `node`, `tsc`, `python`, and `mypy`) on target source code or projects.
- **Project Execution Risk**: Running compilation check on projects (e.g., Cargo workspace, Go module, C# csproj) can execute arbitrary code via build scripts (`build.rs`), compiler plugins, or procedural macros.
- **Explicit Trust Required**: To prevent unauthorized code execution, project-level checks and raw compiler argument passthrough require the `--trusted-project` flag. If this flag is omitted, the planner immediately returns a `PermissionDenied` error.
- **Sandbox Recommendation**: We recommend running `ripex check` in an OS-level sandbox or containerized environment when validating untrusted third-party code.

## License

MIT
