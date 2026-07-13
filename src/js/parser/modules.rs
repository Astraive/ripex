use super::state::Parser;
use crate::diagnostics::DiagnosticCode;
use crate::js::ast::*;
use crate::js::lexer::TokenKind;
use crate::span::Span;

pub fn parse_import(parser: &mut Parser) -> ImportDecl {
    let start = parser.current_pos();
    parser.expect(TokenKind::Import).ok();

    if parser.peek() == TokenKind::String {
        let tok = parser.advance();
        super::recovery::expect_semicolon(parser);
        return ImportDecl {
            span: parser.span_since(start),
            specifiers: Vec::new(),
            source: StrLit {
                span: tok.span,
                value: tok.value,
                raw: String::new(),
            },
            assertions: Vec::new(),
        };
    }

    if parser.peek() == TokenKind::LParen {
        let tok = parser.current_token().clone();
        let err = parser.error(DiagnosticCode::InvalidImport, &tok);
        parser.errors.push(err);
        return ImportDecl {
            span: Span::ZERO,
            specifiers: Vec::new(),
            source: StrLit {
                span: Span::ZERO,
                value: String::new(),
                raw: String::new(),
            },
            assertions: Vec::new(),
        };
    }

    let specifiers = parse_import_clause(parser);

    parser.expect(TokenKind::From).ok();

    let source_tok = parser.advance();
    let source = StrLit {
        span: source_tok.span,
        value: source_tok.value,
        raw: String::new(),
    };

    let assertions =
        if parser.peek() == TokenKind::Assert && parser.options.features.import_attributes {
            parser.advance();
            parse_import_assertions(parser)
        } else {
            Vec::new()
        };

    super::recovery::expect_semicolon(parser);

    ImportDecl {
        span: parser.span_since(start),
        specifiers,
        source,
        assertions,
    }
}

fn parse_import_clause(parser: &mut Parser) -> Vec<ImportSpecifier> {
    let mut specifiers = Vec::new();

    if parser.peek() == TokenKind::Ident || parser.peek() == TokenKind::Default {
        let tok = parser.advance();
        specifiers.push(ImportSpecifier::Default(ImportDefault {
            span: tok.span,
            local: Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            },
        }));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            return specifiers;
        }
    }

    if parser.peek() == TokenKind::Star {
        parser.advance();
        parser.expect(TokenKind::As).ok();
        let tok = parser.advance();
        specifiers.push(ImportSpecifier::Namespace(ImportNamespace {
            span: tok.span,
            local: Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            },
        }));
        return specifiers;
    }

    if parser.peek() == TokenKind::LBrace {
        parser.advance();
        while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
            if parser.peek() == TokenKind::Comma {
                parser.advance();
                continue;
            }
            let tok = parser.advance();
            let mut imported = Ident {
                span: tok.span,
                name: tok.value.clone(),
                optional: false,
            };
            let local;
            if parser.peek() == TokenKind::As {
                parser.advance();
                let local_tok = parser.advance();
                local = Ident {
                    span: local_tok.span,
                    name: local_tok.value,
                    optional: false,
                };
            } else {
                local = Ident {
                    span: tok.span,
                    name: tok.value,
                    optional: false,
                };
                imported = local.clone();
            }
            specifiers.push(ImportSpecifier::Named(ImportNamed {
                span: tok.span,
                imported,
                local,
            }));
            if parser.peek() == TokenKind::Comma {
                parser.advance();
            }
        }
        parser.expect(TokenKind::RBrace).ok();
    }

    specifiers
}

pub fn parse_export(parser: &mut Parser) -> ExportDecl {
    let start = parser.current_pos();
    parser.expect(TokenKind::Export).ok();

    if parser.peek() == TokenKind::Default {
        parser.advance();
        return parse_export_default(parser, start);
    }

    if parser.peek() == TokenKind::Star {
        let star = parser.advance();
        if parser.peek() == TokenKind::As {
            parser.advance();
            let exported = parser.advance();
            parser.expect(TokenKind::From).ok();
            let source_token = parser.advance();
            super::recovery::expect_semicolon(parser);
            return ExportDecl::Named(ExportNamed {
                span: parser.span_since(start),
                specifiers: vec![ExportSpecifier {
                    span: Span::new(star.span.start, exported.span.end),
                    local: Ident {
                        span: star.span,
                        name: "*".to_string(),
                        optional: false,
                    },
                    exported: Ident {
                        span: exported.span,
                        name: exported.value,
                        optional: false,
                    },
                }],
                source: Some(StrLit {
                    span: source_token.span,
                    value: source_token.value,
                    raw: String::new(),
                }),
                decl: None,
            });
        }
        if parser.peek() == TokenKind::From {
            parser.advance();
            let tok = parser.advance();
            let source = StrLit {
                span: tok.span,
                value: tok.value,
                raw: String::new(),
            };
            super::recovery::expect_semicolon(parser);
            return ExportDecl::All(ExportAll {
                span: parser.span_since(start),
                source,
            });
        } else {
            super::recovery::expect_semicolon(parser);
            return ExportDecl::Named(ExportNamed {
                span: parser.span_since(start),
                specifiers: Vec::new(),
                source: None,
                decl: None,
            });
        }
    }

    if parser.peek() == TokenKind::LBrace {
        return parse_export_named(parser, start);
    }

    if let Some(decl) = super::declarations::parse_decl(parser) {
        if matches!(decl, Decl::Var(_)) {
            super::recovery::expect_semicolon(parser);
        }
        return ExportDecl::Named(ExportNamed {
            span: parser.span_since(start),
            specifiers: Vec::new(),
            source: None,
            decl: Some(Box::new(decl)),
        });
    }

    let tok = parser.current_token().clone();
    let err = parser.error(DiagnosticCode::InvalidExport, &tok);
    parser.errors.push(err);
    ExportDecl::Named(ExportNamed {
        span: parser.span_since(start),
        specifiers: Vec::new(),
        source: None,
        decl: None,
    })
}

