use super::super::ast::expr::Expr;
use super::super::ast::stmt::*;
use super::super::lexer::TokenKind;
use super::state::Parser;
use crate::span::{Pos, Span};

impl Parser {
    pub fn parse_stmt(&mut self) -> Stmt {
        if self.bump_recursion().is_err() {
            return Stmt::Empty(Span::ZERO);
        }
        let result = match self.peek() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Fn => Stmt::Item(self.parse_fn_item(), self.peek_token().span),
            TokenKind::Struct => Stmt::Item(self.parse_struct_item(), self.peek_token().span),
            TokenKind::Enum => Stmt::Item(self.parse_enum_item(), self.peek_token().span),
            TokenKind::Trait => Stmt::Item(self.parse_trait_item(), self.peek_token().span),
            TokenKind::Impl => Stmt::Item(self.parse_impl_item(), self.peek_token().span),
            TokenKind::Use => Stmt::Item(self.parse_use_item(), self.peek_token().span),
            TokenKind::Mod => Stmt::Item(self.parse_mod_item(), self.peek_token().span),
            TokenKind::Type_ => Stmt::Item(self.parse_type_item(), self.peek_token().span),
            TokenKind::Static => Stmt::Item(self.parse_static_item(), self.peek_token().span),
            TokenKind::Const => Stmt::Item(self.parse_const_item(), self.peek_token().span),
            TokenKind::Pub => Stmt::Item(self.parse_pub_item(), self.peek_token().span),
            TokenKind::Unsafe => Stmt::Item(self.parse_unsafe_item(), self.peek_token().span),
            TokenKind::Extern => Stmt::Item(self.parse_extern_item(), self.peek_token().span),
            TokenKind::Hash => {
                self.advance();
                let body = self.parse_delimited_group(TokenKind::LBracket, TokenKind::RBracket);
                Stmt::Item(self.parse_item_after_attr(body), self.peek_token().span)
            }
            TokenKind::Semicolon => {
                self.advance();
                Stmt::Empty(Span::ZERO)
            }
            TokenKind::LBrace => {
                let block = self.parse_block();
                Stmt::Expr(Expr::Block(Box::new(block.clone()), block.span), block.span)
            }
            _ => {
                let expr = self.parse_expr();
                self.expect_semicolon();
                Stmt::Expr(expr.clone(), Span::new(expr.span().start, self.prev_end()))
            }
        };
        self.pop_recursion();
        result
    }

    fn parse_let(&mut self) -> Stmt {
        let tok = self.advance();
        let start = tok.span.start;
        let mutable = self.peek() == TokenKind::Mut;
        if mutable {
            self.advance();
        }
        let pattern = self.parse_pattern();
        let type_ann = if self.peek() == TokenKind::Colon {
            self.advance();
            Some(Box::new(self.parse_expr()))
        } else {
            None
        };
        let init = if self.peek() == TokenKind::Eq {
            self.advance();
            Some(Box::new(self.parse_expr()))
        } else {
            None
        };
        self.expect_semicolon();
        Stmt::Let(
            LetDecl {
                pattern,
                mutable,
                type_ann,
                init,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_fn_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let mut generics = Vec::new();
        if self.peek() == TokenKind::Lt {
            self.advance();
            while self.peek() != TokenKind::Gt && self.peek() != TokenKind::Eof {
                if self.peek() == TokenKind::Ident {
                    let pname = self.expect_ident();
                    generics.push(GenericParam {
                        name: pname,
                        bounds: Vec::new(),
                        span: Span::new(start, self.prev_end()),
                    });
                }
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
                // Forward-progress guard: unexpected token inside <...> must
                // still be consumed, else this loop spins forever.
                if self.peek() != TokenKind::Gt && self.peek() != TokenKind::Eof {
                    self.advance();
                }
            }
            self.expect(TokenKind::Gt);
        }
        self.expect(TokenKind::LParen);
        let mut params = Vec::new();
        while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
            let pname = self.parse_pattern();
            let ptype = if self.peek() == TokenKind::Colon {
                self.advance();
                Some(Box::new(
                    self.parse_type_until(&[TokenKind::Comma, TokenKind::RParen]),
                ))
            } else {
                None
            };
            params.push(FnParam {
                pattern: pname,
                type_ann: ptype,
                span: Span::new(start, self.prev_end()),
            });
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RParen);
        let return_type = if self.peek() == TokenKind::Arrow {
            self.advance();
            Some(Box::new(self.parse_type_until(&[
                TokenKind::LBrace,
                TokenKind::Semicolon,
            ])))
        } else {
            None
        };
        let body = if self.peek() == TokenKind::LBrace {
            Some(self.parse_block())
        } else {
            None
        };
        Item::Fn(
            FnDecl {
                name,
                generics,
                params,
                return_type,
                body,
                visibility: Visibility::Private,
                is_async: false,
                is_unsafe: false,
                is_extern: false,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_struct_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        let mut fields = Vec::new();
        if self.peek() == TokenKind::LBrace {
            self.advance();
            while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                let visibility = if self.peek() == TokenKind::Pub {
                    self.advance();
                    Visibility::Pub
                } else {
                    Visibility::Private
                };
                let fname = self.expect_ident();
                self.expect(TokenKind::Colon);
                let ftype = self.parse_type_until(&[TokenKind::Comma, TokenKind::RBrace]);
                fields.push(super::super::ast::stmt::StructField {
                    name: fname,
                    type_ann: Box::new(ftype),
                    visibility,
                    span: Span::new(start, self.prev_end()),
                });
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RBrace);
        }
        Item::Struct(
            StructDecl {
                name,
                generics,
                fields,
                visibility: Visibility::Private,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_enum_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let generics = self.parse_generic_params();
        let mut variants = Vec::new();
        if self.peek() == TokenKind::LBrace {
            self.advance();
            while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                let vname = self.expect_ident();
                let mut fields = Vec::new();
                if self.peek() == TokenKind::LParen {
                    self.advance();
                    while self.peek() != TokenKind::RParen && self.peek() != TokenKind::Eof {
                        fields.push(Box::new(self.parse_expr()));
                        if self.peek() == TokenKind::Comma {
                            self.advance();
                        }
                    }
                    self.expect(TokenKind::RParen);
                }
                variants.push(EnumVariant {
                    name: vname,
                    fields,
                    span: Span::new(start, self.prev_end()),
                });
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
            }
            self.expect(TokenKind::RBrace);
        }
        Item::Enum(
            EnumDecl {
                name,
                generics,
                variants,
                visibility: Visibility::Private,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_trait_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let mut methods = Vec::new();
        if self.peek() == TokenKind::LBrace {
            self.advance();
            while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                let item = if self.peek() == TokenKind::Pub {
                    self.parse_pub_item()
                } else if self.peek() == TokenKind::Fn {
                    self.parse_fn_item()
                } else {
                    self.advance();
                    continue;
                };
                if let Item::Fn(method, _) = item {
                    methods.push(method);
                }
                self.expect_semicolon();
            }
            self.expect(TokenKind::RBrace);
        }
        let span = Span::new(start, self.prev_end());
        Item::Trait(
            TraitDecl {
                name,
                methods,
                visibility: Visibility::Private,
                span,
            },
            span,
        )
    }

    fn parse_impl_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        self.parse_generic_params();
        let first = self.parse_type_until(&[TokenKind::For, TokenKind::LBrace]);
        let (trait_name, type_name) = if self.peek() == TokenKind::For {
            self.advance();
            let trait_name = match first {
                super::super::ast::expr::Expr::Ident(name, _) => Some(name),
                _ => None,
            };
            let target = self.parse_type_until(&[TokenKind::LBrace]);
            (trait_name, Box::new(target))
        } else {
            (None, Box::new(first))
        };
        let mut methods = Vec::new();
        if self.peek() == TokenKind::LBrace {
            self.advance();
            while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                let item = match self.peek() {
                    TokenKind::Fn => Some(self.parse_fn_item()),
                    TokenKind::Pub => Some(self.parse_pub_item()),
                    TokenKind::Const => Some(self.parse_const_item()),
                    _ => {
                        self.advance();
                        None
                    }
                };
                if let Some(Item::Fn(method, _)) = item {
                    methods.push(method);
                }
            }
            self.expect(TokenKind::RBrace);
        }
        let span = Span::new(start, self.prev_end());
        Item::Impl(
            ImplBlock {
                trait_name,
                type_name,
                methods,
                span,
            },
            span,
        )
    }

    fn parse_use_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let path = self.parse_use_tree(String::new(), start);
        self.expect_semicolon();
        let span = Span::new(start, self.prev_end());
        Item::Use(UseDecl { path, span }, span)
    }

    fn parse_use_tree(&mut self, prefix: String, start: crate::span::Pos) -> UsePath {
        let mut path = prefix;
        loop {
            let segment = match self.peek() {
                TokenKind::Ident => self.expect_ident(),
                TokenKind::Self_ => {
                    self.advance();
                    "self".to_string()
                }
                TokenKind::Super => {
                    self.advance();
                    "super".to_string()
                }
                TokenKind::Crate => {
                    self.advance();
                    "crate".to_string()
                }
                _ => break,
            };
            if !path.is_empty() {
                path.push_str("::");
            }
            path.push_str(&segment);
            if self.peek() != TokenKind::ColonColon {
                break;
            }
            self.advance();
            if self.peek() == TokenKind::LBrace {
                self.advance();
                let mut children = Vec::new();
                while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                    children.push(self.parse_use_tree(String::new(), start));
                    if self.peek() == TokenKind::Comma {
                        self.advance();
                    } else if self.peek() != TokenKind::RBrace {
                        // Ensure malformed trees still make progress and are
                        // consumed atomically through the statement semicolon.
                        self.advance();
                    }
                }
                self.expect(TokenKind::RBrace);
                return UsePath::Nested(path, children, Span::new(start, self.prev_end()));
            }
            if self.peek() == TokenKind::Star {
                self.advance();
                return UsePath::Glob(path, Span::new(start, self.prev_end()));
            }
        }
        if self.peek() == TokenKind::Star {
            self.advance();
            return UsePath::Glob(path, Span::new(start, self.prev_end()));
        }
        if path == "self" {
            UsePath::Self_(path, Span::new(start, self.prev_end()))
        } else {
            UsePath::Simple(path, Span::new(start, self.prev_end()))
        }
    }

    fn parse_mod_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let mut items = Vec::new();
        if self.peek() == TokenKind::LBrace {
            self.advance();
            while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
                items.push(self.parse_item());
            }
            self.expect(TokenKind::RBrace);
        }
        Item::Mod(
            ModDecl {
                name,
                items,
                visibility: Visibility::Private,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_type_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let generics = Vec::new();
        self.expect(TokenKind::Eq);
        let type_ = self.parse_type_until(&[TokenKind::Semicolon]);
        self.expect_semicolon();
        Item::Type(
            TypeAlias {
                name,
                generics,
                type_: Box::new(type_),
                visibility: Visibility::Private,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_static_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let mutable = self.peek() == TokenKind::Mut;
        if mutable {
            self.advance();
        }
        let name = self.expect_ident();
        self.expect(TokenKind::Colon);
        let type_ = self.parse_type_until(&[TokenKind::Eq]);
        self.expect(TokenKind::Eq);
        let init = self.parse_expr();
        self.expect_semicolon();
        Item::Static(
            StaticDecl {
                name,
                mutable,
                type_: Box::new(type_),
                init: Box::new(init),
                visibility: Visibility::Private,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_const_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        self.advance();
        let name = self.expect_ident();
        let type_ = if self.peek() == TokenKind::Colon {
            self.advance();
            Some(Box::new(self.parse_type_until(&[TokenKind::Eq])))
        } else {
            None
        };
        self.expect(TokenKind::Eq);
        let init = self.parse_expr();
        self.expect_semicolon();
        Item::Const(
            ConstItem {
                name,
                type_,
                init: Box::new(init),
                visibility: Visibility::Private,
                span: Span::new(start, self.prev_end()),
            },
            Span::new(start, self.prev_end()),
        )
    }

    fn parse_macro_item(&mut self) -> Item {
        let start = self.peek_token().span.start;
        let name = self.expect_ident();
        self.expect(TokenKind::Exclamation);
        let body = match self.peek() {
            TokenKind::LParen => self.parse_delimited_group(TokenKind::LParen, TokenKind::RParen),
            TokenKind::LBracket => {
                self.parse_delimited_group(TokenKind::LBracket, TokenKind::RBracket)
            }
            TokenKind::LBrace => self.parse_delimited_group(TokenKind::LBrace, TokenKind::RBrace),
            _ => String::new(),
        };
        self.expect_semicolon();
        let span = Span::new(start, self.prev_end());
        Item::Macro(MacroInvocation { name, body, span }, span)
    }

    pub fn parse_item(&mut self) -> Item {
        match self.peek() {
            TokenKind::Fn => self.parse_fn_item(),
            TokenKind::Struct => self.parse_struct_item(),
            TokenKind::Enum => self.parse_enum_item(),
            TokenKind::Trait => self.parse_trait_item(),
            TokenKind::Impl => self.parse_impl_item(),
            TokenKind::Use => self.parse_use_item(),
            TokenKind::Mod => self.parse_mod_item(),
            TokenKind::Type_ => self.parse_type_item(),
            TokenKind::Static => self.parse_static_item(),
            TokenKind::Const => self.parse_const_item(),
            TokenKind::Pub => self.parse_pub_item(),
            TokenKind::Unsafe => self.parse_unsafe_item(),
            TokenKind::Extern => self.parse_extern_item(),
            TokenKind::Ident
                if self.peek_ahead(1) == TokenKind::Exclamation
                    && matches!(
                        self.peek_ahead(2),
                        TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace
                    ) =>
            {
                self.parse_macro_item()
            }
            _ => {
                // Keep an unsupported token recoverable without fabricating a
                // semantic macro invocation from punctuation or delimiters.
                let tok = self.advance();
                Item::Macro(
                    MacroInvocation {
                        name: String::new(),
                        body: String::new(),
                        span: tok.span,
                    },
                    tok.span,
                )
            }
        }
    }

    fn parse_pub_item(&mut self) -> Item {
        self.advance();
        let mut item = self.parse_item();
        match item {
            Item::Fn(ref mut d, _) => d.visibility = Visibility::Pub,
            Item::Struct(ref mut d, _) => d.visibility = Visibility::Pub,
            Item::Enum(ref mut d, _) => d.visibility = Visibility::Pub,
            Item::Trait(ref mut d, _) => d.visibility = Visibility::Pub,
            Item::Mod(ref mut d, _) => d.visibility = Visibility::Pub,
            Item::Type(ref mut d, _) => d.visibility = Visibility::Pub,
            Item::Static(ref mut d, _) => d.visibility = Visibility::Pub,
            Item::Const(ref mut d, _) => d.visibility = Visibility::Pub,
            _ => {}
        }
        item
    }

    fn parse_unsafe_item(&mut self) -> Item {
        self.advance();
        self.parse_item()
    }

    fn parse_extern_item(&mut self) -> Item {
        self.advance();
        if self.peek() == TokenKind::Crate {
            self.advance();
            let name = self.expect_ident();
            self.expect_semicolon();
            return Item::ExternCrate(name, Span::new(self.prev_end(), self.prev_end()));
        }
        self.parse_item()
    }

    fn parse_delimited_group(&mut self, open: TokenKind, close: TokenKind) -> String {
        let start = self.peek_token().span.start;
        self.expect(open);
        let mut depth = 1;
        while depth > 0 && self.peek() != TokenKind::Eof {
            if self.peek() == open {
                depth += 1;
            }
            if self.peek() == close {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            self.advance();
        }
        self.expect(close);
        self.scanner_slice(start)
    }

    fn parse_item_after_attr(&mut self, _attr: String) -> Item {
        self.parse_item()
    }

    fn scanner_slice(&self, _start: Pos) -> String {
        String::new()
    }

    /// Consume a Rust type without treating `<`, `>` or `=` as expression
    /// operators. The current AST stores annotations as expressions, so keep
    /// the canonical token spelling in an identifier node until it gains a
    /// dedicated type representation.
    fn parse_type_until(&mut self, stops: &[TokenKind]) -> super::super::ast::expr::Expr {
        use super::super::ast::expr::Expr;

        let start = self.peek_token().span.start;
        let mut text = String::new();
        let mut angle_depth = 0usize;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;

        while self.peek() != TokenKind::Eof {
            let kind = self.peek();
            if angle_depth == 0 && paren_depth == 0 && bracket_depth == 0 && stops.contains(&kind) {
                break;
            }

            match kind {
                TokenKind::Lt => angle_depth += 1,
                TokenKind::Gt => angle_depth = angle_depth.saturating_sub(1),
                TokenKind::LParen => paren_depth += 1,
                TokenKind::RParen => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::LBracket => bracket_depth += 1,
                TokenKind::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }

            let token = self.advance();
            let piece = if token.value.is_empty() {
                match kind {
                    TokenKind::Ampersand => "&",
                    TokenKind::Star => "*",
                    TokenKind::ColonColon => "::",
                    TokenKind::Lt => "<",
                    TokenKind::Gt => ">",
                    TokenKind::Comma => ",",
                    TokenKind::LParen => "(",
                    TokenKind::RParen => ")",
                    TokenKind::LBracket => "[",
                    TokenKind::RBracket => "]",
                    TokenKind::Semicolon => ";",
                    _ => "",
                }
            } else {
                token.value.as_str()
            };
            text.push_str(piece);
        }

        Expr::Ident(text, Span::new(start, self.prev_end()))
    }
}
