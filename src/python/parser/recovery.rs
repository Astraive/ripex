use super::state::Parser;
use crate::python::lexer::TokenKind;

pub fn skip_to_stmt_boundary(parser: &mut Parser) {
    loop {
        let kind = parser.peek();
        match kind {
            TokenKind::Newline | TokenKind::Dedent | TokenKind::RBrace | TokenKind::Eof => return,
            _ if kind.is_keyword() => match kind {
                TokenKind::False
                | TokenKind::None_
                | TokenKind::True
                | TokenKind::And
                | TokenKind::As
                | TokenKind::Assert
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Break
                | TokenKind::Class
                | TokenKind::Continue
                | TokenKind::Def
                | TokenKind::Del
                | TokenKind::Elif
                | TokenKind::Else
                | TokenKind::Except
                | TokenKind::Finally
                | TokenKind::For
                | TokenKind::From
                | TokenKind::Global
                | TokenKind::If
                | TokenKind::Import
                | TokenKind::In
                | TokenKind::Is
                | TokenKind::Lambda
                | TokenKind::Match
                | TokenKind::Nonlocal
                | TokenKind::Not
                | TokenKind::Or
                | TokenKind::Pass
                | TokenKind::Raise
                | TokenKind::Return
                | TokenKind::Try
                | TokenKind::While
                | TokenKind::With
                | TokenKind::Yield
                | TokenKind::Type
                | TokenKind::Self_ => return,
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
    let pos_before = parser.pos;
    skip_to_stmt_boundary(parser);
    if parser.peek() == TokenKind::Newline {
        parser.advance();
    }
    // Force progress: if we are still at the same position, skip one token.
    if parser.pos == pos_before {
        parser.advance();
    }
}

pub fn is_eof(parser: &Parser) -> bool {
    parser.peek() == TokenKind::Eof
}
