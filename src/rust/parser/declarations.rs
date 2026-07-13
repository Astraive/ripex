// Additional Rust declaration parsing helpers
use super::super::ast::stmt::*;
use super::super::lexer::TokenKind;
use super::state::Parser;

impl Parser {
    pub fn parse_generic_params(&mut self) -> Vec<GenericParam> {
        let mut params = Vec::new();
        if self.peek() == TokenKind::Lt {
            self.advance();
            while self.peek() != TokenKind::Gt && self.peek() != TokenKind::Eof {
                if self.peek() == TokenKind::Ident {
                    let name = self.expect_ident();
                    let mut bounds = Vec::new();
                    if self.peek() == TokenKind::Colon {
                        self.advance();
                        bounds.push(self.parse_expr());
                    }
                    params.push(GenericParam {
                        name,
                        bounds,
                        span: self.peek_token().span,
                    });
                }
                if self.peek() == TokenKind::Comma {
                    self.advance();
                }
                // Forward-progress guard: if we didn't consume a token above
                // (unexpected token inside <...>), advance so the loop can't
                // spin forever and OOM.
                if self.peek() != TokenKind::Gt && self.peek() != TokenKind::Eof {
                    self.advance();
                }
            }
            self.expect(TokenKind::Gt);
        }
        params
    }

    pub fn parse_where_clause(&mut self) -> Vec<(String, Vec<super::super::ast::expr::Expr>)> {
        Vec::new()
    }
}
