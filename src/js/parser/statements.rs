use crate::js::ast::*;
use crate::js::lexer::TokenKind;
use crate::span::Span;

use super::declarations;
use super::declarations::PatSpan;
use super::expressions;
use super::patterns;
use super::recovery;
use super::state::Parser;

pub fn parse_stmt(parser: &mut Parser) -> Option<Stmt> {
    Some(match parser.peek() {
        TokenKind::LBrace => Stmt::Block(parse_block(parser)),
        TokenKind::Semicolon => {
            parser.advance();
            Stmt::Empty(EmptyStmt {
                span: parser
                    .previous_token()
                    .map(|t| t.span)
                    .unwrap_or(Span::ZERO),
            })
        }
        TokenKind::If => Stmt::If(parse_if(parser)),
        TokenKind::Switch => Stmt::Switch(parse_switch(parser)),
        TokenKind::For => parse_for(parser),
        TokenKind::While => Stmt::While(parse_while(parser)),
        TokenKind::Do => Stmt::DoWhile(parse_do_while(parser)),
        TokenKind::Break => Stmt::Break(parse_break(parser)),
        TokenKind::Continue => Stmt::Continue(parse_continue(parser)),
        TokenKind::Return => Stmt::Return(parse_return(parser)),
        TokenKind::Throw => Stmt::Throw(parse_throw(parser)),
        TokenKind::Try => Stmt::Try(parse_try(parser)),
        TokenKind::Debugger => Stmt::Debugger(parse_debugger(parser)),
        TokenKind::With => Stmt::With(parse_with(parser)),
        TokenKind::Var | TokenKind::Let | TokenKind::Const => declarations::parse_var_stmt(parser),
        TokenKind::Using if parser.options.features.explicit_resource_management => {
            declarations::parse_using_stmt(parser, false)
        }
        TokenKind::Function => Stmt::Decl(Decl::Fn(declarations::parse_fn_decl(parser))),
        TokenKind::Async => {
            if parser.peek_ahead(1) == TokenKind::Function {
                parser.advance();
                Stmt::Decl(Decl::Fn(declarations::parse_async_fn_decl(parser)))
            } else {
                Stmt::Expr(parse_expr_stmt(parser))
            }
        }
        TokenKind::Class => Stmt::Decl(Decl::Class(declarations::parse_class_decl(parser))),
        TokenKind::At => {
            // Stage-3 decorators precede a class or function declaration.
            // Consume the `@expr` decorators, then dispatch on the
            // declaration keyword that follows them.
            let decorators = super::expressions::parse_decorators(parser);
            match parser.peek() {
                TokenKind::Class => Stmt::Decl(Decl::Class(
                    declarations::parse_class_decl_with_decorators(parser, decorators),
                )),
                TokenKind::Function => Stmt::Decl(Decl::Fn(
                    declarations::parse_fn_decl_with_decorators(parser, decorators),
                )),
                _ => {
                    // Not a known decorated declaration; recover.
                    Stmt::Expr(parse_expr_stmt(parser))
                }
            }
        }
        TokenKind::Interface if parser.options.features.typescript => Stmt::Decl(
            Decl::TsInterface(super::typescript::parse_ts_interface(parser)),
        ),
        TokenKind::Type if parser.options.features.typescript => Stmt::Decl(Decl::TsTypeAlias(
            super::typescript::parse_ts_type_alias(parser),
        )),
        TokenKind::Enum if parser.options.features.typescript => {
            Stmt::Decl(Decl::TsEnum(super::typescript::parse_ts_enum(parser)))
        }
        TokenKind::Module | TokenKind::Namespace if parser.options.features.typescript => {
            Stmt::Decl(Decl::TsModule(super::typescript::parse_ts_module(parser)))
        }
        TokenKind::Abstract if parser.options.features.typescript => {
            parser.advance();
            if parser.peek() == TokenKind::Class {
                Stmt::Decl(Decl::Class(declarations::parse_class_decl(parser)))
            } else {
                Stmt::Expr(parse_expr_stmt(parser))
            }
        }
        TokenKind::Declare if parser.options.features.typescript => {
            parser.advance();
            parse_stmt(parser)?
        }
        TokenKind::Await
            if parser.peek_ahead(1) == TokenKind::Using
                && parser.options.features.explicit_resource_management =>
        {
            parser.advance();
            declarations::parse_using_stmt(parser, true)
        }
        _ => {
            if parser.peek() == TokenKind::Eof {
                return None;
            }
            let start = parser.current_pos();
            let expr = expressions::parse_expr(parser, 0);
            if parser.peek() == TokenKind::Colon && !parser.current_token().has_line_break {
                let label = if let Expr::Ident(ref id) = parser.ast[expr] {
                    Some(id.clone())
                } else {
                    None
                };
                if let Some(label) = label {
                    parser.advance();
                    let body = parse_stmt(parser);
                    return Some(Stmt::Labelled(LabelledStmt {
                        span: parser.span_since(start),
                        label,
                        body: Box::new(body.unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO }))),
                    }));
                }
            }
            recovery::expect_semicolon(parser);
            Stmt::Expr(ExprStmt {
                span: parser.span_since(start),
                expr,
            })
        }
    })
}

