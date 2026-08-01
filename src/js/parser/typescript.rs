use crate::js::ast::*;
use crate::js::lexer::TokenKind;
use crate::span::Span;

use super::state::Parser;

// ---- Type parameter helpers ----

pub fn maybe_parse_ts_type_params(parser: &mut Parser) -> Option<Vec<TypeAnn>> {
    if !parser.options.features.typescript || parser.peek() != TokenKind::Lt {
        return None;
    }
    let saved = parser.current_pos();
    let result = try_parse_ts_type_params_inner(parser);
    if result.is_none() {
        parser.pos = saved;
    }
    result
}

fn try_parse_ts_type_params_inner(parser: &mut Parser) -> Option<Vec<TypeAnn>> {
    let mut params = Vec::new();
    parser.advance();
    loop {
        if parser.peek() == TokenKind::Gt {
            parser.advance();
            return Some(params);
        }
        if parser.is_eof() {
            return None;
        }
        let tok = parser.advance();
        if parser.peek() == TokenKind::Extends {
            parser.advance();
            let _constraint = parse_ts_type(parser);
        }
        if parser.peek() == TokenKind::Eq {
            parser.advance();
            let _default = parse_ts_type(parser);
        }
        params.push(TypeAnn::Ident(Ident {
            span: tok.span,
            name: tok.value,
            optional: false,
        }));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        }
    }
}

// ---- Type argument parsing ----

pub fn try_parse_ts_type_args(parser: &mut Parser) -> Option<Vec<TypeAnn>> {
    if !parser.options.features.typescript || parser.peek() != TokenKind::Lt {
        return None;
    }
    let saved = parser.current_pos();
    parser.advance();
    let mut args = Vec::new();
    let mut count = 0;
    loop {
        if parser.peek() == TokenKind::Gt {
            parser.advance();
            return Some(args);
        }
        if parser.peek() == TokenKind::Eof || count > 10 {
            parser.pos = saved;
            return None;
        }
        let ta = parse_ts_type(parser);
        args.push(ta);
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        }
        count += 1;
    }
}

// ---- Type annotation ----

pub fn maybe_parse_ts_type_ann(parser: &mut Parser) -> Option<TypeAnn> {
    if !parser.options.features.typescript || parser.peek() != TokenKind::Colon {
        return None;
    }
    parser.advance();
    Some(parse_ts_type(parser))
}

// ---- Type expression parsing ----

pub fn parse_ts_type(parser: &mut Parser) -> TypeAnn {
    parse_ts_union_type(parser)
}

fn parse_ts_union_type(parser: &mut Parser) -> TypeAnn {
    let mut types = vec![parse_ts_intersection_type(parser)];
    while parser.peek() == TokenKind::Pipe {
        parser.advance();
        types.push(parse_ts_intersection_type(parser));
    }
    if types.len() == 1 {
        types.into_iter().next().unwrap()
    } else {
        TypeAnn::Union(types)
    }
}

fn parse_ts_intersection_type(parser: &mut Parser) -> TypeAnn {
    let mut types = vec![parse_ts_prefix_type(parser)];
    while parser.peek() == TokenKind::Ampersand {
        parser.advance();
        types.push(parse_ts_prefix_type(parser));
    }
    if types.len() == 1 {
        types.into_iter().next().unwrap()
    } else {
        TypeAnn::Intersection(types)
    }
}

fn parse_ts_prefix_type(parser: &mut Parser) -> TypeAnn {
    let _start = parser.current_pos();

    match parser.peek() {
        TokenKind::Readonly => {
            parser.advance();
            let inner = parse_ts_prefix_type(parser);
            return TypeAnn::Readonly(Box::new(inner));
        }
        TokenKind::KeyOf => {
            parser.advance();
            let inner = parse_ts_prefix_type(parser);
            return TypeAnn::KeyOf(Box::new(inner));
        }
        TokenKind::Unique | TokenKind::Symbol => {
            parser.advance();
            let inner = parse_ts_prefix_type(parser);
            return TypeAnn::Readonly(Box::new(inner));
        }
        _ => {}
    }

    parse_ts_atom_type(parser)
}

