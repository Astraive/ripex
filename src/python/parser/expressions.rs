use super::super::ast::expr::*;
use super::super::ast::literal::Literal;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::Span;

impl Parser {
    pub fn parse_expr(&mut self) -> Expr {
        if self.bump_recursion().is_err() {
            return Expr::Error(Span::ZERO);
        }
        let result = self.parse_test();
        self.pop_recursion();
        result
    }

    pub fn parse_test(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut expr = self.parse_or_expr();
        if self.peek() == TokenKind::If {
            self.advance();
            let cond = self.parse_or_expr();
            self.expect(TokenKind::Else);
            let else_ = self.parse_test();
            expr = Expr::IfElse(
                Box::new(cond),
                Box::new(expr),
                Box::new(else_),
                Span::new(start, self.prev_end()),
            );
        }
        if self.peek() == TokenKind::Walrus {
            self.advance();
            let rhs = self.parse_test();
            expr = Expr::Walrus(
                Box::new(expr),
                Box::new(rhs),
                Span::new(start, self.prev_end()),
            );
        }
        expr
    }

    fn parse_or_expr(&mut self) -> Expr {
        self.parse_binary_op(&mut |p| p.parse_and_expr(), TokenKind::Or, BinaryOp::Or)
    }

    fn parse_and_expr(&mut self) -> Expr {
        self.parse_binary_op(&mut |p| p.parse_not_expr(), TokenKind::And, BinaryOp::And)
    }

    fn parse_not_expr(&mut self) -> Expr {
        if self.peek() == TokenKind::Not {
            let start = self.peek_token().span.start;
            self.advance();
            let expr = self.parse_not_expr();
            let end = expr.span().end;
            Expr::Unary(UnaryOp::Not, Box::new(expr), Span::new(start, end))
        } else {
            self.parse_comparison()
        }
    }

    fn parse_comparison(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut left = self.parse_binary(1);
        let mut ops = Vec::new();
        loop {
            let op = match self.peek() {
                TokenKind::Lt => {
                    self.advance();
                    CmpOp::Lt
                }
                TokenKind::Gt => {
                    self.advance();
                    CmpOp::Gt
                }
                TokenKind::EqEq => {
                    self.advance();
                    CmpOp::Eq
                }
                TokenKind::Ne => {
                    self.advance();
                    CmpOp::Ne
                }
                TokenKind::LtEq => {
                    self.advance();
                    CmpOp::Le
                }
                TokenKind::GtEq => {
                    self.advance();
                    CmpOp::Ge
                }
                TokenKind::In => {
                    self.advance();
                    CmpOp::In
                }
                TokenKind::Is if self.peek_ahead(1) == TokenKind::Not => {
                    self.advance();
                    self.advance();
                    CmpOp::IsNot
                }
                TokenKind::Is => {
                    self.advance();
                    CmpOp::Is
                }
                TokenKind::Not if self.peek_ahead(1) == TokenKind::In => {
                    self.advance();
                    self.advance();
                    CmpOp::NotIn
                }
                _ => break,
            };
            ops.push(op);
            let right = self.parse_binary(1);
            left = Expr::Compare(
                Box::new(left),
                vec![op],
                vec![Box::new(right)],
                Span::new(start, self.prev_end()),
            );
        }
        left
    }

