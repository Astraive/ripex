// Additional C declaration helpers
use super::super::lexer::TokenKind;
use super::state::Parser;

impl Parser {
    /// Reads an optional struct/union/enum tag name. C allows the name to be
    /// absent (anonymous tags, e.g. `struct { int x; }`), so this returns an
    /// empty string when no identifier follows the keyword.
    pub(crate) fn parse_optional_tag_name(&mut self) -> String {
        if self.peek() == TokenKind::Ident {
            self.advance().value
        } else {
            String::new()
        }
    }

    pub fn parse_struct_body(&mut self) -> Vec<super::super::ast::expr::StructField> {
        let mut fields = Vec::new();
        self.expect(TokenKind::LBrace);
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            let type_ = self.parse_type();
            let name = self.expect_ident();
            let bitfield = if self.peek() == TokenKind::Colon {
                self.advance();
                let tok = self.advance();
                tok.value.parse::<usize>().ok()
            } else {
                None
            };
            self.expect(TokenKind::Semicolon);
            fields.push(super::super::ast::expr::StructField {
                type_: Box::new(type_),
                name,
                bitfield,
                span: self.peek_token().span,
            });
        }
        self.expect(TokenKind::RBrace);
        fields
    }

    pub fn parse_enum_body(&mut self) -> Vec<super::super::ast::expr::EnumConstant> {
        let mut values = Vec::new();
        self.expect(TokenKind::LBrace);
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            let name = self.expect_ident();
            let value = if self.peek() == TokenKind::Eq {
                self.advance();
                Some(Box::new(self.parse_expr()))
            } else {
                None
            };
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
            values.push(super::super::ast::expr::EnumConstant {
                name,
                value,
                span: self.peek_token().span,
            });
        }
        self.expect(TokenKind::RBrace);
        values
    }
}
