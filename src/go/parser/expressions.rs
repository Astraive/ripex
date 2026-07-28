use super::super::ast::expr::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::{Pos, Span};

impl Parser {
    pub fn parse_expr(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Ident(String::new(), Span::ZERO);
        }
        let result = self.parse_binary(0);
        self.pop_recursion();
        result
    }

    pub(super) fn parse_expr_without_composite_literal(&mut self) -> Expr {
        let previous = self.allow_composite_literal;
        self.allow_composite_literal = false;
        let expr = self.parse_expr();
        self.allow_composite_literal = previous;
        expr
    }

    fn parse_binary(&mut self, min_prec: u8) -> Expr {
        let mut left = self.parse_unary();
        loop {
            let prec = self.binary_precedence();
            if prec < min_prec {
                break;
            }
            let op = match self.peek() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                TokenKind::Percent => BinaryOp::Mod,
                TokenKind::EqEq => BinaryOp::Eq,
                TokenKind::Ne => BinaryOp::Ne,
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::LtEq => BinaryOp::Le,
                TokenKind::GtEq => BinaryOp::Ge,
                TokenKind::AmpersandAmpersand => BinaryOp::And,
                TokenKind::PipePipe => BinaryOp::Or,
                TokenKind::Ampersand => BinaryOp::BitAnd,
                TokenKind::Pipe => BinaryOp::BitOr,
                TokenKind::Caret => BinaryOp::BitXor,
                TokenKind::LtLt => BinaryOp::Shl,
                TokenKind::GtGt => BinaryOp::Shr,
                TokenKind::AmpersandCaret => BinaryOp::BitClear,
                TokenKind::Eq => BinaryOp::Assign,
                TokenKind::PlusEq => BinaryOp::AddAssign,
                TokenKind::MinusEq => BinaryOp::SubAssign,
                TokenKind::StarEq => BinaryOp::MulAssign,
                TokenKind::SlashEq => BinaryOp::DivAssign,
                TokenKind::PercentEq => BinaryOp::ModAssign,
                TokenKind::AmpersandEq => BinaryOp::AndAssign,
                TokenKind::PipeEq => BinaryOp::OrAssign,
                TokenKind::CaretEq => BinaryOp::XorAssign,
                TokenKind::LtLtEq => BinaryOp::ShlAssign,
                TokenKind::GtGtEq => BinaryOp::ShrAssign,
                _ => break,
            };
            self.advance();
            let right = self.parse_binary(prec + 1);
            let span = left.span().merge(right.span());
            left = Expr::Binary(Box::new(left), op, Box::new(right), span);
        }
        left
    }

    fn binary_precedence(&self) -> u8 {
        match self.peek() {
            TokenKind::PipePipe => 1,
            TokenKind::AmpersandAmpersand => 2,
            TokenKind::EqEq
            | TokenKind::Ne
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::LtEq
            | TokenKind::GtEq => 3,
            TokenKind::Plus | TokenKind::Minus | TokenKind::Pipe | TokenKind::Caret => 4,
            TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::LtLt
            | TokenKind::GtGt
            | TokenKind::Ampersand
            | TokenKind::AmpersandCaret => 5,
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
            | TokenKind::GtGtEq => 0,
            _ => 0,
        }
    }

    fn parse_unary(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Neg,
                    Box::new(expr.clone()),
                    Span::new(start, expr.span().end),
                )
            }
            TokenKind::Exclamation => {
                self.advance();
                let expr = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Not,
                    Box::new(expr.clone()),
                    Span::new(start, expr.span().end),
                )
            }
            TokenKind::Ampersand => {
                self.advance();
                let expr = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Ref,
                    Box::new(expr.clone()),
                    Span::new(start, expr.span().end),
                )
            }
            TokenKind::Star => {
                self.advance();
                let expr = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Deref,
                    Box::new(expr.clone()),
                    Span::new(start, expr.span().end),
                )
            }
            TokenKind::Arrow => {
                self.advance();
                let expr = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Receive,
                    Box::new(expr.clone()),
                    Span::new(start, expr.span().end),
                )
            }
            TokenKind::Plus => {
                self.advance();
                let expr = self.parse_unary_guarded();
                Expr::Unary(
                    UnaryOp::Plus,
                    Box::new(expr.clone()),
                    Span::new(start, expr.span().end),
                )
            }
            _ => self.parse_primary(),
        }
    }
    fn parse_unary_guarded(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Ident(String::new(), Span::ZERO);
        }
        let expr = self.parse_unary();
        self.pop_recursion();
        expr
    }

    fn parse_primary(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut expr = match self.peek() {
            TokenKind::IntLit => {
                let tok = self.advance();
                let val = tok.value.parse::<i64>().unwrap_or(0);
                Expr::Int(val, tok.span)
            }
            TokenKind::FloatLit => {
                let tok = self.advance();
                let val = tok.value.parse::<f64>().unwrap_or(0.0);
                Expr::Float(val, tok.span)
            }
            TokenKind::InterpretedString | TokenKind::RawString => {
                let tok = self.advance();
                Expr::String(tok.value.clone(), tok.span)
            }
            TokenKind::True => {
                let tok = self.advance();
                Expr::Bool(true, tok.span)
            }
            TokenKind::False => {
                let tok = self.advance();
                Expr::Bool(false, tok.span)
            }
            TokenKind::Nil => {
                let tok = self.advance();
                Expr::Nil(tok.span)
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
            TokenKind::LBrace => {
                return self.parse_struct_lit(start);
            }
            TokenKind::LBracket => {
                let typ = self.parse_type();
                if self.peek() == TokenKind::LBrace {
                    return self.parse_composite_lit(typ);
                }
                typ
            }
            TokenKind::Func => self.parse_func_lit(start),
            _ => {
                let tok = self.advance();
                Expr::Ident(format!("{:?}", tok.kind), tok.span)
            }
        };

        loop {
            match self.peek() {
                TokenKind::LParen => {
                    expr = self.parse_call(expr);
                }
                TokenKind::LBracket => {
                    expr = self.parse_index(expr);
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.expect_ident();
                    let tok = self.peek_token();
                    expr = Expr::Selector(
                        Box::new(expr.clone()),
                        name,
                        Span::new(expr.span().start, tok.span.end),
                    );
                }
                TokenKind::LBrace if self.allow_composite_literal => {
                    // Composite literal
                    expr = self.parse_composite_lit(expr);
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_call(&mut self, callee: Expr) -> Expr {
        let start = callee.span().start;
        self.advance();
        let mut args = Vec::new();
        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            args.push(self.parse_expr());
            if self.peek() == TokenKind::DotDotDot {
                self.advance();
            }
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        let end = self.prev_end();
        Expr::Call(Box::new(callee), args, Span::new(start, end))
    }

    fn parse_index(&mut self, expr: Expr) -> Expr {
        let start = expr.span().start;
        self.advance();
        let lower = if self.peek() == TokenKind::Colon {
            None
        } else {
            Some(self.parse_expr())
        };
        if self.peek() == TokenKind::Colon {
            self.advance();
            let upper = if self.peek() == TokenKind::RBracket {
                None
            } else {
                Some(Box::new(self.parse_expr()))
            };
            self.expect(TokenKind::RBracket);
            let end = self.prev_end();
            return Expr::Slice(
                Box::new(expr),
                lower.map(Box::new),
                upper,
                Span::new(start, end),
            );
        }
        self.expect(TokenKind::RBracket);
        let end = self.prev_end();
        Expr::Index(
            Box::new(expr),
            Box::new(lower.unwrap_or_else(|| Expr::Ident(String::new(), Span::ZERO))),
            Span::new(start, end),
        )
    }

    fn parse_struct_lit(&mut self, start: Pos) -> Expr {
        let start_pos = start;
        // This could be a struct literal or a block
        // For simplicity, parse as struct literal if we see Ident: pattern
        self.advance();
        let mut fields = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            if self.peek() == TokenKind::Ident && self.peek_ahead(1) == TokenKind::Colon {
                let name = self.expect_ident();
                self.expect(TokenKind::Colon);
                let value = self.parse_expr();
                fields.push(FieldInit {
                    name,
                    value: Some(Box::new(value)),
                    span: Span::new(start, self.prev_end()),
                });
            } else {
                let value = self.parse_expr();
                fields.push(FieldInit {
                    name: String::new(),
                    value: Some(Box::new(value)),
                    span: Span::new(start, self.prev_end()),
                });
            }
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace);
        let end = self.prev_end();
        Expr::StructLit(String::new(), fields, Span::new(start_pos, end))
    }

    fn parse_func_lit(&mut self, start: Pos) -> Expr {
        self.advance();
        self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            let name = self.expect_ident();
            let typ = self.parse_type();
            params.push((name, Box::new(typ)));
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        let mut returns = Vec::new();
        if self.peek() != TokenKind::LBrace {
            returns.push(Box::new(self.parse_type()));
        }
        let body = self.parse_block();
        Expr::FuncLit(
            Box::new(FuncType {
                params,
                returns,
                span: Span::new(start, self.prev_end()),
            }),
            Box::new(body),
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_composite_lit(&mut self, typ: Expr) -> Expr {
        let start = typ.span().start;
        self.advance();
        let mut elems = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            let first = self.parse_expr();
            if self.peek() == TokenKind::Colon {
                self.advance();
                elems.push(self.parse_expr());
            } else {
                elems.push(first);
            }
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace);
        let end = self.prev_end();
        Expr::CompositeLit(Box::new(typ), elems, Span::new(start, end))
    }

    pub fn parse_type(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Star => {
                self.advance();
                let inner = self.parse_type();
                Expr::Unary(UnaryOp::Deref, Box::new(inner.clone()), inner.span())
            }
            TokenKind::LBracket => {
                self.advance();
                let size = if self.peek() == TokenKind::IntLit {
                    let tok = self.advance();
                    Some(tok.value.clone())
                } else {
                    None
                };
                self.expect(TokenKind::RBracket);
                let elem = self.parse_type();
                let span = Span::new(
                    size.as_ref().map_or(elem.span().start, |_| self.prev_end()),
                    elem.span().end,
                );
                Expr::Ident(
                    format!("[{}]{}", size.unwrap_or_default(), type_name(&elem)),
                    span,
                )
            }
            TokenKind::Map => {
                self.advance();
                self.expect(TokenKind::LBracket);
                let key = self.parse_type();
                self.expect(TokenKind::RBracket);
                let val = self.parse_type();
                Expr::Ident(
                    format!("map[{}]{}", type_name(&key), type_name(&val)),
                    Span::new(key.span().start, val.span().end),
                )
            }
            TokenKind::Chan => {
                self.advance();
                let dir = if self.peek() == TokenKind::Arrow {
                    self.advance();
                    "chan<-"
                } else {
                    "chan"
                };
                let inner = self.parse_type();
                Expr::Ident(format!("{} {}", dir, type_name(&inner)), inner.span())
            }
            TokenKind::Func => {
                self.advance();
                self.expect(TokenKind::LParen);
                let mut params = Vec::new();
                while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                    params.push((String::new(), Box::new(self.parse_type())));
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RParen);
                let mut returns = Vec::new();
                if self.peek() != TokenKind::LBrace
                    && self.peek() != TokenKind::Comma
                    && self.peek() != TokenKind::RParen
                {
                    returns.push(Box::new(self.parse_type()));
                }
                let params = params
                    .iter()
                    .map(|(_, ty)| type_name(ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                let returns = returns.iter().map(|ty| type_name(ty)).collect::<Vec<_>>();
                let result = match returns.as_slice() {
                    [] => String::new(),
                    [single] => format!(" {single}"),
                    many => format!(" ({})", many.join(", ")),
                };
                Expr::Ident(
                    format!("func({params}){result}"),
                    Span::new(start, self.prev_end()),
                )
            }
            TokenKind::Interface => {
                self.advance();
                self.expect(TokenKind::LBrace);
                let _methods = String::new();
                // skip interface body
                let mut depth = 1;
                while depth > 0 && self.peek() != TokenKind::Eof {
                    if self.peek() == TokenKind::LBrace {
                        depth += 1;
                    }
                    if self.peek() == TokenKind::RBrace {
                        depth -= 1;
                    }
                    self.advance();
                }
                Expr::Ident("interface{}".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::Struct => {
                self.advance();
                self.expect(TokenKind::LBrace);
                let mut depth = 1;
                while depth > 0 && self.peek() != TokenKind::Eof {
                    if self.peek() == TokenKind::LBrace {
                        depth += 1;
                    }
                    if self.peek() == TokenKind::RBrace {
                        depth -= 1;
                    }
                    self.advance();
                }
                Expr::Ident("struct{}".to_string(), Span::new(start, self.prev_end()))
            }
            TokenKind::DotDotDot => {
                self.advance();
                let inner = self.parse_type();
                Expr::Ident(
                    format!("...{}", type_name(&inner)),
                    Span::new(start, inner.span().end),
                )
            }
            _ => {
                let tok = self.advance();
                let mut typ = match tok.kind {
                    TokenKind::Ident => Expr::Ident(tok.value.clone(), tok.span),
                    TokenKind::String => Expr::Ident("string".to_string(), tok.span),
                    TokenKind::Int
                    | TokenKind::Int8
                    | TokenKind::Int16
                    | TokenKind::Int32
                    | TokenKind::Int64 => Expr::Ident(tok.value.clone(), tok.span),
                    TokenKind::Uint
                    | TokenKind::Uint8
                    | TokenKind::Uint16
                    | TokenKind::Uint32
                    | TokenKind::Uint64 => Expr::Ident(tok.value.clone(), tok.span),
                    TokenKind::Float32 | TokenKind::Float64 => {
                        Expr::Ident(tok.value.clone(), tok.span)
                    }
                    TokenKind::Bool => Expr::Ident("bool".to_string(), tok.span),
                    TokenKind::Byte => Expr::Ident("byte".to_string(), tok.span),
                    TokenKind::Rune => Expr::Ident("rune".to_string(), tok.span),
                    TokenKind::Any => Expr::Ident("any".to_string(), tok.span),
                    _ => Expr::Ident(tok.value.clone(), tok.span),
                };
                while self.peek() == TokenKind::Dot {
                    self.advance();
                    let name = self.expect_ident();
                    typ = Expr::Selector(
                        Box::new(typ.clone()),
                        name,
                        Span::new(typ.span().start, self.prev_end()),
                    );
                }
                typ
            }
        }
    }

    pub fn parse_block(&mut self) -> Block {
        if self.bump_recursion().is_err() {
            return Block {
                stmts: Vec::new(),
                span: Span::ZERO,
            };
        }
        let start = self.peek_token().span.start;
        self.expect(TokenKind::LBrace);
        let mut stmts = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            stmts.push(self.parse_stmt());
        }
        self.expect(TokenKind::RBrace);
        let end = self.prev_end();
        self.pop_recursion();
        Block {
            stmts,
            span: Span::new(start, end),
        }
    }
}

fn type_name(expr: &Expr) -> String {
    match expr {
        Expr::Ident(name, _) => name.clone(),
        Expr::Selector(object, field, _) => format!("{}.{}", type_name(object), field),
        Expr::Unary(UnaryOp::Deref, inner, _) => format!("*{}", type_name(inner)),
        Expr::Paren(inner, _) => format!("({})", type_name(inner)),
        _ => "any".to_string(),
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Bool(_, s)
            | Expr::Int(_, s)
            | Expr::Float(_, s)
            | Expr::String(_, s)
            | Expr::Nil(s)
            | Expr::Ident(_, s)
            | Expr::Paren(_, s)
            | Expr::Array(_, s)
            | Expr::StructLit(_, _, s)
            | Expr::MapLit(_, s)
            | Expr::FuncLit(_, _, s)
            | Expr::TypeAssert(_, _, s)
            | Expr::CompositeLit(_, _, s) => *s,
            Expr::Binary(_, _, _, s)
            | Expr::Unary(_, _, s)
            | Expr::Call(_, _, s)
            | Expr::Index(_, _, s)
            | Expr::Selector(_, _, s)
            | Expr::Slice(_, _, _, s) => *s,
        }
    }
}
