use crate::diagnostics::DiagnosticCode;
use crate::js::ast::*;
use crate::js::lexer::TokenKind;
use crate::span::Span;

use super::state::Parser;

/// Parses a leading list of stage-3 decorators `@expr` (optionally
/// `@expr(args)`), stopping when the next token is no longer `@`.
///
/// The decorator expression is a member path (`@a.b.c`, `@a(args)`) where each
/// name segment may be a keyword identifier (e.g. `@readonly`), matching
/// TypeScript's allowance of keywords as decorator targets.
pub(crate) fn parse_decorators(parser: &mut Parser) -> Vec<Decorator> {
    let mut decorators = Vec::new();
    while parser.peek() == TokenKind::At {
        let start = parser.current_pos();
        parser.advance(); // consume '@'
        let expr = parse_decorator_expr(parser);
        decorators.push(Decorator {
            span: parser.span_since(start),
            expr,
        });
    }
    decorators
}

/// Parses a single decorator expression: an identifier-or-keyword path joined
/// by `.`, optionally followed by a call `(args)`.
fn parse_decorator_expr(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    let tok = parser.advance();
    let mut node = parser.ast.alloc(Expr::Ident(Ident {
        span: tok.span,
        name: tok.value,
        optional: false,
    }));
    while parser.peek() == TokenKind::Dot {
        parser.advance();
        let prop_tok = parser.advance();
        let property = Box::new(Expr::Ident(Ident {
            span: prop_tok.span,
            name: prop_tok.value,
            optional: false,
        }));
        let span = Span::new(parser.ast[node].span().start, property.span().end);
        node = parser.ast.alloc(Expr::Member(MemberExpr {
            span,
            object: node,
            property,
            computed: false,
        }));
    }
    if parser.peek() == TokenKind::LParen {
        parser.advance();
        let mut args = Vec::new();
        while parser.peek() != TokenKind::RParen && !parser.is_eof() {
            args.push(parse_assign_expr(parser));
            if parser.peek() == TokenKind::Comma {
                parser.advance();
            } else {
                break;
            }
        }
        parser.expect(TokenKind::RParen).ok();
        let span = parser.span_since(start);
        node = parser.ast.alloc(Expr::Call(CallExpr {
            span,
            callee: node,
            args,
        }));
    }
    node
}

// ---- Precedence helpers ----

#[allow(dead_code)]
fn prefix_bp(kind: TokenKind) -> Option<u8> {
    match kind {
        TokenKind::Exclamation
        | TokenKind::Tilde
        | TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Typeof
        | TokenKind::Void
        | TokenKind::Delete => Some(15),
        TokenKind::PlusPlus | TokenKind::MinusMinus => Some(15),
        TokenKind::Await => Some(15),
        _ => None,
    }
}

fn infix_bp(kind: TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::Question => Some((2, 3)),
        TokenKind::QuestionQuestion => Some((3, 4)),
        TokenKind::PipeGt => Some((2, 3)),
        TokenKind::PipePipe => Some((4, 5)),
        TokenKind::AmpersandAmpersand => Some((5, 6)),
        TokenKind::Pipe => Some((6, 7)),
        TokenKind::Caret => Some((7, 8)),
        TokenKind::Ampersand => Some((8, 9)),
        TokenKind::EqEq | TokenKind::EqEqEq => Some((9, 10)),
        TokenKind::Lt
        | TokenKind::Gt
        | TokenKind::LtEq
        | TokenKind::GtEq
        | TokenKind::In
        | TokenKind::Instanceof => Some((10, 11)),
        TokenKind::LtLt | TokenKind::GtGt | TokenKind::GtGtGt => Some((11, 12)),
        TokenKind::Plus | TokenKind::Minus => Some((12, 13)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((13, 14)),
        TokenKind::StarStar => Some((13, 14)),
        TokenKind::Dot | TokenKind::QuestionDot => Some((17, 18)),
        TokenKind::LBracket => Some((17, 18)),
        TokenKind::LParen => Some((17, 18)),
        TokenKind::Backtick | TokenKind::Template | TokenKind::TemplateHead => Some((17, 18)),
        _ => None,
    }
}

fn is_assign_op(kind: TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Eq
            | TokenKind::PlusEq
            | TokenKind::MinusEq
            | TokenKind::StarEq
            | TokenKind::SlashEq
            | TokenKind::PercentEq
            | TokenKind::StarStarEq
            | TokenKind::LtLtEq
            | TokenKind::GtGtEq
            | TokenKind::GtGtGtEq
            | TokenKind::AmpersandEq
            | TokenKind::PipeEq
            | TokenKind::CaretEq
            | TokenKind::AmpersandAmpersandEq
            | TokenKind::PipePipeEq
            | TokenKind::QuestionQuestionEq
    )
}

fn token_to_binary_op(kind: TokenKind) -> Option<BinaryOp> {
    match kind {
        TokenKind::EqEq => Some(BinaryOp::EqEq),
        TokenKind::EqEqEq => Some(BinaryOp::EqEqEq),
        TokenKind::Lt => Some(BinaryOp::Lt),
        TokenKind::LtEq => Some(BinaryOp::LtEq),
        TokenKind::Gt => Some(BinaryOp::Gt),
        TokenKind::GtEq => Some(BinaryOp::GtEq),
        TokenKind::LtLt => Some(BinaryOp::LShift),
        TokenKind::GtGt => Some(BinaryOp::RShift),
        TokenKind::GtGtGt => Some(BinaryOp::RShift3),
        TokenKind::Plus => Some(BinaryOp::Plus),
        TokenKind::Minus => Some(BinaryOp::Minus),
        TokenKind::Star => Some(BinaryOp::Mul),
        TokenKind::Slash => Some(BinaryOp::Div),
        TokenKind::Percent => Some(BinaryOp::Mod),
        TokenKind::StarStar => Some(BinaryOp::Pow),
        TokenKind::Ampersand => Some(BinaryOp::BitAnd),
        TokenKind::Pipe => Some(BinaryOp::BitOr),
        TokenKind::Caret => Some(BinaryOp::BitXor),
        TokenKind::In => Some(BinaryOp::In),
        TokenKind::Instanceof => Some(BinaryOp::Instanceof),
        _ => None,
    }
}

