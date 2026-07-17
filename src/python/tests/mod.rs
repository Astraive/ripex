mod fixtures;

#[test]
fn parses_multi_argument_generic_annotations() {
    let source = "def apply(fn: Callable[P, R]) -> Generator[int, None, None]:\n    pass\n";
    let (_, errors) = crate::python::parse_program(source);
    assert!(errors.is_empty(), "{errors:#?}");
}

use super::facts::{extract_facts, extract_imports, extract_symbols, extract_variables};
use super::lexer::{Lexer, TokenKind};
use super::parser::parse_program;
use crate::facts::*;

#[test]
fn test_lexer_basic_tokens() {
    let (tokens, errors) = Lexer::new("x = 42\n").tokenize();
    assert!(errors.is_empty());
    assert_eq!(tokens[0].kind, TokenKind::Ident);
    assert_eq!(tokens[0].value, "x");
    assert_eq!(tokens[1].kind, TokenKind::Eq);
    assert_eq!(tokens[2].kind, TokenKind::IntLit);
    assert_eq!(tokens[2].value, "42");
}

#[test]
fn test_parse_empty() {
    let (program, errors) = parse_program("");
    assert!(errors.is_empty());
    assert!(program.stmts.is_empty());
}

#[test]
fn test_parse_imports() {
    let (program, errors) = parse_program(fixtures::IMPORT_SIMPLE);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let facts = extract_facts(&program);
    assert_eq!(facts.imports.len(), 1);
    assert_eq!(facts.imports[0].source, "os");
    assert_eq!(facts.imports[0].kind, ImportKind::PythonImport);
}

#[test]
fn test_parse_import_multi() {
    let (program, errors) = parse_program(fixtures::IMPORT_MULTI);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let imports = extract_imports(&program);
    assert_eq!(imports.len(), 2);
    assert_eq!(imports[0].source, "os");
    assert_eq!(imports[1].source, "sys");
}

#[test]
fn test_parse_function_no_args() {
    let (program, errors) = parse_program(fixtures::FUNC_NO_ARGS);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "calc");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
}

#[test]
fn test_parse_function_with_args() {
    let (program, errors) = parse_program(fixtures::FUNC_WITH_ARGS);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "calc");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
}

#[test]
fn test_parse_function_async() {
    let (program, errors) = parse_program(fixtures::FUNC_ASYNC);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "load");
    assert!(symbols[0].is_async);
}

#[test]
fn test_parse_class_empty() {
    let (program, errors) = parse_program(fixtures::CLASS_EMPTY);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].name, "MyClass");
    assert_eq!(symbols[0].kind, SymbolKind::Class);
}

#[test]
fn test_parse_class_with_init() {
    let (program, errors) = parse_program(fixtures::CLASS_WITH_INIT);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].name, "MyClass");
    assert_eq!(symbols[0].kind, SymbolKind::Class);
    assert_eq!(symbols[1].name, "__init__");
    assert_eq!(symbols[1].kind, SymbolKind::Constructor);
}

#[test]
fn test_parse_class_decorated() {
    let (program, errors) = parse_program(fixtures::CLASS_DECORATED);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols[0].name, "Point");
    assert_eq!(symbols[0].kind, SymbolKind::Class);
}

#[test]
fn test_parse_assign_simple() {
    let (program, errors) = parse_program(fixtures::ASSIGN_SIMPLE);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let vars = extract_variables(&program);
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].name, "x");
}

#[test]
fn test_parse_ann_assign() {
    let (program, errors) = parse_program(fixtures::ANN_ASSIGN);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let vars = extract_variables(&program);
    assert_eq!(vars.len(), 1);
    assert_eq!(vars[0].name, "x");
}

#[test]
fn test_parse_aug_assign() {
    let (_program, errors) = parse_program(fixtures::AUG_ASSIGN);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
}

#[test]
fn test_parse_decorator_func() {
    let (program, errors) = parse_program(fixtures::DECORATOR_FUNC);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols[0].name, "index");
    assert_eq!(symbols[0].kind, SymbolKind::Function);
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
        assert!(!program.stmts.is_empty());
    }
}
