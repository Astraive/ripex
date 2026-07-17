use super::state::Parser;
use crate::diagnostics::DiagnosticCode;
use crate::js::ast::*;
use crate::js::lexer::TokenKind;

pub fn parse_jsx_element(parser: &mut Parser) -> ExprRef {
    let _start = parser.current_pos();
    let elem = parse_jsx_element_inner(parser);
    parser.ast.alloc(Expr::JSXElement(elem))
}

fn parse_jsx_element_inner(parser: &mut Parser) -> JSXElement {
    let start = parser.current_pos();
    let opening = parse_jsx_opening(parser);
    let mut children = Vec::new();

    if !opening.self_closing {
        while !(parser.is_eof()
            || parser.peek() == TokenKind::Lt && parser.peek_ahead(1) == TokenKind::Slash)
        {
            if parser.peek() == TokenKind::RBrace || parser.peek() == TokenKind::Eof {
                break;
            }
            children.push(parse_jsx_child(parser));
        }

        let closing = if parser.peek() == TokenKind::Lt {
            Some(parse_jsx_closing(parser))
        } else {
            None
        };
        JSXElement {
            span: parser.span_since(start),
            opening,
            children,
            closing,
        }
    } else {
        JSXElement {
            span: parser.span_since(start),
            opening,
            children,
            closing: None,
        }
    }
}

fn parse_jsx_opening(parser: &mut Parser) -> JSXOpening {
    let start = parser.current_pos();
    parser.expect(TokenKind::Lt).ok();
    let name = parse_jsx_name(parser);
    let mut attrs = Vec::new();

    while parser.peek() != TokenKind::Gt && parser.peek() != TokenKind::Slash && !parser.is_eof() {
        if parser.peek() == TokenKind::LBrace {
            attrs.push(parse_jsx_spread_attr(parser));
        } else {
            attrs.push(parse_jsx_attr(parser));
        }
    }

    let self_closing = parser.peek() == TokenKind::Slash;
    if self_closing {
        parser.advance();
    }

    parser.expect(TokenKind::Gt).ok();

    JSXOpening {
        span: parser.span_since(start),
        name,
        attrs,
        self_closing,
    }
}

fn parse_jsx_closing(parser: &mut Parser) -> JSXClosing {
    let start = parser.current_pos();
    parser.expect(TokenKind::Lt).ok();
    parser.expect(TokenKind::Slash).ok();
    let name = parse_jsx_name(parser);
    parser.expect(TokenKind::Gt).ok();
    JSXClosing {
        span: parser.span_since(start),
        name,
    }
}

fn parse_jsx_name(parser: &mut Parser) -> JSXName {
    let start = parser.current_pos();
    match parser.peek() {
        TokenKind::Ident | TokenKind::This => {
            let tok = parser.advance();
            let id = JSXIdent {
                span: tok.span,
                name: tok.value,
            };

            if parser.peek() == TokenKind::Colon {
                parser.advance();
                let ns = id;
                let tok = parser.advance();
                let name = JSXIdent {
                    span: tok.span,
                    name: tok.value,
                };
                JSXName::Namespace(JSXNamespace {
                    span: parser.span_since(start),
                    namespace: ns,
                    name,
                })
            } else {
                let mut name = JSXName::Ident(id);
                while parser.peek() == TokenKind::Dot {
                    parser.advance();
                    let tok = parser.advance();
                    let prop = JSXIdent {
                        span: tok.span,
                        name: tok.value,
                    };
                    name = JSXName::Member(JSXMember {
                        span: parser.span_since(start),
                        object: Box::new(name),
                        property: prop,
                    });
                }
                name
            }
        }
        _ => {
            let tok = parser.current_token().clone();
            let err = parser.error(DiagnosticCode::InvalidJSX, &tok);
            parser.errors.push(err);
            parser.advance();
            JSXName::Ident(JSXIdent {
                span: tok.span,
                name: String::new(),
            })
        }
    }
}

fn parse_jsx_attr(parser: &mut Parser) -> JSXAttr {
    let start = parser.current_pos();
    let tok = parser.advance();
    let name = JSXName::Ident(JSXIdent {
        span: tok.span,
        name: tok.value,
    });

    let value = if parser.peek() == TokenKind::Eq {
        parser.advance();
        Some(parse_jsx_attr_val(parser))
    } else {
        None
    };

    JSXAttr::Attr(JSXAttribute {
        span: parser.span_since(start),
        name,
        value,
    })
}

fn parse_jsx_spread_attr(parser: &mut Parser) -> JSXAttr {
    let start = parser.current_pos();
    parser.advance();
    let arg = super::expressions::parse_assign_expr(parser);
    parser.expect(TokenKind::RBrace).ok();
    JSXAttr::Spread(SpreadExpr {
        span: parser.span_since(start),
        arg,
    })
}

fn parse_jsx_attr_val(parser: &mut Parser) -> JSXAttrVal {
    let _start = parser.current_pos();
    match parser.peek() {
        TokenKind::String => {
            let tok = parser.advance();
            JSXAttrVal::Str(StrLit {
                span: tok.span,
                value: tok.value,
                raw: String::new(),
            })
        }
        TokenKind::LBrace => {
            parser.advance();
            let expr = super::expressions::parse_assign_expr(parser);
            parser.expect(TokenKind::RBrace).ok();
            JSXAttrVal::Expr(expr)
        }
        TokenKind::Lt => {
            let elem = parse_jsx_element_inner(parser);
            JSXAttrVal::Element(elem)
        }
        _ => {
            let tok = parser.advance();
            JSXAttrVal::Str(StrLit {
                span: tok.span,
                value: tok.value,
                raw: String::new(),
            })
        }
    }
}

fn parse_jsx_child(parser: &mut Parser) -> JSXChild {
    match parser.peek() {
        TokenKind::LBrace => {
            parser.advance();
            let expr = super::expressions::parse_assign_expr(parser);
            parser.expect(TokenKind::RBrace).ok();
            JSXChild::Expr(expr)
        }
        TokenKind::Lt => {
            if parser.peek_ahead(1) == TokenKind::Slash {
                JSXChild::Text(String::new())
            } else if parser.peek_ahead(1) == TokenKind::Gt {
                let frag = parse_jsx_fragment_inner(parser);
                JSXChild::Fragment(frag)
            } else {
                let elem = parse_jsx_element_inner(parser);
                JSXChild::Element(elem)
            }
        }
        _ => {
            let mut text = String::new();
            while !parser.is_eof()
                && parser.peek() != TokenKind::LBrace
                && parser.peek() != TokenKind::Lt
            {
                let tok = parser.advance();
                text.push_str(&tok.value);
            }
            JSXChild::Text(text)
        }
    }
}

pub fn parse_jsx_fragment(parser: &mut Parser) -> ExprRef {
    let frag = parse_jsx_fragment_inner(parser);
    parser.ast.alloc(Expr::JSXFragment(frag))
}

fn parse_jsx_fragment_inner(parser: &mut Parser) -> JSXFragment {
    let start = parser.current_pos();
    parser.expect(TokenKind::Lt).ok();
    parser.expect(TokenKind::Gt).ok();

    let mut children = Vec::new();
    while !parser.is_eof() {
        if parser.peek() == TokenKind::Lt && parser.peek_ahead(1) == TokenKind::Slash {
            break;
        }
        children.push(parse_jsx_child(parser));
    }

    parser.expect(TokenKind::Lt).ok();
    parser.expect(TokenKind::Slash).ok();
    parser.expect(TokenKind::Gt).ok();

    JSXFragment {
        span: parser.span_since(start),
        children,
    }
}
