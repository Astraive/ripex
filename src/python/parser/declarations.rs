// Python declarations are handled inline in statements.rs
// because function and class definitions are parsed directly.
use super::super::lexer::TokenKind;
use super::state::Parser;

impl Parser {
    pub fn parse_docstring(&mut self) -> Option<String> {
        if self.peek() == TokenKind::StringLit {
            let tok = self.advance();
            Some(tok.value.clone())
        } else {
            None
        }
    }
}
