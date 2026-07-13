use super::super::ast::decl::*;
use super::super::ast::expr::Expr;
use super::super::ast::pattern::*;
use super::super::ast::stmt::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::Span;

impl Parser {
    pub fn parse_stmt(&mut self) -> Stmt {
        if self.bump_recursion().is_err() {
            return Stmt::Empty(Span::ZERO);
        }
        let start = self.peek_token().span.start;

        if self.peek() == TokenKind::At {
            let decorators = self.parse_decorators();
            let result = self.parse_decorated(decorators);
            self.pop_recursion();
            return result;
        }

        let result = match self.peek() {
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Try => self.parse_try_stmt(),
            TokenKind::With => self.parse_with_stmt(),
            TokenKind::Match => self.parse_match_stmt(),
            TokenKind::Def | TokenKind::Async
                if self.peek_ahead(1) == TokenKind::Def || self.peek() == TokenKind::Def =>
            {
                self.parse_func_def_stmt()
            }
            TokenKind::Async => {
                self.advance();
                let stmt = self.parse_func_def_stmt();
                Stmt::Async(Box::new(stmt), Span::new(start, self.prev_end()))
            }
            TokenKind::Class => self.parse_class_def_stmt(),
            TokenKind::Return => {
                let tok = self.advance();
                let expr = if self.peek() != TokenKind::Newline
                    && self.peek() != TokenKind::Eof
                    && self.peek() != TokenKind::Dedent
                {
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect_newline();
                Stmt::Return(expr, tok.span)
            }
            TokenKind::Yield => {
                let expr = self.parse_expr_stmt();
                self.expect_newline();
                Stmt::Yield(Some(expr), Span::new(start, self.prev_end()))
            }
            TokenKind::Raise => {
                let tok = self.advance();
                let expr = if self.peek() != TokenKind::Newline && self.peek() != TokenKind::Eof {
                    Some(self.parse_expr())
                } else {
                    None
                };
                let cause = if self.peek() == TokenKind::From {
                    self.advance();
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect_newline();
                Stmt::Raise(expr, cause, tok.span)
            }
            TokenKind::Assert => {
                let tok = self.advance();
                let test = self.parse_expr();
                let msg = if self.peek() == TokenKind::Comma {
                    self.advance();
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect_newline();
                Stmt::Assert(test, msg, tok.span)
            }
            TokenKind::Break => {
                let tok = self.advance();
                self.expect_newline();
                Stmt::Break(tok.span)
            }
            TokenKind::Continue => {
                let tok = self.advance();
                self.expect_newline();
                Stmt::Continue(tok.span)
            }
            TokenKind::Pass => {
                let tok = self.advance();
                self.expect_newline();
                Stmt::Pass(tok.span)
            }
            TokenKind::Del => {
                let tok = self.advance();
                let expr = self.parse_expr();
                self.expect_newline();
                Stmt::Delete(expr, tok.span)
            }
            TokenKind::Global => {
                let tok = self.advance();
                let mut names = vec![self.expect_ident()];
                while self.peek() == TokenKind::Comma {
                    self.advance();
                    names.push(self.expect_ident());
                }
                self.expect_newline();
                Stmt::Global(names, tok.span)
            }
            TokenKind::Nonlocal => {
                let tok = self.advance();
                let mut names = vec![self.expect_ident()];
                while self.peek() == TokenKind::Comma {
                    self.advance();
                    names.push(self.expect_ident());
                }
                self.expect_newline();
                Stmt::Nonlocal(names, tok.span)
            }
            TokenKind::Import => self.parse_import_stmt(),
            TokenKind::From => self.parse_from_import_stmt(),
            TokenKind::Newline => {
                self.advance();
                Stmt::Empty(Span::ZERO)
            }
            TokenKind::Dedent => {
                self.advance();
                Stmt::Empty(Span::ZERO)
            }
            _ => self.parse_simple_stmt(),
        };
        self.pop_recursion();
        result
    }

    fn parse_simple_stmt(&mut self) -> Stmt {
        let start = self.peek_token().span.start;
        // Parse expression or assignment
        let first_lhs = self.parse_test();
        let lhs = if self.peek() == TokenKind::Comma {
            let mut items = vec![first_lhs];
            while self.peek() == TokenKind::Comma {
                self.advance();
                if self.peek() == TokenKind::Eq {
                    break;
                }
                items.push(self.parse_test());
            }
            Expr::Tuple(items, Span::new(start, self.prev_end()))
        } else {
            first_lhs
        };

        if self.peek() == TokenKind::Eq {
            // Assignment (e.g. `x = 1`). Chained assignment (`x = y = z`) is
            // left to error recovery: the extra `= expr` is not consumed here,
            // and the resilient parser records an error and continues.
            self.advance();
            let rhs = self.parse_test();
            let span = Span::new(start, self.prev_end());
            let stmt = if self.peek() == TokenKind::Colon {
                // AnnAssign after the fact - not typical
                Stmt::AnnAssign(Box::new(lhs), Box::new(rhs), None, span)
            } else {
                Stmt::Assign(Box::new(lhs), Box::new(rhs), span)
            };
            if self.peek() == TokenKind::Semicolon {
                self.advance();
            }
            self.expect_newline();
            stmt
        } else if self.peek() == TokenKind::PlusEq
            || self.peek() == TokenKind::MinusEq
            || self.peek() == TokenKind::StarEq
            || self.peek() == TokenKind::SlashEq
            || self.peek() == TokenKind::SlashSlashEq
            || self.peek() == TokenKind::PercentEq
            || self.peek() == TokenKind::StarStarEq
            || self.peek() == TokenKind::AtEq
            || self.peek() == TokenKind::Ampersand
            || self.peek() == TokenKind::Pipe
            || self.peek() == TokenKind::Caret
            || self.peek() == TokenKind::LtLt
            || self.peek() == TokenKind::GtGt
        {
            let op_token = self.advance();
            let op = match op_token.kind {
                TokenKind::PlusEq => super::super::ast::expr::BinaryOp::Add,
                TokenKind::MinusEq => super::super::ast::expr::BinaryOp::Sub,
                TokenKind::StarEq => super::super::ast::expr::BinaryOp::Mul,
                TokenKind::SlashEq => super::super::ast::expr::BinaryOp::Div,
                TokenKind::SlashSlashEq => super::super::ast::expr::BinaryOp::FloorDiv,
                TokenKind::PercentEq => super::super::ast::expr::BinaryOp::Mod,
                TokenKind::StarStarEq => super::super::ast::expr::BinaryOp::Pow,
                TokenKind::AtEq => super::super::ast::expr::BinaryOp::MatMult,
                _ => super::super::ast::expr::BinaryOp::Add,
            };
            let rhs = self.parse_test();
            if self.peek() == TokenKind::Semicolon {
                self.advance();
            }
            self.expect_newline();
            Stmt::AugAssign(
                Box::new(lhs),
                op,
                Box::new(rhs),
                Span::new(start, self.prev_end()),
            )
        } else if self.peek() == TokenKind::Colon {
            // AnnAssign
            self.advance();
            let type_ann = self.parse_test();
            let rhs = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(Box::new(self.parse_test()))
            } else {
                None
            };
            if self.peek() == TokenKind::Semicolon {
                self.advance();
            }
            self.expect_newline();
            Stmt::AnnAssign(
                Box::new(lhs),
                Box::new(type_ann),
                rhs,
                Span::new(start, self.prev_end()),
            )
        } else {
            if self.peek() == TokenKind::Semicolon {
                self.advance();
            }
            self.expect_newline();
            Stmt::Expr(lhs, Span::new(start, self.prev_end()))
        }
    }

    fn parse_if_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        let cond = self.parse_expr();
        self.expect(TokenKind::Colon);
        self.expect_newline();
        let body = self.parse_block();
        let mut elif_else = Vec::new();
        while self.peek() == TokenKind::Elif {
            self.advance();
            let _elif_cond = self.parse_expr();
            self.expect(TokenKind::Colon);
            self.expect_newline();
            let elif_body = self.parse_block();
            elif_else.extend(elif_body);
        }
        if self.peek() == TokenKind::Else {
            self.advance();
            self.expect(TokenKind::Colon);
            self.expect_newline();
            let else_body = self.parse_block();
            elif_else.extend(else_body);
        }
        Stmt::If(Box::new(cond), body, elif_else, tok.span)
    }

    fn parse_while_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        let cond = self.parse_expr();
        self.expect(TokenKind::Colon);
        self.expect_newline();
        let body = self.parse_block();
        let else_body = if self.peek() == TokenKind::Else {
            self.advance();
            self.expect(TokenKind::Colon);
            self.expect_newline();
            Some(self.parse_block())
        } else {
            None
        };
        Stmt::While(Box::new(cond), body, else_body, tok.span)
    }