    fn parse_binary(&mut self, min_prec: u8) -> Expr {
        let mut left = self.parse_term();
        loop {
            let prec = self.binary_prec();
            if prec < min_prec {
                break;
            }
            let span_start = left.span().start;
            let op = match self.peek() {
                TokenKind::Pipe => {
                    self.advance();
                    BinaryOp::BitOr
                }
                TokenKind::Caret => {
                    self.advance();
                    BinaryOp::BitXor
                }
                TokenKind::Ampersand => {
                    self.advance();
                    BinaryOp::BitAnd
                }
                TokenKind::LtLt => {
                    self.advance();
                    BinaryOp::Shl
                }
                TokenKind::GtGt => {
                    self.advance();
                    BinaryOp::Shr
                }
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
                TokenKind::At => {
                    self.advance();
                    BinaryOp::MatMult
                }
                TokenKind::Slash => {
                    self.advance();
                    BinaryOp::Div
                }
                TokenKind::SlashSlash => {
                    self.advance();
                    BinaryOp::FloorDiv
                }
                TokenKind::Percent => {
                    self.advance();
                    BinaryOp::Mod
                }
                TokenKind::StarStar => {
                    self.advance();
                    BinaryOp::Pow
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

    fn binary_prec(&self) -> u8 {
        match self.peek() {
            TokenKind::Pipe => 1,
            TokenKind::Caret => 2,
            TokenKind::Ampersand => 3,
            TokenKind::LtLt | TokenKind::GtGt => 4,
            TokenKind::Plus | TokenKind::Minus => 5,
            TokenKind::Star
            | TokenKind::At
            | TokenKind::Slash
            | TokenKind::SlashSlash
            | TokenKind::Percent => 6,
            TokenKind::StarStar => 7,
            _ => 0,
        }
    }

    fn parse_term(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Plus => {
                self.advance();
                let e = self.parse_term();
                let end = e.span().end;
                Expr::Unary(UnaryOp::Pos, Box::new(e), Span::new(start, end))
            }
            TokenKind::Minus => {
                self.advance();
                let e = self.parse_term();
                let end = e.span().end;
                Expr::Unary(UnaryOp::Neg, Box::new(e), Span::new(start, end))
            }
            TokenKind::Tilde => {
                self.advance();
                let e = self.parse_term();
                let end = e.span().end;
                Expr::Unary(UnaryOp::Invert, Box::new(e), Span::new(start, end))
            }
            TokenKind::Star => {
                self.advance();
                let e = self.parse_term();
                Expr::Starred(Box::new(e), Span::new(start, self.prev_end()))
            }
            TokenKind::Await => {
                self.advance();
                let e = self.parse_term();
                Expr::Await(Box::new(e), Span::new(start, self.prev_end()))
            }
            _ => self.parse_power(),
        }
    }

    fn parse_power(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut expr = self.parse_await_primary();
        if self.peek() == TokenKind::StarStar {
            self.advance();
            let right = self.parse_power();
            expr = Expr::Binary(
                Box::new(expr),
                BinaryOp::Pow,
                Box::new(right),
                Span::new(start, self.prev_end()),
            );
        }
        expr
    }

    fn parse_await_primary(&mut self) -> Expr {
        if self.peek() == TokenKind::Await {
            let start = self.peek_token().span.start;
            self.advance();
            let expr = self.parse_power();
            let end = expr.span().end;
            Expr::Await(Box::new(expr), Span::new(start, end))
        } else {
            self.parse_trailer()
        }
    }

    fn parse_trailer(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut expr = self.parse_primary();

        loop {
            match self.peek() {
                TokenKind::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    let mut keywords = Vec::new();
                    while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                        if self.peek() == TokenKind::Star {
                            self.advance();
                            let argument = self.parse_expr();
                            let argument = if self.peek() == TokenKind::For {
                                self.advance();
                                let target = Box::new(self.parse_primary());
                                self.expect(TokenKind::In);
                                let iter = Box::new(self.parse_primary());
                                let span = Span::new(argument.span().start, self.prev_end());
                                Expr::Generator(
                                    Box::new(argument),
                                    vec![Comprehension {
                                        target,
                                        iter,
                                        ifs: Vec::new(),
                                        is_async: false,
                                        span,
                                    }],
                                    span,
                                )
                            } else {
                                argument
                            };
                            args.push(Expr::Starred(Box::new(argument), self.peek_token().span));
                        } else if self.peek() == TokenKind::StarStar {
                            self.advance();
                            let val = self.parse_expr();
                            keywords.push(Keyword {
                                name: None,
                                value: val,
                                span: self.peek_token().span,
                            });
                        } else if self.peek() == TokenKind::Ident
                            && self.peek_ahead(1) == TokenKind::Eq
                        {
                            let name = self.expect_ident();
                            self.advance();
                            let val = self.parse_expr();
                            keywords.push(Keyword {
                                name: Some(name),
                                value: val,
                                span: self.peek_token().span,
                            });
                        } else {
                            let argument = self.parse_expr();
                            if self.peek() == TokenKind::For {
                                self.advance();
                                let target = Box::new(self.parse_primary());
                                self.expect(TokenKind::In);
                                let iter = Box::new(self.parse_primary());
                                let mut ifs = Vec::new();
                                while self.peek() == TokenKind::If {
                                    self.advance();
                                    ifs.push(Box::new(self.parse_expr()));
                                }
                                let span = Span::new(argument.span().start, self.prev_end());
                                args.push(Expr::Generator(
                                    Box::new(argument),
                                    vec![Comprehension {
                                        target,
                                        iter,
                                        ifs,
                                        is_async: false,
                                        span,
                                    }],
                                    span,
                                ));
                            } else {
                                args.push(argument);
                            }
                        }
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RParen);
                    expr = Expr::Call(
                        Box::new(expr),
                        args,
                        keywords,
                        Span::new(start, self.prev_end()),
                    );
                }
                TokenKind::LBracket => {
                    self.advance();
                    let mut have_slice = false;
                    let mut lower = None;
                    let mut upper = None;
                    let mut step = None;

                    if self.peek() != TokenKind::RBracket && self.peek() != TokenKind::Colon {
                        let first = self.parse_expr();
                        if self.peek() == TokenKind::Colon {
                            have_slice = true;
                            lower = Some(Box::new(first));
                        } else {
                            self.expect(TokenKind::RBracket);
                            expr = Expr::Subscript(
                                Box::new(expr),
                                Box::new(first),
                                Span::new(start, self.prev_end()),
                            );
                            continue;
                        }
                    } else if self.peek() == TokenKind::Colon {
                        have_slice = true;
                        self.advance();
                    }

                    if have_slice {
                        if self.peek() != TokenKind::RBracket && self.peek() != TokenKind::Colon {
                            upper = Some(Box::new(self.parse_expr()));
                        }
                        if self.peek() == TokenKind::Colon {
                            self.advance();
                            if self.peek() != TokenKind::RBracket {
                                step = Some(Box::new(self.parse_expr()));
                            }
                        }
                        self.expect(TokenKind::RBracket);
                        expr = Expr::Slice(lower, upper, step, Span::new(start, self.prev_end()));
                        continue;
                    }

                    self.expect(TokenKind::RBracket);
                    expr = Expr::Subscript(
                        Box::new(expr),
                        Box::new(Expr::Ident(String::new(), Span::ZERO)),
                        Span::new(start, self.prev_end()),
                    );
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.expect_ident();
                    expr = Expr::Attribute(Box::new(expr), name, Span::new(start, self.prev_end()));
                }
                _ => break,
            }
        }

        expr
    }

    fn parse_binary_op<F>(&mut self, sub: &mut F, token: TokenKind, op: BinaryOp) -> Expr
    where
        F: FnMut(&mut Parser) -> Expr,
    {
        let start = self.peek_token().span.start;
        let mut left = sub(self);
        while self.peek() == token {
            self.advance();
            let right = sub(self);
            left = Expr::Binary(
                Box::new(left),
                op,
                Box::new(right),
                Span::new(start, self.prev_end()),
            );
        }
        left
    }

    pub(crate) fn parse_primary(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::False => {
                self.advance();
                Expr::Literal(
                    Literal::Boolean(false, Span::new(start, self.prev_end())),
                    Span::new(start, self.prev_end()),
                )
            }
            TokenKind::None_ => {
                self.advance();
                Expr::Literal(
                    Literal::None_(Span::new(start, self.prev_end())),
                    Span::new(start, self.prev_end()),
                )
            }
            TokenKind::True => {
                self.advance();
                Expr::Literal(
                    Literal::Boolean(true, Span::new(start, self.prev_end())),
                    Span::new(start, self.prev_end()),
                )
            }
            TokenKind::IntLit => {
                let tok = self.advance();
                let val = tok.value.parse::<i64>().unwrap_or(0);
                Expr::Literal(Literal::Int(val, tok.value.clone(), tok.span), tok.span)
            }
            TokenKind::FloatLit => {
                let tok = self.advance();
                let val = tok.value.parse::<f64>().unwrap_or(0.0);
                Expr::Literal(Literal::Float(val, tok.value.clone(), tok.span), tok.span)
            }
            TokenKind::ComplexLit => {
                let tok = self.advance();
                Expr::Literal(
                    Literal::Complex {
                        real: 0.0,
                        imag: 1.0,
                        text: tok.value.clone(),
                        span: tok.span,
                    },
                    tok.span,
                )
            }
            TokenKind::StringLit
            | TokenKind::BytesLit
            | TokenKind::FStringLit
            | TokenKind::FStringHead
            | TokenKind::FStringMid
            | TokenKind::FStringTail => {
                let tok = self.advance();
                Expr::Literal(
                    Literal::String(tok.value.clone(), String::new(), tok.span),
                    tok.span,
                )
            }
            TokenKind::Ellipsis => {
                self.advance();
                Expr::Ellipsis(Span::new(start, self.prev_end()))
            }
            TokenKind::Ident | TokenKind::Self_ | TokenKind::Type => {
                let name = self.expect_ident();
                Expr::Ident(name, Span::new(start, self.prev_end()))
            }
            TokenKind::DotDotDot => {
                self.advance();
                Expr::Ellipsis(Span::new(start, self.prev_end()))
            }
            TokenKind::LParen => {
                self.advance();
                if self.peek() == TokenKind::RParen {
                    self.advance();
                    Expr::Tuple(Vec::new(), Span::new(start, self.prev_end()))
                } else {
                    let first = self.parse_expr();
                    if self.peek() == TokenKind::For {
                        self.advance();
                        let target = Box::new(self.parse_primary());
                        self.expect(TokenKind::In);
                        let iter = Box::new(self.parse_primary());
                        let mut ifs = Vec::new();
                        while self.peek() == TokenKind::If {
                            self.advance();
                            ifs.push(Box::new(self.parse_expr()));
                        }
                        self.expect(TokenKind::RParen);
                        let span = Span::new(start, self.prev_end());
                        Expr::Generator(
                            Box::new(first),
                            vec![Comprehension {
                                target,
                                iter,
                                ifs,
                                is_async: false,
                                span,
                            }],
                            span,
                        )
                    } else if self.peek() == TokenKind::Comma {
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
                if self.peek() == TokenKind::RBracket {
                    self.advance();
                    return Expr::List(Vec::new(), Span::new(start, self.prev_end()));
                }
                let first = self.parse_expr();
                // Could be list, list comprehension, or subscript
                if self.peek() == TokenKind::Comma {
                    self.advance();
                    let mut items = vec![first];
                    while self.peek() != TokenKind::RBracket && self.peek() != TokenKind::Eof {
                        items.push(self.parse_expr());
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RBracket);
                    Expr::List(items, Span::new(start, self.prev_end()))
                } else if self.peek() == TokenKind::For || self.peek() == TokenKind::Async {
                    // List comprehension
                    let mut generators = Vec::new();
                    let is_async = if self.peek() == TokenKind::Async {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    self.expect(TokenKind::For);
                    let target = Box::new(self.parse_primary());
                    self.expect(TokenKind::In);
                    let iter = Box::new(self.parse_primary());
                    let mut ifs = Vec::new();
                    while self.peek() == TokenKind::If {
                        self.advance();
                        ifs.push(Box::new(self.parse_expr()));
                    }
                    generators.push(Comprehension {
                        target,
                        iter,
                        ifs,
                        is_async,
                        span: self.peek_token().span,
                    });
                    self.expect(TokenKind::RBracket);
                    Expr::ListComp(
                        Box::new(first),
                        generators,
                        Span::new(start, self.prev_end()),
                    )
                } else {
                    self.expect(TokenKind::RBracket);
                    Expr::List(vec![first], Span::new(start, self.prev_end()))
                }
            }
            TokenKind::LBrace => {
                self.advance();
                if self.peek() == TokenKind::RBrace {
                    self.advance();
                    return Expr::Dict(Vec::new(), Span::new(start, self.prev_end()));
                }
                let first = self.parse_expr();
                if self.peek() == TokenKind::Colon {
                    // Dict
                    self.advance();
                    let second = self.parse_expr();
                    let mut items = vec![(first, second)];
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        if self.peek() == TokenKind::RBrace {
                            break;
                        }
                        let k = self.parse_expr();
                        self.expect(TokenKind::Colon);
                        let v = self.parse_expr();
                        items.push((k, v));
                    }
                    self.expect(TokenKind::RBrace);
                    Expr::Dict(items, Span::new(start, self.prev_end()))
                } else {
                    // Set
                    let mut items = vec![first];
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        if self.peek() == TokenKind::RBrace {
                            break;
                        }
                        items.push(self.parse_expr());
                    }
                    self.expect(TokenKind::RBrace);
                    Expr::Set(items, Span::new(start, self.prev_end()))
                }
            }
            TokenKind::Lambda => {
                self.advance();
                let mut params = Vec::new();
                if self.peek() != TokenKind::Colon {
                    params.push(self.expect_ident());
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        params.push(self.expect_ident());
                    }
                }
                self.expect(TokenKind::Colon);
                let body = self.parse_expr();
                Expr::Lambda(params, Box::new(body), Span::new(start, self.prev_end()))
            }
            TokenKind::Yield => {
                self.advance();
                let expr = if self.peek() != TokenKind::Newline
                    && self.peek() != TokenKind::RParen
                    && self.peek() != TokenKind::RBrace
                    && self.peek() != TokenKind::RBracket
                    && self.peek() != TokenKind::Colon
                    && self.peek() != TokenKind::Comma
                    && self.peek() != TokenKind::Eof
                {
                    if self.peek() == TokenKind::From {
                        self.advance();
                        let e = self.parse_expr();
                        return Expr::YieldFrom(Box::new(e), Span::new(start, self.prev_end()));
                    }
                    Some(Box::new(self.parse_expr()))
                } else {
                    None
                };
                Expr::Yield(expr, Span::new(start, self.prev_end()))
            }
            _ => {
                let tok = self.advance();
                Expr::Ident(format!("{:?}", tok.kind), tok.span)
            }
        }
    }

