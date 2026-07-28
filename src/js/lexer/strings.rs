use crate::js::lexer::scanner::Scanner;
use crate::js::lexer::token::TokenKind;

pub fn scan_string(scanner: &mut Scanner, quote: char) -> String {
    let mut result = String::new();
    loop {
        match scanner.advance() {
            None | Some('\n') | Some('\r') => {
                return result;
            }
            Some(ch) if ch == quote => {
                return result;
            }
            Some('\\') => {
                if let Some(c) = scan_escape(scanner) {
                    result.push(c);
                }
            }
            Some(ch) => {
                result.push(ch);
            }
        }
    }
}

pub fn scan_template(scanner: &mut Scanner) -> (String, TokenKind) {
    let mut result = String::new();
    loop {
        match scanner.peek() {
            None => {
                return (result, TokenKind::TemplateTail);
            }
            Some('`') => {
                scanner.advance();
                return (result, TokenKind::TemplateTail);
            }
            Some('$') if scanner.peek_ahead(1) == Some('{') => {
                scanner.advance();
                scanner.advance();
                return (result, TokenKind::TemplateHead);
            }
            Some('\\') => {
                scanner.advance();
                if let Some(c) = scan_escape(scanner) {
                    result.push(c);
                }
            }
            Some(ch) => {
                scanner.advance();
                result.push(ch);
            }
        }
    }
}

fn scan_escape(scanner: &mut Scanner) -> Option<char> {
    Some(match scanner.advance()? {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        'b' => '\u{0008}',
        'f' => '\u{000C}',
        'v' => '\u{000B}',
        '\\' => '\\',
        '\'' => '\'',
        '"' => '"',
        '`' => '`',
        '0' => {
            if matches!(scanner.peek(), Some('0'..='7')) {
                let mut code = 0u32;
                for _ in 0..3 {
                    match scanner.peek() {
                        Some(d @ '0'..='7') => {
                            scanner.advance();
                            code = code * 8 + (d as u32 - '0' as u32);
                        }
                        _ => break,
                    }
                }
                char::from_u32(code).unwrap_or('\u{FFFD}')
            } else {
                '\0'
            }
        }
        'x' => {
            let hi = scanner.advance().and_then(hex_val);
            let lo = scanner.advance().and_then(hex_val);
            match (hi, lo) {
                (Some(h), Some(l)) => ((h << 4) | l) as char,
                _ => '\u{FFFD}',
            }
        }
        'u' => {
            if scanner.peek() == Some('{') {
                scanner.advance();
                let mut code = 0u32;
                loop {
                    match scanner.peek() {
                        Some('}') => {
                            scanner.advance();
                            break;
                        }
                        Some(d) if d.is_ascii_hexdigit() => {
                            scanner.advance();
                            code = code * 16 + d.to_digit(16).unwrap();
                        }
                        _ => break,
                    }
                }
                char::from_u32(code).unwrap_or('\u{FFFD}')
            } else {
                let hi = scanner.advance().and_then(hex_val);
                let lo = scanner.advance().and_then(hex_val);
                match (hi, lo) {
                    (Some(h), Some(l)) => ((h << 4) | l) as char,
                    _ => '\u{FFFD}',
                }
            }
        }
        '\r' => {
            if scanner.peek() == Some('\n') {
                scanner.advance();
            }
            return None;
        }
        '\n' => {
            return None;
        }
        ch => ch,
    })
}

fn hex_val(c: char) -> Option<u8> {
    match c {
        '0'..='9' => Some(c as u8 - b'0'),
        'a'..='f' => Some(c as u8 - b'a' + 10),
        'A'..='F' => Some(c as u8 - b'A' + 10),
        _ => None,
    }
}
