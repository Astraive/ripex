mod fixtures;

use super::facts::{extract_facts, extract_imports, extract_symbols, extract_variables};
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
    let (program, errors) = parse_program("");
    assert!(errors.is_empty());
    let imports = extract_imports(&program);
    assert!(imports.is_empty());
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
    let (program, errors) = parse_program(fixtures::STRUCT_VAR);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let variables = extract_variables(&program);
    assert!(!variables.is_empty());
    assert_eq!(variables[0].name, "p");
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
    let src = "struct Point { int x; int y; char* label; };";
    let (program, errors) = parse_program(src);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let result = extract_facts(&program);

    // The struct container is emitted as a symbol.
    let struct_sym = result
        .symbols
        .iter()
        .find(|s| s.kind == SymbolKind::Struct && s.name == "Point");
    assert!(
        struct_sym.is_some(),
        "struct Point not emitted: {:?}",
        result.symbols
    );

    // Each member is emitted as a field variable scoped to Point.
    let field_names: Vec<&str> = result
        .variables
        .iter()
        .filter(|v| v.kind == VarKind::Field)
        .map(|v| v.name.as_str())
        .collect();
    assert!(
        field_names.contains(&"x"),
        "field x missing: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"y"),
        "field y missing: {:?}",
        field_names
    );
    assert!(
        field_names.contains(&"label"),
        "field label missing: {:?}",
        field_names
    );
}

#[test]
fn test_extract_enum_members() {
    let src = "enum Color { RED, GREEN, BLUE };";
    let (program, errors) = parse_program(src);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let result = extract_facts(&program);

    let enum_sym = result
        .symbols
        .iter()
        .find(|s| s.kind == SymbolKind::Enum && s.name == "Color");
    assert!(
        enum_sym.is_some(),
        "enum Color not emitted: {:?}",
        result.symbols
    );

    let members: Vec<&str> = result
        .variables
        .iter()
        .filter(|v| v.kind == VarKind::EnumMember)
        .map(|v| v.name.as_str())
        .collect();
    assert!(
        members.contains(&"RED"),
        "enum member RED missing: {:?}",
        members
    );
    assert!(
        members.contains(&"GREEN"),
        "enum member GREEN missing: {:?}",
        members
    );
    assert!(
        members.contains(&"BLUE"),
        "enum member BLUE missing: {:?}",
        members
    );
}

#[test]
fn test_preprocessor_directives_are_retained() {
    let src = "#include \"config.h\"\n#define ENABLED 1\n#if ENABLED\nint enabled(void) { return 1; }\n#endif\n";
    let (program, errors) = parse_program(src);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    assert!(
        program
            .decls
            .iter()
            .filter(|stmt| matches!(stmt, super::ast::stmt::Stmt::Preprocessor(..)))
            .count()
            >= 4,
        "preprocessor directives were dropped: {:?}",
        program.decls
    );
    let imports = extract_imports(&program);
    assert!(
        imports.iter().any(|import| import.source == "config.h"),
        "include fact missing: {:?}",
        imports
    );
    let symbols = extract_symbols(&program);
    assert!(
        !symbols.iter().any(|symbol| symbol.name == "enabled"),
        "conditional declaration incorrectly reported active: {:?}",
        symbols
    );
}
