use crate::arena::Arena;
use crate::js::ast::Program;

use super::printer::Printer;

pub fn minify(source: &str, _program: &Program) -> String {
    let mut output = String::new();
    let mut in_string = false;
    let mut string_char = '"';
    let mut prev_char = ' ';

    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if in_string {
            output.push(c);
            if c == '\\' && i + 1 < chars.len() {
                i += 1;
                output.push(chars[i]);
            } else if c == string_char {
                in_string = false;
            }
            i += 1;
            continue;
        }

        if c == '"' || c == '\'' || c == '`' {
            in_string = true;
            string_char = c;
            output.push(c);
            i += 1;
            continue;
        }

        if c == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if chars[i + 1] == '*' {
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
        }

        if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
            let next = if i + 1 < chars.len() {
                chars[i + 1]
            } else {
                '\0'
            };
            let prev_is_op = matches!(
                prev_char,
                '+' | '-'
                    | '*'
                    | '/'
                    | '%'
                    | '&'
                    | '|'
                    | '^'
                    | '~'
                    | '='
                    | '!'
                    | '<'
                    | '>'
                    | '?'
                    | ':'
                    | ','
                    | ';'
                    | '('
                    | '['
                    | '{'
            );
            let next_is_op = matches!(
                next,
                '+' | '-'
                    | '*'
                    | '/'
                    | '%'
                    | '&'
                    | '|'
                    | '^'
                    | '~'
                    | '='
                    | '!'
                    | '<'
                    | '>'
                    | '?'
                    | ':'
                    | ','
                    | ';'
                    | ')'
                    | ']'
                    | '}'
                    | '.'
            );
            if !prev_is_op && !next_is_op {
                output.push(' ');
            }
            i += 1;
            continue;
        }

        output.push(c);
        prev_char = c;
        i += 1;
    }

    output
}

pub fn minify_ast(program: &Program) -> String {
    let mut ast = Arena::new();
    let mut printer = Printer::new();
    printer.print_program(program, &mut ast)
}