fn parse_expr_stmt(parser: &mut Parser) -> ExprStmt {
    let start = parser.current_pos();
    let expr = expressions::parse_expr(parser, 0);
    recovery::expect_semicolon(parser);
    ExprStmt {
        span: parser.span_since(start),
        expr,
    }
}

pub fn parse_block(parser: &mut Parser) -> BlockStmt {
    let start = parser.current_pos();
    parser.expect(TokenKind::LBrace).ok();
    let mut stmts = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if let Some(stmt) = parse_stmt(parser) {
            stmts.push(stmt);
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    BlockStmt {
        span: parser.span_since(start),
        stmts,
    }
}

fn parse_if(parser: &mut Parser) -> IfStmt {
    let start = parser.current_pos();
    parser.advance();
    parser.expect(TokenKind::LParen).ok();
    let test = expressions::parse_expr(parser, 0);
    parser.expect(TokenKind::RParen).ok();
    let consequent =
        Box::new(parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })));
    let alternate = if parser.peek() == TokenKind::Else {
        parser.advance();
        Some(Box::new(
            parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })),
        ))
    } else {
        None
    };
    IfStmt {
        span: parser.span_since(start),
        test,
        consequent,
        alternate,
    }
}

fn parse_switch(parser: &mut Parser) -> SwitchStmt {
    let start = parser.current_pos();
    parser.advance();
    parser.expect(TokenKind::LParen).ok();
    let discriminant = expressions::parse_expr(parser, 0);
    parser.expect(TokenKind::RParen).ok();
    parser.expect(TokenKind::LBrace).ok();
    let mut cases = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Case || parser.peek() == TokenKind::Default {
            let case_start = parser.current_pos();
            let is_default = parser.peek() == TokenKind::Default;
            parser.advance();
            let test = if is_default {
                None
            } else {
                Some(expressions::parse_expr(parser, 0))
            };
            parser.expect(TokenKind::Colon).ok();
            let mut consequent = Vec::new();
            while parser.peek() != TokenKind::Case
                && parser.peek() != TokenKind::Default
                && parser.peek() != TokenKind::RBrace
                && !parser.is_eof()
            {
                if let Some(stmt) = parse_stmt(parser) {
                    consequent.push(stmt);
                } else {
                    break;
                }
            }
            cases.push(SwitchCase {
                span: parser.span_since(case_start),
                test,
                consequent,
            });
        } else {
            if let Some(stmt) = parse_stmt(parser) {
                if let Some(last) = cases.last_mut() {
                    last.consequent.push(stmt);
                }
            } else {
                break;
            }
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    SwitchStmt {
        span: parser.span_since(start),
        discriminant,
        cases,
    }
}

fn parse_for(parser: &mut Parser) -> Stmt {
    let start = parser.current_pos();
    parser.advance();
    parser.expect(TokenKind::LParen).ok();

    let init = if parser.peek() == TokenKind::Semicolon {
        None
    } else if parser.peek() == TokenKind::Var
        || parser.peek() == TokenKind::Let
        || parser.peek() == TokenKind::Const
    {
        let vk = match parser.peek() {
            TokenKind::Const => VarKind::Const,
            TokenKind::Let => VarKind::Let,
            _ => VarKind::Var,
        };
        parser.advance();
        let pat = patterns::parse_binding_pat(parser);
        if parser.peek() == TokenKind::In || parser.peek() == TokenKind::Of {
            let is_in = parser.peek() == TokenKind::In;
            let is_await = parser.peek() == TokenKind::Of && parser.in_async_ctx();
            parser.advance();
            let right = expressions::parse_expr(parser, 0);
            parser.expect(TokenKind::RParen).ok();
            let body =
                Box::new(parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })));
            let span = parser.span_since(start);
            let decl = VarDecl {
                span,
                kind: vk,
                decls: vec![VarDeclarator {
                    span: pat.span(),
                    name: pat,
                    init: None,
                }],
                await_: false,
            };
            if is_in {
                return Stmt::ForIn(ForInStmt {
                    span,
                    left: ForInit::Decl(Box::new(Decl::Var(decl))),
                    right,
                    body,
                });
            } else {
                return Stmt::ForOf(ForOfStmt {
                    span,
                    left: ForInit::Decl(Box::new(Decl::Var(decl))),
                    right,
                    body,
                    await_: is_await,
                });
            }
        }
        let init_val = if parser.peek() == TokenKind::Eq {
            parser.advance();
            Some(expressions::parse_assign_expr(parser))
        } else {
            None
        };
        let var_decl = VarDecl {
            span: parser.span_since(start),
            kind: vk,
            decls: vec![VarDeclarator {
                span: pat.span(),
                name: pat,
                init: init_val,
            }],
            await_: false,
        };
        Some(ForInit::Decl(Box::new(Decl::Var(var_decl))))
    } else {
        let _expr_start = parser.current_pos();
        let expr = expressions::parse_expr(parser, 0);
        if parser.peek() == TokenKind::In || parser.peek() == TokenKind::Of {
            let is_in = parser.peek() == TokenKind::In;
            let is_await = parser.peek() == TokenKind::Of && parser.in_async_ctx();
            parser.advance();
            let right = expressions::parse_expr(parser, 0);
            parser.expect(TokenKind::RParen).ok();
            let body =
                Box::new(parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })));
            let span = parser.span_since(start);
            if is_in {
                return Stmt::ForIn(ForInStmt {
                    span,
                    left: ForInit::Expr(expr),
                    right,
                    body,
                });
            } else {
                return Stmt::ForOf(ForOfStmt {
                    span,
                    left: ForInit::Expr(expr),
                    right,
                    body,
                    await_: is_await,
                });
            }
        }
        Some(ForInit::Expr(expr))
    };

    parser.expect(TokenKind::Semicolon).ok();

    let test = if parser.peek() != TokenKind::Semicolon && !parser.is_eof() {
        Some(expressions::parse_expr(parser, 0))
    } else {
        None
    };

    parser.expect(TokenKind::Semicolon).ok();

    let update = if parser.peek() != TokenKind::RParen && !parser.is_eof() {
        Some(expressions::parse_expr(parser, 0))
    } else {
        None
    };

    parser.expect(TokenKind::RParen).ok();

    let body = Box::new(parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })));

    Stmt::For(ForStmt {
        span: parser.span_since(start),
        init,
        test,
        update,
        body,
    })
}

