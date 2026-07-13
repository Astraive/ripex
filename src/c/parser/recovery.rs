use super::state::Parser;
use crate::c::lexer::TokenKind;

pub fn skip_to_stmt_boundary(parser: &mut Parser) {
    loop {
        let kind = parser.peek();
        match kind {
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof => return,
            _ if kind.is_keyword() => match kind {
                TokenKind::Break
                | TokenKind::Case
                | TokenKind::Continue
                | TokenKind::Default
                | TokenKind::Do
                | TokenKind::Else
                | TokenKind::Enum
                | TokenKind::For
                | TokenKind::Goto
                | TokenKind::If
                | TokenKind::Return
                | TokenKind::Struct
                | TokenKind::Switch
                | TokenKind::Typedef
                | TokenKind::Union
                | TokenKind::While => return,
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
    if parser.peek() == TokenKind::Semicolon {
        parser.advance();
    }
    if parser.pos == pos_before {
        parser.advance();
    }
}

pub fn is_eof(parser: &Parser) -> bool {
    parser.peek() == TokenKind::Eof
}
