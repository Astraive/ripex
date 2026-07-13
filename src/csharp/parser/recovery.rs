use super::state::Parser;
use crate::csharp::lexer::TokenKind;

pub fn skip_to_stmt_boundary(parser: &mut Parser) {
    loop {
        let kind = parser.peek();
        match kind {
            TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof => return,
            // Keywords that can start a new statement
            TokenKind::Abstract
            | TokenKind::Async
            | TokenKind::Await
            | TokenKind::Base
            | TokenKind::Break
            | TokenKind::Case
            | TokenKind::Catch
            | TokenKind::Checked
            | TokenKind::Class
            | TokenKind::Const
            | TokenKind::Continue
            | TokenKind::Default
            | TokenKind::Delegate
            | TokenKind::Do
            | TokenKind::Else
            | TokenKind::Enum
            | TokenKind::Event
            | TokenKind::Explicit
            | TokenKind::Extern
            | TokenKind::Finally
            | TokenKind::Fixed
            | TokenKind::For
            | TokenKind::ForEach
            | TokenKind::Goto
            | TokenKind::If
            | TokenKind::Implicit
            | TokenKind::Interface
            | TokenKind::Internal
            | TokenKind::Lock
            | TokenKind::Namespace
            | TokenKind::New
            | TokenKind::Operator
            | TokenKind::Out
            | TokenKind::Override
            | TokenKind::Params
            | TokenKind::Partial
            | TokenKind::Private
            | TokenKind::Protected
            | TokenKind::Public
            | TokenKind::Readonly
            | TokenKind::Record
            | TokenKind::Ref
            | TokenKind::Return
            | TokenKind::Sealed
            | TokenKind::Static
            | TokenKind::Struct
            | TokenKind::Switch
            | TokenKind::Throw
            | TokenKind::Try
            | TokenKind::Unchecked
            | TokenKind::Unsafe
            | TokenKind::Using
            | TokenKind::Virtual
            | TokenKind::Volatile
            | TokenKind::While
            | TokenKind::With
            | TokenKind::Yield
            | TokenKind::From
            | TokenKind::Let
            | TokenKind::Join
            | TokenKind::Orderby
            | TokenKind::Select
            | TokenKind::Group
            | TokenKind::By
            | TokenKind::Descending
            | TokenKind::Equals
            | TokenKind::Into
            | TokenKind::On
            | TokenKind::Where
            | TokenKind::When
            | TokenKind::Init
            | TokenKind::Add
            | TokenKind::Remove
            | TokenKind::Get
            | TokenKind::Set => return,
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
