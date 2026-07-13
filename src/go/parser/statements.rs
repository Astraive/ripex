use super::super::ast::expr::Expr;
use super::super::ast::stmt::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::Span;

impl Parser {
    pub fn parse_stmt(&mut self) -> Stmt {
        if self.bump_recursion().is_err() {
            return Stmt::Empty(Span::ZERO);
        }
        let result = match self.peek() {
            TokenKind::Var => self.parse_var_decl(),
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::Type => self.parse_type_decl(),
            TokenKind::Func => self.parse_func_decl(),
            TokenKind::Import => self.parse_import_decl(),
            TokenKind::Package => self.parse_package(),
            TokenKind::Return => {
                let tok = self.advance();
                let mut exprs = Vec::new();
                if self.peek() != TokenKind::Semicolon
                    && self.peek() != TokenKind::RBrace
                    && self.peek() != TokenKind::Newline
                    && self.peek() != TokenKind::Eof
                {
                    exprs.push(self.parse_expr());
                }
                self.expect_semicolon();
                Stmt::Return(exprs, tok.span)
            }
            TokenKind::If => self.parse_if(),
            TokenKind::For => self.parse_for(),
            TokenKind::Switch => self.parse_switch(),
            TokenKind::Select => self.parse_select(),
            TokenKind::Break => {
                let tok = self.advance();
                let label = if self.peek() == TokenKind::Ident {
                    Some(self.expect_ident())
                } else {
                    None
                };
                self.expect_semicolon();
                Stmt::Break(label, tok.span)
            }
            TokenKind::Continue => {
                let tok = self.advance();
                let label = if self.peek() == TokenKind::Ident {
                    Some(self.expect_ident())
                } else {
                    None
                };
                self.expect_semicolon();
                Stmt::Continue(label, tok.span)
            }
            TokenKind::Defer => {
                let tok = self.advance();
                let expr = self.parse_expr();
                self.expect_semicolon();
                Stmt::Defer(expr, tok.span)
            }
            TokenKind::Go => {
                let tok = self.advance();
                let expr = self.parse_expr();
                self.expect_semicolon();
                Stmt::Go(expr, tok.span)
            }
            TokenKind::LBrace => {
                let block = self.parse_block();
                Stmt::Block(block.clone(), block.span)
            }
            TokenKind::Semicolon | TokenKind::Newline => {
                self.advance();
                Stmt::Empty(Span::ZERO)
            }
            TokenKind::Fallthrough => {
                let tok = self.advance();
                self.expect_semicolon();
                Stmt::Fallthrough(tok.span)
            }
            TokenKind::Goto => {
                let tok = self.advance();
                let label = self.expect_ident();
                self.expect_semicolon();
                Stmt::Goto(label, tok.span)
            }
            TokenKind::Arrow => {
                // Send statement
                self.advance();
                let val = self.parse_expr();
                self.expect_semicolon();
                Stmt::Send(Expr::Ident(String::new(), Span::ZERO), val, Span::ZERO)
            }
            _ => {
                let expr = self.parse_expr();
                if self.peek() == TokenKind::Define {
                    // Short variable declaration
                    self.advance();
                    let val = self.parse_expr();
                    self.expect_semicolon();
                    Stmt::Assign(
                        vec![expr.clone()],
                        vec![val],
                        Span::new(expr.span().start, self.prev_end()),
                    )
                } else if self.peek() == TokenKind::Eq
                    || self.peek() == TokenKind::PlusEq
                    || self.peek() == TokenKind::MinusEq
                    || self.peek() == TokenKind::StarEq
                    || self.peek() == TokenKind::SlashEq
                    || self.peek() == TokenKind::PercentEq
                    || self.peek() == TokenKind::AmpersandEq
                    || self.peek() == TokenKind::PipeEq
                    || self.peek() == TokenKind::CaretEq
                    || self.peek() == TokenKind::LtLtEq
                    || self.peek() == TokenKind::GtGtEq
                {
                    self.advance();
                    let val = self.parse_expr();
                    self.expect_semicolon();
                    Stmt::Assign(
                        vec![expr.clone()],
                        vec![val],
                        Span::new(expr.span().start, self.prev_end()),
                    )
                } else {
                    self.expect_semicolon();
                    Stmt::Expr(expr.clone(), expr.span())
                }
            }
        };
        self.pop_recursion();
        result
    }