fn token_to_assign_op(kind: TokenKind) -> Option<AssignOp> {
    match kind {
        TokenKind::Eq => Some(AssignOp::Assign),
        TokenKind::PlusEq => Some(AssignOp::AddAssign),
        TokenKind::MinusEq => Some(AssignOp::SubAssign),
        TokenKind::StarEq => Some(AssignOp::MulAssign),
        TokenKind::SlashEq => Some(AssignOp::DivAssign),
        TokenKind::PercentEq => Some(AssignOp::ModAssign),
        TokenKind::StarStarEq => Some(AssignOp::PowAssign),
        TokenKind::LtLtEq => Some(AssignOp::LShiftAssign),
        TokenKind::GtGtEq => Some(AssignOp::RShiftAssign),
        TokenKind::GtGtGtEq => Some(AssignOp::RShift3Assign),
        TokenKind::AmpersandEq => Some(AssignOp::BitAndAssign),
        TokenKind::PipeEq => Some(AssignOp::BitOrAssign),
        TokenKind::CaretEq => Some(AssignOp::BitXorAssign),
        TokenKind::AmpersandAmpersandEq => Some(AssignOp::AndAssign),
        TokenKind::PipePipeEq => Some(AssignOp::OrAssign),
        TokenKind::QuestionQuestionEq => Some(AssignOp::NullishAssign),
        _ => None,
    }
}

// ---- Core Pratt parser ----

