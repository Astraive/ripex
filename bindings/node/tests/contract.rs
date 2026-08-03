#[path = "../src/lib.rs"]
mod binding;

use binding::{
    detect_language, parse_sync, supported_languages, Language, ParseOptions, ParseOutput, Status,
    TypeKind,
};

fn options(
    language: Option<&str>,
    filename: Option<&str>,
    extension: Option<&str>,
    include_ast_summary: Option<bool>,
) -> ParseOptions {
    ParseOptions {
        language: language.map(str::to_owned),
        filename: filename.map(str::to_owned),
        extension: extension.map(str::to_owned),
        include_ast_summary,
    }
}

fn parse_typescript(source: &str) -> ParseOutput {
    parse_sync(
        source.to_owned(),
        Some(options(Some("typescript"), None, None, None)),
    )
    .expect("TypeScript source should parse through the binding")
}

fn assert_rejected(result: napi::Result<ParseOutput>, expected: &str) {
    let error = match result {
        Ok(_) => panic!("invalid binding options should reject"),
        Err(error) => error,
    };
    let message = error.to_string().to_ascii_lowercase();
    assert!(
        message.contains(expected),
        "error {message:?} did not contain {expected:?}"
    );
}

fn type_kind_depth(kind: &TypeKind) -> usize {
    if kind.items.is_empty() {
        0
    } else {
        1 + kind.items.iter().map(type_kind_depth).max().unwrap_or(0)
    }
}

fn language_id(language: &Language) -> &'static str {
    match language {
        Language::JavaScript => "javascript",
        Language::TypeScript => "typescript",
        Language::Python => "python",
        Language::Go => "go",
        Language::Rust => "rust",
        Language::C => "c",
        Language::Cpp => "cpp",
        Language::CSharp => "csharp",
    }
}

#[test]
fn explicit_typescript_parsing_extracts_all_fact_vectors() {
    let output = parse_typescript(
        r#"import { readFile } from "node:fs";
export function add(a: number, b: number): number {
  return a + b;
}
const result: Array<number> = add(1, 2);
readFile("input.txt");
"#,
    );

    assert!(matches!(output.language, Language::TypeScript));
    assert!(matches!(output.status, Status::Complete));
    assert!(output.completeness);
    assert!(!output.truncated);
    assert!(!output.effective_mode.is_empty());
    assert!(!output.facts.symbols.is_empty(), "expected symbol facts");
    assert!(!output.facts.imports.is_empty(), "expected import facts");
    assert!(!output.facts.calls.is_empty(), "expected call facts");
    assert!(
        !output.facts.variables.is_empty(),
        "expected variable facts"
    );

    let add = output
        .facts
        .symbols
        .iter()
        .find(|symbol| symbol.name == "add")
        .expect("function symbol should be extracted");
    assert!(add.exported);

    let imported = output
        .facts
        .imports
        .iter()
        .find(|import| import.source == "node:fs")
        .expect("import source should be preserved");
    assert!(!imported.specifiers.is_empty());
    assert!(output
        .facts
        .calls
        .iter()
        .any(|call| call.callee_text.contains("add")));
    assert!(output
        .facts
        .variables
        .iter()
        .any(|variable| variable.name == "result"));

    // An object-shaped DTO must retain all vector fields, including empty vectors
    // on inputs that do not produce a particular category.
    let empty = parse_typescript("");
    assert!(empty.facts.symbols.is_empty());
    assert!(empty.facts.imports.is_empty());
    assert!(empty.facts.calls.is_empty());
    assert!(empty.facts.variables.is_empty());
}

#[test]
fn filename_selection_uses_canonical_language_id() {
    let output = parse_sync(
        "export const answer = 42;".to_owned(),
        Some(options(None, Some("src/index.ts"), None, None)),
    )
    .expect("filename-based language selection should succeed");

    assert!(matches!(output.language, Language::TypeScript));
    assert!(matches!(
        detect_language("src/index.ts".to_owned()),
        Some(Language::TypeScript)
    ));
    assert!(detect_language("header.h".to_owned()).is_none());
}