fn parse_ts_atom_type(parser: &mut Parser) -> TypeAnn {
    let _start = parser.current_pos();

    match parser.peek() {
        TokenKind::LParen => {
            parser.advance();
            if parser.peek() == TokenKind::RParen && parser.peek_ahead(1) == TokenKind::FatArrow {
                parser.advance();
                parser.advance();
                let return_type = Box::new(parse_ts_type(parser));
                return TypeAnn::Fn(Vec::new(), return_type);
            }
            let inner = parse_ts_type(parser);
            parser.expect(TokenKind::RParen).ok();
            if parser.peek() == TokenKind::FatArrow {
                parser.advance();
                let return_type = Box::new(parse_ts_type(parser));
                return TypeAnn::Fn(vec![inner], return_type);
            }
            return TypeAnn::Paren(Box::new(inner));
        }
        TokenKind::LBrace => {
            return parse_ts_object_type(parser);
        }
        TokenKind::LBracket => {
            return parse_ts_tuple_type(parser);
        }
        TokenKind::New => {
            parser.advance();
            parser.expect(TokenKind::LParen).ok();
            let mut params = Vec::new();
            while parser.peek() != TokenKind::RParen && !parser.is_eof() {
                let before = parser.current_pos();
                params.push(parse_ts_type(parser));
                if parser.peek() == TokenKind::Comma {
                    parser.advance();
                }
                if parser.current_pos() == before && !parser.is_eof() {
                    parser.advance();
                }
            }
            parser.expect(TokenKind::RParen).ok();
            parser.expect(TokenKind::FatArrow).ok();
            let return_type = Box::new(parse_ts_type(parser));
            return TypeAnn::Fn(params, return_type);
        }
        TokenKind::Typeof => {
            parser.advance();
            let tok = parser.advance();
            return TypeAnn::Typeof(Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            });
        }
        TokenKind::Infer => {
            parser.advance();
            let tok = parser.advance();
            return TypeAnn::Infer(Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            });
        }
        TokenKind::This => {
            let tok = parser.advance();
            return TypeAnn::This(tok.span);
        }
        TokenKind::Void => {
            let tok = parser.advance();
            return TypeAnn::Void(tok.span);
        }
        TokenKind::Undefined => {
            let tok = parser.advance();
            return TypeAnn::Undefined(tok.span);
        }
        TokenKind::Null => {
            let tok = parser.advance();
            return TypeAnn::Null(tok.span);
        }
        TokenKind::Never => {
            let tok = parser.advance();
            return TypeAnn::Never(tok.span);
        }
        TokenKind::Any => {
            let tok = parser.advance();
            return TypeAnn::Any(tok.span);
        }
        TokenKind::Unknown => {
            let tok = parser.advance();
            return TypeAnn::Unknown(tok.span);
        }
        TokenKind::Boolean => {
            let tok = parser.advance();
            return TypeAnn::Boolean(tok.span);
        }
        TokenKind::Number => {
            let tok = parser.advance();
            return TypeAnn::Number(tok.span);
        }
        TokenKind::StringLocal => {
            let tok = parser.advance();
            return TypeAnn::String(tok.span);
        }
        TokenKind::Symbol => {
            let tok = parser.advance();
            return TypeAnn::Symbol(tok.span);
        }
        TokenKind::BigInt => {
            let tok = parser.advance();
            return TypeAnn::BigInt(tok.span);
        }
        TokenKind::True => {
            let tok = parser.advance();
            return TypeAnn::LitBool(BoolLit {
                span: tok.span,
                value: true,
            });
        }
        TokenKind::False => {
            let tok = parser.advance();
            return TypeAnn::LitBool(BoolLit {
                span: tok.span,
                value: false,
            });
        }
        TokenKind::String => {
            let tok = parser.advance();
            return TypeAnn::Lit(StrLit {
                span: tok.span,
                value: tok.value,
                raw: String::new(),
            });
        }
        _ => {}
    }

    if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
        let tok = parser.advance();
        let name = tok.value;
        let mut type_args = None;

        if parser.peek() == TokenKind::Lt && parser.options.features.typescript {
            type_args = try_parse_ts_type_args(parser);
        }

        let ident = Ident {
            span: tok.span,
            name,
            optional: false,
        };

        let mut ty = if let Some(args) = type_args {
            TypeAnn::Generic(ident, args)
        } else {
            TypeAnn::Ident(ident)
        };

        while parser.peek() == TokenKind::Dot {
            parser.advance();
            let prop_tok = parser.advance();
            let prop_ident = Ident {
                span: prop_tok.span,
                name: prop_tok.value,
                optional: false,
            };
            ty = TypeAnn::Member(Box::new(ty), prop_ident);
        }

        while parser.peek() == TokenKind::LBracket {
            parser.advance();
            let index_type = parse_ts_type(parser);
            parser.expect(TokenKind::RBracket).ok();
            ty = TypeAnn::Indexed(Box::new(ty), Box::new(index_type));
        }

        return ty;
    }

    TypeAnn::Any(Span::ZERO)
}

