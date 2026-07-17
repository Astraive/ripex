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
            TokenKind::New => {
                self.advance();
                let typ = self.parse_type();
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
                Expr::New(Box::new(typ), args, Span::new(start, self.prev_end()))
            }
            TokenKind::Delete => {
                self.advance();
                let e = self.parse_unary();
                Expr::Delete(Box::new(e), Span::new(start, self.prev_end()))
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
            TokenKind::StaticCast
            | TokenKind::DynamicCast
            | TokenKind::ConstCast
            | TokenKind::ReinterpretCast => {
                let cast_kind = self.peek();
                self.advance();
                self.expect(TokenKind::Lt);
                let typ = self.parse_expr();
                self.expect(TokenKind::Gt);
                self.expect(TokenKind::LParen);
                let expr = self.parse_expr();
                self.expect(TokenKind::RParen);
                match cast_kind {
                    TokenKind::StaticCast => Expr::StaticCast(
                        Box::new(typ),
                        Box::new(expr),
                        Span::new(start, self.prev_end()),
                    ),
                    TokenKind::DynamicCast => Expr::DynamicCast(
                        Box::new(typ),
                        Box::new(expr),
                        Span::new(start, self.prev_end()),
                    ),
                    TokenKind::ConstCast => Expr::ConstCast(
                        Box::new(typ),
                        Box::new(expr),
                        Span::new(start, self.prev_end()),
                    ),
                    _ => Expr::ReinterpretCast(
                        Box::new(typ),
                        Box::new(expr),
                        Span::new(start, self.prev_end()),
                    ),
                }
            }
            TokenKind::Lambda => self.parse_lambda(start),
            _ => self.parse_postfix(),
        }
    }

    fn parse_lambda(&mut self, start: Pos) -> Expr {
        self.advance(); // lambda
        let captures = Vec::new();
        self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            let ptype = self.parse_type();
            let pname = if self.peek() == TokenKind::Ident {
                Some(self.expect_ident())
            } else {
                None
            };
            params.push(ParamDecl {
                type_: Box::new(ptype),
                name: pname,
                default: None,
                span: Span::new(start, self.prev_end()),
            });
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        let body = self.parse_block();
        Expr::Lambda(
            LambdaExpr {
                captures,
                params,
                return_type: None,
                body: Box::new(body),
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
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
                TokenKind::ColonColon => {
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
                let val = tok.value.parse::<i64>().unwrap_or(0);
                Expr::Int(val, tok.span)
            }
            TokenKind::FloatLit | TokenKind::DoubleLit | TokenKind::HexFloatLit => {
                let tok = self.advance();
                let val = tok.value.parse::<f64>().unwrap_or(0.0);
                Expr::Float(val, tok.span)
            }
            TokenKind::StringLit
            | TokenKind::RawStringLit
            | TokenKind::LStringLit
            | TokenKind::U16StringLit
            | TokenKind::U32StringLit
            | TokenKind::WStringLit => {
                let tok = self.advance();
                Expr::String(tok.value.clone(), tok.span)
            }
            TokenKind::CharLit
            | TokenKind::Char16Lit
            | TokenKind::Char32Lit
            | TokenKind::WcharLit => {
                let tok = self.advance();
                let ch = tok.value.chars().nth(1).unwrap_or('\0');
                Expr::Char(ch, tok.span)
            }
            TokenKind::True => {
                let tok = self.advance();
                Expr::Bool(true, tok.span)
            }
            TokenKind::False => {
                let tok = self.advance();
                Expr::Bool(false, tok.span)
            }
            TokenKind::Null | TokenKind::Nullptr => {
                let tok = self.advance();
                Expr::NullPtr(tok.span)
            }
            TokenKind::This => {
                let tok = self.advance();
                Expr::This(tok.span)
            }
            TokenKind::Ident => {
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
                let mut captures = Vec::new();
                while self.peek() != TokenKind::RBracket && self.peek() != TokenKind::Eof {
                    let pos_before = self.pos;
                    let by_ref = self.peek() == TokenKind::Ampersand;
                    if by_ref {
                        self.advance();
                    }
                    let name = if self.peek() == TokenKind::Ident {
                        Some(self.expect_ident())
                    } else {
                        None
                    };
                    captures.push(LambdaCapture {
                        by_ref,
                        name,
                        span: Span::new(start, self.prev_end()),
                    });
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                    // Malformed captures such as `[(` must still consume a
                    // token. Without this guard, the loop grows `captures`
                    // forever and eventually aborts the process on OOM.
                    if self.pos == pos_before {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBracket);
                let mut params = Vec::new();
                if self.peek() == TokenKind::LParen {
                    self.advance();
                    while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                        let type_ = self.parse_type();
                        let name = if self.peek() == TokenKind::Ident {
                            Some(self.expect_ident())
                        } else {
                            None
                        };
                        params.push(ParamDecl {
                            type_: Box::new(type_),
                            name,
                            default: None,
                            span: Span::new(start, self.prev_end()),
                        });
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RParen);
                }
                let body = self.parse_block();
                let span = Span::new(start, self.prev_end());
                Expr::Lambda(
                    LambdaExpr {
                        captures,
                        params,
                        return_type: None,
                        body: Box::new(body),
                        span,
                    },
                    span,
                )
            }
            TokenKind::LBrace => {
                self.advance();
                let mut items = Vec::new();
                while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                    items.push(self.parse_expr());
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace);
                Expr::BraceInit(items, Span::new(start, self.prev_end()))
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
        while self.peek() == TokenKind::Star {
            self.advance();
            base = Expr::Unary(
                UnaryOp::Deref,
                Box::new(base),
                Span::new(start, self.prev_end()),
            );
        }
        if self.peek() == TokenKind::Ampersand {
            self.advance();
            base = Expr::Unary(
                UnaryOp::Ref,
                Box::new(base),
                Span::new(start, self.prev_end()),
            );
        }
        if self.peek() == TokenKind::AmpersandAmpersand {
            self.advance();
            base = Expr::Unary(
                UnaryOp::Ref,
                Box::new(base),
                Span::new(start, self.prev_end()),
            );
        }
        base
    }

    fn parse_type_spec(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut qualifiers = Vec::new();
        loop {
            match self.peek() {
                TokenKind::Const => {
                    qualifiers.push("const".to_string());
                    self.advance();
                }
                TokenKind::Volatile => {
                    qualifiers.push("volatile".to_string());
                    self.advance();
                }
                TokenKind::Static => {
                    qualifiers.push("static".to_string());
                    self.advance();
                }
                TokenKind::Extern => {
                    qualifiers.push("extern".to_string());
                    self.advance();
                }
                TokenKind::Mutable => {
                    qualifiers.push("mutable".to_string());
                    self.advance();
                }
                _ => break,
            }
        }
        let name = match self.peek() {
            TokenKind::Void => {
                self.advance();
                "void".to_string()
            }
            TokenKind::Char => {
                self.advance();
                "char".to_string()
            }
            TokenKind::WcharT => {
                self.advance();
                "wchar_t".to_string()
            }
            TokenKind::Char16 => {
                self.advance();
                "char16_t".to_string()
            }
            TokenKind::Char32 => {
                self.advance();
                "char32_t".to_string()
            }
            TokenKind::Short => {
                self.advance();
                "short".to_string()
            }
            TokenKind::Int => {
                self.advance();
                "int".to_string()
            }
            TokenKind::Long => {
                self.advance();
                if self.peek() == TokenKind::Long {
                    self.advance();
                }
                "long".to_string()
            }
            TokenKind::Float => {
                self.advance();
                "float".to_string()
            }
            TokenKind::Double => {
                self.advance();
                "double".to_string()
            }
            TokenKind::Signed => {
                self.advance();
                "signed".to_string()
            }
            TokenKind::Unsigned => {
                self.advance();
                "unsigned".to_string()
            }
            TokenKind::Bool => {
                self.advance();
                "bool".to_string()
            }
            TokenKind::Ident => {
                let mut name = self.expect_ident();
                while self.peek() == TokenKind::ColonColon && self.peek_ahead(1) == TokenKind::Ident
                {
                    self.advance();
                    name.push_str("::");
                    name.push_str(&self.expect_ident());
                }
                if self.peek() == TokenKind::Lt {
                    let mut depth = 0usize;
                    while self.peek() != TokenKind::Eof {
                        match self.peek() {
                            TokenKind::Lt => depth += 1,
                            TokenKind::Gt => {
                                depth = depth.saturating_sub(1);
                                self.advance();
                                if depth == 0 {
                                    break;
                                }
                                continue;
                            }
                            _ => {}
                        }
                        self.advance();
                    }
                }
                name
            }
            TokenKind::Struct | TokenKind::Class => {
                self.advance();
                let name = self.expect_ident();
                return Expr::Ident(
                    format!(
                        "{} {}",
                        if self.pos > 0 && self.tokens[self.pos - 2].kind == TokenKind::Struct {
                            "struct"
                        } else {
                            "class"
                        },
                        name
                    ),
                    Span::new(start, self.prev_end()),
                );
            }
            TokenKind::Enum => {
                self.advance();
                let name = self.expect_ident();
                return Expr::Ident(format!("enum {}", name), Span::new(start, self.prev_end()));
            }
            TokenKind::Union => {
                self.advance();
                let name = self.expect_ident();
                return Expr::Ident(format!("union {}", name), Span::new(start, self.prev_end()));
            }
            _ => {
                let tok = self.advance();
                tok.value.clone()
            }
        };
        Expr::Ident(name, Span::new(start, self.prev_end()))
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
            | Expr::Bool(_, s)
            | Expr::NullPtr(s)
            | Expr::Ident(_, s)
            | Expr::Paren(_, s)
            | Expr::This(s)
            | Expr::BraceInit(_, s)
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
            | Expr::DynamicCast(_, _, s)
            | Expr::StaticCast(_, _, s)
            | Expr::ConstCast(_, _, s)
            | Expr::ReinterpretCast(_, _, s)
            | Expr::Sizeof(_, s)
            | Expr::Alignof(_, s)
            | Expr::Typeid(_, s)
            | Expr::Ternary(_, _, _, s)
            | Expr::Comma(_, s)
            | Expr::Lambda(_, s)
            | Expr::New(_, _, s)
            | Expr::Delete(_, s)
            | Expr::Assign(_, _, s)
            | Expr::Template(_, _, s) => *s,
        }
    }
}
