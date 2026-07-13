// Additional C++ declaration helpers
use super::super::lexer::TokenKind;
use super::state::Parser;

impl Parser {
    pub fn parse_initializer_list(&mut self) -> Vec<super::super::ast::expr::Expr> {
        let mut items = Vec::new();
        self.expect(TokenKind::LBrace);
        while self.peek() != TokenKind::RBrace && self.peek() != TokenKind::Eof {
            items.push(self.parse_expr());
            if self.peek() == TokenKind::Comma {
                self.advance();
            }
        }
        self.expect(TokenKind::RBrace);
        items
    }
}
