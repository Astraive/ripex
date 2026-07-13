use crate::js::lexer::comments::{
    scan_hashbang, scan_multi_line_comment, scan_single_line_comment,
};
use crate::js::lexer::keywords::keyword_to_token;
use crate::js::lexer::numbers::scan_number;
use crate::js::lexer::regex::scan_regex;
use crate::js::lexer::scanner::Scanner;
use crate::js::lexer::strings::{scan_string, scan_template};
pub use crate::js::lexer::token::{Comment, Token, TokenKind};
use crate::span::{Pos, Span};

pub mod comments;
pub mod keywords;
pub mod numbers;
pub mod regex;
pub mod scanner;
pub mod strings;
pub mod token;

pub struct Lexer<'a> {
    scanner: Scanner<'a>,
    comments: Vec<Comment>,
    in_template_expr: bool,
    template_brace_depth: usize,
    is_start_of_expr: bool,
    jsx_enabled: bool,
    previous_kind: Option<TokenKind>,
    hashbang_scanned: bool,
    eof_emitted: bool,
    tokens_count: usize,
}

fn is_ident_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_alphabetic()
}

fn is_ident_continue(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit()
}

fn sets_start_of_expr(kind: TokenKind) -> bool {
    !matches!(
        kind,
        TokenKind::Ident
            | TokenKind::PrivateName
            | TokenKind::Number
            | TokenKind::String
            | TokenKind::Template
            | TokenKind::Regex
            | TokenKind::BigInt
            | TokenKind::TemplateTail
            | TokenKind::Null
            | TokenKind::True
            | TokenKind::False
            | TokenKind::This
            | TokenKind::Super
            | TokenKind::RParen
            | TokenKind::RBracket
            | TokenKind::RBrace
            | TokenKind::PlusPlus
            | TokenKind::MinusMinus
    )
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self::with_jsx(source, false)
    }

    pub fn with_jsx(source: &'a str, jsx_enabled: bool) -> Self {
        Lexer {
            scanner: Scanner::new(source),
            comments: Vec::new(),
            in_template_expr: false,
            template_brace_depth: 0,
            is_start_of_expr: true,
            jsx_enabled,
            previous_kind: None,
            hashbang_scanned: false,
            eof_emitted: false,
            tokens_count: 0,
        }
    }

    /// Get the next token from the lexer.
    pub fn next_token(&mut self) -> Token {
        self.next().unwrap_or(Token {
            kind: TokenKind::Eof,
            value: String::new(),
            span: Span::new(self.scanner.position(), self.scanner.position()),
            leading_comments: Vec::new(),
            has_line_break: false,
        })
    }

    /// Get all comments collected during lexing.
    pub fn comments(&self) -> &[Comment] {
        &self.comments
    }

    fn skip_trivia(&mut self) {
        loop {
            self.scanner.reset_line_break();
            match self.scanner.peek() {
                Some(ch) if ch.is_whitespace() || ch == '\n' || ch == '\r' => {
                    self.scanner.advance();
                }
                Some('/') if self.scanner.peek_ahead(1) == Some('/') => {
                    self.comments
                        .push(scan_single_line_comment(&mut self.scanner));
                }
                Some('/') if self.scanner.peek_ahead(1) == Some('*') => {
                    self.comments
                        .push(scan_multi_line_comment(&mut self.scanner));
                }
                _ => break,
            }
        }
    }

    fn make_token(&mut self, kind: TokenKind) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        let end = self.scanner.position();
        let span = Span::new(start, end);
        let mut token = Token::new(kind, span);
        token.leading_comments = self.comments.drain(..).collect();
        token.has_line_break = self.scanner.has_line_break;
        self.is_start_of_expr = sets_start_of_expr(kind);
        self.previous_kind = Some(kind);
        token
    }

    #[allow(dead_code)]
    fn make_token_value(&mut self, kind: TokenKind, value: impl Into<String>) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        let end = self.scanner.position();
        let span = Span::new(start, end);
        let mut token = Token::with_value(kind, span, value);
        token.leading_comments = self.comments.drain(..).collect();
        token.has_line_break = self.scanner.has_line_break;
        self.is_start_of_expr = sets_start_of_expr(kind);
        self.previous_kind = Some(kind);
        token
    }

    fn finish_token(&mut self, kind: TokenKind, start: Pos, value: impl Into<String>) -> Token {
        let end = self.scanner.position();
        let span = Span::new(start, end);
        let mut token = Token::with_value(kind, span, value);
        token.leading_comments = self.comments.drain(..).collect();
        token.has_line_break = self.scanner.has_line_break;
        self.is_start_of_expr = sets_start_of_expr(kind);
        self.previous_kind = Some(kind);
        token
    }

    #[allow(dead_code)]
    fn advance(&mut self) -> Option<char> {
        self.scanner.advance()
    }

    #[allow(dead_code)]
    fn peek(&self) -> Option<char> {
        self.scanner.peek()
    }

    #[allow(dead_code)]
    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.scanner.peek_ahead(n)
    }

    fn eof_token(&mut self) -> Token {
        let start = self.scanner.position();
        let span = Span::new(start, start);
        let mut token = Token::new(TokenKind::Eof, span);
        token.leading_comments = self.comments.drain(..).collect();
        token
    }

    fn error_token(&mut self, msg: impl Into<String>) -> Token {
        let start = self.scanner.position();
        let mut token = Token::with_value(TokenKind::Error, Span::new(start, start), msg);
        token.leading_comments = self.comments.drain(..).collect();
        token
    }

    fn tokenize(&mut self) -> Token {
        self.skip_trivia();

        if self.scanner.is_eof() {
            return self.eof_token();
        }

        // Hashbang at the very start of source
        if !self.hashbang_scanned
            && self.scanner.peek() == Some('#')
            && self.scanner.peek_ahead(1) == Some('!')
        {
            self.hashbang_scanned = true;
            let start = self.scanner.position();
            scan_hashbang(&mut self.scanner);
            return self.finish_token(TokenKind::Hashbang, start, "");
        }
        self.hashbang_scanned = true;

        // Template expression: handle { and } with depth tracking
        if self.in_template_expr {
            match self.scanner.peek() {
                Some('{') => {
                    self.template_brace_depth += 1;
                    return self.make_token(TokenKind::LBrace);
                }
                Some('}') if self.template_brace_depth == 0 => {
                    return self.scan_template_continuation();
                }
                Some('}') => {
                    self.template_brace_depth -= 1;
                    return self.make_token(TokenKind::RBrace);
                }
                _ => {}
            }
        }

        let ch = self.scanner.peek().unwrap();
        match ch {
            // Single-char punctuators
            '(' => self.make_token(TokenKind::LParen),
            ')' => self.make_token(TokenKind::RParen),
            '{' => self.make_token(TokenKind::LBrace),
            '}' => self.make_token(TokenKind::RBrace),
            '[' => self.make_token(TokenKind::LBracket),
            ']' => self.make_token(TokenKind::RBracket),
            ',' => self.make_token(TokenKind::Comma),
            ';' => self.make_token(TokenKind::Semicolon),
            ':' => self.make_token(TokenKind::Colon),
            '~' => self.make_token(TokenKind::Tilde),

            // Dot and spread
            '.' => self.scan_dot_spread(),

            // Template literal
            '`' => self.scan_template_literal(),

            // Strings
            '\'' | '"' => self.scan_string_token(ch),

            // Identifier, keyword, private name
            c if is_ident_start(c) => self.scan_ident_or_keyword(),
            '#' => self.scan_hash(),
            '@' => self.make_token(TokenKind::At),

            // Numbers
            c if c.is_ascii_digit() => self.scan_number_token(),

            // Operators
            '+' => self.scan_plus(),
            '-' => self.scan_minus(),
            '*' => self.scan_star(),
            '/' => {
                if self.jsx_enabled && self.previous_kind == Some(TokenKind::Lt) {
                    self.scan_slash()
                } else if self.is_start_of_expr {
                    self.scan_regex_token()
                } else {
                    self.scan_slash()
                }
            }
            '%' => self.scan_percent(),
            '&' => self.scan_ampersand(),
            '|' => self.scan_pipe(),
            '^' => self.scan_caret(),
            '<' => self.scan_lt(),
            '>' => self.scan_gt(),
            '=' => self.scan_eq(),
            '!' => self.scan_exclamation(),
            '?' => self.scan_question(),

            _ => {
                let _ = self.scanner.advance();
                self.error_token(format!("unexpected character `{ch}`"))
            }
        }
    }

    fn scan_dot_spread(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        if self.scanner.peek() == Some('.') && self.scanner.peek_ahead(1) == Some('.') {
            self.scanner.advance();
            self.scanner.advance();
            self.finish_token(TokenKind::DotDotDot, start, "...")
        } else {
            self.finish_token(TokenKind::Dot, start, ".")
        }
    }

    fn scan_ident_or_keyword(&mut self) -> Token {
        let start = self.scanner.position();
        let mut word = String::new();
        while let Some(c) = self.scanner.peek() {
            if is_ident_continue(c) {
                self.scanner.advance();
                word.push(c);
            } else {
                break;
            }
        }
        let kind = keyword_to_token(&word).unwrap_or(TokenKind::Ident);
        self.finish_token(kind, start, word)
    }

    fn scan_hash(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance(); // consume '#'
        match self.scanner.peek() {
            Some(c) if is_ident_start(c) => {
                let mut word = String::from("#");
                while let Some(c) = self.scanner.peek() {
                    if is_ident_continue(c) {
                        self.scanner.advance();
                        word.push(c);
                    } else {
                        break;
                    }
                }
                self.finish_token(TokenKind::PrivateName, start, word)
            }
            Some('{') => {
                self.scanner.advance(); // consume '{'
                self.finish_token(TokenKind::HashLBrace, start, "#{")
            }
            Some('[') => {
                self.scanner.advance(); // consume '['
                self.finish_token(TokenKind::HashLBracket, start, "#[")
            }
            _ => self.finish_token(TokenKind::Error, start, "#"),
        }
    }

    fn scan_number_token(&mut self) -> Token {
        let start = self.scanner.position();
        let kind = scan_number(&mut self.scanner);
        let value = self.scanner.slice(start).to_string();
        self.finish_token(kind, start, value)
    }

    fn scan_string_token(&mut self, quote: char) -> Token {
        let start = self.scanner.position();
        self.scanner.advance(); // consume opening quote
        let value = scan_string(&mut self.scanner, quote);
        self.finish_token(TokenKind::String, start, value)
    }

    fn scan_template_literal(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance(); // consume backtick
        let (text, kind) = scan_template(&mut self.scanner);
        let token_kind = match kind {
            TokenKind::TemplateHead => TokenKind::TemplateHead,
            TokenKind::TemplateTail => TokenKind::Template,
            _ => unreachable!(),
        };
        match token_kind {
            TokenKind::TemplateHead => {
                self.in_template_expr = true;
                self.template_brace_depth = 0;
            }
            _ => {
                self.in_template_expr = false;
            }
        }
        self.finish_token(token_kind, start, text)
    }

    fn scan_template_continuation(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance(); // consume the '}'
        let (text, kind) = scan_template(&mut self.scanner);
        let token_kind = match kind {
            TokenKind::TemplateHead => TokenKind::TemplateMiddle,
            TokenKind::TemplateTail => TokenKind::TemplateTail,
            _ => unreachable!(),
        };
        match token_kind {
            TokenKind::TemplateMiddle => {
                self.in_template_expr = true;
                self.template_brace_depth = 0;
            }
            _ => {
                self.in_template_expr = false;
            }
        }
        self.finish_token(token_kind, start, text)
    }

    fn scan_regex_token(&mut self) -> Token {
        let start = self.scanner.position();
        let (pattern, flags) = scan_regex(&mut self.scanner);
        let value = if flags.is_empty() {
            pattern
        } else {
            format!("{}/{}", pattern, flags)
        };
        self.finish_token(TokenKind::Regex, start, value)
    }

    // --- Operator scanners ---

    fn scan_plus(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('+') => {
                self.scanner.advance();
                self.finish_token(TokenKind::PlusPlus, start, "++")
            }
            Some('=') => {
                self.scanner.advance();
                self.finish_token(TokenKind::PlusEq, start, "+=")
            }
            _ => self.finish_token(TokenKind::Plus, start, "+"),
        }
    }

    fn scan_minus(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('-') => {
                self.scanner.advance();
                self.finish_token(TokenKind::MinusMinus, start, "--")
            }
            Some('=') => {
                self.scanner.advance();
                self.finish_token(TokenKind::MinusEq, start, "-=")
            }
            Some('>') => {
                self.scanner.advance();
                self.finish_token(TokenKind::Arrow, start, "->")
            }
            _ => self.finish_token(TokenKind::Minus, start, "-"),
        }
    }

    fn scan_star(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('*') => {
                self.scanner.advance();
                if self.scanner.peek() == Some('=') {
                    self.scanner.advance();
                    self.finish_token(TokenKind::StarStarEq, start, "**=")
                } else {
                    self.finish_token(TokenKind::StarStar, start, "**")
                }
            }
            Some('=') => {
                self.scanner.advance();
                self.finish_token(TokenKind::StarEq, start, "*=")
            }
            _ => self.finish_token(TokenKind::Star, start, "*"),
        }
    }

    fn scan_slash(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        if self.scanner.peek() == Some('=') {
            self.scanner.advance();
            self.finish_token(TokenKind::SlashEq, start, "/=")
        } else {
            self.finish_token(TokenKind::Slash, start, "/")
        }
    }

    fn scan_percent(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        if self.scanner.peek() == Some('=') {
            self.scanner.advance();
            self.finish_token(TokenKind::PercentEq, start, "%=")
        } else {
            self.finish_token(TokenKind::Percent, start, "%")
        }
    }

    fn scan_ampersand(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('&') => {
                self.scanner.advance();
                if self.scanner.peek() == Some('=') {
                    self.scanner.advance();
                    self.finish_token(TokenKind::AmpersandAmpersandEq, start, "&&=")
                } else {
                    self.finish_token(TokenKind::AmpersandAmpersand, start, "&&")
                }
            }
            Some('=') => {
                self.scanner.advance();
                self.finish_token(TokenKind::AmpersandEq, start, "&=")
            }
            _ => self.finish_token(TokenKind::Ampersand, start, "&"),
        }
    }

    fn scan_pipe(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('|') => {
                self.scanner.advance();
                if self.scanner.peek() == Some('=') {
                    self.scanner.advance();
                    self.finish_token(TokenKind::PipePipeEq, start, "||=")
                } else {
                    self.finish_token(TokenKind::PipePipe, start, "||")
                }
            }
            Some('=') => {
                self.scanner.advance();
                self.finish_token(TokenKind::PipeEq, start, "|=")
            }
            Some('>') => {
                self.scanner.advance();
                self.finish_token(TokenKind::PipeGt, start, "|>")
            }
            _ => self.finish_token(TokenKind::Pipe, start, "|"),
        }
    }

    fn scan_caret(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        if self.scanner.peek() == Some('=') {
            self.scanner.advance();
            self.finish_token(TokenKind::CaretEq, start, "^=")
        } else {
            self.finish_token(TokenKind::Caret, start, "^")
        }
    }

    fn scan_lt(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('<') => {
                self.scanner.advance();
                if self.scanner.peek() == Some('=') {
                    self.scanner.advance();
                    self.finish_token(TokenKind::LtLtEq, start, "<<=")
                } else {
                    self.finish_token(TokenKind::LtLt, start, "<<")
                }
            }
            Some('=') => {
                self.scanner.advance();
                self.finish_token(TokenKind::LtEq, start, "<=")
            }
            _ => self.finish_token(TokenKind::Lt, start, "<"),
        }
    }

    fn scan_gt(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('>') => {
                self.scanner.advance();
                match self.scanner.peek() {
                    Some('>') => {
                        self.scanner.advance();
                        if self.scanner.peek() == Some('=') {
                            self.scanner.advance();
                            self.finish_token(TokenKind::GtGtGtEq, start, ">>>=")
                        } else {
                            self.finish_token(TokenKind::GtGtGt, start, ">>>")
                        }
                    }
                    Some('=') => {
                        self.scanner.advance();
                        self.finish_token(TokenKind::GtGtEq, start, ">>=")
                    }
                    _ => self.finish_token(TokenKind::GtGt, start, ">>"),
                }
            }
            Some('=') => {
                self.scanner.advance();
                self.finish_token(TokenKind::GtEq, start, ">=")
            }
            _ => self.finish_token(TokenKind::Gt, start, ">"),
        }
    }

    fn scan_eq(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('=') => {
                self.scanner.advance();
                if self.scanner.peek() == Some('=') {
                    self.scanner.advance();
                    self.finish_token(TokenKind::EqEqEq, start, "===")
                } else {
                    self.finish_token(TokenKind::EqEq, start, "==")
                }
            }
            Some('>') => {
                self.scanner.advance();
                self.finish_token(TokenKind::FatArrow, start, "=>")
            }
            _ => self.finish_token(TokenKind::Eq, start, "="),
        }
    }

    fn scan_exclamation(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('=') => {
                self.scanner.advance();
                if self.scanner.peek() == Some('=') {
                    self.scanner.advance();
                    self.finish_token(TokenKind::Neq, start, "!==")
                } else {
                    self.finish_token(TokenKind::Ne, start, "!=")
                }
            }
            _ => self.finish_token(TokenKind::Exclamation, start, "!"),
        }
    }

    fn scan_question(&mut self) -> Token {
        let start = self.scanner.position();
        self.scanner.advance();
        match self.scanner.peek() {
            Some('?') => {
                self.scanner.advance();
                if self.scanner.peek() == Some('=') {
                    self.scanner.advance();
                    self.finish_token(TokenKind::QuestionQuestionEq, start, "??=")
                } else {
                    self.finish_token(TokenKind::QuestionQuestion, start, "??")
                }
            }
            Some('.') => {
                // Look ahead to make sure it's not a spread (...)
                // ?. is valid only if the dot is not part of ...
                self.scanner.advance();
                self.finish_token(TokenKind::QuestionDot, start, "?.")
            }
            _ => self.finish_token(TokenKind::Question, start, "?"),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tokens_count >= crate::limits::MAX_TOKENS {
            if self.eof_emitted {
                return None;
            }
            self.eof_emitted = true;
            let pos = self.scanner.position();
            return Some(Token::new(TokenKind::Eof, Span::new(pos, pos)));
        }
        if self.scanner.is_eof() && self.comments.is_empty() {
            if self.eof_emitted {
                return None;
            }
            self.eof_emitted = true;
            let pos = self.scanner.position();
            return Some(Token::new(TokenKind::Eof, Span::new(pos, pos)));
        }
        let token = self.tokenize();
        self.tokens_count += 1;
        Some(token)
    }
}
