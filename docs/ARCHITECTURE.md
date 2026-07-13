# Architecture

ripex is a multi-language parser + fact extractor. Each language lives in its own module under
`src/<lang>/` (e.g. `src/js`, `src/python`, `src/cpp`). Every parser implements the
`LanguageParser` trait (`src/lib.rs`) with the same two-phase API:

1. **`parse(&str) -> ParseResult`** — tokenize + build the (possibly partial) AST, recording
   `ParseError`s as it goes. ripex is *resilient*: it recovers from bad tokens instead of aborting,
   so a single syntax error never discards the rest of a file.
2. **`extract(&ParseResult) -> ExtractionResult`** — walk the tree and collect **facts**.

## Fact model

`ExtractionResult` (`src/facts.rs`) holds four vectors:

| Field        | Meaning                              |
|--------------|--------------------------------------|
| `symbols`    | definitions (fn, class, struct, ...) |
| `imports`    | module / file references             |
| `calls`      | call sites                           |
| `variables`  | top-level / notable bindings         |

Every fact carries a `Span` (line/column) so tooling maps it back to source.

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
non-increasing diagnostic budgets for every language.