fn parse_ts_object_type(parser: &mut Parser) -> TypeAnn {
    let start = parser.current_pos();
    parser.advance();
    let mut members = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Semicolon || parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        match parser.peek() {
            TokenKind::LBracket => {
                let _index = parse_ts_index_sig(parser);
            }
            _ => {
                if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
                    let _tok = parser.advance();
                    if parser.peek() == TokenKind::LParen {
                        parser.expect(TokenKind::LParen).ok();
                        let mut params = Vec::new();
                        while parser.peek() != TokenKind::RParen && !parser.is_eof() {
                            let before = parser.current_pos();
                            params.push(parse_ts_type(parser));
                            if parser.peek() == TokenKind::Comma {
                                parser.advance();
                            }
                            if parser.current_pos() == before && !parser.is_eof() {
                                parser.advance();
                            }
                        }
                        parser.expect(TokenKind::RParen).ok();
                        parser.expect(TokenKind::Colon).ok();
                        let _return_type = Box::new(parse_ts_type(parser));
                        members.push(());
                    } else {
                        let _optional = parser.peek() == TokenKind::Question;
                        if _optional {
                            parser.advance();
                        }
                        parser.expect(TokenKind::Colon).ok();
                        let _type_ann = parse_ts_type(parser);
                        members.push(());
                    }
                }
            }
        }
        if parser.peek() == TokenKind::Semicolon || parser.peek() == TokenKind::Comma {
            parser.advance();
        }
        // Forward-progress guard: if we reached here without consuming a token
        // (e.g. a member keyword the parser doesn't recognize, such as
        // `readonly`), advance so the loop cannot spin forever.
        if parser.peek() != TokenKind::RBrace && !parser.is_eof() {
            parser.advance();
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    TypeAnn::Object(parser.span_since(start))
}

fn parse_ts_tuple_type(parser: &mut Parser) -> TypeAnn {
    let _start = parser.current_pos();
    parser.advance();
    let mut elements = Vec::new();
    while parser.peek() != TokenKind::RBracket && !parser.is_eof() {
        let elem = parse_ts_type(parser);
        elements.push(elem);
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBracket).ok();
    TypeAnn::Tuple(elements)
}

pub fn parse_ts_index_sig(parser: &mut Parser) -> TsIndexSig {
    let start = parser.current_pos();
    parser.expect(TokenKind::LBracket).ok();
    let key_tok = parser.advance();
    parser.expect(TokenKind::Colon).ok();
    let value_type = parse_ts_type(parser);
    parser.expect(TokenKind::RBracket).ok();
    parser.expect(TokenKind::Colon).ok();
    let return_type = parse_ts_type(parser);

    let key_pat = Pat::Ident(BindingIdent {
        span: key_tok.span,
        id: Ident {
            span: key_tok.span,
            name: key_tok.value,
            optional: false,
        },
        type_ann: Some(value_type),
        optional: false,
    });

    TsIndexSig {
        span: parser.span_since(start),
        key: Box::new(key_pat),
        value: Box::new(return_type),
    }
}