pub fn parse_expr(parser: &mut Parser, min_bp: u8) -> ExprRef {
    let start = parser.current_pos();

    let mut left = match parser.peek() {
        TokenKind::PlusPlus | TokenKind::MinusMinus => parse_prefix_update(parser),
        TokenKind::Exclamation
        | TokenKind::Tilde
        | TokenKind::Plus
        | TokenKind::Minus
        | TokenKind::Typeof
        | TokenKind::Void
        | TokenKind::Delete => parse_prefix_unary(parser),
        TokenKind::Await => {
            if parser.in_async_ctx() {
                parse_await_expr(parser)
            } else {
                let tok = parser.current_token().clone();
                let err = parser.error(DiagnosticCode::IllegalAwait, &tok);
                parser.errors.push(err);
                parser.advance();
                parse_expr(parser, 15)
            }
        }
        TokenKind::Yield => parse_yield_expr(parser),
        TokenKind::Ident | TokenKind::PrivateName => parse_ident_or_keyword_expr(parser),
        TokenKind::Number
        | TokenKind::String
        | TokenKind::BigInt
        | TokenKind::Null
        | TokenKind::True
        | TokenKind::False
        | TokenKind::Regex => parse_literal(parser),
        TokenKind::This => parse_this_expr(parser),
        TokenKind::Super => parse_super_expr(parser),
        TokenKind::Template | TokenKind::TemplateHead => parse_template_lit(parser),
        TokenKind::LParen => parse_paren_or_arrow(parser),
        TokenKind::LBracket => parse_array_lit(parser),
        TokenKind::LBrace => parse_obj_lit_or_block(parser),
        TokenKind::HashLBrace => parse_record_lit(parser),
        TokenKind::HashLBracket => parse_tuple_lit(parser),
        TokenKind::Function => parse_fn_expr(parser),
        TokenKind::Class => parse_class_expr(parser),
        TokenKind::New => parse_new_expr(parser),
        TokenKind::Slash | TokenKind::SlashEq => parse_regex(parser),
        TokenKind::DotDotDot => parse_spread_expr(parser),
        TokenKind::Import => parse_import_expr(parser),
        TokenKind::Lt => {
            if parser.options.features.jsx {
                if parser.peek_ahead(1) == TokenKind::Gt {
                    super::jsx::parse_jsx_fragment(parser)
                } else {
                    super::jsx::parse_jsx_element(parser)
                }
            } else if parser.options.features.typescript {
                parse_ts_type_assertion(parser)
            } else {
                let tok = parser.advance();
                let right = parse_expr(parser, 13);
                parser.expect(TokenKind::Gt).ok();
                let span = Span::new(tok.span.start, parser.span_since(start).end);
                let right_expr = parser.parse_expr();
                parser.ast.alloc(Expr::Binary(BinaryExpr {
                    span,
                    op: BinaryOp::Lt,
                    left: right,
                    right: right_expr,
                }))
            }
        }
        _ => {
            let tok = parser.current_token().clone();
            let err = parser.error(DiagnosticCode::UnexpectedToken, &tok);
            parser.errors.push(err);
            parser.advance();
            parser.ast.alloc(Expr::Ident(Ident {
                span: tok.span,
                name: String::new(),
                optional: false,
            }))
        }
    };

    loop {
        if parser.is_eof() {
            break;
        }
        let kind = parser.peek();
        if kind == TokenKind::Semicolon
            || kind == TokenKind::RBrace
            || kind == TokenKind::RBracket
            || kind == TokenKind::RParen
            || kind == TokenKind::Comma
            || kind == TokenKind::Colon
            || kind == TokenKind::In && parser.current_token().has_line_break
        {
            break;
        }

        if kind == TokenKind::Eq || is_assign_op(kind) {
            if min_bp >= 1 {
                break;
            }
            left = parse_assign_tail(parser, left);
            continue;
        }

        if kind == TokenKind::Question {
            if min_bp >= 2 {
                break;
            }
            left = parse_cond_tail(parser, left);
            continue;
        }

        if kind == TokenKind::QuestionQuestion && min_bp >= 3 {
            break;
        }
        if kind == TokenKind::PipePipe && min_bp >= 4 {
            break;
        }
        if kind == TokenKind::AmpersandAmpersand && min_bp >= 5 {
            break;
        }
        if kind == TokenKind::Pipe && min_bp >= 6 {
            break;
        }
        if kind == TokenKind::Caret && min_bp >= 7 {
            break;
        }
        if kind == TokenKind::Ampersand && min_bp >= 8 {
            break;
        }
        if matches!(
            kind,
            TokenKind::EqEq | TokenKind::Ne | TokenKind::EqEqEq | TokenKind::Neq
        ) && min_bp >= 9
        {
            break;
        }
        // In TypeScript, `<T>` after an expression is a postfix type
        // instantiation, not a relational expression. Parse it before the
        // relational precedence gate so unary expressions such as
        // `await api.fetch<Response>()` retain the complete call as their arg.
        if kind == TokenKind::Lt && parser.options.features.typescript && min_bp < 17 {
            if let Some(type_params) = super::typescript::try_parse_ts_type_args(parser) {
                let end = parser
                    .previous_token()
                    .map(|token| token.span.end)
                    .unwrap_or_else(|| parser.ast[left].span().end);
                left = parser.ast.alloc(Expr::TSInst(TSInstantiationExpr {
                    span: Span::new(parser.ast[left].span().start, end),
                    expr: left,
                    type_params,
                }));
                if parser.peek() == TokenKind::LParen {
                    left = parse_call_tail(parser, left);
                }
                continue;
            }
        }

        if matches!(
            kind,
            TokenKind::Lt
                | TokenKind::Gt
                | TokenKind::LtEq
                | TokenKind::GtEq
                | TokenKind::In
                | TokenKind::Instanceof
        ) && min_bp >= 10
        {
            break;
        }
        if matches!(kind, TokenKind::LtLt | TokenKind::GtGt | TokenKind::GtGtGt) && min_bp >= 11 {
            break;
        }
        if matches!(kind, TokenKind::Plus | TokenKind::Minus) && min_bp >= 12 {
            break;
        }
        if matches!(
            kind,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent
        ) && min_bp >= 13
        {
            break;
        }
        if kind == TokenKind::StarStar && min_bp >= 14 {
            break;
        }

        if kind == TokenKind::PlusPlus || kind == TokenKind::MinusMinus {
            if min_bp >= 16 {
                break;
            }
            left = parse_postfix_update(parser, left);
            continue;
        }

        if kind == TokenKind::QuestionDot || kind == TokenKind::Dot {
            if min_bp >= 17 {
                break;
            }
            if kind == TokenKind::QuestionDot && parser.peek_ahead(1) == TokenKind::LParen {
                left = parse_optional_call_tail(parser, left);
            } else if kind == TokenKind::QuestionDot && parser.peek_ahead(1) == TokenKind::LBracket
            {
                left = parse_computed_member_tail(parser, left, true);
            } else {
                left = parse_member_tail(parser, left);
            }
            continue;
        }

        if kind == TokenKind::LParen {
            if min_bp >= 17 {
                break;
            }
            left = parse_call_tail(parser, left);
            continue;
        }

        if kind == TokenKind::LBracket {
            if min_bp >= 17 {
                break;
            }
            left = parse_computed_member_tail(parser, left, false);
            continue;
        }

        if kind == TokenKind::As && parser.options.features.typescript {
            left = super::typescript::parse_ts_as_expr(parser, left);
            continue;
        }

        if kind == TokenKind::Satisfies && parser.options.features.typescript {
            left = super::typescript::parse_ts_satisfies_expr(parser, left);
            continue;
        }

        if kind == TokenKind::Exclamation && parser.options.features.typescript {
            if !parser.current_token().has_line_break {
                let tok = parser.advance();
                let span = Span::new(parser.ast[left].span().start, tok.span.end);
                left = parser
                    .ast
                    .alloc(Expr::TSNonNull(TSNonNullExpr { span, expr: left }));
                continue;
            }
            break;
        }

        if kind == TokenKind::Template || kind == TokenKind::TemplateHead {
            if min_bp >= 17 {
                break;
            }
            left = parse_tagged_template_tail(parser, left);
            continue;
        }

        // Binary operators
        if let Some((l_bp, r_bp)) = infix_bp(kind) {
            if l_bp < min_bp {
                break;
            }
            if kind == TokenKind::PipeGt {
                parser.advance();
                let right = parse_expr(parser, r_bp);
                let span = Span::new(parser.ast[left].span().start, parser.ast[right].span().end);
                left = parser.ast.alloc(Expr::Pipeline(PipelineExpr {
                    span,
                    input: left,
                    body: right,
                }));
                continue;
            }
            let _tok = parser.advance();
            let right = parse_expr(parser, r_bp);
            let span = Span::new(parser.ast[left].span().start, parser.ast[right].span().end);
            if matches!(
                kind,
                TokenKind::AmpersandAmpersand | TokenKind::PipePipe | TokenKind::QuestionQuestion
            ) {
                let op = match kind {
                    TokenKind::AmpersandAmpersand => LogicalOp::And,
                    TokenKind::PipePipe => LogicalOp::Or,
                    _ => LogicalOp::Nullish,
                };
                left = parser.ast.alloc(Expr::Logical(LogicalExpr {
                    span,
                    op,
                    left,
                    right,
                }));
            } else if let Some(op) = token_to_binary_op(kind) {
                left = parser.ast.alloc(Expr::Binary(BinaryExpr {
                    span,
                    op,
                    left,
                    right,
                }));
            }
            continue;
        }

        break;
    }

    left
}

