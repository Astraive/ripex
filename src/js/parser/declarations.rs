use crate::diagnostics::DiagnosticCode;
use crate::js::ast::*;
use crate::js::lexer::TokenKind;
use crate::span::Span;

use super::expressions;
use super::patterns;
use super::state::Parser;

pub fn parse_decl(parser: &mut Parser) -> Option<Decl> {
    match parser.peek() {
        TokenKind::Function => Some(Decl::Fn(parse_fn_decl(parser))),
        TokenKind::Async => {
            if parser.peek_ahead(1) == TokenKind::Function {
                parser.advance();
                Some(Decl::Fn(parse_async_fn_decl(parser)))
            } else {
                None
            }
        }
        TokenKind::Class => Some(Decl::Class(parse_class_decl(parser))),
        TokenKind::Const | TokenKind::Let | TokenKind::Var | TokenKind::Using => {
            Some(Decl::Var(parse_var_decl(parser)))
        }
        TokenKind::Interface if parser.options.features.typescript => Some(Decl::TsInterface(
            super::typescript::parse_ts_interface(parser),
        )),
        TokenKind::Type if parser.options.features.typescript => Some(Decl::TsTypeAlias(
            super::typescript::parse_ts_type_alias(parser),
        )),
        TokenKind::Enum if parser.options.features.typescript => {
            Some(Decl::TsEnum(super::typescript::parse_ts_enum(parser)))
        }
        TokenKind::Module | TokenKind::Namespace if parser.options.features.typescript => {
            Some(Decl::TsModule(super::typescript::parse_ts_module(parser)))
        }
        TokenKind::Abstract if parser.options.features.typescript => {
            parser.advance();
            if parser.peek() == TokenKind::Class {
                Some(Decl::Class(parse_class_decl(parser)))
            } else {
                None
            }
        }
        TokenKind::Declare if parser.options.features.typescript => {
            parser.advance();
            parse_decl(parser)
        }
        _ => None,
    }
}

pub fn parse_var_stmt(parser: &mut Parser) -> Stmt {
    let vd = parse_var_decl(parser);
    super::recovery::expect_semicolon(parser);
    Stmt::Decl(Decl::Var(vd))
}

pub fn parse_using_stmt(parser: &mut Parser, await_: bool) -> Stmt {
    let vd = parse_var_decl_with_await(parser, await_);
    super::recovery::expect_semicolon(parser);
    Stmt::Decl(Decl::Var(vd))
}

fn parse_var_decl(parser: &mut Parser) -> VarDecl {
    parse_var_decl_with_await(parser, false)
}

fn parse_var_decl_with_await(parser: &mut Parser, await_: bool) -> VarDecl {
    let start = parser.current_pos();
    let kind = match parser.peek() {
        TokenKind::Const => VarKind::Const,
        TokenKind::Let => VarKind::Let,
        TokenKind::Using => VarKind::Using,
        _ => VarKind::Var,
    };
    parser.advance();

    let mut decls = Vec::new();
    loop {
        let pat = patterns::parse_binding_pat(parser);

        if parser.peek() == TokenKind::Colon && parser.options.features.typescript {
            parser.advance();
            let _ = super::typescript::parse_ts_type(parser);
        }

        let init = if parser.peek() == TokenKind::Eq {
            parser.advance();
            Some(expressions::parse_assign_expr(parser))
        } else {
            None
        };

        let span = if let Some(init_val) = init {
            Span::new(pat.span().start, parser.expr_span(init_val).end)
        } else {
            pat.span()
        };

        decls.push(VarDeclarator {
            span,
            name: pat,
            init,
        });

        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }

    VarDecl {
        span: parser.span_since(start),
        kind,
        decls,
        await_,
    }
}

pub fn parse_fn_decl(parser: &mut Parser) -> FnDecl {
    parse_fn_decl_kind(parser, false)
}

pub fn parse_async_fn_decl(parser: &mut Parser) -> FnDecl {
    parse_fn_decl_kind(parser, true)
}