fn parse_while(parser: &mut Parser) -> WhileStmt {
    let start = parser.current_pos();
    parser.advance();
    parser.expect(TokenKind::LParen).ok();
    let test = expressions::parse_expr(parser, 0);
    parser.expect(TokenKind::RParen).ok();
    let body = Box::new(parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })));
    WhileStmt {
        span: parser.span_since(start),
        test,
        body,
    }
}

fn parse_do_while(parser: &mut Parser) -> DoWhileStmt {
    let start = parser.current_pos();
    parser.advance();
    let body = Box::new(parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })));
    parser.expect(TokenKind::While).ok();
    parser.expect(TokenKind::LParen).ok();
    let test = expressions::parse_expr(parser, 0);
    parser.expect(TokenKind::RParen).ok();
    recovery::expect_semicolon(parser);
    DoWhileStmt {
        span: parser.span_since(start),
        test,
        body,
    }
}

fn parse_break(parser: &mut Parser) -> BreakStmt {
    let start = parser.current_pos();
    parser.advance();
    let label = if parser.peek() == TokenKind::Ident
        && !parser.current_token().has_line_break
        && !matches!(
            parser.peek(),
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof
        ) {
        let tok = parser.advance();
        Some(Ident {
            span: tok.span,
            name: tok.value,
            optional: false,
        })
    } else {
        None
    };
    recovery::expect_semicolon(parser);
    BreakStmt {
        span: parser.span_since(start),
        label,
    }
}