pub fn parse_assign_expr(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    let left = parse_expr(parser, 1);

    if parser.peek() == TokenKind::Eq || is_assign_op(parser.peek()) {
        return parse_assign_tail(parser, left);
    }

    if parser.peek() == TokenKind::FatArrow {
        return parse_arrow_tail(parser, left, start);
    }

    left
}

pub fn parse_cond_expr(parser: &mut Parser) -> ExprRef {
    parse_expr(parser, 0)
}

// ---- Prefix parsing ----

fn parse_prefix_unary(parser: &mut Parser) -> ExprRef {
    let _start = parser.current_pos();
    let tok = parser.advance();
    let op = match tok.kind {
        TokenKind::Plus => UnaryOp::Plus,
        TokenKind::Minus => UnaryOp::Minus,
        TokenKind::Exclamation => UnaryOp::Not,
        TokenKind::Tilde => UnaryOp::BitNot,
        TokenKind::Typeof => UnaryOp::Typeof,
        TokenKind::Void => UnaryOp::Void,
        TokenKind::Delete => UnaryOp::Delete,
        _ => unreachable!(),
    };
    let arg = parse_expr(parser, 15);
    let span = Span::new(tok.span.start, parser.ast[arg].span().end);
    parser.ast.alloc(Expr::Unary(UnaryExpr { span, op, arg }))
}

fn parse_prefix_update(parser: &mut Parser) -> ExprRef {
    let _start = parser.current_pos();
    let tok = parser.advance();
    let op = match tok.kind {
        TokenKind::PlusPlus => UpdateOp::PlusPlus,
        _ => UpdateOp::MinusMinus,
    };
    let arg = parse_expr(parser, 16);
    let span = Span::new(tok.span.start, parser.ast[arg].span().end);
    parser.ast.alloc(Expr::Update(UpdateExpr {
        span,
        op,
        arg,
        prefix: true,
    }))
}

fn parse_postfix_update(parser: &mut Parser, left: ExprRef) -> ExprRef {
    let tok = parser.advance();
    let op = match tok.kind {
        TokenKind::PlusPlus => UpdateOp::PlusPlus,
        _ => UpdateOp::MinusMinus,
    };
    let span = Span::new(parser.ast[left].span().start, tok.span.end);
    parser.ast.alloc(Expr::Update(UpdateExpr {
        span,
        op,
        arg: left,
        prefix: false,
    }))
}

fn parse_await_expr(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    parser.advance();
    let arg = parse_expr(parser, 15);
    let span = parser.span_since(start);
    parser.ast.alloc(Expr::Await(AwaitExpr { span, arg }))
}

fn parse_yield_expr(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    let tok = parser.advance();
    if !parser.in_generator_ctx() {
        let err = parser.error(DiagnosticCode::IllegalYield, &tok);
        parser.errors.push(err);
    }
    let delegate = if parser.peek() == TokenKind::Star {
        parser.advance();
        true
    } else {
        false
    };
    let arg = if !parser.is_eof()
        && parser.peek() != TokenKind::Semicolon
        && parser.peek() != TokenKind::RBrace
        && parser.peek() != TokenKind::RBracket
        && parser.peek() != TokenKind::RParen
        && parser.peek() != TokenKind::Colon
        && parser.peek() != TokenKind::Comma
    {
        Some(parse_expr(parser, 1))
    } else {
        None
    };
    let span = parser.span_since(start);
    parser.ast.alloc(Expr::Yield(YieldExpr {
        span,
        arg,
        delegate,
    }))
}

fn parse_this_expr(parser: &mut Parser) -> ExprRef {
    let tok = parser.advance();
    parser.ast.alloc(Expr::This(ThisExpr { span: tok.span }))
}

fn parse_super_expr(parser: &mut Parser) -> ExprRef {
    let tok = parser.advance();
    parser.ast.alloc(Expr::Super(SuperExpr { span: tok.span }))
}

fn parse_literal(parser: &mut Parser) -> ExprRef {
    let tok = parser.advance();
    match tok.kind {
        TokenKind::Number => {
            let val: f64 = tok.value.parse().unwrap_or(0.0);
            parser.ast.alloc(Expr::Lit(Lit::Num(NumLit {
                span: tok.span,
                value: val,
                raw: tok.value,
            })))
        }
        TokenKind::String => parser.ast.alloc(Expr::Lit(Lit::Str(StrLit {
            span: tok.span,
            value: tok.value,
            raw: String::new(),
        }))),
        TokenKind::BigInt => parser.ast.alloc(Expr::Lit(Lit::BigInt(BigIntLit {
            span: tok.span,
            value: tok.value,
            raw: String::new(),
        }))),
        TokenKind::Null => parser
            .ast
            .alloc(Expr::Lit(Lit::Null(NullLit { span: tok.span }))),
        TokenKind::True => parser.ast.alloc(Expr::Lit(Lit::Bool(BoolLit {
            span: tok.span,
            value: true,
        }))),
        TokenKind::False => parser.ast.alloc(Expr::Lit(Lit::Bool(BoolLit {
            span: tok.span,
            value: false,
        }))),
        TokenKind::Regex => {
            let (pattern, flags) = parse_regex_str(&tok.value);
            parser.ast.alloc(Expr::Lit(Lit::RegExp(RegExpLit {
                span: tok.span,
                pattern,
                flags,
            })))
        }
        _ => parser
            .ast
            .alloc(Expr::Lit(Lit::Null(NullLit { span: tok.span }))),
    }
}

fn parse_regex(parser: &mut Parser) -> ExprRef {
    let tok = parser.advance();
    let (pattern, flags) = parse_regex_str(&tok.value);
    parser.ast.alloc(Expr::Lit(Lit::RegExp(RegExpLit {
        span: tok.span,
        pattern,
        flags,
    })))
}