fn parse_fn_decl_kind(parser: &mut Parser, async_: bool) -> FnDecl {
    let start = parser.current_pos();
    let decorators = super::expressions::parse_decorators(parser);
    parser.expect(TokenKind::Function).ok();

    let generator = parser.peek() == TokenKind::Star;
    if generator {
        parser.advance();
    }

    let id_tok = parser.advance();
    let id = Ident {
        span: id_tok.span,
        name: id_tok.value,
        optional: false,
    };

    let _type_params = super::typescript::maybe_parse_ts_type_params(parser);

    parser.expect(TokenKind::LParen).ok();
    let params = parse_fn_params(parser);
    parser.expect(TokenKind::RParen).ok();

    let _return_type = super::typescript::maybe_parse_ts_type_ann(parser);

    parser.ctx.push(super::state::Context::InFunction);
    if async_ {
        parser.ctx.push(super::state::Context::InAsync);
    }
    if generator {
        parser.ctx.push(super::state::Context::InGenerator);
    }
    let body = parser.parse_block();
    if generator {
        parser.ctx.pop();
    }
    if async_ {
        parser.ctx.pop();
    }
    parser.ctx.pop();

    FnDecl {
        span: parser.span_since(start),
        id,
        params,
        body: Some(body),
        generator,
        async_,
        declare: false,
        decorators,
    }
}

pub fn parse_class_decl(parser: &mut Parser) -> ClassDecl {
    let start = parser.current_pos();
    let decorators = super::expressions::parse_decorators(parser);
    parser.expect(TokenKind::Class).ok();

    let id_tok = parser.advance();
    let id = Ident {
        span: id_tok.span,
        name: id_tok.value,
        optional: false,
    };

    let super_class = if parser.peek() == TokenKind::Extends {
        parser.advance();
        let expr = expressions::parse_expr(parser, 17);
        Some(expr)
    } else {
        None
    };

    let body = parse_class_body(parser);

    ClassDecl {
        span: parser.span_since(start),
        id,
        super_class,
        body,
        declare: false,
        abstract_: false,
        decorators,
    }
}

pub fn parse_fn_decl_with_decorators(parser: &mut Parser, decorators: Vec<Decorator>) -> FnDecl {
    let start = parser.current_pos();
    parser.expect(TokenKind::Function).ok();

    let generator = parser.peek() == TokenKind::Star;
    if generator {
        parser.advance();
    }

    let id_tok = parser.advance();
    let id = Ident {
        span: id_tok.span,
        name: id_tok.value,
        optional: false,
    };

    let _type_params = super::typescript::maybe_parse_ts_type_params(parser);

    parser.expect(TokenKind::LParen).ok();
    let params = parse_fn_params(parser);
    parser.expect(TokenKind::RParen).ok();

    let _return_type = super::typescript::maybe_parse_ts_type_ann(parser);

    let body = parser.parse_block();

    FnDecl {
        span: parser.span_since(start),
        id,
        params,
        body: Some(body),
        generator,
        async_: false,
        declare: false,
        decorators,
    }
}

pub fn parse_class_decl_with_decorators(
    parser: &mut Parser,
    decorators: Vec<Decorator>,
) -> ClassDecl {
    let start = parser.current_pos();
    parser.expect(TokenKind::Class).ok();

    let id_tok = parser.advance();
    let id = Ident {
        span: id_tok.span,
        name: id_tok.value,
        optional: false,
    };

    let super_class = if parser.peek() == TokenKind::Extends {
        parser.advance();
        let expr = expressions::parse_expr(parser, 17);
        Some(expr)
    } else {
        None
    };

    let body = parse_class_body(parser);

    ClassDecl {
        span: parser.span_since(start),
        id,
        super_class,
        body,
        declare: false,
        abstract_: false,
        decorators,
    }
}

