pub mod keywords;
pub mod scanner;
pub mod token;

use crate::diagnostics::{DiagnosticCode, ParseError};
use crate::span::{Pos, Span};
use scanner::Scanner;
pub use token::{Comment, CommentKind, Token, TokenKind};

use std::collections::HashMap;

pub struct Lexer<'a> {
    scanner: Scanner<'a>,
    #[allow(dead_code)]
    source: &'a str,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    current_comments: Vec<Comment>,
    keywords: HashMap<&'a str, TokenKind>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let keywords = keywords::keyword_map();
        Lexer {
            scanner: Scanner::new(source),
            source,
            tokens: Vec::new(),
            errors: Vec::new(),
            current_comments: Vec::new(),
            keywords,
        }
    }

    pub fn tokenize(mut self) -> (Vec<Token>, Vec<ParseError>) {
        loop {
            if self.tokens.len() >= crate::limits::MAX_TOKENS {
                self.errors.push(ParseError::with_message(
                    DiagnosticCode::TokenLimitExceeded,
                    Span::ZERO,
                    "too many tokens",
                ));
                self.tokens.push(Token {
                    kind: TokenKind::Eof,
                    span: Span::ZERO,
                    value: String::new(),
                    leading_comments: Vec::new(),
                    has_line_break: false,
                });
                break;
            }
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            self.tokens.push(token);
            if is_eof {
                break;
            }
        }
        (self.tokens, self.errors)
    }

    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start = self.scanner.position();
        let Some(ch) = self.scanner.peek() else {
            return self.make_token(TokenKind::Eof, start);
        };

        let token = match ch {
            '0'..='9' => return self.read_number(start),
            '"' => return self.read_interpreted_string(start),
            '`' => return self.read_raw_string(start),
            '\'' => return self.read_rune(start),
            'a'..='z' | 'A'..='Z' | '_' => {
                let word = self.scan_ident();
                let kind = self
                    .keywords
                    .get(word.as_str())
                    .copied()
                    .unwrap_or(TokenKind::Ident);
                return self.make_token_with_value(kind, start, word);
            }
            '+' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::PlusEq
                } else if self.scanner.advance_if_eq('+') {
                    TokenKind::PlusPlus
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::MinusEq
                } else if self.scanner.advance_if_eq('-') {
                    TokenKind::MinusMinus
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                self.scanner.advance();
                TokenKind::Slash
            }
            '%' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::PercentEq
                } else {
                    TokenKind::Percent
                }
            }
            '&' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('&') {
                    TokenKind::AmpersandAmpersand
                } else if self.scanner.advance_if_eq('=') {
                    TokenKind::AmpersandEq
                } else if self.scanner.advance_if_eq('^') {
                    TokenKind::AmpersandCaret
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('|') {
                    TokenKind::PipePipe
                } else if self.scanner.advance_if_eq('=') {
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::CaretEq
                } else {
                    TokenKind::Caret
                }
            }
            '<' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('<') {
                    if self.scanner.advance_if_eq('=') {
                        TokenKind::LtLtEq
                    } else {
                        TokenKind::LtLt
                    }
                } else if self.scanner.advance_if_eq('=') {
                    TokenKind::LtEq
                } else if self.scanner.advance_if_eq('-') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('>') {
                    if self.scanner.advance_if_eq('=') {
                        TokenKind::GtGtEq
                    } else {
                        TokenKind::GtGt
                    }
                } else if self.scanner.advance_if_eq('=') {
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            '=' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::EqEq
                } else if self.scanner.advance_if_eq(':') {
                    TokenKind::Define
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::Ne
                } else {
                    TokenKind::Exclamation
                }
            }
            ':' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::Define
                } else {
                    TokenKind::Colon
                }
            }
            ';' => {
                self.scanner.advance();
                TokenKind::Semicolon
            }
            ',' => {
                self.scanner.advance();
                TokenKind::Comma
            }
            '.' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('.') {
                    if self.scanner.advance_if_eq('.') {
                        TokenKind::DotDotDot
                    } else {
                        TokenKind::Dot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            '(' => {
                self.scanner.advance();
                TokenKind::LParen
            }
            ')' => {
                self.scanner.advance();
                TokenKind::RParen
            }
            '{' => {
                self.scanner.advance();
                TokenKind::LBrace
            }
            '}' => {
                self.scanner.advance();
                TokenKind::RBrace
            }
            '[' => {
                self.scanner.advance();
                TokenKind::LBracket
            }
            ']' => {
                self.scanner.advance();
                TokenKind::RBracket
            }
            '~' => {
                self.scanner.advance();
                TokenKind::Tilde
            }
            _ => {
                let err = ParseError::new(
                    DiagnosticCode::UnexpectedToken,
                    Span::new(start, self.scanner.position()),
                );
                self.errors.push(err);
                self.scanner.advance();
                TokenKind::Error
            }
        };

        self.make_token(token, start)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            self.scanner.has_line_break = false;
            match self.scanner.peek() {
                Some(' ') | Some('\t') | Some('\r') | Some('\n') => {
                    self.scanner.advance();
                }
                Some('/') if self.scanner.peek_ahead(1) == Some('/') => {
                    let start = self.scanner.position();
                    self.scanner.advance();
                    self.scanner.advance();
                    while let Some(ch) = self.scanner.peek() {
                        if ch == '\n' || ch == '\r' {
                            break;
                        }
                        self.scanner.advance();
                    }
                    let text = self.scanner.slice(start);
                    self.current_comments.push(Comment::new(
                        CommentKind::Line,
                        Span::new(start, self.scanner.position()),
                        text,
                    ));
                }
                Some('/') if self.scanner.peek_ahead(1) == Some('*') => {
                    let start = self.scanner.position();
                    self.scanner.advance();
                    self.scanner.advance();
                    while let Some(ch) = self.scanner.peek() {
                        if ch == '*' && self.scanner.peek_ahead(1) == Some('/') {
                            self.scanner.advance();
                            self.scanner.advance();
                            break;
                        }
                        self.scanner.advance();
                    }
                    let text = self.scanner.slice(start);
                    self.current_comments.push(Comment::new(
                        CommentKind::Block,
                        Span::new(start, self.scanner.position()),
                        text,
                    ));
                }
                _ => break,
            }
        }
    }

    fn scan_ident(&mut self) -> String {
        let start = self.scanner.position();
        while let Some(ch) = self.scanner.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.scanner.advance();
            } else {
                break;
            }
        }
        self.scanner.slice(start).to_string()
    }

    fn read_number(&mut self, start: Pos) -> Token {
        let mut is_float = false;
        let value = if self.scanner.peek() == Some('0') {
            self.scanner.advance();
            match self.scanner.peek() {
                Some('x') | Some('X') => {
                    self.scanner.advance();
                    self.scan_hex_digits();
                    None
                }
                Some('o') | Some('O') => {
                    self.scanner.advance();
                    self.scan_oct_digits();
                    None
                }
                Some('b') | Some('B') => {
                    self.scanner.advance();
                    self.scan_bin_digits();
                    None
                }
                _ => self.scan_dec_or_float(&mut is_float),
            }
        } else {
            self.scan_dec_or_float(&mut is_float)
        };

        let kind = if is_float {
            TokenKind::FloatLit
        } else {
            TokenKind::IntLit
        };
        let val = value.unwrap_or_else(|| self.scanner.slice(start).to_string());
        self.make_token_with_value(kind, start, val)
    }

    fn scan_dec_or_float(&mut self, is_float: &mut bool) -> Option<String> {
        while let Some(ch) = self.scanner.peek() {
            if ch.is_ascii_digit() {
                self.scanner.advance();
            } else {
                break;
            }
        }
        if self.scanner.peek() == Some('.') {
            let next = self.scanner.peek_ahead(1);
            if next.is_some_and(|c| c.is_ascii_digit()) {
                *is_float = true;
                self.scanner.advance();
                while let Some(ch) = self.scanner.peek() {
                    if ch.is_ascii_digit() {
                        self.scanner.advance();
                    } else {
                        break;
                    }
                }
            }
        }
        None
    }

    fn scan_hex_digits(&mut self) {
        while let Some(ch) = self.scanner.peek() {
            if ch.is_ascii_hexdigit() {
                self.scanner.advance();
            } else {
                break;
            }
        }
    }

    fn scan_oct_digits(&mut self) {
        while let Some(ch) = self.scanner.peek() {
            if matches!(ch, '0'..='7') {
                self.scanner.advance();
            } else {
                break;
            }
        }
    }

    fn scan_bin_digits(&mut self) {
        while let Some(ch) = self.scanner.peek() {
            if matches!(ch, '0'..='1') {
                self.scanner.advance();
            } else {
                break;
            }
        }
    }

    fn read_interpreted_string(&mut self, start: Pos) -> Token {
        self.scanner.advance(); // opening "
        while let Some(ch) = self.scanner.peek() {
            if ch == '"' {
                self.scanner.advance();
                break;
            }
            if ch == '\\' {
                self.scanner.advance();
            }
            if self.scanner.peek().is_some() {
                self.scanner.advance();
            }
        }
        let value = self.scanner.slice(start).to_string();
        self.make_token_with_value(TokenKind::InterpretedString, start, value)
    }

    fn read_raw_string(&mut self, start: Pos) -> Token {
        self.scanner.advance(); // opening `
        while let Some(ch) = self.scanner.peek() {
            if ch == '`' {
                self.scanner.advance();
                break;
            }
            self.scanner.advance();
        }
        let value = self.scanner.slice(start).to_string();
        self.make_token_with_value(TokenKind::RawString, start, value)
    }

    fn read_rune(&mut self, start: Pos) -> Token {
        self.scanner.advance(); // opening '
        while let Some(ch) = self.scanner.peek() {
            if ch == '\'' {
                self.scanner.advance();
                break;
            }
            if ch == '\\' {
                self.scanner.advance();
            }
            if self.scanner.peek().is_some() {
                self.scanner.advance();
            }
        }
        let value = self.scanner.slice(start).to_string();
        self.make_token_with_value(TokenKind::RuneLit, start, value)
    }

    fn make_token(&mut self, kind: TokenKind, start: Pos) -> Token {
        let end = self.scanner.position();
        let mut token = Token::new(kind, Span::new(start, end));
        token.has_line_break = self.scanner.has_line_break;
        token.leading_comments = std::mem::take(&mut self.current_comments);
        token
    }

    fn make_token_with_value(&mut self, kind: TokenKind, start: Pos, value: String) -> Token {
        let end = self.scanner.position();
        let mut token = Token::with_value(kind, Span::new(start, end), value);
        token.has_line_break = self.scanner.has_line_break;
        token.leading_comments = std::mem::take(&mut self.current_comments);
        token
    }
}
