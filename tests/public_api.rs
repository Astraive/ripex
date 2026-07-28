#[allow(unused_imports)]
use ripex::{detect_language, parser_for_ext, CommentKind, Language};

#[test]
fn detects_all_documented_extensions_case_insensitively() {
    let cases = [
        ("index.JSX", Language::JavaScript),
        ("types.mts", Language::TypeScript),
        ("component.TSX", Language::TypeScript),
        ("stub.pyi", Language::Python),
        ("main.go", Language::Go),
        ("lib.rs", Language::Rust),
        ("header.c", Language::C),
        ("header.hxx", Language::Cpp),
        ("Program.cs", Language::CSharp),
    ];

    for (path, expected) in cases {
        assert_eq!(detect_language(path), Some(expected), "{path}");
    }
    assert_eq!(detect_language("header.h"), None);
}

#[cfg(feature = "lang-js")]
#[test]
fn typescript_ids_enable_typescript_without_an_extension() {
    let parser = parser_for_ext("typescript", "").expect("TypeScript parser");
    let result = parser.parse("interface User { id: number }");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
}

#[cfg(feature = "lang-js")]
#[test]
fn tsx_extension_enables_typescript_and_jsx() {
    let source = "const view = <div id=\"app\">Hello</div>;";
    let parser = parser_for_ext("typescript", "tsx").expect("TSX parser");
    let result = parser.parse(source);
    assert!(result.errors.is_empty(), "{:?}", result.errors);
}

#[cfg(feature = "lang-js")]
#[test]
fn checked_in_tsx_fixture_parses_cleanly() {
    let source = include_str!("lang-test/javascript/src/app.tsx");
    let parser = parser_for_ext("typescript", "tsx").expect("TSX parser");
    let result = parser.parse(source);
    assert!(result.errors.is_empty(), "{:#?}", result.errors);
}

#[cfg(feature = "lang-js")]
#[test]
fn module_and_function_await_are_legal() {
    let parser = parser_for_ext("javascript", "mjs").expect("JavaScript module parser");
    let result = parser.parse(
        "export async function load() { return await fetch('/data'); }\nconst ready = await load();",
    );
    assert!(result.errors.is_empty(), "{:?}", result.errors);
}

#[cfg(feature = "lang-js")]
#[test]
fn javascript_parser_accepts_standard_import_attributes() {
    let parser = parser_for_ext("javascript", "mjs").expect("JavaScript module parser");
    let result = parser.parse("import config from './config.json' with { type: 'json' };");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
}

#[cfg(feature = "lang-js")]
#[test]
fn generator_yield_is_legal() {
    let parser = parser_for_ext("javascript", "js").expect("JavaScript parser");
    let result = parser.parse("export function* values() { yield 1; }");
    assert!(result.errors.is_empty(), "{:?}", result.errors);
}

#[cfg(feature = "lang-js")]
#[test]
fn javascript_parser_retains_comments_and_hashbang_spans() {
    let parser = parser_for_ext("javascript", "mjs").expect("JavaScript parser");
    let result = parser.parse(
        "#!/usr/bin/env node\n// leading comment\nconst answer = 42;\n/* trailing comment */",
    );

    assert!(result.errors.is_empty(), "{:?}", result.errors);
    assert_eq!(result.comments.len(), 3, "{:#?}", result.comments);
    assert_eq!(result.comments[0].kind, CommentKind::Hashbang);
    assert_eq!(result.comments[0].text, "#!/usr/bin/env node");
    assert_eq!(result.comments[0].span.start.line, 1);
    assert_eq!(result.comments[1].kind, CommentKind::Line);
    assert_eq!(result.comments[1].span.start.line, 2);
    assert_eq!(result.comments[2].kind, CommentKind::Block);
    assert_eq!(result.comments[2].span.start.line, 4);
}

#[cfg(all(feature = "lang-js", feature = "cli"))]
#[test]
fn every_extracted_fact_kind_is_serializable() {
    let parser = parser_for_ext("typescript", "ts").expect("TypeScript parser");
    let result = parser.parse(
        "import { value } from 'pkg'; const answer: number = value(); export function f() {}",
    );
    let facts = parser.extract(&result).expect("complete extraction");
    let json = serde_json::to_value(facts).expect("serialize extraction result");

    assert!(json["symbols"].is_array());
    assert!(json["imports"].is_array());
    assert!(json["calls"].is_array());
    assert!(json["variables"].is_array());
}