fn parse_continue(parser: &mut Parser) -> ContinueStmt {
    let start = parser.current_pos();
    parser.advance();
    let label = if parser.peek() == TokenKind::Ident
        && !parser.current_token().has_line_break
        && !matches!(
            parser.peek(),
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof
        ) {
        let tok = parser.advance();
        Some(Ident {
            span: tok.span,
            name: tok.value,
            optional: false,
        })
    } else {
        None
    };
    recovery::expect_semicolon(parser);
    ContinueStmt {
        span: parser.span_since(start),
        label,
    }
}

fn parse_return(parser: &mut Parser) -> ReturnStmt {
    let start = parser.current_pos();
    parser.advance();
    let arg = if !parser.is_eof()
        && !parser.current_token().has_line_break
        && parser.peek() != TokenKind::Semicolon
        && parser.peek() != TokenKind::RBrace
    {
        Some(expressions::parse_expr(parser, 0))
    } else {
        None
    };
    recovery::expect_semicolon(parser);
    ReturnStmt {
        span: parser.span_since(start),
        arg,
    }
}

fn parse_throw(parser: &mut Parser) -> ThrowStmt {
    let start = parser.current_pos();
    parser.advance();
    let arg = expressions::parse_expr(parser, 0);
    recovery::expect_semicolon(parser);
    ThrowStmt {
        span: parser.span_since(start),
        arg,
    }
}

fn parse_try(parser: &mut Parser) -> TryStmt {
    let start = parser.current_pos();
    parser.advance();
    let block = parse_block(parser);

    let handler = if parser.peek() == TokenKind::Catch {
        let catch_start = parser.current_pos();
        parser.advance();
        let param = if parser.peek() == TokenKind::LParen {
            parser.advance();
            let p = if parser.peek() != TokenKind::RParen {
                Some(patterns::parse_binding_pat(parser))
            } else {
                None
            };
            parser.expect(TokenKind::RParen).ok();
            p
        } else {
            None
        };
        let catch_body = parse_block(parser);
        Some(CatchClause {
            span: parser.span_since(catch_start),
            param,
            body: catch_body,
        })
    } else {
        None
    };

    let finalizer = if parser.peek() == TokenKind::Finally {
        parser.advance();
        Some(parse_block(parser))
    } else {
        None
    };

    TryStmt {
        span: parser.span_since(start),
        block,
        handler,
        finalizer,
    }
}

fn parse_debugger(parser: &mut Parser) -> DebuggerStmt {
    let start = parser.current_pos();
    parser.advance();
    recovery::expect_semicolon(parser);
    DebuggerStmt {
        span: parser.span_since(start),
    }
}

fn parse_with(parser: &mut Parser) -> WithStmt {
    let start = parser.current_pos();
    parser.advance();
    parser.expect(TokenKind::LParen).ok();
    let object = expressions::parse_expr(parser, 0);
    parser.expect(TokenKind::RParen).ok();
    let body = Box::new(parse_stmt(parser).unwrap_or(Stmt::Empty(EmptyStmt { span: Span::ZERO })));
    WithStmt {
        span: parser.span_since(start),
        object,
        body,
    }
}

// Span helper for Stmt
#[allow(dead_code)]
trait StmtSpan {
    fn span(&self) -> Span;
}

impl StmtSpan for Stmt {
    fn span(&self) -> Span {
        match self {
            Stmt::Block(b) => b.span,
            Stmt::Empty(e) => e.span,
            Stmt::Expr(e) => e.span,
            Stmt::If(i) => i.span,
            Stmt::Switch(s) => s.span,
            Stmt::For(f) => f.span,
            Stmt::ForIn(f) => f.span,
            Stmt::ForOf(f) => f.span,
            Stmt::While(w) => w.span,
            Stmt::DoWhile(d) => d.span,
            Stmt::Break(b) => b.span,
            Stmt::Continue(c) => c.span,
            Stmt::Return(r) => r.span,
            Stmt::Throw(t) => t.span,
            Stmt::Try(t) => t.span,
            Stmt::Debugger(d) => d.span,
            Stmt::Labelled(l) => l.span,
            Stmt::With(w) => w.span,
            Stmt::Decl(d) => match d {
                Decl::Var(v) => v.span,
                Decl::Fn(f) => f.span,
                Decl::Class(c) => c.span,
                Decl::TsInterface(i) => i.span,
                Decl::TsTypeAlias(t) => t.span,
                Decl::TsEnum(e) => e.span,
                Decl::TsModule(m) => m.span,
            },
        }
    }
}
