mod fixtures;

use crate::go::facts::extract_facts;
use crate::go::lexer::{Lexer, TokenKind};
use crate::go::parser::parse_program;
use fixtures::*;

#[test]
fn test_lexer_basic_tokens() {
    let src = "package main\nfunc main() {}";
    let (tokens, errors) = Lexer::new(src).tokenize();
    assert!(errors.is_empty());
    let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Package,
            TokenKind::Ident,
            TokenKind::Func,
            TokenKind::Ident,
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Eof,
        ]
    );
    assert_eq!(tokens[1].value, "main");
    assert_eq!(tokens[3].value, "main");
}

#[test]
fn test_parse_empty() {
    let (program, errors) = parse_program("");
    assert!(errors.is_empty());
    assert!(program.decls.is_empty());
}

#[test]
fn test_parse_imports() {
    for (name, source) in &ALL_FIXTURES[..2] {
        let (program, errors) = parse_program(source);
        assert!(errors.is_empty(), "{} produced errors: {:?}", name, errors);
        let facts = extract_facts(&program);
        assert!(
            !facts.imports.is_empty(),
            "expected imports from '{}'",
            name
        );
    }
}

#[test]
fn test_parse_functions() {
    for (name, source) in &ALL_FIXTURES[2..5] {
        let (program, errors) = parse_program(source);
        assert!(errors.is_empty(), "{} produced errors: {:?}", name, errors);
        let facts = extract_facts(&program);
        assert!(
            !facts.symbols.is_empty(),
            "expected symbols from '{}'",
            name
        );
    }
}

#[test]
fn test_parse_types() {
    for (name, source) in &ALL_FIXTURES[5..8] {
        let (program, errors) = parse_program(source);
        assert!(errors.is_empty(), "{} produced errors: {:?}", name, errors);
        let facts = extract_facts(&program);
        assert!(
            !facts.symbols.is_empty(),
            "expected symbols from '{}'",
            name
        );
    }
}

#[test]
fn test_parse_variables() {
    let (program, errors) = parse_program(VAR_DECL);
    assert!(errors.is_empty(), "var decl produced errors: {:?}", errors);
    let facts = extract_facts(&program);
    assert!(
        !facts.variables.is_empty(),
        "expected variables from var decl"
    );

    let (program, errors) = parse_program(CONST_DECL);
    assert!(
        errors.is_empty(),
        "const decl produced errors: {:?}",
        errors
    );
    let facts = extract_facts(&program);
    assert!(
        !facts.variables.is_empty(),
        "expected variables from const decl"
    );
}

#[test]
fn test_parse_all_fixtures() {
    for (name, source) in ALL_FIXTURES {
        let (program, errors) = parse_program(source);
        assert!(
            errors.is_empty(),
            "fixture '{}' produced errors: {:?}",
            name,
            errors
        );
        // Facts extraction must not panic
        let _facts = extract_facts(&program);
    }
}
