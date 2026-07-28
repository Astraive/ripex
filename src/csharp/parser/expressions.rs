use super::super::ast::expr::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::{Pos, Span};

impl Parser {
    pub fn parse_expr(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Error(Span::ZERO);
        }
        let result = self.parse_assignment();
        self.pop_recursion();
        result
    }

    fn parse_assignment(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let left = self.parse_ternary();
        if self.peek() == TokenKind::FatArrow {
            self.advance();
            return self.parse_assignment();
        }
        if matches!(
            self.peek(),
            TokenKind::Eq
                | TokenKind::PlusEq
                | TokenKind::MinusEq
                | TokenKind::StarEq
                | TokenKind::SlashEq
                | TokenKind::PercentEq
                | TokenKind::AmpersandEq
                | TokenKind::PipeEq
                | TokenKind::CaretEq
                | TokenKind::LtLtEq
                | TokenKind::GtGtEq
                | TokenKind::QuestionQuestionEq
        ) {
            self.advance();
            let right = self.parse_assignment();
            Expr::Assign(
                Box::new(left),
                Box::new(right),
                Span::new(start, self.prev_end()),
            )
        } else {
            left
        }
    }

    fn parse_ternary(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut cond = self.parse_null_coalesce();
        if self.peek() == TokenKind::Question {
            self.advance();
            let then = self.parse_expr();
            self.expect(TokenKind::Colon);
            let else_ = self.parse_ternary_guarded();
            cond = Expr::Conditional(
                Box::new(cond),
                Box::new(then),
                Box::new(else_),
                Span::new(start, self.prev_end()),
            );
        }
        cond
    }
    fn parse_ternary_guarded(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Error(Span::ZERO);
        }
        let expr = self.parse_ternary();
        self.pop_recursion();
        expr
    }

    fn parse_null_coalesce(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut left = self.parse_binary(0);
        while self.peek() == TokenKind::QuestionQuestion {
            self.advance();
            let right = self.parse_null_coalesce();
            left = Expr::NullCoalesce(
                Box::new(left),
                Box::new(right),
                Span::new(start, self.prev_end()),
            );
        }
        left
    }

    fn parse_binary(&mut self, min_prec: u8) -> Expr {
        let mut left = self.parse_unary();
        loop {
            let prec = self.binary_precedence();
            if prec < min_prec {
                break;
            }
            let span_start = left.span().start;
            let op = match self.peek() {
                TokenKind::Plus => {
                    self.advance();
                    BinaryOp::Add
                }
                TokenKind::Minus => {
                    self.advance();
                    BinaryOp::Sub
                }
                TokenKind::Star => {
                    self.advance();
                    BinaryOp::Mul
                }
                TokenKind::Slash => {
                    self.advance();
                    BinaryOp::Div
                }
                TokenKind::Percent => {
                    self.advance();
                    BinaryOp::Mod
                }
                TokenKind::EqEq => {
                    self.advance();
                    BinaryOp::Eq
                }
                TokenKind::Ne => {
                    self.advance();
                    BinaryOp::Ne
                }
                TokenKind::Lt => {
                    self.advance();
                    BinaryOp::Lt
                }
                TokenKind::Gt => {
                    self.advance();
                    BinaryOp::Gt
                }
                TokenKind::LtEq => {
                    self.advance();
                    BinaryOp::Le
                }
                TokenKind::GtEq => {
                    self.advance();
                    BinaryOp::Ge
                }
                TokenKind::AmpersandAmpersand => {
                    self.advance();
                    BinaryOp::And
                }
                TokenKind::PipePipe => {
                    self.advance();
                    BinaryOp::Or
                }
                TokenKind::Ampersand => {
                    self.advance();
                    BinaryOp::BitAnd
                }
                TokenKind::Pipe => {
                    self.advance();
                    BinaryOp::BitOr
                }
                TokenKind::Caret => {
                    self.advance();
                    BinaryOp::BitXor
                }
                TokenKind::LtLt => {
                    self.advance();
                    BinaryOp::Shl
                }
                TokenKind::GtGt => {
                    self.advance();
                    BinaryOp::Shr
                }
                TokenKind::Is => {
                    self.advance();
                    BinaryOp::Is
                }
                TokenKind::As => {
                    self.advance();
                    BinaryOp::As
                }
                _ => break,
            };
            let right = self.parse_binary(prec + 1);
            left = Expr::Binary(
                Box::new(left),
                op,
                Box::new(right),
                Span::new(span_start, self.prev_end()),
            );
        }
        left
    }

    fn binary_precedence(&self) -> u8 {
        match self.peek() {
            TokenKind::PipePipe => 1,
            TokenKind::AmpersandAmpersand => 2,
            TokenKind::Pipe => 3,
            TokenKind::Caret => 4,
            TokenKind::Ampersand => 5,
            TokenKind::EqEq | TokenKind::Ne => 6,
            TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq
            | TokenKind::Is
            | TokenKind::As => 7,
            TokenKind::LtLt | TokenKind::GtGt => 8,
            TokenKind::Plus | TokenKind::Minus => 9,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 10,
            _ => 0,
        }
    }

    fn parse_unary(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Neg,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Exclamation => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Not,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Tilde => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::BitNot,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Plus => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Plus,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::PlusPlus => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::PreInc,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::MinusMinus => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::PreDec,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Ampersand => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Ref,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Star => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Deref,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Await => {
                self.advance();
                let e = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Await,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            _ => self.parse_postfix(),
        }
    }
    fn parse_unary_guarded(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Error(Span::ZERO);
        }
        let expr = self.parse_unary();
        self.pop_recursion();
        expr
    }

    fn parse_postfix(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut expr = self.parse_primary();
        loop {
            match self.peek() {
                TokenKind::PlusPlus => {
                    self.advance();
                    expr = Expr::Unary(
                        UnaryOp::PostInc,
                        Box::new(expr),
                        Span::new(start, self.prev_end()),
                    );
                }
                TokenKind::MinusMinus => {
                    self.advance();
                    expr = Expr::Unary(
                        UnaryOp::PostDec,
                        Box::new(expr),
                        Span::new(start, self.prev_end()),
                    );
                }
                TokenKind::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                        args.push(self.parse_expr());
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RParen);
                    expr = Expr::Call(Box::new(expr), args, Span::new(start, self.prev_end()));
                }
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr();
                    self.expect(TokenKind::RBracket);
                    expr = Expr::Index(
                        Box::new(expr),
                        Box::new(index),
                        Span::new(start, self.prev_end()),
                    );
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.expect_ident();
                    if self.peek() == TokenKind::LParen {
                        self.advance();
                        let mut args = Vec::new();
                        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                            args.push(self.parse_expr());
                            if self.peek() == TokenKind::Comma {
                                self.advance();
                            }
                        }
                        self.expect(TokenKind::RParen);
                        expr = Expr::Call(Box::new(expr), args, Span::new(start, self.prev_end()));
                    } else {
                        expr =
                            Expr::Member(Box::new(expr), name, Span::new(start, self.prev_end()));
                    }
                }
                TokenKind::QuestionDot => {
                    self.advance();
                    let name = self.expect_ident();
                    expr = Expr::NullConditional(
                        Box::new(expr),
                        name,
                        Span::new(start, self.prev_end()),
                    );
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::IntLit | TokenKind::UIntLit | TokenKind::LongLit | TokenKind::ULongLit => {
                let tok = self.advance();
                let val = tok.value.parse::<i64>().unwrap_or(0);
                Expr::Int(val, tok.span)
            }
            TokenKind::FloatLit | TokenKind::DoubleLit | TokenKind::DecimalLit => {
                let tok = self.advance();
                let val = tok.value.parse::<f64>().unwrap_or(0.0);
                Expr::Double(val, tok.span)
            }
            TokenKind::StringLit
            | TokenKind::VerbatimStringLit
            | TokenKind::InterpolatedStringLit => {
                let tok = self.advance();
                Expr::String(tok.value.clone(), tok.span)
            }
            TokenKind::CharLit => {
                let tok = self.advance();
                let ch = tok.value.chars().nth(1).unwrap_or('\0');
                Expr::Char(ch, tok.span)
            }
            TokenKind::TrueLit => {
                let tok = self.advance();
                Expr::Bool(true, tok.span)
            }
            TokenKind::FalseLit => {
                let tok = self.advance();
                Expr::Bool(false, tok.span)
            }
            TokenKind::NullLit => {
                let tok = self.advance();
                Expr::Null(tok.span)
            }
            TokenKind::Ident | TokenKind::Var | TokenKind::This | TokenKind::Base => {
                let tok = self.advance();
                Expr::Ident(tok.value.clone(), tok.span)
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(TokenKind::RParen);
                Expr::Paren(Box::new(expr), Span::new(start, self.prev_end()))
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                while self.peek() != TokenKind::RBracket && self.peek() != TokenKind::Eof {
                    items.push(self.parse_expr());
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBracket);
                Expr::Array(items, Span::new(start, self.prev_end()))
            }
            TokenKind::New => self.parse_new_expr(start),
            TokenKind::Typeof => {
                self.advance();
                self.expect(TokenKind::LParen);
                let e = self.parse_type();
                self.expect(TokenKind::RParen);
                Expr::Typeof(Box::new(e), Span::new(start, self.prev_end()))
            }
            TokenKind::Nameof => {
                self.advance();
                self.expect(TokenKind::LParen);
                let e = self.parse_expr();
                self.expect(TokenKind::RParen);
                Expr::Nameof(Box::new(e), Span::new(start, self.prev_end()))
            }
            TokenKind::Sizeof => {
                self.advance();
                self.expect(TokenKind::LParen);
                let e = self.parse_type();
                self.expect(TokenKind::RParen);
                Expr::Sizeof(Box::new(e), Span::new(start, self.prev_end()))
            }
            TokenKind::Default => {
                self.advance();
                self.expect(TokenKind::LParen);
                let e = self.parse_type();
                self.expect(TokenKind::RParen);
                Expr::Default(Box::new(e), Span::new(start, self.prev_end()))
            }
            TokenKind::Throw => {
                let tok = self.advance();
                let e = self.parse_expr();
                Expr::Throw(Box::new(e), tok.span)
            }
            _ => {
                let tok = self.advance();
                Expr::Ident(format!("{:?}", tok.kind), tok.span)
            }
        }
    }

    fn parse_new_expr(&mut self, start: Pos) -> Expr {
        self.advance();
        let type_ = if self.peek() == TokenKind::LBracket {
            self.advance();
            self.expect(TokenKind::RBracket);
            Expr::Ident("implicit[]".to_string(), Span::new(start, self.prev_end()))
        } else if self.peek() == TokenKind::LParen {
            Expr::Ident(
                "target-typed".to_string(),
                Span::new(start, self.prev_end()),
            )
        } else {
            self.parse_type()
        };
        let mut args = Vec::new();
        if self.peek() == TokenKind::LParen {
            self.advance();
            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                args.push(self.parse_expr());
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RParen);
        }
        if self.peek() == TokenKind::LBrace {
            self.advance();
            while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                args.push(self.parse_expr());
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RBrace);
        }
        Expr::New(Box::new(type_), args, Span::new(start, self.prev_end()))
    }

    pub fn parse_type(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let name = match self.peek() {
            TokenKind::Bool => {
                self.advance();
                "bool".to_string()
            }
            TokenKind::Byte => {
                self.advance();
                "byte".to_string()
            }
            TokenKind::Sbyte => {
                self.advance();
                "sbyte".to_string()
            }
            TokenKind::Short => {
                self.advance();
                "short".to_string()
            }
            TokenKind::Ushort => {
                self.advance();
                "ushort".to_string()
            }
            TokenKind::Int => {
                self.advance();
                "int".to_string()
            }
            TokenKind::Uint => {
                self.advance();
                "uint".to_string()
            }
            TokenKind::Long => {
                self.advance();
                "long".to_string()
            }
            TokenKind::Ulong => {
                self.advance();
                "ulong".to_string()
            }
            TokenKind::Float => {
                self.advance();
                "float".to_string()
            }
            TokenKind::Double => {
                self.advance();
                "double".to_string()
            }
            TokenKind::Decimal => {
                self.advance();
                "decimal".to_string()
            }
            TokenKind::Char => {
                self.advance();
                "char".to_string()
            }
            TokenKind::String => {
                self.advance();
                "string".to_string()
            }
            TokenKind::Object => {
                self.advance();
                "object".to_string()
            }
            TokenKind::Void => {
                self.advance();
                "void".to_string()
            }
            TokenKind::Nint => {
                self.advance();
                "nint".to_string()
            }
            TokenKind::Nuint => {
                self.advance();
                "nuint".to_string()
            }
            TokenKind::Ident => self.expect_ident(),
            TokenKind::Var => {
                self.advance();
                "var".to_string()
            }
            _ => {
                let tok = self.advance();
                tok.value.clone()
            }
        };
        let mut full_name = name;
        if self.peek() == TokenKind::Lt {
            let mut depth = 0usize;
            while self.peek() != TokenKind::Eof {
                let token = self.advance();
                match token.kind {
                    TokenKind::Lt => {
                        depth += 1;
                        full_name.push('<');
                    }
                    TokenKind::Gt => {
                        depth = depth.saturating_sub(1);
                        full_name.push('>');
                        if depth == 0 {
                            break;
                        }
                    }
                    TokenKind::GtGt => {
                        depth = depth.saturating_sub(2);
                        full_name.push_str(">>");
                        if depth == 0 {
                            break;
                        }
                    }
                    TokenKind::Comma => full_name.push_str(", "),
                    TokenKind::Dot => full_name.push('.'),
                    _ => full_name.push_str(&type_token_text(&token)),
                }
            }
        }
        let mut typ = Expr::Ident(full_name, Span::new(start, self.prev_end()));
        while self.peek() == TokenKind::LBracket {
            self.advance();
            let mut commas = 0usize;
            while self.peek() == TokenKind::Comma {
                commas += 1;
                self.advance();
            }
            self.expect(TokenKind::RBracket);
            let base = expr_type_name(&typ);
            typ = Expr::Ident(
                format!("{base}[{}]", ",".repeat(commas)),
                Span::new(start, self.prev_end()),
            );
        }
        if self.peek() == TokenKind::Nullable {
            self.advance();
            typ = Expr::Ident(
                format!("{}?", expr_type_name(&typ)),
                Span::new(start, self.prev_end()),
            );
        }
        typ
    }
}

