use super::super::ast::stmt::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::Span;

impl Parser {
    fn parse_qualified_name(&mut self) -> String {
        let mut name = self.expect_ident();
        while self.peek() == TokenKind::Dot {
            self.advance();
            name.push('.');
            name.push_str(&self.expect_ident());
        }
        name
    }

    fn skip_angle_group(&mut self) {
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
                TokenKind::GtGt => {
                    depth = depth.saturating_sub(2);
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

    pub fn parse_stmt(&mut self) -> Stmt {
        if self.bump_recursion().is_err() {
            return Stmt::Empty(Span::ZERO);
        }
        let _start = self.peek_token().span.start;
        let result = match self.peek() {
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_stmt(),
            TokenKind::ForEach => self.parse_foreach_stmt(),
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
            TokenKind::Yield => {
                let tok = self.advance();
                if self.peek() == TokenKind::Return {
                    self.advance();
                    let e = self.parse_expr();
                    self.expect(TokenKind::Semicolon);
                    Stmt::YieldReturn(e, tok.span)
                } else if self.peek() == TokenKind::Break {
                    self.advance();
                    self.expect(TokenKind::Semicolon);
                    Stmt::YieldBreak(tok.span)
                } else {
                    Stmt::Empty(tok.span)
                }
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
                if self.peek() == TokenKind::Case {
                    self.advance();
                    let e = self.parse_expr();
                    self.expect(TokenKind::Semicolon);
                    Stmt::GotoCase(e, tok.span)
                } else if self.peek() == TokenKind::Default {
                    self.advance();
                    self.expect(TokenKind::Semicolon);
                    Stmt::GotoDefault(tok.span)
                } else {
                    let label = self.expect_ident();
                    self.expect(TokenKind::Semicolon);
                    Stmt::Goto(label, tok.span)
                }
            }
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
            TokenKind::Try => self.parse_try(),
            TokenKind::Checked => {
                self.advance();
                let body = Box::new(self.parse_stmt());
                Stmt::Checked(body, self.peek_token().span)
            }
            TokenKind::Unchecked => {
                self.advance();
                let body = Box::new(self.parse_stmt());
                Stmt::Unchecked(body, self.peek_token().span)
            }
            TokenKind::Lock => {
                self.advance();
                self.expect(TokenKind::LParen);
                let e = self.parse_expr();
                self.expect(TokenKind::RParen);
                let body = Box::new(self.parse_stmt());
                Stmt::Lock(e, body, self.peek_token().span)
            }
            TokenKind::Using => {
                self.advance();
                if self.peek() == TokenKind::LParen {
                    self.advance();
                    let e = self.parse_expr();
                    self.expect(TokenKind::RParen);
                    let body = Box::new(self.parse_stmt());
                    Stmt::Using(e, body, self.peek_token().span)
                } else {
                    let ns = self.parse_namespace_or_using();
                    Stmt::Decl(ns, self.peek_token().span)
                }
            }
            TokenKind::Fixed => {
                self.advance();
                self.expect(TokenKind::LParen);
                let e = self.parse_expr();
                self.expect(TokenKind::RParen);
                let body = Box::new(self.parse_stmt());
                Stmt::Fixed(e, body, self.peek_token().span)
            }
            TokenKind::Unsafe => {
                self.advance();
                let body = Box::new(self.parse_stmt());
                Stmt::Unsafe(body, self.peek_token().span)
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
            TokenKind::Namespace => {
                let d = self.parse_namespace_decl();
                Stmt::Decl(d, self.peek_token().span)
            }
            TokenKind::Class | TokenKind::Struct | TokenKind::Record | TokenKind::Interface => {
                let d = self.parse_type_decl();
                Stmt::Decl(d, self.peek_token().span)
            }
            TokenKind::Enum => {
                let d = self.parse_enum_decl();
                Stmt::Decl(d, self.peek_token().span)
            }
            TokenKind::Delegate => {
                let d = self.parse_delegate_decl();
                Stmt::Decl(d, self.peek_token().span)
            }
            TokenKind::Event => {
                let d = self.parse_event_decl();
                Stmt::Decl(d, self.peek_token().span)
            }
            // Visibility modifiers
            TokenKind::Public | TokenKind::Private | TokenKind::Protected | TokenKind::Internal => {
                let vis = self.parse_visibility();
                if matches!(
                    self.peek(),
                    TokenKind::Class
                        | TokenKind::Struct
                        | TokenKind::Record
                        | TokenKind::Interface
                        | TokenKind::Enum
                ) {
                    let mut decl = if self.peek() == TokenKind::Enum {
                        self.parse_enum_decl()
                    } else {
                        self.parse_type_decl()
                    };
                    Self::set_decl_visibility(&mut decl, vis);
                    Stmt::Decl(decl, self.peek_token().span)
                } else {
                    self.parse_member_decl(vis)
                }
            }
            // Member declarations
            TokenKind::Static
            | TokenKind::Virtual
            | TokenKind::Override
            | TokenKind::Abstract
            | TokenKind::Sealed
            | TokenKind::Async
            | TokenKind::Readonly
            | TokenKind::Partial
            | TokenKind::Extern
            | TokenKind::Implicit
            | TokenKind::Explicit => self.parse_member_decl(Visibility::None),
            TokenKind::Const => self.parse_const_decl(),
            TokenKind::Int
            | TokenKind::String
            | TokenKind::Bool
            | TokenKind::Double
            | TokenKind::Float
            | TokenKind::Char
            | TokenKind::Byte
            | TokenKind::Short
            | TokenKind::Long
            | TokenKind::Uint
            | TokenKind::Ushort
            | TokenKind::Ulong
            | TokenKind::Sbyte
            | TokenKind::Decimal
            | TokenKind::Object
            | TokenKind::Var
            | TokenKind::Void => {
                if self.peek_ahead(1) == TokenKind::Ident && self.peek_ahead(2) == TokenKind::LParen
                {
                    return self.parse_member_decl(Visibility::None);
                }
                if let Some((stmts, span)) = self.try_parse_local_decl() {
                    return Self::wrap_local_decls(stmts, span);
                }
                let expr = self.parse_expr();
                self.expect(TokenKind::Semicolon);
                let span = expr.span();
                Stmt::Expr(expr, span)
            }
            _ => {
                let is_named_type = self.peek() == TokenKind::Ident
                    && matches!(
                        self.peek_ahead(1),
                        TokenKind::Ident | TokenKind::This | TokenKind::LBracket
                    );
                if is_named_type {
                    if let Some((stmts, span)) = self.try_parse_local_decl() {
                        return Self::wrap_local_decls(stmts, span);
                    }
                }
                let expr = self.parse_expr();
                self.expect(TokenKind::Semicolon);
                let span = expr.span();
                Stmt::Expr(expr, span)
            }
        };
        self.pop_recursion();
        result
    }

    /// Tries to parse a local variable declaration statement, e.g.
    /// `int total = a;` or `var x = 1, y = 2;`. Returns `None` when the
    /// current token does not begin a declaration (so the caller can fall
    /// back to an expression statement). A declaration is recognized when
    /// the statement begins with a primitive type keyword, or with an
    /// identifier followed by another identifier (a type name then a name).
    fn try_parse_local_decl(&mut self) -> Option<(Vec<Stmt>, Span)> {
        let is_type_keyword = matches!(
            self.peek(),
            TokenKind::Int
                | TokenKind::String
                | TokenKind::Bool
                | TokenKind::Double
                | TokenKind::Float
                | TokenKind::Char
                | TokenKind::Byte
                | TokenKind::Short
                | TokenKind::Long
                | TokenKind::Uint
                | TokenKind::Ushort
                | TokenKind::Ulong
                | TokenKind::Sbyte
                | TokenKind::Decimal
                | TokenKind::Object
                | TokenKind::Var
                | TokenKind::Void
        );
        let is_named_type = self.peek() == TokenKind::Ident
            && matches!(
                self.peek_ahead(1),
                TokenKind::Ident | TokenKind::This | TokenKind::LBracket
            );
        if !is_type_keyword && !is_named_type {
            return None;
        }

        let start = self.peek_token().span.start;
        let mut stmts = Vec::new();
        loop {
            let type_ = self.parse_type();
            let name = self.expect_ident();
            let init = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(self.parse_expr())
            } else {
                None
            };
            let span = Span::new(start, self.prev_end());
            stmts.push(Stmt::Decl(
                Decl::Field(
                    FieldDecl {
                        type_: Box::new(type_),
                        name,
                        init,
                        is_const: false,
                        is_readonly: false,
                        is_static: false,
                        visibility: Visibility::None,
                        span,
                    },
                    span,
                ),
                span,
            ));
            match self.peek() {
                TokenKind::Comma => {
                    self.advance();
                    continue;
                }
                TokenKind::Semicolon => {
                    self.advance();
                    break;
                }
                _ => {
                    // Not a valid declarator separator; recover.
                    self.expect(TokenKind::Semicolon);
                    break;
                }
            }
        }
        Some((stmts, Span::new(start, self.prev_end())))
    }

    /// Wraps a list of local declaration statements into a single `Stmt`:
    /// if there is exactly one, that `Stmt` is returned directly; otherwise
    /// the list is wrapped in a `Stmt::Block`.
    fn wrap_local_decls(stmts: Vec<Stmt>, span: Span) -> Stmt {
        if stmts.len() == 1 {
            return stmts.into_iter().next().unwrap();
        }
        Stmt::Block(Block { stmts, span }, span)
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
    fn set_decl_visibility(decl: &mut Decl, visibility: Visibility) {
        match decl {
            Decl::Class(d, _) | Decl::Record(d, _) => d.visibility = visibility,
            Decl::Struct(d, _) => d.visibility = visibility,
            Decl::Interface(d, _) => d.visibility = visibility,
            Decl::Enum(d, _) => d.visibility = visibility,
            Decl::Delegate(d, _) => d.visibility = visibility,
            Decl::Event(d, _) => d.visibility = visibility,
            _ => {}
        }
    }

    fn set_decl_modifiers(
        decl: &mut Decl,
        is_static: bool,
        is_abstract: bool,
        is_sealed: bool,
        is_partial: bool,
        is_readonly: bool,
    ) {
        match decl {
            Decl::Class(d, _) | Decl::Record(d, _) => {
                d.is_static = is_static;
                d.is_abstract = is_abstract;
                d.is_sealed = is_sealed;
                d.is_partial = is_partial;
                d.is_readonly = is_readonly;
            }
            Decl::Struct(d, _) => {
                d.is_partial = is_partial;
                d.is_readonly = is_readonly;
            }
            Decl::Interface(d, _) => d.is_partial = is_partial,
            _ => {}
        }
    }

    fn parse_visibility(&mut self) -> Visibility {
        match self.peek() {
            TokenKind::Public => {
                self.advance();
                Visibility::Public
            }
            TokenKind::Private => {
                self.advance();
                Visibility::Private
            }
            TokenKind::Protected => {
                self.advance();
                if self.peek() == TokenKind::Internal {
                    self.advance();
                    Visibility::ProtectedInternal
                } else {
                    Visibility::Protected
                }
            }
            TokenKind::Internal => {
                self.advance();
                if self.peek() == TokenKind::Protected {
                    self.advance();
                    Visibility::ProtectedInternal
                } else {
                    Visibility::Internal
                }
            }
            _ => Visibility::None,
        }
    }

    fn parse_member_decl(&mut self, vis: Visibility) -> Stmt {
        let start = self.peek_token().span.start;
        let mut is_static = false;
        let mut is_virtual = false;
        let mut is_override = false;
        let mut is_abstract = false;
        let mut is_sealed = false;
        let mut is_async = false;
        let mut is_readonly = false;
        let mut is_partial = false;
        let mut is_extern = false;
        let mut is_unsafe = false;

        loop {
            match self.peek() {
                TokenKind::Static => {
                    is_static = true;
                    self.advance();
                }
                TokenKind::Virtual => {
                    is_virtual = true;
                    self.advance();
                }
                TokenKind::Override => {
                    is_override = true;
                    self.advance();
                }
                TokenKind::Abstract => {
                    is_abstract = true;
                    self.advance();
                }
                TokenKind::Sealed => {
                    is_sealed = true;
                    self.advance();
                }
                TokenKind::Async => {
                    is_async = true;
                    self.advance();
                }
                TokenKind::Readonly => {
                    is_readonly = true;
                    self.advance();
                }
                TokenKind::Partial => {
                    is_partial = true;
                    self.advance();
                }
                TokenKind::Extern => {
                    is_extern = true;
                    self.advance();
                }
                TokenKind::Unsafe => {
                    is_unsafe = true;
                    self.advance();
                }
                TokenKind::Implicit => {
                    self.advance();
                }
                TokenKind::Explicit => {
                    self.advance();
                }
                _ => break,
            }
        }

        if matches!(
            self.peek(),
            TokenKind::Class | TokenKind::Struct | TokenKind::Record | TokenKind::Interface
        ) {
            let mut decl = self.parse_type_decl();
            Self::set_decl_visibility(&mut decl, vis);
            Self::set_decl_modifiers(
                &mut decl,
                is_static,
                is_abstract,
                is_sealed,
                is_partial,
                is_readonly,
            );
            return Stmt::Decl(decl, Span::new(start, self.prev_end()));
        }

        // A constructor has no return type: the first identifier is the type
        // name and is immediately followed by its parameter list. This also
        // covers static constructors after the modifier loop above.
        if self.peek() == TokenKind::Ident && self.peek_ahead(1) == TokenKind::LParen {
            let _name = self.expect_ident();
            self.advance();
            let mut params = Vec::new();
            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                let is_ref = matches!(self.peek(), TokenKind::Ref);
                if is_ref {
                    self.advance();
                }
                let ptype = self.parse_type();
                let pname = self.expect_ident();
                params.push(ParamDecl {
                    type_: Box::new(ptype),
                    name: pname,
                    is_ref,
                    is_out: false,
                    is_in: false,
                    is_params: false,
                    is_this: false,
                    default: None,
                    span: self.peek_token().span,
                });
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RParen);
            let initializer = if self.peek() == TokenKind::Colon {
                self.advance();
                let is_base = self.peek() == TokenKind::Base;
                if self.peek() == TokenKind::Base || self.peek() == TokenKind::This {
                    self.advance();
                } else {
                    self.expect_ident();
                }
                self.expect(TokenKind::LParen);
                let mut args = Vec::new();
                while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                    args.push(self.parse_expr());
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    }
                }
                self.expect(TokenKind::RParen);
                Some(if is_base {
                    ConstructorInit::Base(args)
                } else {
                    ConstructorInit::This(args)
                })
            } else {
                None
            };
            let body = if self.peek() == TokenKind::LBrace {
                Some(self.parse_block())
            } else {
                self.expect(TokenKind::Semicolon);
                None
            };
            let span = Span::new(start, self.prev_end());
            return Stmt::Decl(
                Decl::Constructor(
                    ConstructorDecl {
                        params,
                        body,
                        initializer,
                        visibility: vis,
                        is_static,
                        span,
                    },
                    span,
                ),
                span,
            );
        }

        let return_type = self.parse_type();
        let name = if self.peek() == TokenKind::Ident {
            self.expect_ident()
        } else {
            "__ctor".to_string()
        };

        if self.peek() == TokenKind::Lt {
            self.skip_angle_group();
        }

        if self.peek() == TokenKind::LParen {
            // Method
            self.advance();
            let mut params = Vec::new();
            while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                while matches!(
                    self.peek(),
                    TokenKind::Ref
                        | TokenKind::Out
                        | TokenKind::In
                        | TokenKind::Params
                        | TokenKind::This
                ) {
                    self.advance();
                }
                let ptype = self.parse_type();
                let pname = self.expect_ident();
                let default = if self.peek() == TokenKind::Eq {
                    self.advance();
                    Some(Box::new(self.parse_expr()))
                } else {
                    None
                };
                params.push(ParamDecl {
                    type_: Box::new(ptype),
                    name: pname,
                    is_ref: false,
                    is_out: false,
                    is_in: false,
                    is_params: false,
                    is_this: false,
                    default,
                    span: self.peek_token().span,
                });
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RParen);
            if self.peek() == TokenKind::Colon {
                while !matches!(
                    self.peek(),
                    TokenKind::LBrace | TokenKind::FatArrow | TokenKind::Eof
                ) {
                    self.advance();
                }
            }
            if self.peek() == TokenKind::Where {
                while !matches!(
                    self.peek(),
                    TokenKind::LBrace | TokenKind::FatArrow | TokenKind::Eof
                ) {
                    self.advance();
                }
            }
            let body = if self.peek() == TokenKind::LBrace {
                Some(self.parse_block())
            } else if self.peek() == TokenKind::FatArrow {
                self.advance();
                let expr = self.parse_expr();
                let expr_span = expr.span();
                self.expect(TokenKind::Semicolon);
                Some(Block {
                    stmts: vec![Stmt::Return(Some(expr), expr_span)],
                    span: Span::new(start, self.prev_end()),
                })
            } else {
                self.expect(TokenKind::Semicolon);
                None
            };
            Stmt::Decl(
                Decl::Method(
                    FuncDecl {
                        name,
                        return_type: Box::new(return_type),
                        params,
                        body,
                        is_async,
                        is_static,
                        is_virtual,
                        is_override,
                        is_abstract,
                        is_sealed,
                        is_unsafe,
                        is_extern,
                        is_partial,
                        visibility: vis,
                        type_params: Vec::new(),
                        span: Span::new(start, self.prev_end()),
                    },
                    Span::new(start, self.prev_end()),
                ),
                Span::new(start, self.prev_end()),
            )
        } else if self.peek() == TokenKind::LBrace || self.peek() == TokenKind::FatArrow {
            // Property or field. Expression-bodied properties are retained in
            // PropertyDecl::init so fact extraction can traverse the value.
            let (getter, setter, init, is_auto) = if self.peek() == TokenKind::FatArrow {
                self.advance();
                let expr = self.parse_expr();
                self.expect(TokenKind::Semicolon);
                (None, None, Some(Box::new(expr)), false)
            } else {
                self.advance();
                let getter = if self.peek() == TokenKind::Get {
                    self.advance();
                    Some(Box::new(self.parse_accessor_body()))
                } else {
                    None
                };
                let setter = if self.peek() == TokenKind::Set {
                    self.advance();
                    Some(Box::new(self.parse_accessor_body()))
                } else {
                    None
                };
                self.expect(TokenKind::RBrace);
                let init = if self.peek() == TokenKind::Eq {
                    self.advance();
                    let expr = self.parse_expr();
                    self.expect(TokenKind::Semicolon);
                    Some(Box::new(expr))
                } else {
                    None
                };
                let is_auto = getter.is_none() && setter.is_none();
                (getter, setter, init, is_auto)
            };
            let span = Span::new(start, self.prev_end());
            Stmt::Decl(
                Decl::Property(
                    PropertyDecl {
                        type_: Box::new(return_type),
                        name,
                        getter,
                        setter,
                        init,
                        is_auto,
                        visibility: vis,
                        span,
                    },
                    span,
                ),
                span,
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
                Decl::Field(
                    FieldDecl {
                        type_: Box::new(return_type),
                        name,
                        init,
                        is_const: false,
                        is_readonly,
                        is_static,
                        visibility: vis,
                        span: Span::new(start, self.prev_end()),
                    },
                    Span::new(start, self.prev_end()),
                ),
                Span::new(start, self.prev_end()),
            )
        }
    }

    fn parse_accessor_body(&mut self) -> Stmt {
        if self.peek() == TokenKind::Semicolon {
            self.advance();
            Stmt::Empty(Span::ZERO)
        } else if self.peek() == TokenKind::FatArrow {
            self.advance();
            let e = self.parse_expr();
            self.expect(TokenKind::Semicolon);
            Stmt::Expr(e, self.peek_token().span)
        } else {
            let block = self.parse_block();
            let span = block.span;
            Stmt::Block(block, span)
        }
    }

    fn parse_const_decl(&mut self) -> Stmt {
        let start = self.peek_token().span.start;
        self.advance();
        let type_ = self.parse_type();
        let name = self.expect_ident();
        self.expect(TokenKind::Eq);
        let init = self.parse_expr();
        self.expect(TokenKind::Semicolon);
        Stmt::Decl(
            Decl::Field(
                FieldDecl {
                    type_: Box::new(type_),
                    name,
                    init: Some(init),
                    is_const: true,
                    is_readonly: false,
                    is_static: false,
                    visibility: Visibility::None,
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            ),
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_namespace_decl(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.parse_qualified_name();
        self.expect(TokenKind::LBrace);
        let mut members = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            if let Stmt::Decl(d, _) = self.parse_stmt() {
                members.push(d);
            }
        }
        self.expect(TokenKind::RBrace);
        Decl::Namespace(name, members, Span::new(start, self.prev_end()))
    }

    fn parse_type_decl(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        let kind = self.peek();
        self.advance();
        let is_partial = false;
        let name = self.expect_ident();
        let mut type_params = Vec::new();
        if self.peek() == TokenKind::Lt {
            self.advance();
            while self.peek() != TokenKind::Gt && self.peek() != TokenKind::Eof {
                let pname = self.expect_ident();
                type_params.push(TypeParam {
                    name: pname,
                    constraints: Vec::new(),
                    span: self.peek_token().span,
                });
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::Gt);
        }
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
        match kind {
            TokenKind::Class | TokenKind::Record => {
                let mut interfaces = Vec::new();
                if self.peek() == TokenKind::Colon {
                    self.advance();
                    interfaces.push(Box::new(self.parse_type()));
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        interfaces.push(Box::new(self.parse_type()));
                    }
                }
                self.expect(TokenKind::LBrace);
                let mut members = Vec::new();
                while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                    let member = if self.peek() == TokenKind::Ident
                        && (self.peek_ahead(1) == TokenKind::LParen
                            || (self.peek_ahead(1) == TokenKind::Ident
                                && self.peek_ahead(2) == TokenKind::LParen))
                    {
                        self.parse_member_decl(Visibility::None)
                    } else {
                        self.parse_stmt()
                    };
                    if let Stmt::Decl(d, _) = member {
                        members.push(d);
                    }
                }
                self.expect(TokenKind::RBrace);
                Decl::Class(
                    ClassDecl {
                        name,
                        base: None,
                        interfaces,
                        members,
                        is_static: false,
                        is_abstract: false,
                        is_sealed: false,
                        is_partial,
                        is_readonly: false,
                        visibility: Visibility::None,
                        type_params,
                        span: Span::new(start, self.prev_end()),
                    },
                    Span::new(start, self.prev_end()),
                )
            }
            TokenKind::Struct => {
                let mut interfaces = Vec::new();
                if self.peek() == TokenKind::Colon {
                    self.advance();
                    interfaces.push(Box::new(self.parse_type()));
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        interfaces.push(Box::new(self.parse_type()));
                    }
                }
                self.expect(TokenKind::LBrace);
                let mut members = Vec::new();
                while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                    let member = if self.peek() == TokenKind::Ident
                        && (self.peek_ahead(1) == TokenKind::LParen
                            || (self.peek_ahead(1) == TokenKind::Ident
                                && self.peek_ahead(2) == TokenKind::LParen))
                    {
                        self.parse_member_decl(Visibility::None)
                    } else {
                        self.parse_stmt()
                    };
                    if let Stmt::Decl(d, _) = member {
                        members.push(d);
                    }
                }
                self.expect(TokenKind::RBrace);
                Decl::Struct(
                    StructDecl {
                        name,
                        interfaces,
                        members,
                        is_readonly: false,
                        is_partial,
                        visibility: Visibility::None,
                        type_params,
                        span: Span::new(start, self.prev_end()),
                    },
                    Span::new(start, self.prev_end()),
                )
            }
            TokenKind::Interface => {
                let mut bases = Vec::new();
                if self.peek() == TokenKind::Colon {
                    self.advance();
                    bases.push(Box::new(self.parse_type()));
                    while self.peek() == TokenKind::Comma {
                        self.advance();
                        bases.push(Box::new(self.parse_type()));
                    }
                }
                self.expect(TokenKind::LBrace);
                let mut members = Vec::new();
                while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                    let member = if self.peek() == TokenKind::Ident
                        && (self.peek_ahead(1) == TokenKind::LParen
                            || (self.peek_ahead(1) == TokenKind::Ident
                                && self.peek_ahead(2) == TokenKind::LParen))
                    {
                        self.parse_member_decl(Visibility::None)
                    } else {
                        self.parse_stmt()
                    };
                    if let Stmt::Decl(d, _) = member {
                        members.push(d);
                    }
                }
                self.expect(TokenKind::RBrace);
                Decl::Interface(
                    InterfaceDecl {
                        name,
                        bases,
                        members,
                        is_partial,
                        visibility: Visibility::None,
                        type_params,
                        span: Span::new(start, self.prev_end()),
                    },
                    Span::new(start, self.prev_end()),
                )
            }
            _ => Decl::Class(
                ClassDecl {
                    name,
                    base: None,
                    interfaces: Vec::new(),
                    members: Vec::new(),
                    is_static: false,
                    is_abstract: false,
                    is_sealed: false,
                    is_partial: false,
                    is_readonly: false,
                    visibility: Visibility::None,
                    type_params,
                    span: Span::new(start, self.prev_end()),
                },
                Span::new(start, self.prev_end()),
            ),
        }
    }

    fn parse_enum_decl(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        self.expect(TokenKind::LBrace);
        let mut members = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            let ename = self.expect_ident();
            let value = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(self.parse_expr())
            } else {
                None
            };
            members.push(EnumMember {
                name: ename,
                value,
                span: self.peek_token().span,
            });
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace);
        Decl::Enum(
            EnumDecl {
                name,
                members,
                visibility: Visibility::None,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_delegate_decl(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        let return_type = self.parse_type();
        let name = self.expect_ident();
        self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            let ptype = self.parse_type();
            let pname = self.expect_ident();
            params.push(ParamDecl {
                type_: Box::new(ptype),
                name: pname,
                is_ref: false,
                is_out: false,
                is_in: false,
                is_params: false,
                is_this: false,
                default: None,
                span: self.peek_token().span,
            });
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        self.expect(TokenKind::Semicolon);
        Decl::Delegate(
            DelegateDecl {
                name,
                return_type: Box::new(return_type),
                params,
                visibility: Visibility::None,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_event_decl(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        self.advance();
        let type_ = self.parse_type();
        let name = self.expect_ident();
        self.expect(TokenKind::Semicolon);
        Decl::Event(
            EventDecl {
                type_: Box::new(type_),
                name,
                visibility: Visibility::None,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_namespace_or_using(&mut self) -> Decl {
        let start = self.peek_token().span.start;
        let name = self.parse_qualified_name();
        self.expect(TokenKind::Semicolon);
        Decl::Using(
            UsingDecl {
                namespace: name,
                alias: None,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
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
        self.expect(TokenKind::Semicolon);
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

    fn parse_foreach_stmt(&mut self) -> Stmt {
        let tok = self.advance();
        self.expect(TokenKind::LParen);
        let _type_ = self.parse_type();
        let name = self.expect_ident();
        self.expect(TokenKind::In);
        let expr = self.parse_expr();
        self.expect(TokenKind::RParen);
        let body = Box::new(self.parse_stmt());
        Stmt::Foreach(name, expr, body, tok.span)
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
        let mut sections = Vec::new();
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            let mut labels = Vec::new();
            while self.peek() == TokenKind::Case || self.peek() == TokenKind::Default {
                if self.peek() == TokenKind::Case {
                    self.advance();
                    let case_expr = self.parse_expr();
                    labels.push(CaseLabel::Case(case_expr, self.peek_token().span));
                } else {
                    self.advance();
                    labels.push(CaseLabel::Default(self.peek_token().span));
                }
                self.expect(TokenKind::Colon);
            }
            let mut stmts = Vec::new();
            while self.peek() != TokenKind::Case
                && self.peek() != TokenKind::Default
                && self.peek() != TokenKind::RBrace
                && self.peek() != TokenKind::Eof
            {
                stmts.push(self.parse_stmt());
            }
            sections.push(CaseSection {
                labels,
                stmts,
                span: self.peek_token().span,
            });
        }
        self.expect(TokenKind::RBrace);
        Stmt::Switch(expr, sections, tok.span)
    }

    fn parse_try(&mut self) -> Stmt {
        let tok = self.advance();
        let body = Box::new(self.parse_stmt());
        let mut catches = Vec::new();
        while self.peek() == TokenKind::Catch {
            self.advance();
            let type_ = if self.peek() == TokenKind::LParen {
                self.advance();
                let t = self.parse_type();
                let _name = if self.peek() == TokenKind::Ident {
                    Some(self.expect_ident())
                } else {
                    None
                };
                self.expect(TokenKind::RParen);
                Some(Box::new(t))
            } else {
                None
            };
            let when = if self.peek() == TokenKind::When {
                self.advance();
                self.expect(TokenKind::LParen);
                let e = self.parse_expr();
                self.expect(TokenKind::RParen);
                Some(Box::new(e))
            } else {
                None
            };
            let catch_body = Box::new(self.parse_stmt());
            catches.push(CatchClause {
                type_,
                name: None,
                when,
                body: catch_body,
                span: self.peek_token().span,
            });
        }
        let finally = if self.peek() == TokenKind::Finally {
            self.advance();
            Some(Box::new(self.parse_stmt()))
        } else {
            None
        };
        Stmt::Try(body, catches, finally, tok.span)
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
