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
        let start = self.peek_token().span.start;
        let result: Stmt = match self.peek() {
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::Do => self.parse_do_stmt(),
            TokenKind::Switch => self.parse_switch_stmt(),
            TokenKind::Return => {
                let tok = self.advance();
                let expr = if self.peek() != TokenKind::Semicolon {
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon);
                Stmt::Return(expr, tok.span)
            }
            TokenKind::Break => {
                let tok = self.advance();
                self.expect(TokenKind::Semicolon);
                Stmt::Break(tok.span)
            }
            TokenKind::Continue => {
                let tok = self.advance();
                self.expect(TokenKind::Semicolon);
                Stmt::Continue(tok.span)
            }
            TokenKind::Goto => {
                let tok = self.advance();
                let label = self.expect_ident();
                self.expect(TokenKind::Semicolon);
                Stmt::Goto(label, tok.span)
            }
            TokenKind::LBrace => {
                let block = self.parse_block();
                let span = block.span;
                Stmt::Block(block, span)
            }
            TokenKind::Semicolon => {
                self.advance();
                Stmt::Empty(Span::ZERO)
            }
            TokenKind::Newline => {
                self.advance();
                Stmt::Empty(Span::ZERO)
            }
            TokenKind::Hash => {
                let directive = self.advance();
                let name = directive.value;
                let span_start = start;
                let directive = match name.as_str() {
                    "include" => {
                        let path = self
                            .parse_preproc_path()
                            .trim()
                            .trim_start_matches('<')
                            .trim_end_matches('>')
                            .trim_matches('"')
                            .to_string();
                        self.finish_preprocessor_line();
                        PreprocDirective::Include(path, Span::new(span_start, self.prev_end()))
                    }
                    "define" => {
                        let macro_name = if self.peek() == TokenKind::Ident {
                            self.expect_ident()
                        } else {
                            String::new()
                        };
                        let body = self.parse_preproc_text();
                        PreprocDirective::Define(
                            macro_name,
                            (!body.is_empty()).then_some(body),
                            Span::new(span_start, self.prev_end()),
                        )
                    }
                    "undef" => {
                        let symbol = self.parse_preproc_text();
                        PreprocDirective::Undef(symbol, Span::new(span_start, self.prev_end()))
                    }
                    "ifdef" => {
                        let symbol = self.parse_preproc_text();
                        PreprocDirective::Ifdef(symbol, Span::new(span_start, self.prev_end()))
                    }
                    "ifndef" => {
                        let symbol = self.parse_preproc_text();
                        PreprocDirective::Ifndef(symbol, Span::new(span_start, self.prev_end()))
                    }
                    "if" => {
                        let expr = self.parse_preproc_text();
                        PreprocDirective::If(expr, Span::new(span_start, self.prev_end()))
                    }
                    "elif" => {
                        let expr = self.parse_preproc_text();
                        PreprocDirective::Elif(expr, Span::new(span_start, self.prev_end()))
                    }
                    "else" => {
                        self.finish_preprocessor_line();
                        PreprocDirective::Else(Span::new(span_start, self.prev_end()))
                    }
                    "endif" => {
                        self.finish_preprocessor_line();
                        PreprocDirective::Endif(Span::new(span_start, self.prev_end()))
                    }
                    "error" => {
                        let message = self.parse_preproc_text();
                        PreprocDirective::Error(message, Span::new(span_start, self.prev_end()))
                    }
                    "pragma" => {
                        let text = self.parse_preproc_text();
                        PreprocDirective::Pragma(text, Span::new(span_start, self.prev_end()))
                    }
                    "line" => {
                        let text = self.parse_preproc_text();
                        PreprocDirective::Line(text, Span::new(span_start, self.prev_end()))
                    }
                    other => {
                        let text = self.parse_preproc_text();
                        let detail = if text.is_empty() {
                            other.to_string()
                        } else {
                            format!("{other} {text}")
                        };
                        PreprocDirective::Error(detail, Span::new(span_start, self.prev_end()))
                    }
                };
                Stmt::Preprocessor(directive, Span::new(span_start, self.prev_end()))
            }
            TokenKind::Int
            | TokenKind::Char
            | TokenKind::Short
            | TokenKind::Long
            | TokenKind::Float
            | TokenKind::Double
            | TokenKind::Signed
            | TokenKind::Unsigned
            | TokenKind::Void
            | TokenKind::Bool
            | TokenKind::Complex
            | TokenKind::Struct
            | TokenKind::Union
            | TokenKind::Enum
            | TokenKind::Const
            | TokenKind::Volatile
            | TokenKind::Extern
            | TokenKind::Static
            | TokenKind::Register
            | TokenKind::Auto
            | TokenKind::Inline
            | TokenKind::Typedef
            | TokenKind::Restrict
            | TokenKind::ThreadLocal => {
                // Type spec - could be function or variable declaration
                self.parse_declaration()
            }
            _ => {
                if self.peek() == TokenKind::Ident
                    && matches!(self.peek_ahead(1), TokenKind::Ident | TokenKind::Star)
                {
                    let declaration = self.parse_declaration();
                    self.pop_recursion();
                    return declaration;
                }
                let expr = self.parse_expr();
                if self.peek() == TokenKind::LParen && matches!(&expr, Expr::Ident(_, _)) {
                    // Function call that looks like a declaration
                    self.pos -= 1; // Push back ident
                    self.parse_declaration()
                } else if self.peek() == TokenKind::Eq
                    || self.peek() == TokenKind::PlusEq
                    || self.peek() == TokenKind::MinusEq
                    || self.peek() == TokenKind::StarEq
                    || self.peek() == TokenKind::SlashEq
                    || self.peek() == TokenKind::PercentEq
                {
                    self.advance();
                    let _val = self.parse_expr();
                    self.expect(TokenKind::Semicolon);
                    Stmt::VarDecl(
                        VarDecl {
                            type_: Box::new(Expr::Ident("int".to_string(), Span::ZERO)),
                            name: String::new(),
                            init: None,
                            is_const: false,
                            storage_class: None,
                            span: Span::ZERO,
                        },
                        Span::new(start, self.prev_end()),
                    )
                } else {
                    self.expect(TokenKind::Semicolon);
                    let span = expr.span();
                    Stmt::Expr(expr, span)
                }
            }
        };
        self.pop_recursion();
        result
    }

    fn parse_declaration(&mut self) -> Stmt {
        let start = self.peek_token().span.start;
        let mut storage_class = None;
        let mut type_qual = Vec::new();
        let mut is_const = false;

        loop {
            match self.peek() {
                TokenKind::Extern => {
                    storage_class = Some("extern".to_string());
                    self.advance();
                }
                TokenKind::Static => {
                    storage_class = Some("static".to_string());
                    self.advance();
                }
                TokenKind::Auto => {
                    storage_class = Some("auto".to_string());
                    self.advance();
                }
                TokenKind::Register => {
                    storage_class = Some("register".to_string());
                    self.advance();
                }
                TokenKind::Typedef => {
                    storage_class = Some("typedef".to_string());
                    self.advance();
                }
                TokenKind::ThreadLocal => {
                    storage_class = Some("_Thread_local".to_string());
                    self.advance();
                }
                TokenKind::Const => {
                    is_const = true;
                    self.advance();
                }
                TokenKind::Volatile => {
                    type_qual.push("volatile".to_string());
                    self.advance();
                }
                TokenKind::Restrict => {
                    type_qual.push("restrict".to_string());
                    self.advance();
                }
                TokenKind::Inline => {
                    self.advance();
                }
                _ => break,
            }
        }

        let base_type = self.parse_type();
        // A declaration may omit the declarator entirely when it is a bare
        // tag definition, e.g. `struct Point { int x; };` or `enum E { A, B };`.
        let name = if self.peek() == TokenKind::Ident {
            self.expect_ident()
        } else {
            String::new()
        };

        // Function-pointer declarator, e.g. `typedef int (*binary_op)(int, int)`.
        if name.is_empty()
            && self.peek() == TokenKind::LParen
            && self.peek_ahead(1) == TokenKind::Star
        {
            self.advance();
            self.advance();
            let pointer_name = self.expect_ident();
            self.expect(TokenKind::RParen);
            if self.peek() == TokenKind::LParen {
                let mut depth = 0usize;
                while self.peek() != TokenKind::Eof {
                    match self.peek() {
                        TokenKind::LParen => depth += 1,
                        TokenKind::RParen => {
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
            self.expect(TokenKind::Semicolon);
            return Stmt::VarDecl(
                VarDecl {
                    type_: Box::new(base_type),
                    name: pointer_name,
                    init: None,
                    is_const,
                    storage_class,
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            );
        }

        // Check if function declaration
        if self.peek() == TokenKind::LParen {
            // Function
            self.advance();
            let mut params = Vec::new();
            if self.peek() == TokenKind::Void && self.peek_ahead(1) == TokenKind::RParen {
                self.advance();
            } else if self.peek() != TokenKind::RParen {
                while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                    let ptype = self.parse_type();
                    let pname =
                        if self.peek() == TokenKind::Comma || self.peek() == TokenKind::RParen {
                            None
                        } else {
                            Some(self.expect_ident())
                        };
                    params.push(ParamDecl {
                        type_: Box::new(ptype),
                        name: pname,
                        span: Span::new(start, self.prev_end()),
                    });
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
            }
            self.expect(TokenKind::RParen);

            let body = if self.peek() == TokenKind::LBrace {
                Some(self.parse_block())
            } else {
                self.expect(TokenKind::Semicolon);
                None
            };

            Stmt::Decl(
                FuncDecl {
                    name,
                    return_type: Box::new(base_type),
                    params,
                    is_variadic: false,
                    is_knr: false,
                    body,
                    storage_class,
                    is_inline: false,
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            )
        } else {
            // Variable declaration
            while self.peek() == TokenKind::LBracket {
                self.advance();
                if self.peek() != TokenKind::RBracket {
                    self.parse_expr();
                }
                self.expect(TokenKind::RBracket);
            }
            let init = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(self.parse_expr())
            } else {
                None
            };
            self.expect(TokenKind::Semicolon);

            Stmt::VarDecl(
                VarDecl {
                    type_: Box::new(base_type),
                    name,
                    init,
                    is_const,
                    storage_class,
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            )
        }
    }

    fn parse_preproc_path(&mut self) -> String {
        if self.peek() == TokenKind::StringLit {
            return self.advance().value;
        }

        let mut path = String::new();
        while self.peek() != TokenKind::Newline && self.peek() != TokenKind::Eof {
            let token = self.advance();
            let text = match token.kind {
                TokenKind::Lt => "<",
                TokenKind::Gt => ">",
                TokenKind::Slash => "/",
                TokenKind::Dot => ".",
                _ => token.value.as_str(),
            };
            path.push_str(text);
        }
        path
    }

    fn parse_preproc_text(&mut self) -> String {
        let mut text = String::new();
        while self.peek() != TokenKind::Newline && self.peek() != TokenKind::Eof {
            let token = self.advance();
            let piece = match token.kind {
                TokenKind::Lt => "<",
                TokenKind::Gt => ">",
                TokenKind::Slash => "/",
                TokenKind::Dot => ".",
                TokenKind::Comma => ",",
                TokenKind::Colon => ":",
                TokenKind::Plus => "+",
                TokenKind::Minus => "-",
                TokenKind::Star => "*",
                TokenKind::Eq => "=",
                _ => token.value.as_str(),
            };
            if !piece.is_empty() {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.push_str(piece);
            }
        }
        if self.peek() == TokenKind::Newline {
            self.advance();
        }
        text
    }

    fn finish_preprocessor_line(&mut self) {
        while self.peek() != TokenKind::Newline && self.peek() != TokenKind::Eof {
            self.advance();
        }
        if self.peek() == TokenKind::Newline {
            self.advance();
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

    fn parse_if_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        self.expect(TokenKind::LParen);
        let cond = self.parse_expr();
        self.expect(TokenKind::RParen);
        let body = Box::new(self.parse_stmt());
        let else_branch = if self.peek() == TokenKind::Else {
            self.advance();
            Some(Box::new(self.parse_stmt()))
        } else {
            None
        };
        Stmt::If(cond, body, else_branch, tok.span)
    }

    fn parse_while_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        self.expect(TokenKind::LParen);
        let cond = self.parse_expr();
        self.expect(TokenKind::RParen);
        let body = Box::new(self.parse_stmt());
        Stmt::While(cond, body, tok.span)
    }

    fn parse_for_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        self.expect(TokenKind::LParen);
        let init = if self.peek() != TokenKind::Semicolon {
            Some(Box::new(self.parse_stmt()))
        } else {
            self.advance();
            None
        };
        let cond = if self.peek() != TokenKind::Semicolon {
            Some(self.parse_expr())
        } else {
            None
        };
        self.expect(TokenKind::Semicolon);
        let post = if self.peek() != TokenKind::RParen {
            Some(Box::new(self.parse_expr_stmt()))
        } else {
            None
        };
        self.expect(TokenKind::RParen);
        let body = Box::new(self.parse_stmt());
        Stmt::For(init, cond, post, body, tok.span)
    }

    fn parse_expr_stmt(&mut self) -> Stmt {
        let expr = self.parse_expr();
        Stmt::Expr(expr.clone(), expr.span())
    }

    fn parse_do_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        let body = Box::new(self.parse_stmt());
        self.expect(TokenKind::While);
        self.expect(TokenKind::LParen);
        let cond = self.parse_expr();
        self.expect(TokenKind::RParen);
        self.expect(TokenKind::Semicolon);
        Stmt::Do(body, cond, tok.span)
    }

    fn parse_switch_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        self.expect(TokenKind::LParen);
        let expr = self.parse_expr();
        self.expect(TokenKind::RParen);
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
                    stmts: body,
                    span: Span::new(tok.span.start, self.prev_end()),
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
                    stmts: body,
                    span: Span::new(tok.span.start, self.prev_end()),
                });
            } else {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace);
        Stmt::Switch(expr, cases, tok.span)
    }

    pub fn parse_compilation_unit(&mut self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        while self.peek() != TokenKind::Eof {
            let pos_before = self.pos;
            stmts.push(self.parse_stmt_recovery());
            if self.pos == pos_before {
                self.advance();
            }
        }
        stmts
    }
}
