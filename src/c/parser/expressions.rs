use super::super::ast::expr::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::Span;

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
        ) {
            let _op = self.advance();
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
        let mut cond = self.parse_binary(0);
        if self.peek() == TokenKind::Question {
            self.advance();
            let then = self.parse_expr();
            self.expect(TokenKind::Colon);
            let else_ = self.parse_ternary();
            cond = Expr::Ternary(
                Box::new(cond),
                Box::new(then),
                Box::new(else_),
                Span::new(start, self.prev_end()),
            );
        }
        cond
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
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => 7,
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
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::Neg,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Exclamation => {
                self.advance();
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::Not,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Tilde => {
                self.advance();
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::BitNot,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Ampersand => {
                self.advance();
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::Ref,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Star => {
                self.advance();
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::Deref,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Plus => {
                self.advance();
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::Plus,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::PlusPlus => {
                self.advance();
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::PreInc,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::MinusMinus => {
                self.advance();
                let e = self.parse_unary();
                Expr::Unary(
                    UnaryOp::PreDec,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Sizeof => {
                self.advance();
                if self.peek() == TokenKind::LParen {
                    self.advance();
                    let e = self.parse_expr();
                    self.expect(TokenKind::RParen);
                    Expr::Sizeof(Box::new(e), Span::new(start, self.prev_end()))
                } else {
                    let e = self.parse_unary();
                    Expr::Sizeof(Box::new(e.clone()), Span::new(start, e.span().end))
                }
            }
            _ => self.parse_postfix(),
        }
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
                    expr = Expr::Member(Box::new(expr), name, Span::new(start, self.prev_end()));
                }
                TokenKind::Arrow => {
                    self.advance();
                    let name = self.expect_ident();
                    expr = Expr::Arrow(Box::new(expr), name, Span::new(start, self.prev_end()));
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::IntLit
            | TokenKind::UIntLit
            | TokenKind::LongLit
            | TokenKind::ULongLit
            | TokenKind::LongLongLit
            | TokenKind::ULongLongLit => {
                let tok = self.advance();
                let val = tok
                    .value
                    .trim_end_matches(|c: char| {
                        !(c.is_ascii_digit() || matches!(c, 'a'..='f' | 'A'..='F' | 'x' | 'X'))
                    })
                    .parse::<i64>()
                    .unwrap_or(0);
                Expr::Int(val, tok.span)
            }
            TokenKind::FloatLit | TokenKind::DoubleLit | TokenKind::HexFloatLit => {
                let tok = self.advance();
                let val = tok.value.parse::<f64>().unwrap_or(0.0);
                Expr::Float(val, tok.span)
            }
            TokenKind::StringLit | TokenKind::LStringLit | TokenKind::UStringLit => {
                let tok = self.advance();
                Expr::String(tok.value.clone(), tok.span)
            }
            TokenKind::CharLit | TokenKind::LCharLit | TokenKind::UCharLit => {
                let tok = self.advance();
                let ch = tok.value.chars().nth(1).unwrap_or('\0');
                Expr::Char(ch, tok.span)
            }
            TokenKind::True => {
                let tok = self.advance();
                Expr::Int(1, tok.span)
            }
            TokenKind::False => {
                let tok = self.advance();
                Expr::Int(0, tok.span)
            }
            TokenKind::Null => {
                let tok = self.advance();
                Expr::Int(0, tok.span)
            }
            TokenKind::Ident => {
                let tok = self.advance();
                Expr::Ident(tok.value.clone(), tok.span)
            }
            TokenKind::LParen => {
                self.advance();
                let is_cast = matches!(
                    self.peek(),
                    TokenKind::Void
                        | TokenKind::Char
                        | TokenKind::Short
                        | TokenKind::Int
                        | TokenKind::Long
                        | TokenKind::Float
                        | TokenKind::Double
                        | TokenKind::Signed
                        | TokenKind::Unsigned
                        | TokenKind::Bool
                        | TokenKind::Struct
                        | TokenKind::Union
                        | TokenKind::Enum
                        | TokenKind::Const
                        | TokenKind::Volatile
                ) || (self.peek() == TokenKind::Ident
                    && self.peek_ahead(1) == TokenKind::Star);
                if is_cast {
                    let type_ = self.parse_type();
                    self.expect(TokenKind::RParen);
                    let value = self.parse_unary();
                    return Expr::Cast(
                        Box::new(type_),
                        Box::new(value),
                        Span::new(start, self.prev_end()),
                    );
                }
                let expr = self.parse_expr();
                self.expect(TokenKind::RParen);
                Expr::Paren(Box::new(expr), Span::new(start, self.prev_end()))
            }
            TokenKind::LBrace => {
                // Compound literal or block
                self.advance();
                let mut items = Vec::new();
                while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                    items.push(self.parse_expr());
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace);
                let end = self.prev_end();
                Expr::StmtExpr(vec![], Span::new(start, end))
            }
            _ => {
                let tok = self.advance();
                Expr::Ident(format!("{:?}", tok.kind), tok.span)
            }
        }
    }

    pub fn parse_type(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut base = self.parse_type_spec();
        // Pointer declarators
        while self.peek() == TokenKind::Star {
            self.advance();
            base = Expr::Unary(
                UnaryOp::Deref,
                Box::new(base),
                Span::new(start, self.prev_end()),
            );
        }
        base
    }

    fn parse_type_spec(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Void => {
                self.advance();
                Expr::Ident("void".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Char => {
                self.advance();
                Expr::Ident("char".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Short => {
                self.advance();
                Expr::Ident("short".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Int => {
                self.advance();
                Expr::Ident("int".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Long => {
                self.advance();
                // Consume trailing `long`/`int` qualifiers (e.g. `long long int`).
                while matches!(self.peek(), TokenKind::Long | TokenKind::Int) {
                    self.advance();
                }
                Expr::Ident("long".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Float => {
                self.advance();
                Expr::Ident("float".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Double => {
                self.advance();
                Expr::Ident("double".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Signed => {
                self.advance();
                Expr::Ident("signed".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Unsigned => {
                self.advance();
                Expr::Ident("unsigned".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Bool => {
                self.advance();
                Expr::Ident("_Bool".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Struct => {
                self.advance();
                let name = self.parse_optional_tag_name();
                let fields = if self.peek() == TokenKind::LBrace {
                    Some(self.parse_struct_body())
                } else {
                    None
                };
                if name.is_empty() && fields.is_none() {
                    Expr::Ident("struct".to_string(), Span::new(start, self.prev_end()))
                } else {
                    Expr::DeclSpec(
                        DeclSpec::Struct(name, fields),
                        Span::new(start, self.prev_end()),
                    )
                }
            }
            TokenKind::Union => {
                self.advance();
                let name = self.parse_optional_tag_name();
                let fields = if self.peek() == TokenKind::LBrace {
                    Some(self.parse_struct_body())
                } else {
                    None
                };
                if name.is_empty() && fields.is_none() {
                    Expr::Ident("union".to_string(), Span::new(start, self.prev_end()))
                } else {
                    Expr::DeclSpec(
                        DeclSpec::Union(name, fields),
                        Span::new(start, self.prev_end()),
                    )
                }
            }
            TokenKind::Enum => {
                self.advance();
                let name = self.parse_optional_tag_name();
                let constants = if self.peek() == TokenKind::LBrace {
                    Some(self.parse_enum_body())
                } else {
                    None
                };
                if name.is_empty() && constants.is_none() {
                    Expr::Ident("enum".to_string(), Span::new(start, self.prev_end()))
                } else {
                    Expr::DeclSpec(
                        DeclSpec::Enum(name, constants),
                        Span::new(start, self.prev_end()),
                    )
                }
            }
            TokenKind::Const => {
                self.advance();
                self.parse_type_spec()
            }
            TokenKind::Volatile => {
                self.advance();
                self.parse_type_spec()
            }
            TokenKind::Extern => {
                self.advance();
                self.parse_type_spec()
            }
            TokenKind::Static => {
                self.advance();
                self.parse_type_spec()
            }
            TokenKind::Ident => {
                let tok = self.advance();
                Expr::Ident(tok.value.clone(), tok.span)
            }
            _ => {
                let tok = self.advance();
                Expr::Ident(tok.value.clone(), tok.span)
            }
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int(_, s)
            | Expr::UInt(_, s)
            | Expr::Float(_, s)
            | Expr::String(_, s)
            | Expr::Char(_, s)
            | Expr::Ident(_, s)
            | Expr::Paren(_, s)
            | Expr::StmtExpr(_, s)
            | Expr::Error(s) => *s,
            Expr::Binary(_, _, _, s)
            | Expr::Unary(_, _, s)
            | Expr::Call(_, _, s)
            | Expr::Index(_, _, s)
            | Expr::Member(_, _, s)
            | Expr::Arrow(_, _, s)
            | Expr::Deref(_, s)
            | Expr::Ref(_, s)
            | Expr::Cast(_, _, s)
            | Expr::Sizeof(_, s)
            | Expr::Alignof(_, s)
            | Expr::Ternary(_, _, _, s)
            | Expr::Comma(_, s)
            | Expr::Assign(_, _, s)
            | Expr::StringConcat(_, s)
            | Expr::DeclSpec(_, s) => *s,
        }
    }
}
