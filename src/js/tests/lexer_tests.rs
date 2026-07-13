use crate::js::lexer::*;

#[test]
fn test_empty_source() {
    let mut lexer = Lexer::new("");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Eof);
}

#[test]
fn test_identifier() {
    let mut lexer = Lexer::new("goodbye");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident);
    assert_eq!(token.value, "goodbye");
}

#[test]
fn test_keyword() {
    let mut lexer = Lexer::new("function");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Function);
}

#[test]
fn test_number_literal() {
    let mut lexer = Lexer::new("42");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
    assert_eq!(token.value, "42");
}

#[test]
fn test_float_literal() {
    let mut lexer = Lexer::new("3.14");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
    assert_eq!(token.value, "3.14");
}

#[test]
fn test_string_double() {
    let mut lexer = Lexer::new("\"goodbye\"");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String);
    assert_eq!(token.value, "goodbye");
}

#[test]
fn test_string_single() {
    let mut lexer = Lexer::new("'goodbye'");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String);
    assert_eq!(token.value, "goodbye");
}

#[test]
fn test_operators() {
    let mut lexer = Lexer::new("+");
    assert_eq!(lexer.next_token().kind, TokenKind::Plus);
}

#[test]
fn test_punctuators() {
    let mut lexer = Lexer::new(";");
    assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
    let mut lexer = Lexer::new(",");
    assert_eq!(lexer.next_token().kind, TokenKind::Comma);
    let mut lexer = Lexer::new(".");
    assert_eq!(lexer.next_token().kind, TokenKind::Dot);
    let mut lexer = Lexer::new("(");
    assert_eq!(lexer.next_token().kind, TokenKind::LParen);
    let mut lexer = Lexer::new(")");
    assert_eq!(lexer.next_token().kind, TokenKind::RParen);
    let mut lexer = Lexer::new("{");
    assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
    let mut lexer = Lexer::new("}");
    assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
    let mut lexer = Lexer::new("[");
    assert_eq!(lexer.next_token().kind, TokenKind::LBracket);
    let mut lexer = Lexer::new("]");
    assert_eq!(lexer.next_token().kind, TokenKind::RBracket);
    let mut lexer = Lexer::new(":");
    assert_eq!(lexer.next_token().kind, TokenKind::Colon);
    let mut lexer = Lexer::new("?");
    assert_eq!(lexer.next_token().kind, TokenKind::Question);
    let mut lexer = Lexer::new("=>");
    assert_eq!(lexer.next_token().kind, TokenKind::FatArrow);
    let mut lexer = Lexer::new("...");
    assert_eq!(lexer.next_token().kind, TokenKind::DotDotDot);
}

#[test]
fn test_line_comment() {
    let mut lexer = Lexer::new("// this is a comment\n42");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
    assert_eq!(token.leading_comments.len(), 1);
}

#[test]
fn test_block_comment() {
    let mut lexer = Lexer::new("/* block */42");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
    assert_eq!(token.leading_comments.len(), 1);
}

#[test]
fn test_multi_line_block_comment() {
    let mut lexer = Lexer::new("/* line1\nline2 */42");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
}

#[test]
fn test_regex_literal() {
    let mut lexer = Lexer::new("/test/gi");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Regex);
}

#[test]
fn test_template_literal() {
    let mut lexer = Lexer::new("`goodbye world`");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Template);
}

#[test]
fn test_template_with_expr() {
    let mut lexer = Lexer::new("`goodbye ${name}`");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::TemplateHead);
}

#[test]
fn test_bigint() {
    let mut lexer = Lexer::new("42n");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::BigInt);
}

#[test]
fn test_hex_number() {
    let mut lexer = Lexer::new("0xFF");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
}

#[test]
fn test_octal_number() {
    let mut lexer = Lexer::new("0o77");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
}

#[test]
fn test_binary_number() {
    let mut lexer = Lexer::new("0b1010");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Number);
}

#[test]
fn test_multiple_tokens() {
    let mut lexer = Lexer::new("let x = 42;");
    assert_eq!(lexer.next_token().kind, TokenKind::Let);
    assert_eq!(lexer.next_token().kind, TokenKind::Ident);
    assert_eq!(lexer.next_token().kind, TokenKind::Eq);
    assert_eq!(lexer.next_token().kind, TokenKind::Number);
    assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
    assert_eq!(lexer.next_token().kind, TokenKind::Eof);
}

#[test]
fn test_unicode_identifier() {
    let mut lexer = Lexer::new("café");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident);
}

#[test]
fn test_dollar_underscore_identifier() {
    let mut lexer = Lexer::new("$foo_bar");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Ident);
}

#[test]
fn test_empty_string() {
    let mut lexer = Lexer::new("\"\"");
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::String);
    assert_eq!(token.value, "");
}

#[test]
fn test_eof_after_token() {
    let mut lexer = Lexer::new("42");
    let _ = lexer.next_token();
    let token = lexer.next_token();
    assert_eq!(token.kind, TokenKind::Eof);
}
