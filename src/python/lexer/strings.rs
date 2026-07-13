use super::scanner::Scanner;
use super::TokenKind;

pub fn scan_string(scanner: &mut Scanner, start_char: char) -> (TokenKind, bool) {
    let mut is_fstring = false;
    let mut is_bytes = false;

    // Check prefixes before the first quote
    if start_char == 'f' || start_char == 'F' {
        is_fstring = true;
    }
    if start_char == 'b' || start_char == 'B' {
        is_bytes = true;
    }

    // We consumed the prefix char already, now check for more prefixes and the quote
    let ch = scanner.peek().unwrap_or('\0');
    let mut quote_char = ch;

    if quote_char == '"' || quote_char == '\'' {
        scanner.advance(); // consume first quote
    } else {
        let prefixes = if ch == 'r' || ch == 'R' {
            scanner.advance();
            scanner.peek().unwrap_or('\0')
        } else if ch == 'f' || ch == 'F' {
            is_fstring = true;
            scanner.advance();
            scanner.peek().unwrap_or('\0')
        } else {
            '\0'
        };

        quote_char = prefixes;
        if quote_char != '"' && quote_char != '\'' {
            return (TokenKind::StringLit, is_fstring);
        }
        scanner.advance();
    }

    // Check for triple quotes
    let is_triple = if scanner.peek() == Some(quote_char) {
        scanner.advance();
        if scanner.peek() == Some(quote_char) {
            scanner.advance();
            true
        } else {
            // Was just two quotes in a row
            let kind = if is_bytes {
                TokenKind::BytesLit
            } else {
                TokenKind::StringLit
            };
            return (kind, is_fstring);
        }
    } else {
        false
    };

    // Read string content
    loop {
        match scanner.peek() {
            None => break,
            Some('\\') => {
                scanner.advance();
                if scanner.peek().is_some() {
                    scanner.advance();
                }
            }
            Some(ch) if ch == quote_char => {
                if is_triple {
                    scanner.advance();
                    if scanner.peek() == Some(quote_char) {
                        scanner.advance();
                        if scanner.peek() == Some(quote_char) {
                            scanner.advance();
                            break;
                        }
                    }
                } else {
                    scanner.advance();
                    break;
                }
            }
            Some('{') if is_fstring && !is_triple => {
                // Keep the complete f-string in one token. Expression-aware
                // splitting belongs in the parser and must not leave the
                // lexer stranded halfway through a string.
                scanner.advance();
            }
            _ => {
                scanner.advance();
            }
        }
    }

    let kind = if is_bytes {
        TokenKind::BytesLit
    } else if is_fstring {
        TokenKind::FStringLit
    } else {
        TokenKind::StringLit
    };
    (kind, false)
}

pub fn scan_fstring_interior(scanner: &mut Scanner) -> (TokenKind, bool) {
    // Read until matching }
    let mut depth = 1u32;
    loop {
        match scanner.peek() {
            None => break,
            Some('{') => {
                depth += 1;
                scanner.advance();
            }
            Some('}') => {
                depth -= 1;
                scanner.advance();
                if depth == 0 {
                    // Check if more string follows
                    match scanner.peek() {
                        Some('"') => {
                            let _saved_pos = scanner.position();
                            scanner.advance();
                            // Check for fstring continuation
                            if scanner.peek() == Some('"') {
                                scanner.advance();
                            }
                            // Return to string
                            let kind = TokenKind::FStringMid;
                            return (kind, true);
                        }
                        Some('\'') => {
                            scanner.advance();
                            if scanner.peek() == Some('\'') {
                                scanner.advance();
                            }
                            let kind = TokenKind::FStringMid;
                            return (kind, true);
                        }
                        _ => {
                            // Expression is done
                            let kind = TokenKind::FStringTail;
                            return (kind, false);
                        }
                    }
                }
            }
            Some('\'') | Some('"') => {
                scanner.advance();
            }
            Some(_ch) => {
                scanner.advance();
            }
        }
    }
    (TokenKind::Error, false)
}