    fn parse_for_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        let target = self.parse_primary();
        self.expect(TokenKind::In);
        let iter = self.parse_expr();
        self.expect(TokenKind::Colon);
        self.expect_newline();
        let body = self.parse_block();
        let else_body = if self.peek() == TokenKind::Else {
            self.advance();
            self.expect(TokenKind::Colon);
            self.expect_newline();
            Some(self.parse_block())
        } else {
            None
        };
        Stmt::For(Box::new(target), Box::new(iter), body, else_body, tok.span)
    }

    fn parse_try_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        self.expect(TokenKind::Colon);
        self.expect_newline();
        let body = self.parse_block();
        let mut handlers = Vec::new();
        while self.peek() == TokenKind::Except {
            self.advance();
            let type_ = if self.peek() != TokenKind::Colon {
                Some(Box::new(self.parse_expr()))
            } else {
                None
            };
            let name = if self.peek() == TokenKind::As {
                self.advance();
                Some(self.expect_ident())
            } else {
                None
            };
            self.expect(TokenKind::Colon);
            self.expect_newline();
            let handler_body = self.parse_block();
            handlers.push(ExceptHandler {
                type_,
                name,
                body: handler_body,
                span: self.peek_token().span,
            });
        }
        let else_body = if self.peek() == TokenKind::Else {
            self.advance();
            self.expect(TokenKind::Colon);
            self.expect_newline();
            Some(self.parse_block())
        } else {
            None
        };
        let finally_body = if self.peek() == TokenKind::Finally {
            self.advance();
            self.expect(TokenKind::Colon);
            self.expect_newline();
            Some(self.parse_block())
        } else {
            None
        };
        Stmt::Try(body, handlers, else_body, finally_body, tok.span)
    }

    fn parse_with_stmt(&mut self) -> Stmt {
        let start = self.peek_token().span.start;
        self.advance();
        let mut items = Vec::new();
        loop {
            let context = Box::new(self.parse_expr());
            let target = if self.peek() == TokenKind::As {
                self.advance();
                Some(Box::new(self.parse_expr()))
            } else {
                None
            };
            items.push(WithItem {
                context,
                target,
                span: self.peek_token().span,
            });
            if self.peek() != TokenKind::Comma {
                break;
            }
            self.advance();
        }
        self.expect(TokenKind::Colon);
        self.expect_newline();
        let body = self.parse_block();
        Stmt::With(items, body, Span::new(start, self.prev_end()))
    }

    fn parse_match_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        let subject = self.parse_expr();
        self.expect(TokenKind::Colon);
        self.expect_newline();
        self.expect_indent();
        let mut cases = Vec::new();
        while self.peek() == TokenKind::Case {
            self.advance();
            let pattern = self.parse_pattern();
            let guard = if self.peek() == TokenKind::If {
                self.advance();
                Some(Box::new(self.parse_expr()))
            } else {
                None
            };
            self.expect(TokenKind::Colon);
            self.expect_newline();
            let body = self.parse_block();
            cases.push(MatchCase {
                pattern: Box::new(pattern),
                guard,
                body,
                span: self.peek_token().span,
            });
        }
        self.expect_dedent();
        Stmt::Match(Box::new(subject), cases, tok.span)
    }

    fn parse_func_def_stmt(&mut self) -> Stmt {
        let start = self.peek_token().span.start;
        let is_async = if self.peek() == TokenKind::Async {
            self.advance();
            true
        } else {
            false
        };
        self.expect(TokenKind::Def);
        let name = self.expect_ident();
        self.expect(TokenKind::LParen);
        let mut args = Vec::new();
        let mut defaults = Vec::new();
        let mut vararg = None;
        let mut kwarg = None;
        let kw_defaults = Vec::new();

        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            let _pos_before = self.pos;
            if self.peek() == TokenKind::Star {
                if self.peek_ahead(1) == TokenKind::Comma {
                    self.advance();
                    self.advance();
                    continue;
                }
                self.advance();
                let arg_name = self.expect_ident();
                let type_ann = if self.peek() == TokenKind::Colon {
                    self.advance();
                    Some(Box::new(self.parse_expr()))
                } else {
                    None
                };
                vararg = Some(Box::new(Arg {
                    name: arg_name,
                    type_ann,
                    span: self.peek_token().span,
                }));
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
                continue;
            }
            if self.peek() == TokenKind::StarStar {
                self.advance();
                let arg_name = self.expect_ident();
                let type_ann = if self.peek() == TokenKind::Colon {
                    self.advance();
                    Some(Box::new(self.parse_expr()))
                } else {
                    None
                };
                kwarg = Some(Box::new(Arg {
                    name: arg_name,
                    type_ann,
                    span: self.peek_token().span,
                }));
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
                continue;
            }
            let arg_name = self.expect_ident();
            let type_ann = if self.peek() == TokenKind::Colon {
                self.advance();
                Some(Box::new(self.parse_expr()))
            } else {
                None
            };
            let has_eq = self.peek() == TokenKind::Eq;
            if has_eq {
                self.advance();
                defaults.push(self.parse_expr());
            }
            args.push(Arg {
                name: arg_name,
                type_ann,
                span: self.peek_token().span,
            });
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        let returns = if self.peek() == TokenKind::Arrow {
            self.advance();
            Some(Box::new(self.parse_expr()))
        } else {
            None
        };
        self.expect(TokenKind::Colon);
        self.expect_newline();
        let body = self.parse_block();
        Stmt::FuncDef(
            FuncDef {
                name,
                args,
                body,
                decorators: Vec::new(),
                returns,
                is_async,
                is_generator: false,
                defaults,
                kw_defaults,
                vararg,
                kwarg,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_class_def_stmt(&mut self) -> Stmt {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let mut bases = Vec::new();
        let mut keywords = Vec::new();
        if self.peek() == TokenKind::LParen {
            self.advance();
            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                if self.peek() == TokenKind::Ident && self.peek_ahead(1) == TokenKind::Eq {
                    let kw = self.expect_ident();
                    self.advance();
                    let val = self.parse_expr();
                    keywords.push(super::super::ast::expr::Keyword {
                        name: Some(kw),
                        value: val,
                        span: self.peek_token().span,
                    });
                } else {
                    bases.push(self.parse_expr());
                }
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RParen);
        }
        self.expect(TokenKind::Colon);
        self.expect_newline();
        let body = self.parse_block();
        Stmt::ClassDef(
            ClassDef {
                name,
                bases,
                keywords,
                body,
                decorators: Vec::new(),
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_import_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        let mut names = Vec::new();
        loop {
            let name = self.parse_dotted_name();
            let asname = if self.peek() == TokenKind::As {
                self.advance();
                Some(self.expect_ident())
            } else {
                None
            };
            names.push(Alias {
                name,
                asname,
                span: self.peek_token().span,
            });
            if self.peek() != TokenKind::Comma {
                break;
            }
            self.advance();
        }
        self.expect_newline();
        Stmt::Import(names, tok.span)
    }

    fn parse_from_import_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        let mut level = 0usize;
        while self.peek() == TokenKind::Dot {
            level += 1;
            self.advance();
        }
        let module = if self.peek() != TokenKind::Import {
            Some(self.parse_dotted_name())
        } else {
            None
        };
        self.expect(TokenKind::Import);
        let mut names = Vec::new();
        if self.peek() == TokenKind::LParen {
            self.advance();
            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                if self.peek() == TokenKind::Star {
                    self.advance();
                    names.push(Alias {
                        name: "*".to_string(),
                        asname: None,
                        span: self.peek_token().span,
                    });
                } else {
                    let name = self.expect_ident();
                    let asname = if self.peek() == TokenKind::As {
                        self.advance();
                        Some(self.expect_ident())
                    } else {
                        None
                    };
                    names.push(Alias {
                        name,
                        asname,
                        span: self.peek_token().span,
                    });
                }
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RParen);
        } else {
            loop {
                if self.peek() == TokenKind::Star {
                    self.advance();
                    names.push(Alias {
                        name: "*".to_string(),
                        asname: None,
                        span: self.peek_token().span,
                    });
                } else {
                    let name = self.expect_ident();
                    let asname = if self.peek() == TokenKind::As {
                        self.advance();
                        Some(self.expect_ident())
                    } else {
                        None
                    };
                    names.push(Alias {
                        name,
                        asname,
                        span: self.peek_token().span,
                    });
                }
                if self.peek() != TokenKind::Comma {
                    break;
                }
                self.advance();
            }
        }
        self.expect_newline();
        Stmt::ImportFrom(module, names, level, tok.span)
    }

    fn parse_dotted_name(&mut self) -> String {
        let mut name = self.expect_ident();
        while self.peek() == TokenKind::Dot {
            self.advance();
            name = format!("{}.{}", name, self.expect_ident());
        }
        name
    }

    fn parse_decorators(&mut self) -> Vec<Expr> {
        let mut decorators = Vec::new();
        while self.peek() == TokenKind::At {
            self.advance();
            let start = self.peek_token().span.start;
            let name = self.expect_ident();
            let mut expr = Expr::Ident(name, Span::new(start, self.prev_end()));
            while self.peek() == TokenKind::Dot {
                self.advance();
                let attr = self.expect_ident();
                expr = Expr::Attribute(Box::new(expr), attr, Span::new(start, self.prev_end()));
            }
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
                expr = Expr::Call(
                    Box::new(expr),
                    args,
                    Vec::new(),
                    Span::new(start, self.prev_end()),
                );
            }
            decorators.push(expr);
            self.expect_newline();
        }
        decorators
    }

    fn parse_decorated(&mut self, decorators: Vec<Expr>) -> Stmt {
        let _start = self.peek_token().span.start;
        let mut stmt = self.parse_stmt();
        match &mut stmt {
            Stmt::FuncDef(func, _) => func.decorators = decorators,
            Stmt::ClassDef(class, _) => class.decorators = decorators,
            _ => {}
        }
        stmt
    }

    pub fn parse_block(&mut self) -> Vec<Stmt> {
        if self.bump_recursion().is_err() {
            return Vec::new();
        }
        let mut stmts = Vec::new();
        self.expect_indent();
        while self.peek() != TokenKind::Dedent && self.peek() != TokenKind::Eof {
            if self.peek() == TokenKind::Newline {
                self.advance();
                continue;
            }
            let pos_before = self.pos;
            stmts.push(self.parse_stmt());
            if self.pos == pos_before {
                self.advance();
            }
        }
        self.expect_dedent();
        self.pop_recursion();
        stmts
    }

    fn parse_pattern(&mut self) -> Pattern {
        let start = self.peek_token().span.start;
        match self.peek() {
            TokenKind::Ident if self.peek_ahead(1) == TokenKind::Dot => {
                let mut name = self.expect_ident();
                while self.peek() == TokenKind::Dot {
                    self.advance();
                    name = format!("{}.{}", name, self.expect_ident());
                }
                Pattern::Value(name, Span::new(start, self.prev_end()))
            }
            TokenKind::Ident if self.peek_ahead(1) == TokenKind::LParen => {
                let name = self.expect_ident();
                self.advance();
                let mut args = Vec::new();
                let mut kwargs = Vec::new();
                while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                    if self.peek() == TokenKind::Ident && self.peek_ahead(1) == TokenKind::Eq {
                        let kw = self.expect_ident();
                        self.advance();
                        let val = self.parse_pattern();
                        kwargs.push((kw, val));
                    } else {
                        args.push(self.parse_pattern());
                    }
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RParen);
                Pattern::Class(name, args, kwargs, Span::new(start, self.prev_end()))
            }
            TokenKind::IntLit
            | TokenKind::FloatLit
            | TokenKind::True
            | TokenKind::False
            | TokenKind::None_
            | TokenKind::StringLit => {
                let lit = self.parse_expr();
                Pattern::Literal(Box::new(lit), Span::new(start, self.prev_end()))
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                while self.peek() != TokenKind::RBracket && self.peek() != TokenKind::Eof {
                    items.push(self.parse_pattern());
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBracket);
                Pattern::Sequence(items, Span::new(start, self.prev_end()))
            }
            TokenKind::LBrace => {
                self.advance();
                let mut items = Vec::new();
                let mut rest = None;
                while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                    if self.peek() == TokenKind::StarStar {
                        self.advance();
                        let name = self.expect_ident();
                        rest = Some(Box::new(Pattern::Capture(
                            name,
                            Span::new(start, self.prev_end()),
                        )));
                    } else {
                        let key = self.parse_pattern();
                        self.expect(TokenKind::Colon);
                        let val = self.parse_pattern();
                        items.push((key, val));
                    }
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace);
                Pattern::Mapping(items, rest, Span::new(start, self.prev_end()))
            }
            TokenKind::Underscore => {
                self.advance();
                Pattern::Wildcard(Span::new(start, self.prev_end()))
            }
            TokenKind::Ident => {
                let name = self.expect_ident();
                if self.peek() == TokenKind::As {
                    self.advance();
                    let inner = self.parse_pattern();
                    Pattern::As(Box::new(inner), name, Span::new(start, self.prev_end()))
                } else {
                    Pattern::Capture(name, Span::new(start, self.prev_end()))
                }
            }
            TokenKind::LParen => {
                self.advance();
                let first = self.parse_pattern();
                if self.peek() == TokenKind::Comma {
                    self.advance();
                    let mut items = vec![first];
                    while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                        items.push(self.parse_pattern());
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RParen);
                    Pattern::Sequence(items, Span::new(start, self.prev_end()))
                } else {
                    self.expect(TokenKind::RParen);
                    Pattern::Group(Box::new(first), Span::new(start, self.prev_end()))
                }
            }
            _ => {
                self.advance();
                Pattern::Wildcard(Span::new(start, self.prev_end()))
            }
        }
    }
}
