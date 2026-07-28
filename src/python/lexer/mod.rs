pub mod comments;
pub mod keywords;
pub mod numbers;
pub mod scanner;
pub mod strings;
pub mod token;

use crate::diagnostics::{DiagnosticCode, ParseError};
use crate::span::{Pos, Span};
use scanner::Scanner;
pub use token::{Comment, CommentKind, Token, TokenKind};

use numbers::scan_number;
use std::collections::HashMap;
use strings::scan_string;

pub struct Lexer<'a> {
    scanner: Scanner<'a>,
    #[allow(dead_code)]
    source: &'a str,
    tokens: Vec<Token>,
    errors: Vec<ParseError>,
    current_comments: Vec<Comment>,
    keywords: HashMap<&'a str, TokenKind>,
    indent_stack: Vec<usize>,
    pending: Vec<Token>,
    at_line_start: bool,
    paren_depth: u32,
    bracket_depth: u32,
    brace_depth: u32,
    fstring_expr_depth: u32,
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
            indent_stack: vec![0],
            pending: Vec::new(),
            at_line_start: true,
            paren_depth: 0,
            bracket_depth: 0,
            brace_depth: 0,
            fstring_expr_depth: 0,
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
        if !self.pending.is_empty() {
            return self.pending.remove(0);
        }

        loop {
            if self.at_line_start {
                if let Some(tok) = self.handle_indent() {
                    return tok;
                }
            }

            let token = self.read_token();
            if token.kind == TokenKind::Newline {
                if self.paren_depth > 0
                    || self.bracket_depth > 0
                    || self.brace_depth > 0
                    || self.fstring_expr_depth > 0
                {
                    // Implicit line continuation
                    continue;
                }
                self.at_line_start = true;
                return token;
            }
            if token.kind == TokenKind::Eof {
                // Emit DEDENTs to close all indentation levels
                for _ in 1..self.indent_stack.len() {
                    let tok = self.make_token(TokenKind::Dedent, self.scanner.position());
                    self.pending.push(tok);
                }
                self.indent_stack.truncate(1);
                return token;
            }
            return token;
        }
    }

    fn handle_indent(&mut self) -> Option<Token> {
        self.at_line_start = false;
        let start = self.scanner.position();
        let mut indent = 0usize;

        loop {
            match self.scanner.peek() {
                Some(' ') => {
                    indent += 1;
                    self.scanner.advance();
                }
                Some('\t') => {
                    indent += 8;
                    self.scanner.advance();
                }
                Some('\n') | Some('\r') => {
                    self.scanner.advance();
                    indent = 0;
                }
                Some('#') => {
                    self.skip_comment_line();
                }
                Some('\x0C') => {
                    indent = 0;
                    self.scanner.advance();
                }
                _ => break,
            }
        }

        if self.scanner.is_eof() {
            return None;
        }

        let current = *self.indent_stack.last().unwrap();
        match indent.cmp(&current) {
            std::cmp::Ordering::Greater => {
                self.indent_stack.push(indent);
                let tok = self.make_token(TokenKind::Indent, start);
                Some(tok)
            }
            std::cmp::Ordering::Less => {
                // Pop until we find the matching level
                let mut dedents = 0;
                while self.indent_stack.len() > 1 && indent < *self.indent_stack.last().unwrap() {
                    self.indent_stack.pop();
                    dedents += 1;
                }
                if dedents > 0 {
                    let tok = self.make_token(TokenKind::Dedent, start);
                    // Queue any extra dedents
                    for _ in 1..dedents {
                        let tok = self.make_token(TokenKind::Dedent, start);
                        self.pending.push(tok);
                    }
                    Some(tok)
                } else {
                    None
                }
            }
            std::cmp::Ordering::Equal => None,
        }
    }

    fn skip_comment_line(&mut self) {
        let start = self.scanner.position();
        while let Some(ch) = self.scanner.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.scanner.advance();
        }
        let text = self.scanner.slice(start).to_string();
        self.current_comments.push(Comment::new(
            CommentKind::Line,
            Span::new(start, self.scanner.position()),
            text,
        ));
    }

    fn read_token(&mut self) -> Token {
        self.skip_whitespace_no_nl();

        let start = self.scanner.position();
        let Some(ch) = self.scanner.peek() else {
            return self.make_token(TokenKind::Eof, start);
        };

        let token_kind = match ch {
            '\n' | '\r' => {
                self.scanner.advance();
                // Skip blank lines
                return self.make_token(TokenKind::Newline, start);
            }
            '0'..='9' => return self.read_number(start),
            // String prefixes
            'f' | 'F' | 'b' | 'B' | 'r' | 'R' => {
                let ch2 = self.scanner.peek_ahead(1).unwrap_or('\0');
                if ch2 == '"'
                    || ch2 == '\''
                    || ch2 == 'f'
                    || ch2 == 'F'
                    || ch2 == 'r'
                    || ch2 == 'R'
                    || ch2 == 'b'
                    || ch2 == 'B'
                {
                    return self.read_string(start);
                }
                let word = self.scan_ident();
                let kind = self
                    .keywords
                    .get(word.as_str())
                    .copied()
                    .unwrap_or(TokenKind::Ident);
                return self.make_token_with_value(kind, start, word);
            }
            'a'..='e' | 'g'..='q' | 's'..='z' | 'A'..='E' | 'G'..='Q' | 'S'..='Z' | '_' => {
                let word = self.scan_ident();
                let kind = self
                    .keywords
                    .get(word.as_str())
                    .copied()
                    .unwrap_or(TokenKind::Ident);
                return self.make_token_with_value(kind, start, word);
            }
            '"' | '\'' => return self.read_string(start),
            '#' => {
                let comment = comments::skip_comment(&mut self.scanner, start);
                self.current_comments.push(comment);
                return self.read_token();
            }
            '+' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::MinusEq
                } else if self.scanner.advance_if_eq('>') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('*') {
                    if self.scanner.advance_if_eq('=') {
                        TokenKind::StarStarEq
                    } else {
                        TokenKind::StarStar
                    }
                } else if self.scanner.advance_if_eq('=') {
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('/') {
                    if self.scanner.advance_if_eq('=') {
                        TokenKind::SlashSlashEq
                    } else {
                        TokenKind::SlashSlash
                    }
                } else if self.scanner.advance_if_eq('=') {
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            '%' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::PercentEq
                } else {
                    TokenKind::Percent
                }
            }
            '@' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::AtEq
                } else {
                    TokenKind::At
                }
            }
            '&' => {
                self.scanner.advance();
                TokenKind::Ampersand
            }
            '|' => {
                self.scanner.advance();
                TokenKind::Pipe
            }
            '^' => {
                self.scanner.advance();
                TokenKind::Caret
            }
            '~' => {
                self.scanner.advance();
                TokenKind::Tilde
            }
            '<' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('<') {
                    TokenKind::LtLt
                } else if self.scanner.advance_if_eq('=') {
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            '>' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('>') {
                    TokenKind::GtGt
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
                } else {
                    TokenKind::Eq
                }
            }
            '!' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::Ne
                } else {
                    TokenKind::Error
                }
            }
            ':' => {
                self.scanner.advance();
                if self.scanner.advance_if_eq('=') {
                    TokenKind::Walrus
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
                self.paren_depth += 1;
                TokenKind::LParen
            }
            ')' => {
                self.scanner.advance();
                self.paren_depth = self.paren_depth.saturating_sub(1);
                TokenKind::RParen
            }
            '[' => {
                self.scanner.advance();
                self.bracket_depth += 1;
                TokenKind::LBracket
            }
            ']' => {
                self.scanner.advance();
                self.bracket_depth = self.bracket_depth.saturating_sub(1);
                TokenKind::RBracket
            }
            '{' => {
                self.scanner.advance();
                self.brace_depth += 1;
                TokenKind::LBrace
            }
            '}' => {
                self.scanner.advance();
                self.brace_depth = self.brace_depth.saturating_sub(1);
                TokenKind::RBrace
            }
            '\\' => {
                // Line continuation
                self.scanner.advance();
                // Skip until newline
                while let Some(ch) = self.scanner.peek() {
                    if ch == '\n' || ch == '\r' {
                        self.scanner.advance();
                        break;
                    }
                    self.scanner.advance();
                }
                return self.read_token();
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

        self.make_token(token_kind, start)
    }

    fn skip_whitespace_no_nl(&mut self) {
        while let Some(ch) = self.scanner.peek() {
            if ch == ' ' || ch == '\t' || ch == '\x0C' {
                self.scanner.advance();
            } else {
                break;
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
        let kind = scan_number(&mut self.scanner);
        let value = self.scanner.slice(start).to_string();
        self.make_token_with_value(kind, start, value)
    }

    fn read_string(&mut self, start: Pos) -> Token {
        let first_char = self.scanner.slice(start).chars().next().unwrap_or('"');
        let (kind, _is_fstring) = scan_string(&mut self.scanner, first_char);
        let value = self.scanner.slice(start).to_string();
        self.make_token_with_value(kind, start, value)
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