fn parse_regex_str(s: &str) -> (String, String) {
    if s.starts_with('/') {
        if let Some(idx) = s.rfind('/') {
            if idx > 1
                && s.is_char_boundary(1)
                && s.is_char_boundary(idx)
                && s.is_char_boundary(idx + 1)
            {
                let pattern = s[1..idx].to_string();
                let flags = s[idx + 1..].to_string();
                return (pattern, flags);
            }
        }
    }
    (s.to_string(), String::new())
}

fn parse_ident_or_keyword_expr(parser: &mut Parser) -> ExprRef {
    let tok = parser.advance();
    let name = tok.value.clone();
    let span = tok.span;
    let ident = Ident {
        span,
        name,
        optional: false,
    };

    if parser.peek() == TokenKind::FatArrow {
        let type_ann = super::typescript::maybe_parse_ts_type_ann(parser);
        let params = vec![Pat::Ident(BindingIdent {
            span,
            id: ident.clone(),
            type_ann,
            optional: false,
        })];
        return parse_arrow(parser, params, false, span);
    }

    parser.ast.alloc(Expr::Ident(ident))
}

fn parse_paren_or_arrow(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();

    if parser.peek_ahead(0) == TokenKind::RParen && parser.peek_ahead(1) == TokenKind::FatArrow {
        parser.advance();
        parser.advance();
        return parse_arrow(parser, Vec::new(), false, parser.span_since(start));
    }

    parser.advance();
    let content_pos = parser.pos;

    let mut params = Vec::new();

    if parser.peek() != TokenKind::RParen && !parser.is_eof() {
        loop {
            if parser.peek() == TokenKind::DotDotDot {
                let rest = super::patterns::parse_rest_pat(parser);
                params.push(rest);
                if parser.peek() == TokenKind::Comma {
                    parser.advance();
                }
                break;
            }

            let ppos = parser.current_pos();
            if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
                let id_tok = parser.advance();
                let id_name = id_tok.value.clone();
                let id_span = id_tok.span;

                if parser.peek() == TokenKind::Colon
                    || parser.peek() == TokenKind::Eq
                    || parser.peek() == TokenKind::Comma
                    || parser.peek() == TokenKind::RParen
                {
                    let type_ann = if parser.peek() == TokenKind::Colon {
                        parser.advance();
                        Some(super::typescript::parse_ts_type(parser))
                    } else {
                        None
                    };
                    let init = if parser.peek() == TokenKind::Eq {
                        parser.advance();
                        Some(parse_assign_expr(parser))
                    } else {
                        None
                    };
                    let pat = Pat::Ident(BindingIdent {
                        span: id_span,
                        id: Ident {
                            span: id_span,
                            name: id_name,
                            optional: false,
                        },
                        type_ann,
                        optional: false,
                    });
                    if let Some(init_val) = init {
                        let span = Span::new(id_span.start, parser.ast[init_val].span().end);
                        params.push(Pat::Assign(AssignPat {
                            span,
                            left: Box::new(pat),
                            right: init_val,
                        }));
                    } else {
                        params.push(pat);
                    }
                } else {
                    parser.pos = ppos;
                    break;
                }
            } else if parser.peek() == TokenKind::LBrace || parser.peek() == TokenKind::LBracket {
                let pat = super::patterns::parse_binding_pat(parser);
                params.push(pat);
            } else {
                break;
            }

            if parser.peek() == TokenKind::Comma {
                parser.advance();
            } else {
                break;
            }
        }
    }

    if parser.peek() == TokenKind::RParen && parser.peek_ahead(1) == TokenKind::FatArrow {
        parser.advance();
        return parse_arrow(parser, params, false, parser.span_since(start));
    }

    // The parameter parse above is speculative. If no arrow follows, rewind
    // and parse the contents as a normal expression; otherwise operators such
    // as `>` or `? :` after an identifier are mistaken for a closing paren.
    parser.pos = content_pos;
    let inner = parse_expr(parser, 0);
    parser.expect(TokenKind::RParen).ok();
    parser.ast.alloc(Expr::Parenthesized(ParenthesizedExpr {
        span: parser.span_since(start),
        expr: inner,
    }))
}

fn parse_arrow(parser: &mut Parser, params: Vec<Pat>, async_: bool, span: Span) -> ExprRef {
    parser.expect(TokenKind::FatArrow).ok();
    let body = if parser.peek() == TokenKind::LBrace {
        let body_block = parser.parse_block();
        ArrowBody::Block(body_block)
    } else {
        let expr = parse_expr(parser, 0);
        ArrowBody::Expr(expr)
    };
    let span = Span::new(
        span.start,
        parser
            .previous_token()
            .map(|t| t.span.end)
            .unwrap_or(span.start),
    );
    parser.ast.alloc(Expr::Arrow(ArrowExpr {
        span,
        params,
        body,
        async_,
    }))
}

fn parse_array_lit(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    parser.advance();
    let mut elements = Vec::new();
    while parser.peek() != TokenKind::RBracket && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            elements.push(None);
            parser.advance();
            continue;
        }
        let elem = parse_assign_expr(parser);
        elements.push(Some(elem));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBracket).ok();
    parser.ast.alloc(Expr::Array(ArrayExpr {
        span: parser.span_since(start),
        elements,
    }))
}

fn parse_obj_lit_or_block(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    parser.advance();
    let mut props = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        if parser.peek() == TokenKind::DotDotDot {
            let tok = parser.advance();
            let arg = parse_assign_expr(parser);
            let span = Span::new(tok.span.start, parser.ast[arg].span().end);
            props.push(ObjProp::Spread(SpreadExpr { span, arg }));
            if parser.peek() == TokenKind::Comma {
                parser.advance();
            }
            continue;
        }
        props.push(parse_obj_prop(parser));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    parser.ast.alloc(Expr::Object(ObjectExpr {
        span: parser.span_since(start),
        props,
    }))
}

