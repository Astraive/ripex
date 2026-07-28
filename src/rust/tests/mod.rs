mod fixtures;

use crate::rust::facts;
use crate::rust::lexer::{Lexer, TokenKind};
use crate::rust::parse_program;

#[test]
fn test_lexer_basic_tokens() {
    let (tokens, errors) = Lexer::new("fn main() { return 42; }").tokenize();
    assert!(errors.is_empty(), "lexer errors: {:?}", errors);
    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Fn,
            TokenKind::Ident,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::Return,
            TokenKind::IntLit,
            TokenKind::Semicolon,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn test_parse_empty() {
    let (program, errors) = parse_program("");
    assert!(
        errors.is_empty(),
        "expected no parse errors, got: {:?}",
        errors
    );
    assert!(program.items.is_empty());
}

#[test]
fn test_parse_imports() {
    let (program, errors) = parse_program(fixtures::IMPORT_SIMPLE);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let result = facts::extract_facts(&program);
    assert!(!result.imports.is_empty(), "expected imports");
}

#[test]
fn test_parse_functions() {
    let func_sources = &[
        fixtures::FN_EMPTY,
        fixtures::FN_PARAMS,
        fixtures::FN_GENERIC,
    ];
    for &src in func_sources {
        let (program, errors) = parse_program(src);
        assert!(errors.is_empty(), "parse errors in {:?}: {:?}", src, errors);
        let result = facts::extract_facts(&program);
        assert!(!result.symbols.is_empty(), "expected symbols in: {src}");
    }
}

#[test]
fn test_macro_calls_extracted() {
    let (program, errors) = parse_program("fn main() { let v = vec![1, 2, 3]; println!(\"hi\"); }");
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let calls = facts::extract_calls(&program);
    let names: Vec<&str> = calls.iter().map(|c| c.callee_text.as_str()).collect();
    assert!(
        names.contains(&"vec"),
        "macro vec! not extracted: {:?}",
        names
    );
    assert!(
        names.contains(&"println"),
        "macro println! not extracted: {:?}",
        names
    );
}

#[test]
fn test_parse_structs_enums() {
    let type_sources = &[
        fixtures::STRUCT_SIMPLE,
        fixtures::ENUM_SIMPLE,
        fixtures::TRAIT_DECL,
        fixtures::IMPL_BLOCK,
        fixtures::CONSTRUCTOR,
        fixtures::ASSOCIATED_FN,
    ];
    for &src in type_sources {
        let (program, errors) = parse_program(src);
        assert!(errors.is_empty(), "parse errors in {:?}: {:?}", src, errors);
        let result = facts::extract_facts(&program);
        assert!(!result.symbols.is_empty(), "expected symbols in: {src}");
    }
}

#[test]
fn test_parse_variables() {
    let var_sources = &[fixtures::LET_VAR, fixtures::LET_MUT];
    for &src in var_sources {
        let (program, errors) = parse_program(src);
        assert!(errors.is_empty(), "parse errors in {:?}: {:?}", src, errors);
        let result = facts::extract_facts(&program);
        assert!(!result.variables.is_empty(), "expected variables in: {src}");
    }
}

#[test]
fn test_struct_simple() {
    let (p, e) = parse_program(fixtures::STRUCT_SIMPLE);
    assert!(e.is_empty(), "{:?}", e);
    let r = facts::extract_facts(&p);
    assert!(!r.symbols.is_empty());
}
#[test]
fn test_enum_simple() {
    let (p, e) = parse_program(fixtures::ENUM_SIMPLE);
    assert!(e.is_empty(), "{:?}", e);
    let r = facts::extract_facts(&p);
    assert!(!r.symbols.is_empty());
}
#[test]
fn test_trait_decl() {
    let (p, e) = parse_program(fixtures::TRAIT_DECL);
    assert!(e.is_empty(), "{:?}", e);
    let r = facts::extract_facts(&p);
    assert!(!r.symbols.is_empty());
}
#[test]
fn test_impl_block() {
    let (p, e) = parse_program(fixtures::IMPL_BLOCK);
    assert!(e.is_empty(), "{:?}", e);
    let r = facts::extract_facts(&p);
    assert!(!r.symbols.is_empty());
}
#[test]
fn test_constructor() {
    let (p, e) = parse_program(fixtures::CONSTRUCTOR);
    assert!(e.is_empty(), "{:?}", e);
    let r = facts::extract_facts(&p);
    assert!(!r.symbols.is_empty());
}
#[test]
fn test_associated_fn() {
    let (p, e) = parse_program(fixtures::ASSOCIATED_FN);
    assert!(e.is_empty(), "{:?}", e);
    let r = facts::extract_facts(&p);
    assert!(!r.symbols.is_empty());
}
#[test]
fn test_import_nested_is_atomic_and_has_no_blank_calls() {
    let (program, errors) = parse_program(fixtures::IMPORT_NESTED);
    assert!(errors.is_empty(), "parse errors: {:?}", errors);
    let result = facts::extract_facts(&program);
    let names: Vec<&str> = result.imports.iter().map(|import| import.source.as_str()).collect();
    assert!(names.contains(&"std::io"), "missing self import: {:?}", names);
    assert!(
        names.contains(&"std::io::Read"),
        "missing nested import: {:?}",
        names
    );
    assert!(
        result.calls.iter().all(|call| !call.callee_text.is_empty()),
        "blank call fact emitted: {:?}",
        result.calls
    );
}
#[test]
fn test_fn_async() {
    parse_program(fixtures::FN_ASYNC);
}
#[test]
fn test_if_else() {
    parse_program(fixtures::IF_ELSE);
}
#[test]
fn test_for_loop() {
    parse_program(fixtures::FOR_LOOP);
}
#[test]
fn test_while_loop() {
    parse_program(fixtures::WHILE_LOOP);
}
#[test]
fn test_loop_inf() {
    parse_program(fixtures::LOOP_INF);
}
#[test]
fn test_match_expr() {
    parse_program(fixtures::MATCH_EXPR);
}
#[test]
fn test_macro_vec() {
    parse_program(fixtures::MACRO_VEC);
}
#[test]
fn test_macro_println() {
    parse_program(fixtures::MACRO_PRINTLN);
}
#[test]
fn test_attr_derive() {
    parse_program(fixtures::ATTR_DERIVE);
}
#[test]
fn test_attr_test() {
    parse_program(fixtures::ATTR_TEST);
}
