use super::super::ast::expr::{Expr, ParamDecl};
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
        let result = match self.peek() {
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
            TokenKind::Try => self.parse_try(),
            TokenKind::Throw => {
                let tok = self.advance();
                let expr = if self.peek() != TokenKind::Semicolon {
                    Some(self.parse_expr())
                } else {
                    None
                };
                self.expect(TokenKind::Semicolon);
                Stmt::Throw(expr, tok.span)
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
            TokenKind::Hash => {
                let directive = self.advance();
                let raw = directive.value.trim();
                if let Some(rest) = raw.strip_prefix("#include") {
                    let path = rest
                        .trim()
                        .trim_start_matches('<')
                        .trim_end_matches('>')
                        .trim_matches('"')
                        .to_string();
                    let span = Span::new(start, directive.span.end);
                    if !path.is_empty() {
                        Stmt::Decl(Decl::Using(path, span), span)
                    } else {
                        Stmt::Empty(span)
                    }
                } else {
                    // Non-include directives are consumed as a single lexer
                    // token; the AST has no directive node, so do not invent
                    // declaration/import facts for them.
                    Stmt::Empty(directive.span)
                }
            }
            TokenKind::Newline => {
                self.advance();
                Stmt::Empty(Span::ZERO)
            }
            TokenKind::Namespace => {
                let ns = self.parse_namespace();
                Stmt::Decl(ns, Span::new(start, self.prev_end()))
            }
            TokenKind::Using => {
                let d = self.parse_using();
                Stmt::Decl(d, Span::new(start, self.prev_end()))
            }
            TokenKind::Template => {
                let d = self.parse_template();
                Stmt::Decl(d, Span::new(start, self.prev_end()))
            }
            TokenKind::Class | TokenKind::Struct => {
                let d = self.parse_class_or_struct();
                Stmt::Decl(d, Span::new(start, self.prev_end()))
            }
            TokenKind::Enum => {
                let d = self.parse_enum();
                Stmt::Decl(d, Span::new(start, self.prev_end()))
            }
            TokenKind::Typedef => {
                let d = self.parse_typedef();
                Stmt::Decl(d, Span::new(start, self.prev_end()))
            }
            TokenKind::Extern if self.peek_ahead(1) == TokenKind::String => {
                self.advance();
                let _ = self.advance();
                let d = self.parse_block();
                let span = d.span;
                Stmt::Block(d, span)
            }
            TokenKind::StaticAssert => {
                let tok = self.advance();
                self.expect(TokenKind::LParen);
                let _e = self.parse_expr();
                if self.peek() == TokenKind::Comma {
                    self.advance();
                    self.expect_ident();
                }
                self.expect(TokenKind::RParen);
                self.expect(TokenKind::Semicolon);
                Stmt::Expr(Expr::Bool(true, Span::ZERO), tok.span)
            }
            _ if self.is_type_spec_start(self.peek()) => self.parse_declaration(),
            _ => {
                if self.looks_like_named_type_declaration() {
                    let declaration = self.parse_declaration();
                    self.pop_recursion();
                    return declaration;
                }
                let expr = self.parse_expr();
                if self.peek() == TokenKind::LParen && matches!(&expr, Expr::Ident(_, _)) {
                    self.pos -= 1;
                    self.parse_declaration()
                } else if matches!(
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
                    let val = self.parse_expr();
                    self.expect(TokenKind::Semicolon);
                    Stmt::Expr(
                        Expr::Assign(
                            Box::new(expr),
                            Box::new(val),
                            Span::new(start, self.prev_end()),
                        ),
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

    fn is_type_spec_start(&self, kind: TokenKind) -> bool {
        matches!(
            kind,
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
                | TokenKind::WcharT
                | TokenKind::Char16
                | TokenKind::Char32
                | TokenKind::Const
                | TokenKind::Volatile
                | TokenKind::Extern
                | TokenKind::Static
                | TokenKind::Mutable
                | TokenKind::Register
                | TokenKind::Inline
                | TokenKind::Typedef
                | TokenKind::Constexpr
                | TokenKind::Virtual
                | TokenKind::Explicit
                | TokenKind::Friend
                | TokenKind::Auto
                | TokenKind::Decltype
                | TokenKind::Typename
        )
    }

    fn looks_like_named_type_declaration(&self) -> bool {
        if self.peek() != TokenKind::Ident {
            return false;
        }
        let mut offset = 1usize;
        let mut qualified = false;
        while self.peek_ahead(offset) == TokenKind::ColonColon
            && self.peek_ahead(offset + 1) == TokenKind::Ident
        {
            qualified = true;
            offset += 2;
        }
        if self.peek_ahead(offset) == TokenKind::Lt {
            let mut depth = 0usize;
            loop {
                match self.peek_ahead(offset) {
                    TokenKind::Lt => depth += 1,
                    TokenKind::Gt => {
                        depth = depth.saturating_sub(1);
                        if depth == 0 {
                            offset += 1;
                            break;
                        }
                    }
                    TokenKind::Eof => return false,
                    _ => {}
                }
                offset += 1;
            }
        }
        if self.peek_ahead(offset) == TokenKind::ColonColon
            && self.peek_ahead(offset + 1) == TokenKind::Ident
        {
            return true;
        }
        if qualified && self.peek_ahead(offset) == TokenKind::LParen {
            let first = self.tokens.get(self.pos).map(|token| token.value.as_str());
            let last = self
                .tokens
                .get(self.pos + offset - 1)
                .map(|token| token.value.as_str());
            return first == last;
        }
        while matches!(
            self.peek_ahead(offset),
            TokenKind::Star | TokenKind::Ampersand | TokenKind::AmpersandAmpersand
        ) {
            offset += 1;
        }
        self.peek_ahead(offset) == TokenKind::Ident
    }

    fn parse_declaration(&mut self) -> Stmt {
        let start = self.peek_token().span.start;
        let mut is_const = false;
        let mut is_static = false;
        let mut is_extern = false;
        let mut is_virtual = false;
        let mut is_override = false;
        let mut is_constexpr = false;
        let mut is_explicit = false;
        let mut is_friend = false;
        let mut is_inline = false;

        loop {
            match self.peek() {
                TokenKind::Const => {
                    is_const = true;
                    self.advance();
                }
                TokenKind::Static => {
                    is_static = true;
                    self.advance();
                }
                TokenKind::Extern => {
                    is_extern = true;
                    self.advance();
                }
                TokenKind::Virtual => {
                    is_virtual = true;
                    self.advance();
                }
                TokenKind::Constexpr => {
                    is_constexpr = true;
                    self.advance();
                }
                TokenKind::Explicit => {
                    is_explicit = true;
                    self.advance();
                }
                TokenKind::Friend => {
                    is_friend = true;
                    self.advance();
                }
                TokenKind::Inline => {
                    is_inline = true;
                    self.advance();
                }
                TokenKind::Mutable => {
                    self.advance();
                }
                _ => break,
            }
        }

        let base_type = self.parse_type();
        let mut name = if self.peek() == TokenKind::Ident {
            self.expect_ident()
        } else {
            String::new()
        };
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
        while self.peek() == TokenKind::ColonColon {
            self.advance();
            name = self.expect_ident();
        }

        if self.peek() == TokenKind::LParen {
            self.advance();
            let mut params = Vec::new();
            if self.peek() != TokenKind::RParen {
                while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                    let ptype = self.parse_type();
                    let pname =
                        if self.peek() == TokenKind::Ident && self.peek_ahead(1) != TokenKind::Eq {
                            Some(self.expect_ident())
                        } else {
                            None
                        };
                    let default = if self.peek() == TokenKind::Eq {
                        self.advance();
                        Some(Box::new(self.parse_expr()))
                    } else {
                        None
                    };
                    params.push(ParamDecl {
                        type_: Box::new(ptype),
                        name: pname,
                        default,
                        span: Span::new(start, self.prev_end()),
                    });
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
            }
            self.expect(TokenKind::RParen);
            if self.peek() == TokenKind::Const {
                self.advance();
            }
            if self.peek() == TokenKind::Override {
                self.advance();
                is_override = true;
            }
            if self.peek() == TokenKind::Eq
                && matches!(
                    self.peek_ahead(1),
                    TokenKind::IntLit
                        | TokenKind::Default
                        | TokenKind::CppDefault
                        | TokenKind::Delete
                )
            {
                self.advance();
                self.advance();
            }
            if self.peek() == TokenKind::Colon {
                self.advance();
                while self.peek() != TokenKind::LBrace && self.peek() != TokenKind::Eof {
                    self.advance();
                }
            }
            let body = if self.peek() == TokenKind::LBrace {
                Some(self.parse_block())
            } else {
                self.expect(TokenKind::Semicolon);
                None
            };
            Stmt::Decl(
                Decl::Func(
                    FuncDecl {
                        name,
                        return_type: Box::new(base_type),
                        params,
                        is_variadic: false,
                        body,
                        is_virtual,
                        is_override,
                        is_const,
                        is_pure: false,
                        is_constexpr,
                        is_inline,
                        is_explicit,
                        is_static,
                        is_friend,
                        span: Span::new(start, self.prev_end()),
                    },
                    Span::new(start, self.prev_end()),
                ),
                Span::new(start, self.prev_end()),
            )
        } else {
            let init = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(self.parse_expr())
            } else {
                None
            };
            self.expect(TokenKind::Semicolon);
            Stmt::Decl(
                Decl::Var(
                    VarDecl {
                        type_: Box::new(base_type),
                        name,
                        init,
                        is_const,
                        is_constexpr,
                        is_static,
                        is_extern,
                        span: Span::new(start, self.prev_end()),
                    },
                    Span::new(start, self.prev_end()),
                ),
                Span::new(start, self.prev_end()),
            )
        }
    }

    pub(super) fn parse_block(&mut self) -> Block {
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
        if self.peek() == TokenKind::Constexpr {
            self.advance();
        }
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
        let mut scan = 0usize;
        while !matches!(
            self.peek_ahead(scan),
            TokenKind::Colon | TokenKind::Semicolon | TokenKind::RParen | TokenKind::Eof
        ) {
            scan += 1;
        }
        if self.peek_ahead(scan) == TokenKind::Colon {
            while self.peek() != TokenKind::Colon && self.peek() != TokenKind::Eof {
                self.advance();
            }
            self.advance();
            let range = self.parse_expr();
            self.expect(TokenKind::RParen);
            let body = Box::new(self.parse_stmt());
            return Stmt::RangeFor(Box::new(Stmt::Empty(tok.span)), range, body, tok.span);
        }
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
        let span = expr.span();
        Stmt::Expr(expr, span)
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
                    span: self.peek_token().span,
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
                    span: self.peek_token().span,
                });
            } else {
                break;
            }
        }
        self.expect(TokenKind::RBrace);
        Stmt::Switch(expr, cases, tok.span)
    }

    fn parse_try(&mut self) -> Stmt {
        let tok = self.advance();
        let body = Box::new(self.parse_stmt());
        let mut catches = Vec::new();
        while self.peek() == TokenKind::Catch {
            self.advance();
            self.expect(TokenKind::LParen);
            let type_ = self.parse_type();
            let name = if self.peek() == TokenKind::Ident {
                Some(self.expect_ident())
            } else {
                None
            };
            self.expect(TokenKind::RParen);
            let catch_body = Box::new(self.parse_stmt());
            catches.push(CatchClause {
                type_: Box::new(type_),
                name,
                body: catch_body,
                span: self.peek_token().span,
            });
        }
        Stmt::Try(body, catches, None, tok.span)
    }

    fn parse_namespace(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        let mut name = self.expect_ident();
        while self.peek() == TokenKind::ColonColon {
            self.advance();
            name.push_str("::");
            name.push_str(&self.expect_ident());
        }
        self.expect(TokenKind::LBrace);
        let mut decls = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            if let Stmt::Decl(d, _) = self.parse_stmt() {
                decls.push(d);
            }
        }
        self.expect(TokenKind::RBrace);
        Decl::Namespace(name, decls, Span::new(start, self.prev_end()))
    }

    fn parse_using(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        if self.peek() == TokenKind::Namespace {
            self.advance();
            let ns = self.expect_ident();
            self.expect(TokenKind::Semicolon);
            return Decl::UsingNamespace(ns, Span::new(start, self.prev_end()));
        }
        let name = self.expect_ident();
        if self.peek() == TokenKind::Eq {
            self.advance();
            let _type = self.parse_type();
        }
        self.expect(TokenKind::Semicolon);
        Decl::Using(name, Span::new(start, self.prev_end()))
    }

    fn parse_template(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        if self.peek() != TokenKind::Lt {
            while self.peek() != TokenKind::Semicolon && self.peek() != TokenKind::Eof {
                self.advance();
            }
            self.expect(TokenKind::Semicolon);
            return Decl::Using(
                "template-instantiation".to_string(),
                Span::new(start, self.prev_end()),
            );
        }
        self.expect(TokenKind::Lt);
        let mut params = Vec::new();
        while self.peek() != TokenKind::Gt && self.peek() != TokenKind::Eof {
            if self.peek() == TokenKind::Typename || self.peek() == TokenKind::Class {
                self.advance();
                let name = self.expect_ident();
                params.push(TemplateParam::Type(name, self.peek_token().span));
            }
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
            // Forward-progress guard: a template param that isn't `typename`/`class`
            // (e.g. `Product<Category>`) must still be consumed or this loop spins.
            if self.peek() != TokenKind::Gt && self.peek() != TokenKind::Eof {
                self.advance();
            }
        }
        self.expect(TokenKind::Gt);
        let decl = Box::new(match self.parse_stmt() {
            Stmt::Decl(d, _) => d,
            _ => Decl::Var(
                VarDecl {
                    type_: Box::new(Expr::Ident("int".to_string(), Span::ZERO)),
                    name: String::new(),
                    init: None,
                    is_const: false,
                    is_constexpr: false,
                    is_static: false,
                    is_extern: false,
                    span: Span::ZERO,
                },
                Span::ZERO,
            ),
        });
        Decl::Template(
            TemplateDecl {
                params,
                decl,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_class_or_struct(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        let is_class = self.peek() == TokenKind::Class;
        self.advance();
        let name = self.expect_ident();
        let mut bases = Vec::new();
        if self.peek() == TokenKind::Colon {
            self.advance();
            loop {
                let access = match self.peek() {
                    TokenKind::Public => {
                        self.advance();
                        AccessSpec::Public
                    }
                    TokenKind::Private => {
                        self.advance();
                        AccessSpec::Private
                    }
                    TokenKind::Protected => {
                        self.advance();
                        AccessSpec::Protected
                    }
                    _ => {
                        if is_class {
                            AccessSpec::Private
                        } else {
                            AccessSpec::Public
                        }
                    }
                };
                let is_virtual = if self.peek() == TokenKind::Virtual {
                    self.advance();
                    true
                } else {
                    false
                };
                let base_name = self.expect_ident();
                bases.push(BaseSpec {
                    name: base_name,
                    access,
                    is_virtual,
                    span: self.peek_token().span,
                });
                if self.peek() != TokenKind::Comma {
                    break;
                }
                self.advance();
            }
        }
        self.expect(TokenKind::LBrace);
        let mut members = Vec::new();
        let mut _current_access = if is_class {
            AccessSpec::Private
        } else {
            AccessSpec::Public
        };
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            match self.peek() {
                TokenKind::Public => {
                    self.advance();
                    self.expect(TokenKind::Colon);
                    _current_access = AccessSpec::Public;
                }
                TokenKind::Private => {
                    self.advance();
                    self.expect(TokenKind::Colon);
                    _current_access = AccessSpec::Private;
                }
                TokenKind::Protected => {
                    self.advance();
                    self.expect(TokenKind::Colon);
                    _current_access = AccessSpec::Protected;
                }
                _ => {
                    if let Stmt::Decl(d, _) = self.parse_stmt() {
                        members.push(ClassMember::Decl(d, self.peek_token().span));
                    }
                }
            }
        }
        self.expect(TokenKind::RBrace);
        self.expect(TokenKind::Semicolon);
        Decl::Class(
            ClassDecl {
                name,
                bases,
                members,
                is_final: false,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_enum(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        if self.peek() == TokenKind::Class || self.peek() == TokenKind::Struct {
            self.advance();
        }
        let name = self.expect_ident();
        if self.peek() == TokenKind::Colon {
            self.advance();
            self.parse_type();
        }
        self.expect(TokenKind::LBrace);
        let mut values = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            let ename = self.expect_ident();
            let value = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(Box::new(self.parse_expr()))
            } else {
                None
            };
            values.push(EnumValue {
                name: ename,
                value,
                span: self.peek_token().span,
            });
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace);
        self.expect(TokenKind::Semicolon);
        Decl::Enum(
            EnumDecl {
                name,
                values,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_typedef(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        let type_ = self.parse_type();
        let name = self.expect_ident();
        self.expect(TokenKind::Semicolon);
        Decl::Typedef(
            TypedefDecl {
                name,
                type_: Box::new(type_),
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
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
