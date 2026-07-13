use super::scanner::Scanner;
use super::TokenKind;

pub fn scan_number(scanner: &mut Scanner) -> TokenKind {
    if scanner.peek() == Some('0') {
        scanner.advance();
        match scanner.peek() {
            Some('x') | Some('X') => {
                scanner.advance();
                scan_hex(scanner);
                TokenKind::IntLit
            }
            Some('o') | Some('O') => {
                scanner.advance();
                scan_oct(scanner);
                TokenKind::IntLit
            }
            Some('b') | Some('B') => {
                scanner.advance();
                scan_bin(scanner);
                TokenKind::IntLit
            }
            _ => scan_dec_or_float(scanner),
        }
    } else {
        scan_dec_or_float(scanner)
    }
}

fn scan_dec_or_float(scanner: &mut Scanner) -> TokenKind {
    while let Some(ch) = scanner.peek() {
        if ch.is_ascii_digit() || ch == '_' {
            scanner.advance();
        } else {
            break;
        }
    }
    if scanner.peek() == Some('.') {
        let next = scanner.peek_ahead(1);
        if next.is_some_and(|c| c.is_ascii_digit()) {
            scanner.advance();
            while let Some(ch) = scanner.peek() {
                if ch.is_ascii_digit() || ch == '_' {
                    scanner.advance();
                } else {
                    break;
                }
            }
            // Check for complex 'j'
            if scanner.peek() == Some('j') || scanner.peek() == Some('J') {
                scanner.advance();
                return TokenKind::ComplexLit;
            }
            return TokenKind::FloatLit;
        }
    }
    if scanner.peek() == Some('e') || scanner.peek() == Some('E') {
        scanner.advance();
        if scanner.peek() == Some('+') || scanner.peek() == Some('-') {
            scanner.advance();
        }
        while let Some(ch) = scanner.peek() {
            if ch.is_ascii_digit() || ch == '_' {
                scanner.advance();
            } else {
                break;
            }
        }
        if scanner.peek() == Some('j') || scanner.peek() == Some('J') {
            scanner.advance();
            return TokenKind::ComplexLit;
        }
        return TokenKind::FloatLit;
    }
    if scanner.peek() == Some('j') || scanner.peek() == Some('J') {
        scanner.advance();
        return TokenKind::ComplexLit;
    }
    TokenKind::IntLit
}

fn scan_hex(scanner: &mut Scanner) {
    while let Some(ch) = scanner.peek() {
        if ch.is_ascii_hexdigit() || ch == '_' {
            scanner.advance();
        } else {
            break;
        }
    }
}

fn scan_oct(scanner: &mut Scanner) {
    while let Some(ch) = scanner.peek() {
        if matches!(ch, '0'..='7') || ch == '_' {
            scanner.advance();
        } else {
            break;
        }
    }
}

fn scan_bin(scanner: &mut Scanner) {
    while let Some(ch) = scanner.peek() {
        if matches!(ch, '0'..='1') || ch == '_' {
            scanner.advance();
        } else {
            break;
        }
    }
}