fn parse_record_lit(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    // Consume the `#{` token (already matched).
    parser.advance();
    let mut props = Vec::new();
    while parser.peek() != TokenKind::RBrace && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            parser.advance();
            continue;
        }
        props.push(parse_obj_prop(parser));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBrace).ok();
    parser.ast.alloc(Expr::Record(RecordExpr {
        span: parser.span_since(start),
        props,
    }))
}

fn parse_tuple_lit(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    // Consume the `#[` token (already matched).
    parser.advance();
    let mut elements = Vec::new();
    while parser.peek() != TokenKind::RBracket && !parser.is_eof() {
        if parser.peek() == TokenKind::Comma {
            elements.push(None);
            parser.advance();
            continue;
        }
        let elem = parse_assign_expr(parser);
        elements.push(Some(elem));
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RBracket).ok();
    parser.ast.alloc(Expr::Tuple(TupleExpr {
        span: parser.span_since(start),
        elements,
    }))
}

fn parse_obj_prop(parser: &mut Parser) -> ObjProp {
    let start = parser.current_pos();

    let is_get = parser.peek() == TokenKind::Get
        && parser.peek_ahead(1) != TokenKind::LParen
        && parser.peek_ahead(1) != TokenKind::Colon;
    let is_set = parser.peek() == TokenKind::Set
        && parser.peek_ahead(1) != TokenKind::LParen
        && parser.peek_ahead(1) != TokenKind::Colon;

    if is_get {
        parser.advance();
        let key = super::patterns::parse_prop_name(parser);
        let body = parser.parse_block();
        return ObjProp::Getter(GetterProp {
            span: parser.span_since(start),
            key,
            body: Some(body),
        });
    }

    if is_set {
        parser.advance();
        let key = super::patterns::parse_prop_name(parser);
        parser.expect(TokenKind::LParen).ok();
        let param = super::patterns::parse_binding_pat(parser);
        parser.expect(TokenKind::RParen).ok();
        let body = parser.parse_block();
        return ObjProp::Setter(SetterProp {
            span: parser.span_since(start),
            key,
            param,
            body: Some(body),
        });
    }

    let key = super::patterns::parse_prop_name(parser);

    if parser.peek() == TokenKind::LParen {
        let fn_expr = super::declarations::parse_fn_body(parser, &[]);
        let mut fn_expr_clone = fn_expr.clone();
        fn_expr_clone.id = None;
        return ObjProp::Method(MethodProp {
            span: parser.span_since(start),
            key,
            function: fn_expr_clone,
        });
    }

    if parser.peek() == TokenKind::Colon {
        parser.advance();
        let value = parse_assign_expr(parser);
        return ObjProp::KeyValue(KeyValueProp {
            span: parser.span_since(start),
            key,
            value,
        });
    }

    if parser.peek() == TokenKind::Eq {
        parser.advance();
        let _value = parse_assign_expr(parser);
        if let PropName::Ident(ref id) = key {
            ObjProp::Shorthand(id.clone())
        } else {
            ObjProp::Shorthand(Ident {
                span: Span::ZERO,
                name: String::new(),
                optional: false,
            })
        }
    } else if let PropName::Ident(ref id) = key {
        ObjProp::Shorthand(id.clone())
    } else {
        ObjProp::Shorthand(Ident {
            span: Span::ZERO,
            name: String::new(),
            optional: false,
        })
    }
}

// ---- Infix parsing ----

fn parse_assign_tail(parser: &mut Parser, left: ExprRef) -> ExprRef {
    let tok = parser.advance();
    let op = token_to_assign_op(tok.kind).unwrap_or(AssignOp::Assign);
    let right = parse_assign_expr(parser);
    let span = Span::new(parser.ast[left].span().start, parser.ast[right].span().end);
    parser.ast.alloc(Expr::Assignment(AssignmentExpr {
        span,
        op,
        left,
        right,
    }))
}

fn parse_cond_tail(parser: &mut Parser, left: ExprRef) -> ExprRef {
    parser.advance();
    let cons = parse_assign_expr(parser);
    parser.expect(TokenKind::Colon).ok();
    let alt = parse_assign_expr(parser);
    let span = Span::new(parser.ast[left].span().start, parser.ast[alt].span().end);
    parser.ast.alloc(Expr::Conditional(ConditionalExpr {
        span,
        test: left,
        consequent: cons,
        alternate: alt,
    }))
}

fn parse_member_tail(parser: &mut Parser, left: ExprRef) -> ExprRef {
    let optional = parser.peek() == TokenKind::QuestionDot;
    // `?.` and plain `.` both just consume one token before the member name.
    parser.advance();

    if parser.peek() == TokenKind::Ident || parser.peek().is_keyword() {
        let tok = parser.advance();
        let prop = Expr::Ident(Ident {
            span: tok.span,
            name: tok.value,
            optional: false,
        });
        let span = Span::new(parser.ast[left].span().start, tok.span.end);
        if optional {
            parser.ast.alloc(Expr::OptionalMember(OptionalMemberExpr {
                span,
                object: left,
                property: Box::new(prop),
                computed: false,
            }))
        } else {
            parser.ast.alloc(Expr::Member(MemberExpr {
                span,
                object: left,
                property: Box::new(prop),
                computed: false,
            }))
        }
    } else if parser.peek() == TokenKind::PrivateName {
        let tok = parser.advance();
        let prop = Expr::PrivateName(PrivateNameExpr {
            span: tok.span,
            name: Ident {
                span: tok.span,
                name: tok.value,
                optional: false,
            },
        });
        let span = Span::new(parser.ast[left].span().start, tok.span.end);
        if optional {
            parser.ast.alloc(Expr::OptionalMember(OptionalMemberExpr {
                span,
                object: left,
                property: Box::new(prop),
                computed: false,
            }))
        } else {
            parser.ast.alloc(Expr::Member(MemberExpr {
                span,
                object: left,
                property: Box::new(prop),
                computed: false,
            }))
        }
    } else {
        let tok = parser.current_token().clone();
        let err = parser.error(DiagnosticCode::UnexpectedToken, &tok);
        parser.errors.push(err);
        let prop = Expr::Ident(Ident {
            span: tok.span,
            name: tok.value.clone(),
            optional: false,
        });
        parser.advance();
        let span = Span::new(parser.ast[left].span().start, tok.span.end);
        if optional {
            parser.ast.alloc(Expr::OptionalMember(OptionalMemberExpr {
                span,
                object: left,
                property: Box::new(prop),
                computed: false,
            }))
        } else {
            parser.ast.alloc(Expr::Member(MemberExpr {
                span,
                object: left,
                property: Box::new(prop),
                computed: false,
            }))
        }
    }
}

