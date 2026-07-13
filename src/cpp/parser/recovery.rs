use super::state::Parser;
use crate::cpp::lexer::TokenKind;

pub fn skip_to_stmt_boundary(parser: &mut Parser) {
    loop {
        let kind = parser.peek();
        match kind {
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof => return,
            // Keywords that can start a new statement
            TokenKind::Break
            | TokenKind::Case
            | TokenKind::Catch
            | TokenKind::Class
            | TokenKind::Constexpr
            | TokenKind::Constinit
            | TokenKind::Continue
            | TokenKind::Default
            | TokenKind::Delete
            | TokenKind::Do
            | TokenKind::Else
            | TokenKind::Enum
            | TokenKind::Explicit
            | TokenKind::Export
            | TokenKind::Extern
            | TokenKind::For
            | TokenKind::Friend
            | TokenKind::Goto
            | TokenKind::If
            | TokenKind::IfConstexpr
            | TokenKind::Mutable
            | TokenKind::Namespace
            | TokenKind::Operator
            | TokenKind::Return
            | TokenKind::StaticAssert
            | TokenKind::Struct
            | TokenKind::Switch
            | TokenKind::Template
            | TokenKind::Throw
            | TokenKind::Try
            | TokenKind::Typedef
            | TokenKind::Union
            | TokenKind::Using
            | TokenKind::Virtual
            | TokenKind::While
            | TokenKind::CoAwait
            | TokenKind::CoReturn
            | TokenKind::CoYield
            | TokenKind::ConstCast
            | TokenKind::DynamicCast
            | TokenKind::ReinterpretCast
            | TokenKind::StaticCast
            | TokenKind::Typeid
            | TokenKind::Typename
            | TokenKind::Requires
            | TokenKind::Concept
            | TokenKind::Asm
            | TokenKind::Nullptr
            | TokenKind::New => return,
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
