use super::super::lexer::token::{Token, TokenKind};
use super::super::lexer::Lexer;
use crate::diagnostics::{DiagnosticCode, DiagnosticReporter, ParseError};
use crate::limits::MAX_INPUT_SIZE;
use crate::span::{Pos, Span};

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub errors: Vec<ParseError>,
    pub reporter: DiagnosticReporter,
    pub depth: u32,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        if source.len() > MAX_INPUT_SIZE {
            let err = ParseError::new(DiagnosticCode::InputTooLarge, Span::ZERO);
            return Parser {
                tokens: Vec::new(),
                pos: 0,
                errors: vec![err],
                reporter: DiagnosticReporter::new(),
                depth: 0,
            };
        }
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.tokenize();
        Parser {
            tokens,
            pos: 0,
            errors,
            reporter: DiagnosticReporter::new(),
            depth: 0,
        }
    }

    pub fn peek(&self) -> TokenKind {
        self.tokens
            .get(self.pos)
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    pub fn peek_ahead(&self, n: usize) -> TokenKind {
        self.tokens
            .get(self.pos + n)
            .map(|t| t.kind)
            .unwrap_or(TokenKind::Eof)
    }

    pub fn peek_token(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    /// Start position of the first token, or `Pos::ZERO` when the token stream
    /// is empty (e.g. input larger than `MAX_INPUT_SIZE`, which yields no
    /// tokens). Safe to call even when `tokens` is empty.
    pub fn token_start(&self) -> Pos {
        self.tokens
            .first()
            .map(|t| t.span.start)
            .unwrap_or(Pos::ZERO)
    }

    pub fn advance(&mut self) -> Token {
        let token = self.peek_token().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        token
    }

    pub fn expect(&mut self, kind: TokenKind) -> Token {
        if self.peek() == kind {
            self.advance()
        } else {
            let token = self.peek_token().clone();
            let err = ParseError::with_message(
                DiagnosticCode::UnexpectedToken,
                Span::new(self.prev_end(), token.span.start),
                format!("Expected {:?}, found {:?}", kind, self.peek()),
            );
            self.errors.push(err);
            self.advance();
            token
        }
    }

    pub fn expect_ident(&mut self) -> String {
        if self.peek() == TokenKind::Ident {
            let token = self.advance();
            token.value.clone()
        } else {
            let token = self.peek_token();
            let err = ParseError::with_message(
                DiagnosticCode::UnexpectedToken,
                Span::new(self.prev_end(), token.span.start),
                format!("Expected identifier, found {:?}", self.peek()),
            );
            self.errors.push(err);
            self.advance();
            String::new()
        }
    }

    pub fn prev_end(&self) -> Pos {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span.end
        } else {
            Pos::ZERO
        }
    }

    // ---- Recursion guard ----

    pub fn bump_recursion(&mut self) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > crate::limits::MAX_RECURSION {
            let err = ParseError::new(DiagnosticCode::MaxRecursionExceeded, self.peek_token().span);
            if !self
                .errors
                .iter()
                .any(|existing| existing.code == DiagnosticCode::MaxRecursionExceeded)
            {
                self.errors.push(err.clone());
            }
            self.depth -= 1;
            return Err(err);
        }
        Ok(())
    }

    pub fn pop_recursion(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub fn is_eof(&self) -> bool {
        self.peek() == TokenKind::Eof
    }

    pub fn check(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    pub fn parse_stmt_recovery(&mut self) -> super::super::ast::stmt::Stmt {
        let pos = self.pos;
        let err_count = self.errors.len();
        let stmt = self.parse_stmt();
        if self.pos == pos && self.errors.len() == err_count {
            super::recovery::recover_from_error(self);
            super::super::ast::stmt::Stmt::Empty(crate::span::Span::ZERO)
        } else {
            stmt
        }
    }

    pub fn expect_semicolon(&mut self) {
        if self.check(TokenKind::Semicolon) {
            self.advance();
        }
    }
}
