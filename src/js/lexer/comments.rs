use crate::js::lexer::scanner::Scanner;
use crate::js::lexer::token::Comment;
use crate::span::Span;

pub fn scan_single_line_comment(scanner: &mut Scanner) -> Comment {
    let start = scanner.position();
    scanner.advance();
    scanner.advance();

    loop {
        match scanner.peek() {
            Some('\n') | Some('\r') | None => break,
            _ => _ = scanner.advance(),
        }
    }

    let end = scanner.position();
    let text = scanner.slice(start).to_string();
    let span = Span::new(start, end);
    Comment::new(span, text, false)
}

pub fn scan_multi_line_comment(scanner: &mut Scanner) -> Comment {
    let start = scanner.position();
    scanner.advance();
    scanner.advance();

    loop {
        match scanner.advance() {
            None => break,
            Some('*') if scanner.peek() == Some('/') => {
                scanner.advance();
                break;
            }
            _ => {}
        }
    }

    let end = scanner.position();
    let text = scanner.slice(start).to_string();
    let span = Span::new(start, end);
    Comment::new(span, text, true)
}

pub fn scan_hashbang(scanner: &mut Scanner) -> Comment {
    let start = scanner.position();
    scanner.advance();
    scanner.advance();

    loop {
        match scanner.peek() {
            Some('\n') | Some('\r') | None => break,
            _ => _ = scanner.advance(),
        }
    }

    let end = scanner.position();
    let text = scanner.slice(start).to_string();
    let span = Span::new(start, end);
    Comment::new(span, text, false)
}