    fn parse_var_decl(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        let mut names = Vec::new();
        names.push(self.expect_ident());
        while self.peek() == TokenKind::Comma {
            self.advance();
            names.push(self.expect_ident());
        }
        let mut kind = None;
        if self.peek() == TokenKind::Ident
            || self.peek() == TokenKind::Star
            || self.peek() == TokenKind::LBracket
            || self.peek() == TokenKind::Map
            || self.peek() == TokenKind::Chan
            || self.peek() == TokenKind::Func
            || self.peek() == TokenKind::Interface
            || self.peek() == TokenKind::Struct
        {
            kind = Some(Box::new(self.parse_type()));
        }
        let mut values = Vec::new();
        if self.peek() == TokenKind::Eq {
            self.advance();
            values.push(self.parse_expr());
            while self.peek() == TokenKind::Comma {
                self.advance();
                values.push(self.parse_expr());
            }
        }
        self.expect_semicolon();
        Stmt::Decl(
            Decl::Var(
                VarDecl {
                    names,
                    kind,
                    values,
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            ),
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_const_decl(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        if self.peek() == TokenKind::LParen {
            self.advance();
            let mut names = Vec::new();
            let mut values = Vec::new();
            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                if matches!(self.peek(), TokenKind::Semicolon | TokenKind::Newline) {
                    self.advance();
                    continue;
                }
                names.push(self.expect_ident());
                if is_type_start(self.peek()) {
                    let _ = self.parse_type();
                }
                if self.peek() == TokenKind::Eq {
                    self.advance();
                    values.push(self.parse_expr());
                }
                self.expect_semicolon();
            }
            self.expect(TokenKind::RParen);
            let span = Span::new(start, self.prev_end());
            return Stmt::Decl(
                Decl::Const(
                    ConstDecl {
                        names,
                        kind: None,
                        values,
                        span,
                    },
                    span,
                ),
                span,
            );
        }
        let name = self.expect_ident();
        let mut kind = None;
        if self.peek() == TokenKind::Ident
            || self.peek() == TokenKind::Star
            || self.peek() == TokenKind::LBracket
            || self.peek() == TokenKind::Map
            || self.peek() == TokenKind::Chan
            || self.peek() == TokenKind::Func
        {
            kind = Some(Box::new(self.parse_type()));
        }
        let mut values = Vec::new();
        if self.peek() == TokenKind::Eq {
            self.advance();
            values.push(self.parse_expr());
        }
        self.expect_semicolon();
        Stmt::Decl(
            Decl::Const(
                ConstDecl {
                    names: vec![name],
                    kind,
                    values,
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            ),
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_type_decl(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        let name = self.expect_ident();
        let kind = self.parse_type();
        self.expect_semicolon();
        Stmt::Decl(
            Decl::Type(
                TypeDecl {
                    name,
                    kind: Box::new(kind),
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            ),
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_func_decl(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        let receiver = if self.peek() == TokenKind::LParen {
            self.parse_receiver()
        } else {
            None
        };
        let name = self.expect_ident();
        if self.peek() == TokenKind::LBracket {
            self.skip_type_parameters();
        }
        self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            let mut names = vec![self.expect_ident()];
            while self.peek() == TokenKind::Comma
                && self.peek_ahead(1) == TokenKind::Ident
                && is_type_start(self.peek_ahead(2))
            {
                self.advance();
                names.push(self.expect_ident());
            }
            let ptype = self.parse_type();
            for pname in names {
                params.push((pname, Box::new(ptype.clone())));
            }
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        let mut returns = Vec::new();
        if self.peek() != TokenKind::LBrace && self.peek() != TokenKind::Semicolon {
            returns.push(Box::new(self.parse_type()));
        }
        let body = if self.peek() == TokenKind::LBrace {
            Some(self.parse_block())
        } else {
            self.expect_semicolon();
            None
        };
        let end = self.prev_end();
        Stmt::Decl(
            Decl::Func(
                FuncDecl {
                    name,
                    receiver,
                    params,
                    returns,
                    body,
                    span: Span::new(start, end),
                },
                Span::new(start, end),
            ),
            Span::new(start, end),
        )
    }

    fn parse_import_decl(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        let mut imports = Vec::new();
        if self.peek() == TokenKind::LParen {
            self.advance();
            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                if self.peek() == TokenKind::Semicolon || self.peek() == TokenKind::Newline {
                    self.advance();
                    continue;
                }
                let alias =
                    if self.peek() == TokenKind::Ident && self.peek_ahead(1) == TokenKind::String {
                        Some(self.expect_ident())
                    } else {
                        None
                    };
                let path = self.advance().value.clone();
                imports.push(ImportDecl {
                    path,
                    alias,
                    span: Span::new(start, self.prev_end()),
                });
                if self.peek() == TokenKind::Semicolon || self.peek() == TokenKind::Newline {
                    self.advance();
                }
            }
            self.expect(TokenKind::RParen);
        } else {
            let alias =
                if self.peek() == TokenKind::Ident && self.peek_ahead(1) == TokenKind::String {
                    Some(self.expect_ident())
                } else {
                    None
                };
            let path = self.advance().value.clone();
            imports.push(ImportDecl {
                path,
                alias,
                span: Span::new(start, self.prev_end()),
            });
        }
        Stmt::Decl(
            Decl::Import(
                ImportDecl {
                    path: String::new(),
                    alias: None,
                    span: Span::ZERO,
                },
                Span::new(start, self.prev_end()),
            ),
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_package(&mut self) -> Stmt {
        let tok = self.advance();
        let name = self.expect_ident();
        Stmt::Decl(Decl::Package(name, tok.span), tok.span)
    }

    fn parse_if(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        let parenthesized = self.peek() == TokenKind::LParen;
        if parenthesized {
            self.advance();
        }
        let cond = self.parse_expr();
        if parenthesized {
            self.expect(TokenKind::RParen);
        }
        let body = Box::new(self.parse_stmt());
        let else_branch = if self.peek() == TokenKind::Else {
            self.advance();
            Some(Box::new(self.parse_stmt()))
        } else {
            None
        };
        Stmt::If(cond, body, else_branch, Span::new(start, self.prev_end()))
    }

    fn parse_for(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        if self.peek() == TokenKind::Range {
            self.advance();
            let val = self.parse_expr();
            let body = Box::new(self.parse_stmt());
            return Stmt::ForRange(
                val,
                String::new(),
                None,
                body,
                Span::new(start, self.prev_end()),
            );
        }

        if self.peek() == TokenKind::Ident {
            let checkpoint = self.pos;
            let first = self.expect_ident();
            let second = if self.peek() == TokenKind::Comma {
                self.advance();
                Some(self.expect_ident())
            } else {
                None
            };
            if self.peek() == TokenKind::Define && self.peek_ahead(1) == TokenKind::Range {
                self.advance();
                self.advance();
                let val = self.parse_expr();
                let body = Box::new(self.parse_stmt());
                return Stmt::ForRange(val, first, second, body, Span::new(start, self.prev_end()));
            }
            self.pos = checkpoint;
        }

        let parenthesized = self.peek() == TokenKind::LParen;
        if parenthesized {
            self.advance();
        }
        let init = if self.peek() != TokenKind::Semicolon && self.peek() != TokenKind::RParen {
            Some(Box::new(self.parse_stmt()))
        } else {
            None
        };
        self.expect(TokenKind::Semicolon);
        let cond = if self.peek() != TokenKind::Semicolon && self.peek() != TokenKind::RParen {
            Some(self.parse_expr())
        } else {
            None
        };
        self.expect(TokenKind::Semicolon);
        let post = if self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            Some(Box::new(self.parse_stmt()))
        } else {
            None
        };
        if parenthesized {
            self.expect(TokenKind::RParen);
        }
        let body = Box::new(self.parse_stmt());
        Stmt::For(init, cond, post, body, Span::new(start, self.prev_end()))
    }

    fn skip_type_parameters(&mut self) {
        self.expect(TokenKind::LBracket);
        let mut depth = 1usize;
        while depth > 0 && self.peek() != TokenKind::Eof {
            match self.advance().kind {
                TokenKind::LBracket => depth += 1,
                TokenKind::RBracket => depth -= 1,
                _ => {}
            }
        }
    }

    fn parse_switch(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        let expr = if self.peek() != TokenKind::LBrace {
            self.expect(TokenKind::LParen);
            let e = self.parse_expr();
            self.expect(TokenKind::RParen);
            Some(e)
        } else {
            None
        };
        self.expect(TokenKind::LBrace);
        let mut cases = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            if self.peek() == TokenKind::Case {
                self.advance();
                let case_expr = self.parse_expr();
                self.expect(TokenKind::Colon);
                let mut body = Vec::new();
                while self.peek() != TokenKind::Case
                    && self.peek() != TokenKind::Default
                    && self.peek() != TokenKind::RBrace
                    && self.peek() != TokenKind::Eof
                {
                    body.push(self.parse_stmt());
                }
                cases.push(CaseClause {
                    expr: Some(case_expr),
                    body,
                    span: Span::new(start, self.prev_end()),
                });
            } else if self.peek() == TokenKind::Default {
                self.advance();
                self.expect(TokenKind::Colon);
                let mut body = Vec::new();
                while self.peek() != TokenKind::Case
                    && self.peek() != TokenKind::Default
                    && self.peek() != TokenKind::RBrace
                    && self.peek() != TokenKind::Eof
                {
                    body.push(self.parse_stmt());
                }
                cases.push(CaseClause {
                    expr: None,
                    body,
                    span: Span::new(start, self.prev_end()),
                });
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBrace);
        Stmt::Switch(expr, cases, Span::new(start, self.prev_end()))
    }

    fn parse_select(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        self.expect(TokenKind::LBrace);
        let mut cases = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            if self.peek() == TokenKind::Case {
                self.advance();
                let expr = self.parse_expr();
                self.expect(TokenKind::Colon);
                let mut body = Vec::new();
                while self.peek() != TokenKind::Case
                    && self.peek() != TokenKind::Default
                    && self.peek() != TokenKind::RBrace
                    && self.peek() != TokenKind::Eof
                {
                    body.push(self.parse_stmt());
                }
                cases.push(CaseClause {
                    expr: Some(expr),
                    body,
                    span: Span::new(start, self.prev_end()),
                });
            } else if self.peek() == TokenKind::Default {
                self.advance();
                self.expect(TokenKind::Colon);
                let mut body = Vec::new();
                while self.peek() != TokenKind::Case
                    && self.peek() != TokenKind::Default
                    && self.peek() != TokenKind::RBrace
                    && self.peek() != TokenKind::Eof
                {
                    body.push(self.parse_stmt());
                }
                cases.push(CaseClause {
                    expr: None,
                    body,
                    span: Span::new(start, self.prev_end()),
                });
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBrace);
        Stmt::Select(cases, Span::new(start, self.prev_end()))
    }
}

fn is_type_start(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Ident
            | TokenKind::Star
            | TokenKind::LBracket
            | TokenKind::Map
            | TokenKind::Chan
            | TokenKind::Func
            | TokenKind::Interface
            | TokenKind::Struct
            | TokenKind::DotDotDot
            | TokenKind::String
            | TokenKind::Int
            | TokenKind::Int8
            | TokenKind::Int16
            | TokenKind::Int32
            | TokenKind::Int64
            | TokenKind::Uint
            | TokenKind::Uint8
            | TokenKind::Uint16
            | TokenKind::Uint32
            | TokenKind::Uint64
            | TokenKind::Float32
            | TokenKind::Float64
            | TokenKind::Bool
            | TokenKind::Byte
            | TokenKind::Rune
            | TokenKind::Any
    )
}
