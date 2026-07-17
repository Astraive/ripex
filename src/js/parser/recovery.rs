use super::state::Parser;
use crate::diagnostics::DiagnosticCode;
use crate::js::lexer::TokenKind;

pub fn skip_to_stmt_boundary(parser: &mut Parser) {
    loop {
        let kind = parser.peek();
        match kind {
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof => return,
            _ if kind.is_keyword() => match kind {
                TokenKind::Var
                | TokenKind::Let
                | TokenKind::Const
                | TokenKind::Function
                | TokenKind::Class
                | TokenKind::Return
                | TokenKind::If
                | TokenKind::For
                | TokenKind::While
                | TokenKind::Do
                | TokenKind::Switch
                | TokenKind::Try
                | TokenKind::Throw
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Debugger
                | TokenKind::Import
                | TokenKind::Export
                | TokenKind::Async => return,
                _ => {
                    parser.advance();
                }
            },
            _ => {
                parser.advance();
            }
        }
    }
}

pub fn recover_from_error(parser: &mut Parser) {
    let pos_before = parser.current_pos();
    skip_to_stmt_boundary(parser);
    if parser.peek() == TokenKind::Semicolon {
        parser.advance();
    }
    // Anti-spin guard: if neither skip_to_stmt_boundary nor the semicolon
    // consumed a token (e.g. a lone unknown token with no recovery point),
    // force one advance so the outer parse loop always makes progress and
    // cannot loop forever / OOM. Mirrors the C-family recovery guard.
    if parser.current_pos() == pos_before && !parser.is_eof() {
        parser.advance();
    }
}

pub fn expect_semicolon(parser: &mut Parser) {
    if parser.peek() == TokenKind::Semicolon {
        parser.advance();
        return;
    }
    if parser.is_eof() || parser.peek() == TokenKind::RBrace {
        return;
    }
    if parser
        .previous_token()
        .is_some_and(|token| token.kind == TokenKind::RBrace)
    {
        return;
    }
    if parser.current_token().has_line_break {
        return;
    }
    let tok = parser.current_token().clone();
    let err = parser.error_msg(
        DiagnosticCode::UnexpectedToken,
        tok.span,
        format!("expected semicolon before {:?}", tok.kind),
    );
    parser.errors.push(err);
}