fn expr_type_name(expr: &Expr) -> &str {
    match expr {
        Expr::Ident(name, _) => name,
        _ => "object",
    }
}

fn type_token_text(token: &super::super::lexer::Token) -> String {
    if !token.value.is_empty() {
        return token.value.clone();
    }
    match token.kind {
        TokenKind::Int => "int",
        TokenKind::String => "string",
        TokenKind::Bool => "bool",
        TokenKind::Double => "double",
        TokenKind::Float => "float",
        TokenKind::Char => "char",
        TokenKind::Byte => "byte",
        TokenKind::Short => "short",
        TokenKind::Long => "long",
        TokenKind::Uint => "uint",
        TokenKind::Ushort => "ushort",
        TokenKind::Ulong => "ulong",
        TokenKind::Sbyte => "sbyte",
        TokenKind::Decimal => "decimal",
        TokenKind::Object => "object",
        TokenKind::Void => "void",
        TokenKind::Nint => "nint",
        TokenKind::Nuint => "nuint",
        _ => "object",
    }
    .to_string()
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s)
            | Expr::UInt(_, s)
            | Expr::Long(_, s)
            | Expr::ULong(_, s)
            | Expr::Float(_, s)
            | Expr::Double(_, s)
            | Expr::Decimal(_, s)
            | Expr::String(_, s)
            | Expr::Char(_, s)
            | Expr::Bool(_, s)
            | Expr::Null(s)
            | Expr::Ident(_, s)
            | Expr::Paren(_, s)
            | Expr::Array(_, s)
            | Expr::Typeof(_, s)
            | Expr::Nameof(_, s)
            | Expr::Sizeof(_, s)
            | Expr::Default(_, s)
            | Expr::Error(s) => *s,
            Expr::Binary(_, _, _, s)
            | Expr::Unary(_, _, s)
            | Expr::Call(_, _, s)
            | Expr::Index(_, _, s)
            | Expr::Member(_, _, s)
            | Expr::Conditional(_, _, _, s)
            | Expr::NullCoalesce(_, _, s)
            | Expr::NullConditional(_, _, s)
            | Expr::Lambda(_, s)
            | Expr::New(_, _, s)
            | Expr::Assign(_, _, s)
            | Expr::IsPattern(_, _, s)
            | Expr::SwitchExpr(_, _, s)
            | Expr::Throw(_, s)
            | Expr::Await(_, s) => *s,
            Expr::ObjectInit(_, _, s)
            | Expr::CollectionInit(_, s)
            | Expr::AnonymousMethod(_, _, s)
            | Expr::InterpolatedString(_, s) => *s,
        }
    }
}