    pub fn parse_expr_stmt(&mut self) -> Expr {
        let start = self.peek_token().span.start;
        let mut expr = self.parse_test();

        // Check for assignment
        if self.peek() == TokenKind::Eq {
            self.advance();
            let rhs = self.parse_test();
            expr = Expr::Walrus(
                Box::new(expr),
                Box::new(rhs),
                Span::new(start, self.prev_end()),
            );
        }

        expr
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal(_, s)
            | Expr::Ident(_, s)
            | Expr::List(_, s)
            | Expr::Tuple(_, s)
            | Expr::Dict(_, s)
            | Expr::Set(_, s)
            | Expr::Lambda(_, _, s)
            | Expr::Ellipsis(s)
            | Expr::Error(s)
            | Expr::Paren(_, s) => *s,
            Expr::Attribute(_, _, s)
            | Expr::Subscript(_, _, s)
            | Expr::Slice(_, _, _, s)
            | Expr::Call(_, _, _, s)
            | Expr::Binary(_, _, _, s)
            | Expr::Unary(_, _, s)
            | Expr::IfElse(_, _, _, s)
            | Expr::ListComp(_, _, s)
            | Expr::SetComp(_, _, s)
            | Expr::DictComp(_, _, s)
            | Expr::Generator(_, _, s)
            | Expr::Await(_, s)
            | Expr::Yield(_, s)
            | Expr::YieldFrom(_, s)
            | Expr::Starred(_, s)
            | Expr::Walrus(_, _, s)
            | Expr::FString(_, s)
            | Expr::Compare(_, _, _, s)
            | Expr::Match(_, _, s) => *s,
        }
    }
}