fn parse_export_default(parser: &mut Parser, start: usize) -> ExportDecl {
    let _decl_start = parser.current_pos();
    match parser.peek() {
        TokenKind::Function => {
            let fn_decl = super::declarations::parse_fn_decl(parser);
            let expr = parser.ast.alloc(Expr::Fn(FnExpr {
                span: fn_decl.span,
                id: Some(fn_decl.id),
                params: fn_decl.params,
                body: fn_decl.body,
                generator: fn_decl.generator,
                async_: fn_decl.async_,
            }));
            return ExportDecl::Default(ExportDefault {
                span: parser.span_since(start),
                decl: expr,
                has_assign: false,
            });
        }
        TokenKind::Async if parser.peek_ahead(1) == TokenKind::Function => {
            parser.advance();
            let fn_decl = super::declarations::parse_async_fn_decl(parser);
            let expr = parser.ast.alloc(Expr::Fn(FnExpr {
                span: fn_decl.span,
                id: Some(fn_decl.id),
                params: fn_decl.params,
                body: fn_decl.body,
                generator: fn_decl.generator,
                async_: fn_decl.async_,
            }));
            return ExportDecl::Default(ExportDefault {
                span: parser.span_since(start),
                decl: expr,
                has_assign: false,
            });
        }
        TokenKind::Class => {
            let class_decl = super::declarations::parse_class_decl(parser);
            let expr = parser.ast.alloc(Expr::Class(ClassExpr {
                span: class_decl.span,
                id: Some(class_decl.id),
                super_class: class_decl.super_class,
                body: class_decl.body,
            }));
            return ExportDecl::Default(ExportDefault {
                span: parser.span_since(start),
                decl: expr,
                has_assign: false,
            });
        }
        TokenKind::Interface if parser.options.features.typescript => {
            let iface = super::typescript::parse_ts_interface(parser);
            return ExportDecl::Named(ExportNamed {
                span: parser.span_since(start),
                specifiers: Vec::new(),
                source: None,
                decl: Some(Box::new(Decl::TsInterface(iface))),
            });
        }
        _ => {}
    }
    let expr = super::expressions::parse_assign_expr(parser);
    super::recovery::expect_semicolon(parser);
    ExportDecl::Default(ExportDefault {
        span: parser.span_since(start),
        decl: expr,
        has_assign: true,
    })
}

fn parse_import_assertions(parser: &mut Parser) -> Vec<ImportAttribute> {
    let mut assertions = Vec::new();
    parser.expect(TokenKind::LBrace).ok();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        let key_start = parser.current_pos();
        let key = if parser.peek() == TokenKind::String {
            let tok = parser.advance();
            ImportAttributeKey::StrLit(StrLit {
                span: tok.span,
                value: tok.value,
                raw: String::new(),
            })
        } else {
            let tok = parser.advance();
            ImportAttributeKey::Ident(Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            })
        };
        parser.expect(TokenKind::Colon).ok();
        let tok = parser.advance();
        let value = StrLit {
            span: tok.span,
            value: tok.value,
            raw: String::new(),
        };
        assertions.push(ImportAttribute {
            span: parser.span_since(key_start),
            key,
            value,
        });
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    assertions
}

fn parse_export_named(parser: &mut Parser, start: usize) -> ExportDecl {
    parser.expect(TokenKind::LBrace).ok();
    let mut specifiers = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        let tok = parser.advance();
        if parser.peek() == TokenKind::As {
            parser.advance();
            let local_tok = parser.advance();
            specifiers.push(ExportSpecifier {
                span: tok.span,
                local: Ident {
                    span: local_tok.span,
                    name: local_tok.value,
                    optional: false,
                },
                exported: Ident {
                    span: tok.span,
                    name: tok.value,
                    optional: false,
                },
            });
        } else {
            let name = tok.value.clone();
            specifiers.push(ExportSpecifier {
                span: tok.span,
                local: Ident {
                    span: tok.span,
                    name: name.clone(),
                    optional: false,
                },
                exported: Ident {
                    span: tok.span,
                    name,
                    optional: false,
                },
            });
        }
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        }
    }
    parser.expect(TokenKind::RBrace).ok();

    let source = if parser.peek() == TokenKind::From {
        parser.advance();
        let tok = parser.advance();
        Some(StrLit {
            span: tok.span,
            value: tok.value,
            raw: String::new(),
        })
    } else {
        None
    };

    super::recovery::expect_semicolon(parser);

    ExportDecl::Named(ExportNamed {
        span: parser.span_since(start),
        specifiers,
        source,
        decl: None,
    })
}
