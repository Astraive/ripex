use crate::js::lexer::scanner::Scanner;
use crate::js::lexer::token::TokenKind;

fn scan_digits(scanner: &mut Scanner, radix: u32, allow_separator: bool) -> bool {
    let mut found = false;
    loop {
        match scanner.peek() {
            Some(ch) if ch.is_digit(radix) || (allow_separator && ch == '_') => {
                scanner.advance();
                found = true;
            }
            _ => break,
        }
    }
    found
}

fn scan_exponent(scanner: &mut Scanner) {
    match scanner.peek() {
        Some('e') | Some('E') => {
            scanner.advance();
            let _ = scanner.advance_if(|c| c == '+' || c == '-');
            scan_digits(scanner, 10, true);
        }
        _ => {}
    }
}

pub fn scan_number(scanner: &mut Scanner) -> TokenKind {
    if scanner.peek() == Some('0') {
        scanner.advance();
        match scanner.peek() {
            // Hex 0x, 0X
            Some('x') | Some('X') => {
                scanner.advance();
                let _ = scan_digits(scanner, 16, true);
                return check_bigint(scanner);
            }
            // Binary 0b, 0B
            Some('b') | Some('B') => {
                scanner.advance();
                let _ = scan_digits(scanner, 2, true);
                return check_bigint(scanner);
            }
            // Octal 0o, 0O (ES2021+)
            Some('o') | Some('O') => {
                scanner.advance();
                let _ = scan_digits(scanner, 8, true);
                return check_bigint(scanner);
            }
            // Legacy octal like 0777
            Some('0'..='7') => {
                let _ = scan_digits(scanner, 8, false);
                // If we see 8 or 9, it's a decimal number
                if matches!(scanner.peek(), Some('8') | Some('9')) {
                    scan_digits(scanner, 10, true);
                    scan_exponent(scanner);
                    return TokenKind::Number;
                }
                // Check for decimal separator or exponent
                if scanner.peek() == Some('.') || matches!(scanner.peek(), Some('e') | Some('E')) {
                    scan_fraction(scanner);
                    scan_exponent(scanner);
                }
                return check_bigint(scanner);
            }
            Some('_') => {
                // 0_... is a decimal
                scan_digits(scanner, 10, true);
                scan_fraction_part(scanner);
                scan_exponent(scanner);
                return check_bigint(scanner);
            }
            Some('.') => {
                scan_fraction(scanner);
                scan_exponent(scanner);
                return TokenKind::Number;
            }
            Some('e') | Some('E') => {
                scan_exponent(scanner);
                return TokenKind::Number;
            }
            Some('n') => {
                scanner.advance();
                return TokenKind::BigInt;
            }
            _ => {}
        }
        // Just a single '0'
        return check_bigint(scanner);
    }

    // Decimal: can start with '.' like .5
    if scanner.peek() == Some('.') {
        scan_fraction(scanner);
        scan_exponent(scanner);
        return TokenKind::Number;
    }

    // Integer part
    scan_digits(scanner, 10, true);
    scan_fraction_part(scanner);
    scan_exponent(scanner);
    check_bigint(scanner)
}

fn scan_fraction_part(scanner: &mut Scanner) {
    if scanner.peek() == Some('.') {
        scan_fraction(scanner);
    }
}

fn scan_fraction(scanner: &mut Scanner) {
    scanner.advance(); // consume '.'
    let _ = scan_digits(scanner, 10, true);
}

fn check_bigint(scanner: &mut Scanner) -> TokenKind {
    if scanner.peek() == Some('n') && is_bigint_end(scanner) {
        scanner.advance();
        TokenKind::BigInt
    } else {
        TokenKind::Number
    }
}

fn is_bigint_end(scanner: &mut Scanner) -> bool {
    // BigInt cannot be followed by a decimal point or exponent
    if scanner
        .peek_ahead(1)
        .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.')
    {
        return false;
    }
    true
}
