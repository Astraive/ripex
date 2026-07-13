// Go declaration parsing is inline in statements.rs since
// Go uses a simple statement-oriented declaration system.
// This file provides extra helpers for method receivers and interfaces.

use super::super::ast::expr::Expr;
use super::super::lexer::TokenKind;
use super::state::Parser;

impl Parser {
    pub fn parse_receiver(&mut self) -> Option<(String, String)> {
        // Parse (t *Type) or (t Type)
        let _start_pos = self.pos;
        if self.peek() != TokenKind::LParen {
            return None;
        }
        self.advance();
        let name = self.expect_ident();
        let typ = self.parse_type();
        self.expect(TokenKind::RParen);
        if let Expr::Ident(type_name, _) = typ {
            Some((name, type_name))
        } else if let Expr::Unary(_, ref inner, _) = typ {
            if let Expr::Ident(type_name, _) = inner.as_ref() {
                Some((name, type_name.clone()))
            } else {
                None
            }
        } else {
            None
        }
    }
}