pub fn parse_fn_expr(parser: &mut Parser) -> FnExpr {
    let start = parser.current_pos();
    parser.expect(TokenKind::Function).ok();

    let generator = parser.peek() == TokenKind::Star;
    if generator {
        parser.advance();
    }

    let id = if parser.peek() == TokenKind::Ident
        || parser.peek() == TokenKind::Yield
        || parser.peek() == TokenKind::Async
    {
        let tok = parser.advance();
        Some(Ident {
            span: tok.span,
            name: tok.value,
            optional: false,
        })
    } else {
        None
    };

    let _type_params = super::typescript::maybe_parse_ts_type_params(parser);

    parser.expect(TokenKind::LParen).ok();
    let params = parse_fn_params(parser);
    parser.expect(TokenKind::RParen).ok();

    let _return_type = super::typescript::maybe_parse_ts_type_ann(parser);

    let body = Some(if parser.peek() == TokenKind::LBrace {
        parser.parse_block()
    } else {
        BlockStmt {
            span: Span::ZERO,
            stmts: Vec::new(),
        }
    });

    FnExpr {
        span: parser.span_since(start),
        id,
        params,
        body,
        generator,
        async_: false,
    }
}

fn parse_fn_params(parser: &mut Parser) -> Vec<Pat> {
    let mut params = Vec::new();
    while parser.peek() != TokenKind::RParen && !parser.is_eof() {
        if parser.peek() == TokenKind::DotDotDot {
            let rest = patterns::parse_rest_pat(parser);
            params.push(rest);
            break;
        }
        let pat = patterns::parse_binding_pat(parser);
        if parser.peek() == TokenKind::Eq {
            let _start = parser.current_pos();
            parser.advance();
            let init = expressions::parse_assign_expr(parser);
            let span = Span::new(pat.span().start, parser.expr_span(init).end);
            params.push(Pat::Assign(AssignPat {
                span,
                left: Box::new(pat),
                right: init,
            }));
        } else {
            params.push(pat);
        }
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    params
}

pub fn parse_fn_body(parser: &mut Parser, _params: &[TokenKind]) -> FnExpr {
    let start = parser.current_pos();
    let _type_params = super::typescript::maybe_parse_ts_type_params(parser);

    parser.expect(TokenKind::LParen).ok();
    let params = parse_fn_params(parser);
    parser.expect(TokenKind::RParen).ok();

    let _return_type = super::typescript::maybe_parse_ts_type_ann(parser);

    let body = parser.parse_block();

    FnExpr {
        span: parser.span_since(start),
        id: None,
        params,
        body: Some(body),
        generator: false,
        async_: false,
    }
}

pub fn parse_class_expr(parser: &mut Parser) -> ClassExpr {
    let start = parser.current_pos();
    parser.expect(TokenKind::Class).ok();

    let id = if parser.peek() == TokenKind::Ident {
        let tok = parser.advance();
        Some(Ident {
            span: tok.span,
            name: tok.value,
            optional: false,
        })
    } else {
        None
    };

    let super_class = if parser.peek() == TokenKind::Extends {
        parser.advance();
        let expr = expressions::parse_expr(parser, 17);
        Some(expr)
    } else {
        None
    };

    let body = parse_class_body(parser);

    ClassExpr {
        span: parser.span_since(start),
        id,
        super_class,
        body,
    }
}

fn parse_class_body(parser: &mut Parser) -> Vec<ClassMember> {
    let mut members = Vec::new();
    parser.expect(TokenKind::LBrace).ok();

    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Semicolon {
            parser.advance();
            continue;
        }
        let member = parse_class_member(parser);
        members.push(member);
    }

    parser.expect(TokenKind::RBrace).ok();
    members
}