fn parse_computed_member_tail(parser: &mut Parser, left: ExprRef, optional: bool) -> ExprRef {
    if optional {
        parser.advance();
    }
    parser.expect(TokenKind::LBracket).ok();
    let expr = parse_assign_expr(parser);
    parser.expect(TokenKind::RBracket).ok();
    let end = parser
        .previous_token()
        .map(|token| token.span.end)
        .unwrap_or_else(|| parser.ast[expr].span().end);
    let span = Span::new(parser.ast[left].span().start, end);
    let prop = parser.ast[expr].clone();
    if optional {
        parser.ast.alloc(Expr::OptionalMember(OptionalMemberExpr {
            span,
            object: left,
            property: Box::new(prop),
            computed: true,
        }))
    } else {
        parser.ast.alloc(Expr::Member(MemberExpr {
            span,
            object: left,
            property: Box::new(prop),
            computed: true,
        }))
    }
}

fn parse_call_tail(parser: &mut Parser, left: ExprRef) -> ExprRef {
    parser.advance();
    let mut args = Vec::new();
    while parser.peek() != TokenKind::RParen && !parser.is_eof() {
        if parser.peek() == TokenKind::DotDotDot {
            let spread = parse_spread_expr(parser);
            args.push(spread);
        } else {
            args.push(parse_assign_expr(parser));
        }
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RParen).ok();
    let end = parser
        .previous_token()
        .map(|token| token.span.end)
        .unwrap_or_else(|| parser.ast[left].span().end);
    let span = Span::new(parser.ast[left].span().start, end);
    parser.ast.alloc(Expr::Call(CallExpr {
        span,
        callee: left,
        args,
    }))
}

fn parse_optional_call_tail(parser: &mut Parser, left: ExprRef) -> ExprRef {
    parser.expect(TokenKind::QuestionDot).ok();
    parser.expect(TokenKind::LParen).ok();
    let mut args = Vec::new();
    while parser.peek() != TokenKind::RParen && !parser.is_eof() {
        if parser.peek() == TokenKind::DotDotDot {
            args.push(parse_spread_expr(parser));
        } else {
            args.push(parse_assign_expr(parser));
        }
        if parser.peek() == TokenKind::Comma {
            parser.advance();
        } else {
            break;
        }
    }
    parser.expect(TokenKind::RParen).ok();
    let end = parser
        .previous_token()
        .map(|token| token.span.end)
        .unwrap_or_else(|| parser.ast[left].span().end);
    let span = Span::new(parser.ast[left].span().start, end);
    parser.ast.alloc(Expr::OptionalCall(OptionalCallExpr {
        span,
        callee: left,
        args,
    }))
}

fn parse_new_expr(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    parser.advance();
    let callee = parse_expr(parser, 17);

    let args = if parser.peek() == TokenKind::LParen {
        parser.advance();
        let mut args = Vec::new();
        while parser.peek() != TokenKind::RParen && !parser.is_eof() {
            if parser.peek() == TokenKind::DotDotDot {
                args.push(parse_spread_expr(parser));
            } else {
                args.push(parse_assign_expr(parser));
            }
            if parser.peek() == TokenKind::Comma {
                parser.advance();
            } else {
                break;
            }
        }
        parser.expect(TokenKind::RParen).ok();
        args
    } else {
        Vec::new()
    };

    let span = parser.span_since(start);
    parser.ast.alloc(Expr::New(NewExpr { span, callee, args }))
}

fn parse_fn_expr(parser: &mut Parser) -> ExprRef {
    let fn_expr = super::declarations::parse_fn_expr(parser);
    parser.ast.alloc(Expr::Fn(fn_expr))
}

fn parse_class_expr(parser: &mut Parser) -> ExprRef {
    let class_expr = super::declarations::parse_class_expr(parser);
    parser.ast.alloc(Expr::Class(class_expr))
}

fn parse_spread_expr(parser: &mut Parser) -> ExprRef {
    let _start = parser.current_pos();
    let tok = parser.advance();
    let arg = parse_assign_expr(parser);
    let span = Span::new(tok.span.start, parser.ast[arg].span().end);
    parser.ast.alloc(Expr::Spread(SpreadExpr { span, arg }))
}

fn parse_template_lit(parser: &mut Parser) -> ExprRef {
    parse_template_tail(parser, None)
}

