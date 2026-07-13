use super::state::Parser;
use crate::rust::lexer::TokenKind;

pub fn skip_to_stmt_boundary(parser: &mut Parser) {
    loop {
        let kind = parser.peek();
        match kind {
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof => return,
            _ if kind.is_keyword() => match kind {
                TokenKind::Fn
                | TokenKind::Let
                | TokenKind::Mut
                | TokenKind::Const
                | TokenKind::Static
                | TokenKind::Impl
                | TokenKind::Trait
                | TokenKind::Pub
                | TokenKind::Crate
                | TokenKind::Self_
                | TokenKind::Super
                | TokenKind::Use
                | TokenKind::Mod
                | TokenKind::Struct
                | TokenKind::Enum
                | TokenKind::Union
                | TokenKind::Type_
                | TokenKind::Where
                | TokenKind::For
                | TokenKind::In
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::Match
                | TokenKind::While
                | TokenKind::Loop
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::Return
                | TokenKind::Async
                | TokenKind::Await
                | TokenKind::Unsafe
                | TokenKind::Extern
                | TokenKind::Ref
                | TokenKind::Move
                | TokenKind::Dyn
                | TokenKind::As
                | TokenKind::True
                | TokenKind::False
                | TokenKind::None_ => return,
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