fn parse_class_member(parser: &mut Parser) -> ClassMember {
    let start = parser.current_pos();

    let mut is_static = false;
    let mut decorators = Vec::new();

    loop {
        match parser.peek() {
            TokenKind::Static => {
                is_static = true;
                parser.advance();
            }
            TokenKind::Public
            | TokenKind::Protected
            | TokenKind::Private
            | TokenKind::Abstract
            | TokenKind::Readonly
            | TokenKind::Accessor => {
                parser.advance();
            }
            TokenKind::At => {
                decorators.append(&mut super::expressions::parse_decorators(parser));
            }
            _ => break,
        }
    }

    if is_static && parser.peek() == TokenKind::LBrace {
        let body = super::statements::parse_block(parser);
        return ClassMember::StaticBlock(StaticBlock {
            span: parser.span_since(start),
            body,
        });
    }

    let accessor_kind = match parser.peek() {
        TokenKind::Get if parser.peek_ahead(1) != TokenKind::LParen => {
            parser.advance();
            Some(MethodKind::Get)
        }
        TokenKind::Set if parser.peek_ahead(1) != TokenKind::LParen => {
            parser.advance();
            Some(MethodKind::Set)
        }
        _ => None,
    };

    if let Some(kind) = accessor_kind {
        let key = patterns::parse_prop_name(parser);
        let function = parse_fn_body(parser, &[]);
        return ClassMember::Method(MethodDef {
            span: parser.span_since(start),
            key,
            function,
            is_static,
            kind,
            decorators,
        });
    }

    if parser.peek() == TokenKind::Star {
        parser.advance();
        let key = patterns::parse_prop_name(parser);
        let mut fn_expr = parse_fn_body(parser, &[]);
        fn_expr.generator = true;
        return ClassMember::Method(MethodDef {
            span: parser.span_since(start),
            key,
            function: fn_expr,
            is_static,
            kind: MethodKind::Method,
            decorators,
        });
    }

    if parser.peek() == TokenKind::LBracket
        || parser.peek() == TokenKind::Ident
        || parser.peek() == TokenKind::PrivateName
        || parser.peek() == TokenKind::Number
        || parser.peek() == TokenKind::String
        || parser.peek() == TokenKind::BigInt
        || parser.peek().is_keyword()
    {
        let key = patterns::parse_prop_name(parser);

        if parser.peek() == TokenKind::LParen {
            let fn_expr = parse_fn_body(parser, &[]);

            if let PropName::Ident(ref id) = key {
                if id.name == "constructor" {
                    return ClassMember::Ctor(CtorDef {
                        span: parser.span_since(start),
                        params: fn_expr.params,
                        body: fn_expr.body,
                    });
                }
            }

            let method_kind = if let PropName::Ident(ref id) = key {
                match id.name.as_str() {
                    "get" => MethodKind::Get,
                    "set" => MethodKind::Set,
                    _ => MethodKind::Method,
                }
            } else {
                MethodKind::Method
            };

            return ClassMember::Method(MethodDef {
                span: parser.span_since(start),
                key,
                function: fn_expr,
                is_static,
                kind: method_kind,
                decorators,
            });
        }

        let _type_ann = super::typescript::maybe_parse_ts_type_ann(parser);
        let value = if parser.peek() == TokenKind::Eq {
            parser.advance();
            Some(expressions::parse_assign_expr(parser))
        } else {
            None
        };
        super::recovery::expect_semicolon(parser);

        return ClassMember::Prop(ClassProp {
            span: parser.span_since(start),
            key,
            value,
            is_static,
            decorators,
        });
    }

    if parser.peek() == TokenKind::LBracket && parser.options.features.typescript {
        let member = super::typescript::parse_ts_index_sig(parser);
        return ClassMember::TSIndex(member);
    }

    let tok = parser.current_token().clone();
    let err = parser.error(DiagnosticCode::UnexpectedToken, &tok);
    parser.errors.push(err);
    parser.advance();
    ClassMember::Prop(ClassProp {
        span: tok.span,
        key: PropName::Ident(Ident {
            span: tok.span,
            name: String::new(),
            optional: false,
        }),
        value: None,
        is_static: false,
        decorators,
    })
}

// Span helper for Pat
pub trait PatSpan {
    fn span(&self) -> Span;
}

impl PatSpan for Pat {
    fn span(&self) -> Span {
        match self {
            Pat::Ident(bi) => bi.span,
            Pat::Object(op) => op.span,
            Pat::Array(ap) => ap.span,
            Pat::Rest(rp) => rp.span,
            Pat::Assign(ap) => ap.span,
            Pat::Expr(_) => Span::ZERO,
            Pat::Invalid(ip) => ip.span,
        }
    }
}
