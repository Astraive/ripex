use super::scanner::Scanner;
use super::{Comment, CommentKind};
use crate::span::{Pos, Span};

pub fn skip_comment(scanner: &mut Scanner, start: Pos) -> Comment {
    // Line comment starting with #
    while let Some(ch) = scanner.peek() {
        if ch == '\n' || ch == '\r' {
            break;
        }
        scanner.advance();
    }
    let end = scanner.position();
    let text = scanner.slice(start).to_string();
    Comment::new(CommentKind::Line, Span::new(start, end), text)
}
