use crate::diagnostics::DiagnosticCode;
use crate::js::ast::*;
use crate::js::lexer::TokenKind;
use crate::span::Span;

use super::state::Parser;

pub fn parse_pat(parser: &mut Parser) -> Pat {
    parse_binding_pat(parser)
}

pub fn parse_binding_pat(parser: &mut Parser) -> Pat {
    let start = parser.current_pos();
    match parser.peek() {
        TokenKind::LBrace => parse_object_pat(parser),
        TokenKind::LBracket => parse_array_pat(parser),
        TokenKind::DotDotDot => parse_rest_pat(parser),
        _ => {
            if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
                let ident = parse_binding_ident(parser);
                if parser.peek() == TokenKind::Colon {
                    let tok = parser.current_token().clone();
                    let err = parser.error(DiagnosticCode::UnexpectedToken, &tok);
                    parser.errors.push(err);
                    parser.advance();
                    let _inner = parse_binding_pat(parser);
                    let _span = parser.span_since(start);
                    if let Pat::Ident(bi) = ident {
                        Pat::Object(ObjectPat {
                            span: bi.span,
                            props: vec![ObjectPatProp::Shorthand(bi)],
                            rest: None,
                        })
                    } else {
                        ident
                    }
                } else {
                    ident
                }
            } else {
                let tok = parser.current_token().clone();
                let err = parser.error(DiagnosticCode::UnexpectedToken, &tok);
                parser.errors.push(err);
                parser.advance();
                Pat::Ident(BindingIdent {
                    span: tok.span,
                    id: Ident {
                        span: tok.span,
                        name: String::new(),
                        optional: false,
                    },
                    type_ann: None,
                    optional: false,
                })
            }
        }
    }
}

pub fn parse_assignment_pat(parser: &mut Parser) -> Pat {
    match parser.peek() {
        TokenKind::LBrace => parse_object_pat(parser),
        TokenKind::LBracket => parse_array_pat(parser),
        _ => {
            if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
                let ident = parse_binding_ident(parser);
                if parser.peek() == TokenKind::Eq {
                    parse_assign_pat(parser, ident)
                } else {
                    ident
                }
            } else {
                let expr = parser.parse_expr();
                Pat::Expr(expr)
            }
        }
    }
}

pub fn parse_binding_ident(parser: &mut Parser) -> Pat {
    let _start = parser.current_pos();
    let tok = parser.advance();
    let name = tok.value.clone();
    let ident = Ident {
        span: tok.span,
        name,
        optional: false,
    };
    let type_ann = super::typescript::maybe_parse_ts_type_ann(parser);
    let span = if let Some(ref ta) = type_ann {
        Span::new(tok.span.start, ta.span().end)
    } else {
        tok.span
    };
    Pat::Ident(BindingIdent {
        span,
        id: ident,
        type_ann,
        optional: false,
    })
}

pub fn parse_object_pat(parser: &mut Parser) -> Pat {
    let start = parser.current_pos();
    parser.expect(TokenKind::LBrace).ok();
    let mut props = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        if parser.peek() == TokenKind::DotDotDot {
            let rest = parse_rest_pat(parser);
            if let Pat::Rest(r) = rest {
                props.push(ObjectPatProp::Rest(r));
            }
            break;
        }
        props.push(parse_obj_pat_prop(parser));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    let span = parser.span_since(start);
    Pat::Object(ObjectPat {
        span,
        props,
        rest: None,
    })
}