// ---- TS declarations ----

pub fn parse_ts_type_alias(parser: &mut Parser) -> TsTypeAliasDecl {
    let start = parser.current_pos();
    parser.advance();
    let tok = parser.advance();
    let id = Ident {
        span: tok.span,
        name: tok.value,
        optional: false,
    };
    let _type_params = maybe_parse_ts_type_params(parser);
    parser.expect(TokenKind::Eq).ok();
    let type_ann = parse_ts_type(parser);
    super::recovery::expect_semicolon(parser);
    TsTypeAliasDecl {
        span: parser.span_since(start),
        id,
        type_ann,
    }
}

pub fn parse_ts_interface(parser: &mut Parser) -> TsInterfaceDecl {
    let start = parser.current_pos();
    parser.advance();
    let tok = parser.advance();
    let id = Ident {
        span: tok.span,
        name: tok.value,
        optional: false,
    };
    let _type_params = maybe_parse_ts_type_params(parser);

    let mut extends = Vec::new();
    if parser.peek() == TokenKind::Extends {
        parser.advance();
        loop {
            let ty = parse_ts_type(parser);
            extends.push(ty);
            if parser.peek() == TokenKind::Comma {
                parser.advance();
            } else {
                break;
            }
        }
    }

    let body = if parser.peek() == TokenKind::LBrace {
        parser.advance();
        let mut members = Vec::new();
        while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
            if parser.peek() == TokenKind::Semicolon {
                parser.advance();
                continue;
            }
            let ms = parser.current_pos();
            match parser.peek() {
                TokenKind::LBracket => {
                    let _index = parse_ts_index_sig(parser);
                }
                TokenKind::Readonly => {
                    // `readonly` member modifier — consume it so the loop makes
                    // forward progress (otherwise it spins on `readonly`).
                    parser.advance();
                }
                _ => {
                    if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
                        let tok = parser.advance();
                        let optional = parser.peek() == TokenKind::Question;
                        if optional {
                            parser.advance();
                        }
                        parser.expect(TokenKind::Colon).ok();
                        let value = parse_ts_type(parser);
                        members.push(TsInterfaceBody {
                            span: parser.span_since(ms),
                            key: PropName::Ident(Ident {
                                span: tok.span,
                                name: tok.value,
                                optional: false,
                            }),
                            value,
                            optional,
                            readonly: false,
                        });
                    }
                }
            }
            if parser.peek() == TokenKind::Semicolon {
                parser.advance();
            }
            // Forward-progress guard: if nothing above consumed a token
            // (unexpected token inside the interface body), advance so the
            // loop can't spin forever and OOM.
            if parser.peek() != TokenKind::RBrace && !parser.is_eof() {
                parser.advance();
            }
        }
        parser.expect(TokenKind::RBrace).ok();
        members
    } else {
        Vec::new()
    };

    TsInterfaceDecl {
        span: parser.span_since(start),
        id,
        extends,
        body,
    }
}

pub fn parse_ts_enum(parser: &mut Parser) -> TsEnumDecl {
    let start = parser.current_pos();
    parser.advance();
    let tok = parser.advance();
    let id = Ident {
        span: tok.span,
        name: tok.value,
        optional: false,
    };
    parser.expect(TokenKind::LBrace).ok();
    let mut members = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        let tok = parser.advance();
        let init = if parser.peek() == TokenKind::Eq {
            parser.advance();
            Some(super::expressions::parse_assign_expr(parser))
        } else {
            None
        };
        members.push(TsEnumMember {
            span: tok.span,
            id: Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            },
            init,
        });
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    TsEnumDecl {
        span: parser.span_since(start),
        id,
        members,
        is_const: false,
    }
}

