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
            is_type_only: false,
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
            is_type_only: false,
            assertions: Vec::new(),
        };
    }

    let is_type_only = parser.options.features.typescript && parser.peek() == TokenKind::Type;
    if is_type_only {
        parser.advance();
    }
    let specifiers = parse_import_clause(parser, is_type_only);

    parser.expect(TokenKind::From).ok();

    let source_tok = parser.advance();
    let source = StrLit {
        span: source_tok.span,
        value: source_tok.value,
        raw: String::new(),
    };

    let assertions = if matches!(parser.peek(), TokenKind::Assert | TokenKind::With)
        && parser.options.features.import_attributes
    {
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
        is_type_only,
        assertions,
    }
}

fn parse_import_clause(parser: &mut Parser, declaration_type_only: bool) -> Vec<ImportSpecifier> {
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
            let is_type_only = declaration_type_only
                || (parser.options.features.typescript
                    && parser.peek() == TokenKind::Type
                    && parser.peek_ahead(1) != TokenKind::As);
            if is_type_only && parser.peek() == TokenKind::Type {
                parser.advance();
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
                is_type_only,
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

    // Preserve `export type { ... }` and `export type * from ...` while
    // leaving `export type Name = ...` for the TypeScript declaration parser.
    let is_type_only = parser.options.features.typescript
        && parser.peek() == TokenKind::Type
        && matches!(parser.peek_ahead(1), TokenKind::LBrace | TokenKind::Star);
    if is_type_only {
        parser.advance();
    }

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
                    is_type_only,
                }],
                source: Some(StrLit {
                    span: source_token.span,
                    value: source_token.value,
                    raw: String::new(),
                }),
                decl: None,
                is_type_only,
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
                is_type_only,
            });
        } else {
            super::recovery::expect_semicolon(parser);
            return ExportDecl::Named(ExportNamed {
                span: parser.span_since(start),
                specifiers: Vec::new(),
                source: None,
                decl: None,
                is_type_only,
            });
        }
    }

    if parser.peek() == TokenKind::LBrace {
        return parse_export_named(parser, start, is_type_only);
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
            is_type_only: false,
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
        is_type_only: false,
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
                is_type_only: false,
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

fn parse_export_named(
    parser: &mut Parser,
    start: usize,
    declaration_type_only: bool,
) -> ExportDecl {
    parser.expect(TokenKind::LBrace).ok();
    let mut specifiers = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        let is_type_only = declaration_type_only
            || (parser.options.features.typescript
                && parser.peek() == TokenKind::Type
                && parser.peek_ahead(1) != TokenKind::As);
        let specifier_start = parser.current_pos();
        if is_type_only && parser.peek() == TokenKind::Type {
            parser.advance();
        }
        let tok = parser.advance();
        if parser.peek() == TokenKind::As {
            parser.advance();
            let exported_tok = parser.advance();
            specifiers.push(ExportSpecifier {
                span: parser.span_since(specifier_start),
                local: Ident {
                    span: tok.span,
                    name: tok.value,
                    optional: false,
                },
                exported: Ident {
                    span: exported_tok.span,
                    name: exported_tok.value,
                    optional: false,
                },
                is_type_only,
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
                is_type_only,
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
        is_type_only: declaration_type_only,
    })
}