fn parse_obj_pat_prop(parser: &mut Parser) -> ObjectPatProp {
    let start = parser.current_pos();
    if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
        let ident = parse_ident(parser);
        if parser.peek() == TokenKind::Colon {
            parser.advance();
            let value = Box::new(parse_binding_pat(parser));
            let span = parser.span_since(start);
            let key = PropName::Ident(ident.clone());
            ObjectPatProp::KeyValue(KeyValuePatProp { span, key, value })
        } else if parser.peek() == TokenKind::Eq {
            parser.advance();
            let right = parser.parse_assignment_expr();
            let binding = Pat::Ident(BindingIdent {
                span: ident.span,
                id: ident.clone(),
                type_ann: None,
                optional: false,
            });
            let value = Pat::Assign(AssignPat {
                span: parser.span_since(start),
                left: Box::new(binding),
                right,
            });
            ObjectPatProp::KeyValue(KeyValuePatProp {
                span: parser.span_since(start),
                key: PropName::Ident(ident),
                value: Box::new(value),
            })
        } else {
            let type_ann = super::typescript::maybe_parse_ts_type_ann(parser);
            let span = if type_ann.is_some() {
                parser.span_since(start)
            } else {
                ident.span
            };
            ObjectPatProp::Shorthand(BindingIdent {
                span,
                id: ident,
                type_ann,
                optional: false,
            })
        }
    } else {
        let tok = parser.current_token().clone();
        let err = parser.error(DiagnosticCode::UnexpectedToken, &tok);
        parser.errors.push(err);
        parser.advance();
        ObjectPatProp::Shorthand(BindingIdent {
            span: tok.span,
            id: Ident {
                span: tok.span,
                name: String::new(),
                optional: false,
            },
            type_ann: None,
            optional: false,
        })
    }
}

pub fn parse_array_pat(parser: &mut Parser) -> Pat {
    let start = parser.current_pos();
    parser.expect(TokenKind::LBracket).ok();
    let mut elements = Vec::new();
    let mut rest = None;
    while parser.peek() != TokenKind::RBracket && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            elements.push(None);
            parser.advance();
            continue;
        }
        if parser.peek() == TokenKind::DotDotDot {
            if let Pat::Rest(rp) = parse_rest_pat(parser) {
                rest = Some(Box::new(rp));
            }
            break;
        }
        elements.push(Some(parse_binding_pat(parser)));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBracket).ok();
    let span = parser.span_since(start);
    Pat::Array(ArrayPat {
        span,
        elements,
        rest,
    })
}

pub fn parse_rest_pat(parser: &mut Parser) -> Pat {
    let start = parser.current_pos();
    parser.expect(TokenKind::DotDotDot).ok();
    let arg = Box::new(parse_binding_pat(parser));
    let span = parser.span_since(start);
    Pat::Rest(RestPat { span, arg })
}

pub fn parse_assign_pat(parser: &mut Parser, left: Pat) -> Pat {
    let start = parser.current_pos();
    parser.expect(TokenKind::Eq).ok();
    let right = parser.parse_assignment_expr();
    let span = parser.span_since(start);
    Pat::Assign(AssignPat {
        span,
        left: Box::new(left),
        right,
    })
}

pub fn parse_ident(parser: &mut Parser) -> Ident {
    let tok = parser.advance();
    Ident {
        span: tok.span,
        name: tok.value,
        optional: false,
    }
}

pub fn convert_expr_to_pat(parser: &mut Parser, expr: ExprRef, _span: Span) -> Pat {
    match parser.ast[expr].clone() {
        Expr::Ident(id) => Pat::Ident(BindingIdent {
            span: id.span,
            id,
            type_ann: None,
            optional: false,
        }),
        _ => Pat::Expr(expr),
    }
}

pub fn parse_prop_name(parser: &mut Parser) -> PropName {
    let _start = parser.current_pos();
    match parser.peek() {
        TokenKind::String => {
            let tok = parser.advance();
            PropName::Str(StrLit {
                span: tok.span,
                value: tok.value,
                raw: String::new(),
            })
        }
        TokenKind::Number | TokenKind::BigInt => {
            let tok = parser.advance();
            let val: f64 = tok.value.parse().unwrap_or(0.0);
            PropName::Num(NumLit {
                span: tok.span,
                value: val,
                raw: tok.value,
            })
        }
        TokenKind::LBracket => {
            parser.advance();
            let expr = parser.parse_assignment_expr();
            parser.expect(TokenKind::RBracket).ok();
            PropName::Computed(expr)
        }
        TokenKind::PrivateName => {
            let tok = parser.advance();
            PropName::Ident(Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            })
        }
        _ => {
            let tok = parser.advance();
            PropName::Ident(Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            })
        }
    }
}