pub fn parse_ts_module(parser: &mut Parser) -> TsModuleDecl {
    let start = parser.current_pos();
    parser.advance();

    let is_namespace = parser.peek() == TokenKind::Namespace;
    if is_namespace {
        parser.advance();
    }

    let tok = parser.advance();
    let id = Ident {
        span: tok.span,
        name: tok.value,
        optional: false,
    };

    let body = if parser.peek() == TokenKind::LBrace {
        parser.advance();
        let mut stmts = Vec::new();
        while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
            if let Some(stmt) = super::statements::parse_stmt(parser) {
                stmts.push(stmt);
            }
        }
        parser.expect(TokenKind::RBrace).ok();
        stmts
    } else {
        super::recovery::expect_semicolon(parser);
        Vec::new()
    };

    TsModuleDecl {
        span: parser.span_since(start),
        id,
        body,
        is_namespace,
    }
}

// ---- TS expression extensions ----

pub fn parse_ts_as_expr(parser: &mut Parser, left: ExprRef) -> ExprRef {
    parser.advance();
    let type_ann = Box::new(parse_ts_type(parser));
    let span = Span::new(parser.expr_span(left).start, type_ann.span().end);
    parser.ast.alloc(Expr::TSAs(TSAsExpr {
        span,
        expr: left,
        type_ann,
    }))
}

pub fn parse_ts_satisfies_expr(parser: &mut Parser, left: ExprRef) -> ExprRef {
    parser.advance();
    let type_ann = Box::new(parse_ts_type(parser));
    let span = Span::new(parser.expr_span(left).start, type_ann.span().end);
    parser.ast.alloc(Expr::TSSatisfies(TSSatisfiesExpr {
        span,
        expr: left,
        type_ann,
    }))
}

use crate::js::ast::node::AstNode;

impl AstNode for TypeAnn {
    fn span(&self) -> Span {
        match self {
            TypeAnn::Any(s)
            | TypeAnn::String(s)
            | TypeAnn::Number(s)
            | TypeAnn::Boolean(s)
            | TypeAnn::Void(s)
            | TypeAnn::Never(s)
            | TypeAnn::Unknown(s)
            | TypeAnn::Null(s)
            | TypeAnn::Undefined(s)
            | TypeAnn::Object(s)
            | TypeAnn::Symbol(s)
            | TypeAnn::BigInt(s)
            | TypeAnn::This(s)
            | TypeAnn::TsNull(s) => *s,
            TypeAnn::Ident(id) => id.span(),
            TypeAnn::Array(inner) => inner.span(),
            TypeAnn::Union(types) | TypeAnn::Intersection(types) => {
                types.first().map(|t| t.span()).unwrap_or(Span::ZERO)
            }
            TypeAnn::Fn(params, _) => params.first().map(|t| t.span()).unwrap_or(Span::ZERO),
            TypeAnn::Lit(l) => l.span,
            TypeAnn::LitNum(l) => l.span,
            TypeAnn::LitBool(l) => l.span,
            TypeAnn::Generic(id, _) => id.span(),
            TypeAnn::Tuple(types) => types.first().map(|t| t.span()).unwrap_or(Span::ZERO),
            TypeAnn::Rest(inner)
            | TypeAnn::Optional(inner)
            | TypeAnn::Readonly(inner)
            | TypeAnn::KeyOf(inner)
            | TypeAnn::Paren(inner) => inner.span(),
            TypeAnn::Typeof(id) | TypeAnn::Infer(id) => id.span(),
            TypeAnn::Member(obj, prop) => Span::new(obj.span().start, prop.span().end),
            TypeAnn::Mapped(id, _) => id.span(),
            TypeAnn::Conditional(cons, _, _, alt) => Span::new(cons.span().start, alt.span().end),
            TypeAnn::Pred(_, inner) => inner.span(),
            TypeAnn::Indexed(obj, index) => Span::new(obj.span().start, index.span().end),
        }
    }
}
