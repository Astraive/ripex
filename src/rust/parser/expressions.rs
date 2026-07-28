use super::super::ast::expr::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::Span;

impl Parser {
    pub fn parse_expr(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Error(Span::ZERO);
        }
        let result = self.parse_binary(0);
        self.pop_recursion();
        result
    }

    fn parse_binary(&mut self, min_prec: u8) -> Expr {
        let mut left = self.parse_prefix();
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
                    BinaryOp::Rem
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
                TokenKind::Eq => {
                    self.advance();
                    BinaryOp::Assign
                }
                TokenKind::PlusEq => {
                    self.advance();
                    BinaryOp::AddAssign
                }
                TokenKind::MinusEq => {
                    self.advance();
                    BinaryOp::SubAssign
                }
                TokenKind::StarEq => {
                    self.advance();
                    BinaryOp::MulAssign
                }
                TokenKind::SlashEq => {
                    self.advance();
                    BinaryOp::DivAssign
                }
                TokenKind::PercentEq => {
                    self.advance();
                    BinaryOp::RemAssign
                }
                TokenKind::AmpersandEq => {
                    self.advance();
                    BinaryOp::AndAssign
                }
                TokenKind::PipeEq => {
                    self.advance();
                    BinaryOp::OrAssign
                }
                TokenKind::CaretEq => {
                    self.advance();
                    BinaryOp::XorAssign
                }
                TokenKind::LtLtEq => {
                    self.advance();
                    BinaryOp::ShlAssign
                }
                TokenKind::GtGtEq => {
                    self.advance();
                    BinaryOp::ShrAssign
                }
                TokenKind::DotDot => {
                    self.advance();
                    BinaryOp::Range
                }
                TokenKind::DotDotEq => {
                    self.advance();
                    BinaryOp::RangeInclusive
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
            TokenKind::EqEq | TokenKind::Ne => 3,
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => 4,
            TokenKind::Pipe => 5,
            TokenKind::Ampersand => 6,
            TokenKind::Caret => 7,
            TokenKind::LtLt | TokenKind::GtGt => 8,
            TokenKind::Plus | TokenKind::Minus => 9,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => 10,
            TokenKind::DotDot | TokenKind::DotDotEq => 0,
            TokenKind::As => 0,
            _ => 0,
        }
    }

    fn parse_prefix(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                let e = self.parse_prefix_guarded();
                Expr::Unary(
                    UnaryOp::Neg,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Exclamation => {
                self.advance();
                let e = self.parse_prefix_guarded();
                Expr::Unary(
                    UnaryOp::Not,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Ampersand => {
                self.advance();
                let e = self.parse_prefix_guarded();
                Expr::Unary(
                    UnaryOp::Ref,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Star => {
                self.advance();
                let e = self.parse_prefix_guarded();
                Expr::Unary(
                    UnaryOp::Deref,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Plus => {
                self.advance();
                let e = self.parse_prefix_guarded();
                Expr::Unary(
                    UnaryOp::Neg,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            TokenKind::Tilde => {
                self.advance();
                let e = self.parse_prefix_guarded();
                Expr::Unary(
                    UnaryOp::Not,
                    Box::new(e.clone()),
                    Span::new(start, e.span().end),
                )
            }
            _ => self.parse_primary(),
        }
    }
    fn parse_prefix_guarded(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Error(Span::ZERO);
        }
        let expr = self.parse_prefix();
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
            TokenKind::StringLit | TokenKind::RawStringLit => {
                let tok = self.advance();
                Expr::String(tok.value.clone(), tok.span)
            }
            TokenKind::CharLit => {
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
            TokenKind::Self_ => {
                let tok = self.advance();
                Expr::Ident("self".to_string(), tok.span)
            }
            TokenKind::Ident => {
                let tok = self.advance();
                let mut segments = vec![tok.value.clone()];
                while self.peek() == TokenKind::ColonColon {
                    self.advance();
                    if self.peek() == TokenKind::Ident {
                        segments.push(self.expect_ident());
                    } else if self.peek() == TokenKind::Self_ {
                        segments.push("self".to_string());
                        self.advance();
                    } else if self.peek() == TokenKind::Super {
                        segments.push("super".to_string());
                        self.advance();
                    } else if self.peek() == TokenKind::Crate {
                        segments.push("crate".to_string());
                        self.advance();
                    }
                }
                if segments.len() == 1 {
                    Expr::Ident(segments.into_iter().next().unwrap(), tok.span)
                } else {
                    Expr::Ident(segments.join("::"), Span::new(start, self.prev_end()))
                }
            }
            TokenKind::LParen => {
                self.advance();
                if self.peek() == TokenKind::RParen {
                    self.advance();
                    Expr::Tuple(Vec::new(), Span::new(start, self.prev_end()))
                } else {
                    let first = self.parse_expr();
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                        let mut items = vec![first];
                        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                            items.push(self.parse_expr());
                            if self.peek() == TokenKind::Comma {
                                self.advance();
                            }
                        }
                        self.expect(TokenKind::RParen);
                        Expr::Tuple(items, Span::new(start, self.prev_end()))
                    } else {
                        self.expect(TokenKind::RParen);
                        Expr::Paren(Box::new(first), Span::new(start, self.prev_end()))
                    }
                }
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
            TokenKind::If => return self.parse_if_expr(),
            TokenKind::Match => return self.parse_match_expr(),
            TokenKind::While => return self.parse_while_expr(),
            TokenKind::Loop => return self.parse_loop_expr(),
            TokenKind::For => return self.parse_for_expr(),
            TokenKind::Return => {
                let tok = self.advance();
                Expr::Return(None, tok.span)
            }
            TokenKind::Break => {
                let tok = self.advance();
                Expr::Break(None, tok.span)
            }
            TokenKind::Continue => {
                let tok = self.advance();
                Expr::Continue(tok.span)
            }
            TokenKind::LBrace => return self.parse_block_expr(),
            TokenKind::Async => return self.parse_async_expr(),
            TokenKind::Underscore => {
                let tok = self.advance();
                Expr::Ident("_".to_string(), tok.span)
            }
            _ => {
                let tok = self.advance();
                Expr::Ident(format!("{:?}", tok.kind), tok.span)
            }
        };

        loop {
            match self.peek() {
                TokenKind::LParen => {
                    let call_start = expr.span().start;
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                        args.push(self.parse_expr());
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RParen);
                    expr = Expr::Call(Box::new(expr), args, Span::new(call_start, self.prev_end()));
                }
                TokenKind::LBracket => {
                    let start = expr.span().start;
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
                    let start = expr.span().start;
                    self.advance();
                    if self.peek() == TokenKind::Ident {
                        let name = self.expect_ident();
                        if self.peek() == TokenKind::LParen {
                            // Method call
                            self.advance();
                            let mut args = Vec::new();
                            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof
                            {
                                args.push(self.parse_expr());
                                if self.peek() == TokenKind::Comma {
                                    self.advance();
                                }
                            }
                            self.expect(TokenKind::RParen);
                            expr = Expr::MethodCall(
                                Box::new(expr),
                                name,
                                args,
                                Span::new(start, self.prev_end()),
                            );
                        } else {
                            expr = Expr::Field(
                                Box::new(expr),
                                name,
                                Span::new(start, self.prev_end()),
                            );
                        }
                    } else {
                        break;
                    }
                }
                TokenKind::Question => {
                    self.advance();
                    // try operator
                    expr = Expr::Unary(
                        UnaryOp::Deref,
                        Box::new(expr.clone()),
                        Span::new(expr.span().start, self.prev_end()),
                    );
                }
                TokenKind::Exclamation => {
                    // Macro invocation, e.g. `vec![..]`, `println!(..)`, `foo! { .. }`.
                    let start = expr.span().start;
                    self.advance();
                    // Consume the delimited token group (the macro "arguments").
                    match self.peek() {
                        TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                            let open = self.peek();
                            let close = match open {
                                TokenKind::LParen => TokenKind::RParen,
                                TokenKind::LBracket => TokenKind::RBracket,
                                _ => TokenKind::RBrace,
                            };
                            self.advance();
                            let mut depth = 1usize;
                            while depth > 0 && self.peek() != TokenKind::Eof {
                                let k = self.peek();
                                if k == open {
                                    depth += 1;
                                } else if k == close {
                                    depth -= 1;
                                }
                                if depth == 0 {
                                    break;
                                }
                                self.advance();
                            }
                            if self.peek() == close {
                                self.advance();
                            }
                        }
                        _ => {}
                    }
                    // Represented as a call so fact extraction captures the callee name.
                    expr = Expr::Call(
                        Box::new(expr),
                        Vec::new(),
                        Span::new(start, self.prev_end()),
                    );
                }
                _ => break,
            }
        }
        expr
    }

    fn parse_if_expr(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        self.advance();
        let cond = self.parse_expr();
        let block = self.parse_block();
        let else_branch = if self.peek() == TokenKind::Else {
            self.advance();
            if self.peek() == TokenKind::If {
                Some(Box::new(self.parse_if_expr()))
            } else {
                Some(Box::new(Expr::Block(
                    Box::new(self.parse_block()),
                    Span::new(start, self.prev_end()),
                )))
            }
        } else {
            None
        };
        let span = Span::new(start, self.prev_end());
        Expr::If(Box::new(cond), Box::new(block), else_branch, span)
    }

    fn parse_match_expr(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        self.advance();
        let expr = self.parse_expr();
        self.expect(TokenKind::LBrace);
        let mut arms = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            let arm_start = self.peek_token().span.start;
            let patterns = vec![self.parse_pattern()];
            let guard = if self.peek() == TokenKind::If {
                self.advance();
                Some(Box::new(self.parse_expr()))
            } else {
                None
            };
            self.expect(TokenKind::FatArrow);
            let body = self.parse_expr();
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
            arms.push(MatchArm {
                patterns,
                guard,
                body: Box::new(body),
                span: Span::new(arm_start, self.prev_end()),
            });
        }
        self.expect(TokenKind::RBrace);
        Expr::Match(Box::new(expr), arms, Span::new(start, self.prev_end()))
    }

    fn parse_while_expr(&mut self) -> Expr {
        let tok = self.advance();
        let cond = self.parse_expr();
        let block = self.parse_block();
        Expr::While(Box::new(cond), Box::new(block), tok.span)
    }

    fn parse_loop_expr(&mut self) -> Expr {
        let tok = self.advance();
        let block = self.parse_block();
        Expr::Loop(Box::new(block), tok.span)
    }

    fn parse_for_expr(&mut self) -> Expr {
        let tok = self.advance();
        let pat = self.parse_pattern();
        self.expect(TokenKind::In);
        let expr = self.parse_expr();
        let block = self.parse_block();
        Expr::For(Box::new(pat), Box::new(expr), Box::new(block), tok.span)
    }

    fn parse_block_expr(&mut self) -> Expr {
        let block = self.parse_block();
        Expr::Block(Box::new(block.clone()), block.span)
    }

    fn parse_async_expr(&mut self) -> Expr {
        let tok = self.advance();
        let expr = self.parse_expr();
        Expr::Async(Box::new(expr), tok.span)
    }

    pub fn parse_block(&mut self) -> super::super::ast::stmt::Block {
        if self.bump_recursion().is_err() {
            return super::super::ast::stmt::Block {
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
        super::super::ast::stmt::Block {
            stmts,
            span: Span::new(start, end),
        }
    }

    pub fn parse_pattern(&mut self) -> Pattern {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Underscore => {
                self.advance();
                Pattern::Wildcard(Span::new(start, self.prev_end()))
            }
            TokenKind::Ref => {
                self.advance();
                let p = self.parse_pattern();
                Pattern::Ref(Box::new(p), false, Span::new(start, self.prev_end()))
            }
            TokenKind::Ampersand => {
                self.advance();
                let p = self.parse_pattern();
                Pattern::Ref(Box::new(p), false, Span::new(start, self.prev_end()))
            }
            TokenKind::Mut => {
                self.advance();
                let p = self.parse_pattern();
                Pattern::Ref(Box::new(p), true, Span::new(start, self.prev_end()))
            }
            TokenKind::IntLit | TokenKind::True | TokenKind::False => {
                let lit = self.parse_expr();
                Pattern::Lit(Box::new(lit), Span::new(start, self.prev_end()))
            }
            TokenKind::Self_ => {
                self.advance();
                Pattern::Ident("self".to_string(), Span::new(start, self.prev_end()))
            }
            _ => {
                let name = self.expect_ident();
                Pattern::Ident(name, Span::new(start, self.prev_end()))
            }
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Bool(_, s)
            | Expr::Int(_, s)
            | Expr::Float(_, s)
            | Expr::String(_, s)
            | Expr::Char(_, s)
            | Expr::Ident(_, s)
            | Expr::Tuple(_, s)
            | Expr::Array(_, s)
            | Expr::Closure(_, _, s)
            | Expr::Block(_, s)
            | Expr::If(_, _, _, s)
            | Expr::Match(_, _, s)
            | Expr::While(_, _, s)
            | Expr::Loop(_, s)
            | Expr::For(_, _, _, s)
            | Expr::Return(_, s)
            | Expr::Break(_, s)
            | Expr::Continue(s)
            | Expr::Paren(_, s)
            | Expr::Async(_, s)
            | Expr::Await(_, s)
            | Expr::Ref(_, _, s)
            | Expr::Deref(_, s)
            | Expr::Cast(_, _, s)
            | Expr::Error(s)
            | Expr::Path(_, s)
            | Expr::Struct(_, _, _, s) => *s,
            Expr::Binary(_, _, _, s)
            | Expr::Unary(_, _, s)
            | Expr::Call(_, _, s)
            | Expr::MethodCall(_, _, _, s)
            | Expr::Index(_, _, s)
            | Expr::Field(_, _, s) => *s,
        }
    }
}
