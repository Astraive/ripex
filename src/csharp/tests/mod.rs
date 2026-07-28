mod fixtures;

use super::facts::{extract_calls, extract_imports, extract_symbols, extract_variables};
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
    let (program, errors) = parse_program(fixtures::USING_SYSTEM);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let imports = extract_imports(&program);
    assert_eq!(imports.len(), 1);
    assert_eq!(imports[0].source, "System");
    assert_eq!(imports[0].kind, ImportKind::NamedImport);
}

#[test]
fn test_parse_functions_methods() {
    let (program, errors) = parse_program(fixtures::INTERFACE);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    let baz = symbols
        .iter()
        .find(|s| s.name == "Baz" || s.name.ends_with(".Baz"));
    assert!(baz.is_some());
    assert_eq!(baz.unwrap().kind, SymbolKind::Method);
}

#[test]
fn test_parse_classes_structs() {
    let (program, errors) = parse_program(fixtures::CLASS_SIMPLE);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    assert_eq!(symbols[0].name, "Foo");
    assert_eq!(symbols[0].kind, SymbolKind::Class);
}

#[test]
fn test_method_parameters_extracted() {
    let (program, errors) =
        parse_program("class Foo { public int Add(int a, int b) { return a + b; } }");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let variables = extract_variables(&program);
    let param_names: Vec<&str> = variables
        .iter()
        .filter(|v| v.kind == VarKind::Parameter)
        .map(|v| v.name.as_str())
        .collect();
    assert!(
        param_names.contains(&"a"),
        "param a missing: {:?}",
        param_names
    );
    assert!(
        param_names.contains(&"b"),
        "param b missing: {:?}",
        param_names
    );
}

#[test]
fn test_local_variables_in_method_body() {
    let (program, errors) = parse_program(
        "class Foo { public int Add(int a) { int total = a; string label = \"x\"; return total; } }",
    );
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let variables = extract_variables(&program);
    let locals: Vec<&str> = variables
        .iter()
        .filter(|v| v.kind == VarKind::Var)
        .map(|v| v.name.as_str())
        .collect();
    assert!(
        locals.contains(&"total"),
        "local total missing: {:?}",
        locals
    );
    assert!(
        locals.contains(&"label"),
        "local label missing: {:?}",
        locals
    );
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
fn test_constructors_visibility_and_expression_bodies() {
    let src = r#"
class User : Base {
    public User(int id) : base(id) { Initialize(id); }
    public static User Create(int id) => new User(id);
    public string Name => GetName();
}
"#;
    let (program, errors) = parse_program(src);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let symbols = extract_symbols(&program);
    let ctor = symbols
        .iter()
        .find(|symbol| symbol.kind == SymbolKind::Constructor && symbol.name == "User")
        .expect("User constructor missing");
    assert_eq!(ctor.visibility, Visibility::Public);
    let create = symbols
        .iter()
        .find(|symbol| symbol.name == "User.Create")
        .expect("expression-bodied method missing");
    assert_eq!(create.visibility, Visibility::Public);
    let calls = extract_calls(&program);
    assert!(
        calls.iter().any(|call| call.callee_text == "User"),
        "constructor call from expression body missing: {:?}",
        calls
    );
    assert!(
        calls.iter().any(|call| call.callee_text == "GetName"),
        "property expression body call missing: {:?}",
        calls
    );
}