#[test]
fn supported_languages_are_exact_and_sorted() {
    let actual = supported_languages()
        .iter()
        .map(language_id)
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        vec![
            "c",
            "cpp",
            "csharp",
            "go",
            "javascript",
            "python",
            "rust",
            "typescript",
        ]
    );
}

#[test]
fn selector_errors_cover_missing_unknown_and_unimplemented_languages() {
    assert_rejected(parse_sync("const value = 1;".to_owned(), None), "language");
    assert_rejected(
        parse_sync(
            "const value = 1;".to_owned(),
            Some(options(Some("brainfuck"), None, None, None)),
        ),
        "unknown",
    );
    assert_rejected(
        parse_sync(
            "class Example {}".to_owned(),
            Some(options(Some("java"), None, None, None)),
        ),
        "parser",
    );
}

#[test]
fn malformed_source_returns_recovered_diagnostics_without_rejecting() {
    let output = parse_typescript("const = ;");

    assert!(matches!(output.status, Status::Recovered));
    assert!(!output.completeness);
    assert!(!output.truncated);
    assert!(!output.diagnostics.is_empty());
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| !diagnostic.code.is_empty() && !diagnostic.message.is_empty()));
}

#[test]
fn input_limit_is_reported_as_a_value_not_a_panic() {
    let source = "x".repeat(ripex::limits::MAX_INPUT_SIZE + 1);
    let output = parse_sync(source, Some(options(Some("typescript"), None, None, None)))
        .expect("oversized input is a parse result, not an API rejection");

    assert!(matches!(output.status, Status::LimitExceeded));
    assert!(!output.completeness);
    assert!(output.truncated);
    assert!(output
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "input_too_large"));
}

#[test]
fn ast_summary_is_optional_and_preserves_recursive_type_kinds() {
    let source = r#"type Box<T> = { value: T };
const boxed: Box<number> = { value: 1 };
"#;
    let without_summary = parse_sync(
        source.to_owned(),
        Some(options(Some("typescript"), None, None, Some(false))),
    )
    .expect("parse without AST summary");
    assert!(without_summary.ast_summary.is_none());

    let with_summary = parse_sync(
        source.to_owned(),
        Some(options(Some("typescript"), None, None, Some(true))),
    )
    .expect("parse with AST summary");
    let summary = with_summary
        .ast_summary
        .as_ref()
        .expect("AST summary should be present when requested");
    assert!(!summary.kind.is_empty());
    assert!(summary.top_level_nodes > 0);

    let nested = with_summary
        .facts
        .variables
        .iter()
        .map(|variable| &variable.type_kind)
        .find(|type_kind| type_kind_depth(type_kind) >= 1)
        .expect("generic TypeKind should retain nested child items");
    assert!(!nested.items.is_empty());
}

#[test]
fn repeated_sync_calls_have_stable_conversion_results() {
    let source = "export function add(a: number, b: number) { return a + b; }";
    let first = parse_typescript(source);
    let second = parse_typescript(source);

    assert!(matches!(first.language, Language::TypeScript));
    assert!(matches!(second.language, Language::TypeScript));
    assert!(matches!(first.status, Status::Complete));
    assert!(matches!(second.status, Status::Complete));
    assert_eq!(first.completeness, second.completeness);
    assert_eq!(first.truncated, second.truncated);
    assert_eq!(first.effective_mode, second.effective_mode);
    assert_eq!(first.diagnostics.len(), second.diagnostics.len());
    assert_eq!(first.comments.len(), second.comments.len());
    assert_eq!(first.facts.symbols.len(), second.facts.symbols.len());
    assert_eq!(first.facts.imports.len(), second.facts.imports.len());
    assert_eq!(first.facts.calls.len(), second.facts.calls.len());
    assert_eq!(first.facts.variables.len(), second.facts.variables.len());
}
