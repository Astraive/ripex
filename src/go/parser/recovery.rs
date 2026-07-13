use super::state::Parser;
use crate::go::lexer::TokenKind;

pub fn skip_to_stmt_boundary(parser: &mut Parser) {
    loop {
        let kind = parser.peek();
        match kind {
            TokenKind::Semicolon | TokenKind::Newline | TokenKind::RBrace | TokenKind::Eof => {
                return
            }
            _ if kind.is_keyword() => match kind {
                TokenKind::Package
                | TokenKind::Import
                | TokenKind::Func
                | TokenKind::Var
                | TokenKind::Const
                | TokenKind::Type
                | TokenKind::Struct
                | TokenKind::Interface
                | TokenKind::Map
                | TokenKind::Chan
                | TokenKind::Defer
                | TokenKind::Go
                | TokenKind::Select
                | TokenKind::Case
                | TokenKind::Switch
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::For
                | TokenKind::Range
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Return
                | TokenKind::Fallthrough
                | TokenKind::Default
                | TokenKind::Goto => return,
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
    if parser.peek() == TokenKind::Semicolon || parser.peek() == TokenKind::Newline {
        parser.advance();
    }
    if parser.pos == pos_before {
        parser.advance();
    }
}

pub fn is_eof(parser: &Parser) -> bool {
    parser.peek() == TokenKind::Eof
}