fn parse_template_tail(parser: &mut Parser, tag: Option<ExprRef>) -> ExprRef {
    let start = parser.current_pos();
    let mut quasis = Vec::new();
    let mut expressions = Vec::new();

    if tag.is_some() && parser.peek() == TokenKind::Template {
        let tok = parser.advance();
        quasis.push(TemplateElement {
            span: tok.span,
            value: tok.value,
            tail: true,
        });
    } else {
        loop {
            match parser.peek() {
                TokenKind::Template | TokenKind::TemplateTail => {
                    let tok = parser.advance();
                    quasis.push(TemplateElement {
                        span: tok.span,
                        value: tok.value,
                        tail: true,
                    });
                    break;
                }
                TokenKind::TemplateHead | TokenKind::TemplateMiddle => {
                    let tok = parser.advance();
                    quasis.push(TemplateElement {
                        span: tok.span,
                        value: tok.value,
                        tail: false,
                    });
                    let expr = parse_expr(parser, 0);
                    expressions.push(expr);
                    // The lexer consumes the closing `}` while producing the
                    // following TemplateMiddle/TemplateTail token.
                }
                _ => {
                    if parser.peek() == TokenKind::RBrace {
                        parser.advance();
                    }
                    break;
                }
            }
        }
    }

    let span = parser.span_since(start);
    let tl = TemplateLit {
        span,
        quasis,
        expressions,
    };

    if let Some(tag_expr) = tag {
        let span = Span::new(parser.ast[tag_expr].span().start, tl.span.end);
        parser.ast.alloc(Expr::TaggedTemplate(TaggedTemplateExpr {
            span,
            tag: tag_expr,
            template: tl,
        }))
    } else {
        parser.ast.alloc(Expr::Template(tl))
    }
}

fn parse_tagged_template_tail(parser: &mut Parser, left: ExprRef) -> ExprRef {
    parse_template_tail(parser, Some(left))
}

fn parse_import_expr(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    parser.advance();
    parser.expect(TokenKind::LParen).ok();
    let source = parse_assign_expr(parser);
    while parser.peek() == TokenKind::Comma {
        parser.advance();
        if parser.peek() != TokenKind::RParen {
            let _ = parse_assign_expr(parser);
        }
    }
    parser.expect(TokenKind::RParen).ok();
    let span = parser.span_since(start);
    parser.ast.alloc(Expr::Import(ImportExpr { span, source }))
}

fn parse_arrow_tail(parser: &mut Parser, left: ExprRef, start: usize) -> ExprRef {
    let mut params = Vec::new();
    // Extract params from the left expression that was parsed as a parenthesized sequence
    match &parser.ast[left] {
        Expr::Ident(id) => {
            params.push(Pat::Ident(BindingIdent {
                span: id.span,
                id: id.clone(),
                type_ann: None,
                optional: false,
            }));
        }
        Expr::Sequence(seq) => {
            for e in &seq.expressions {
                if let Expr::Ident(id) = &parser.ast[*e] {
                    params.push(Pat::Ident(BindingIdent {
                        span: id.span,
                        id: id.clone(),
                        type_ann: None,
                        optional: false,
                    }));
                }
            }
        }
        _ => {}
    }
    let span = parser.span_since(start);
    parse_arrow(parser, params, false, span)
}

fn parse_ts_type_assertion(parser: &mut Parser) -> ExprRef {
    let start = parser.current_pos();
    parser.advance();
    let type_ann = Box::new(super::typescript::parse_ts_type(parser));
    parser.expect(TokenKind::Gt).ok();
    let expr = parse_expr(parser, 17);
    let span = parser.span_since(start);
    parser.ast.alloc(Expr::TSTypeAssertion(TSTypeAssertionExpr {
        span,
        expr,
        type_ann,
    }))
}

// ---- Span helper for Expr ----

pub trait HasSpan {
    fn span(&self) -> Span;
}

impl HasSpan for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Ident(i) => i.span,
            Expr::Lit(l) => match l {
                Lit::Str(s) => s.span,
                Lit::Num(n) => n.span,
                Lit::Bool(b) => b.span,
                Lit::Null(n) => n.span,
                Lit::RegExp(r) => r.span,
                Lit::BigInt(b) => b.span,
                Lit::Template(t) => t.span,
            },
            Expr::This(t) => t.span,
            Expr::Super(s) => s.span,
            Expr::Array(a) => a.span,
            Expr::Object(o) => o.span,
            Expr::Fn(f) => f.span,
            Expr::Arrow(a) => a.span,
            Expr::Class(c) => c.span,
            Expr::New(n) => n.span,
            Expr::Call(c) => c.span,
            Expr::OptionalCall(c) => c.span,
            Expr::Member(m) => m.span,
            Expr::OptionalMember(m) => m.span,
            Expr::Unary(u) => u.span,
            Expr::UnaryOp(u) => u.span,
            Expr::Binary(b) => b.span,
            Expr::Logical(l) => l.span,
            Expr::Conditional(c) => c.span,
            Expr::Assignment(a) => a.span,
            Expr::Sequence(s) => s.span,
            Expr::Update(u) => u.span,
            Expr::Await(a) => a.span,
            Expr::Yield(y) => y.span,
            Expr::Spread(s) => s.span,
            Expr::Template(t) => t.span,
            Expr::TaggedTemplate(t) => t.span,
            Expr::MetaProperty(m) => m.span,
            Expr::Import(i) => i.span,
            Expr::JSXElement(j) => j.span,
            Expr::JSXFragment(j) => j.span,
            Expr::TSAs(t) => t.span,
            Expr::TSSatisfies(t) => t.span,
            Expr::TSTypeAssertion(t) => t.span,
            Expr::TSNonNull(t) => t.span,
            Expr::TSInst(t) => t.span,
            Expr::Parenthesized(p) => p.span,
            Expr::PrivateName(p) => p.span,
            Expr::Chain(c) => c.span,
            Expr::Invalid(i) => i.span,
            Expr::Record(r) => r.span,
            Expr::Tuple(t) => t.span,
            Expr::Pipeline(p) => p.span,
        }
    }
}
