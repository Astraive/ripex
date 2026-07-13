use crate::js::lexer::scanner::Scanner;

pub fn scan_regex(scanner: &mut Scanner) -> (String, String) {
    scanner.advance(); // consume opening '/'
    let pattern_start = scanner.offset();
    let mut in_char_class = false;

    loop {
        match scanner.advance() {
            None => break,
            Some('[') => {
                in_char_class = true;
            }
            Some(']') => {
                in_char_class = false;
            }
            Some('\\') => {
                // Skip next character (escape sequence)
                scanner.advance();
            }
            Some('/') if !in_char_class => {
                let pattern = &scanner.source()[pattern_start..scanner.offset() - 1];
                let flags_start = scanner.offset();
                while let Some(ch) = scanner.peek() {
                    if ch.is_ascii_alphabetic() {
                        scanner.advance();
                    } else {
                        break;
                    }
                }
                let flags = &scanner.source()[flags_start..scanner.offset()];
                return (pattern.to_string(), flags.to_string());
            }
            Some('\n') | Some('\r') => {
                break;
            }
            _ => {}
        }
    }

    ("".to_string(), "".to_string())
}
