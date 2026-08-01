# Architecture

ripex is a multi-language parser and fact extractor. Each language lives in its own module under
`src/<lang>/` (e.g. `src/js`, `src/python`, `src/cpp`). Every parser implements the
`LanguageParser` trait (`src/lib.rs`) with the same two-phase API:

Ripex v0.3.0 exposes eight language modes through seven parser families:
JavaScript and TypeScript are distinct modes backed by the JavaScript parser
family. Cargo features (`lang-js`, `lang-python`, `lang-go`, `lang-rust`,
`lang-c`, `lang-cpp`, and `lang-csharp`) select parser families; `lang-all`
enables them all, while the `cli` feature remains an explicit opt-in for the
`ripex` command.

1. **`parse(&str) -> ParseResult`** — tokenize + build the (possibly partial) AST, recording
   `ParseError`s as it goes. ripex is *resilient*: it recovers from bad tokens instead of aborting,
   so a single syntax error never discards the rest of a file.
2. **`extract(&ParseResult) -> ExtractionResult`** — walk the tree and collect **facts**.

## Fact model

`ExtractionResult` (`src/facts.rs`) holds four vectors:

| Field | Meaning |
|-------|---------|
| `symbols` | Definitions such as functions, methods, classes, structs, traits, and enums |
| `imports` | Module/file references, bindings, specifiers, and re-export metadata |
| `calls` | Call sites, including receiver/object, source position, and call modifiers |
| `variables` | Bindings with declaration scope, type, storage, and usage-site metadata |

Facts preserve source positions appropriate to their kind. `ParseResult::comments`
also retains source-preserving JavaScript/TypeScript hashbang, line, and block
comments with spans.

### Rich structural payloads

The four top-level collections are deliberately small; their payloads retain the
language-native detail needed by a downstream indexer without requiring it to
reparse the file:

- `ParsedSymbol` carries the full signature, visibility, return type, async and
  constructor/destructor/static flags, parameters (including annotations and
  defaults), docstring, attributes, base classes, storage, template parameters,
  and type classification when the language provides them.
- `ParsedImport` preserves type-only and re-export flags, namespace/star imports,
  module paths, and individual import specifiers.
- `ParsedCall` records the semantic callee text, optional receiver, line/column,
  awaited and optional-call flags, and generic type arguments. Extractors use
  `try_build` so an empty callee is rejected instead of becoming a fabricated
  fact.
- `ParsedVariable` records declaration scope, usage sites, mutability, type and
  storage metadata, and import/static/constexpr/thread-local/extern qualifiers
  where applicable.

All of these payloads are serializable behind the `serde` feature. Consumers
should preserve them losslessly and derive normalized cross-file relationships
as a separate stage.

## Registration

`ripex::registry()` returns `(language_id, Box<dyn LanguageParser>)` entries for
the enabled parser families. `Language::TypeScript` selects the JavaScript
parser with TypeScript syntax enabled rather than adding a duplicate registry entry.
`parser_for_ext(lang, ext)` resolves a concrete parser from a file extension.

## No-hang rule

Recursive-descent loops must always make forward progress. A loop that only advances on a narrow
token set spins forever on an unexpected token — this was the bug class caught by the
`lang-test` corpus and fixed in several parsers. The corpus gate
(`tests/ripex_lang_test_repos.rs`) hard-asserts zero panics and zero hangs, plus
zero parser diagnostics for every language.

## Optional External Validation

`src/compiler.rs` is intentionally independent from the structural ASTs and
is not part of Ripex's parser implementation. It
plans bounded external toolchain stages, executes them with captured output,
normalizes common diagnostic formats, aggregates stage status, and cleans any
temporary compiler artifacts. File checks use the language compiler directly;
project checks discover Cargo, Go, TypeScript/JavaScript, and .NET manifests.
For C and C++, a `compile_commands.json` database is treated as the project
source of truth: its compilation entries are converted to no-output semantic
checks while preserving compiler selection, include paths, defines, target
flags, and generated-header configuration.

The separation is required for correctness: standards-conforming type checking
depends on project graphs, SDKs, target configuration, macros, and compiler
versions that are outside a structural parser's AST. `unavailable`, timeout,
and invocation failures are first-class non-success statuses.

The corpus gate and curated evidence report therefore establish structural
parser acceptance and fact extraction only. They do not substitute for
compiler-level semantic validation; that evidence comes from the separate
production-toolchain checks described above.