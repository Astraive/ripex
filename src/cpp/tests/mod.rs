mod fixtures;

use super::facts::{extract_facts, extract_imports, extract_symbols};
use super::lexer::{Lexer, TokenKind};
use super::parser::parse_program;
use crate::facts::*;

#[test]
fn test_lexer_basic_tokens() {
    let (tokens, errors) = Lexer::new("int x = 42;").tokenize();
    assert!(errors.is_empty());
    assert_eq!(tokens[0].kind, TokenKind::Int);
    assert_eq!(tokens[1].kind, TokenKind::Ident);
    assert_eq!(tokens[1].value, "x");
    assert_eq!(tokens[2].kind, TokenKind::Eq);
    assert_eq!(tokens[3].kind, TokenKind::IntLit);
    assert_eq!(tokens[3].value, "42");
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
}

#[test]
fn test_parse_empty() {
    let (program, errors) = parse_program("");
    assert!(errors.is_empty());
    assert!(program.decls.is_empty());
}

#[test]
fn test_parse_imports() {
    let (program, errors) = parse_program(fixtures::USING_NAMESPACE);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let imports = extract_imports(&program);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "std");
    assert_eq!(imports[0].kind, ImportKind::NamespaceImport);
}

#[test]
fn test_parse_functions_methods() {
    let (program, errors) = parse_program(fixtures::FUNC_ADD);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "add");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
}

#[test]
fn test_parse_classes_structs() {
    let (program, errors) = parse_program(fixtures::CLASS_SIMPLE);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols[0].name, "MyClass");
    assert_eq!(symbols[0].kind, SymbolKind::Class);
}

#[test]
fn test_parse_all_fixtures() {
    for (i, fixture) in fixtures::ALL_FIXTURES.iter().enumerate() {
        let (program, errors) = parse_program(fixture);
        assert!(
            errors.is_empty(),
            "fixture {} produced parse errors: {:?}\nsource: {:?}",
            i,
            errors,
            fixture
        );
        assert!(!program.decls.is_empty());
    }
}

#[test]
fn test_extract_struct_fields() {
    let src = "struct Point { int x; int y; const char* label; };";
    let (program, errors) = parse_program(src);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let result = extract_facts(&program);

    // C++ folds `struct` into a class declaration at parse time.
    assert!(
        result
            .symbols
            .iter()
            .any(|s| s.kind == SymbolKind::Class && s.name == "Point"),
        "struct Point not emitted: {:?}",
        result.symbols
    );
    let fields: Vec<&str> = result
        .variables
        .iter()
        .filter(|v| v.kind == VarKind::Field)
        .map(|v| v.name.as_str())
        .collect();
    assert!(fields.contains(&"x"), "field x missing: {:?}", fields);
    assert!(fields.contains(&"y"), "field y missing: {:?}", fields);
    assert!(
        fields.contains(&"label"),
        "field label missing: {:?}",
        fields
    );
}

#[test]
fn test_extract_enum_members() {
    let src = "enum Color { RED, GREEN, BLUE };";
    let (program, errors) = parse_program(src);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let result = extract_facts(&program);

    assert!(
        result
            .symbols
            .iter()
            .any(|s| s.kind == SymbolKind::Enum && s.name == "Color"),
        "enum Color not emitted: {:?}",
        result.symbols
    );
    let members: Vec<&str> = result
        .variables
        .iter()
        .filter(|v| v.kind == VarKind::EnumMember)
        .map(|v| v.name.as_str())
        .collect();
    assert!(members.contains(&"RED"), "RED missing: {:?}", members);
    assert!(members.contains(&"GREEN"), "GREEN missing: {:?}", members);
    assert!(members.contains(&"BLUE"), "BLUE missing: {:?}", members);
}

#[test]
fn test_include_new_and_lambda_facts() {
    let src = r#"#include "widget.hpp"
#include <vector>
Widget* make() {
    auto value = new Widget(1);
    auto run = []() { target(); };
    run();
}
"#;
    let (program, _errors) = parse_program(src);
    let result = extract_facts(&program);
    let imports: Vec<&str> = result.imports.iter().map(|item| item.source.as_str()).collect();
    assert!(imports.contains(&"widget.hpp"), "quoted include missing: {:?}", imports);
    assert!(imports.contains(&"vector"), "system include missing: {:?}", imports);
    assert!(
        result
            .calls
            .iter()
            .any(|call| call.kind == CallKind::ConstructorCall && call.callee_text == "Widget"),
        "new Widget constructor call missing: {:?}",
        result.calls
    );
    assert!(
        result.calls.iter().any(|call| call.callee_text == "target"),
        "lambda body call missing: {:?}",
        result.calls
    );
}
