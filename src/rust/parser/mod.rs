pub mod declarations;
pub mod expressions;
pub mod recovery;
pub mod state;
pub mod statements;

pub use state::Parser;

use super::ast::Program;
use crate::span::Span;

pub fn parse_program(source: &str) -> (Program, Vec<crate::diagnostics::ParseError>) {
    let mut parser = Parser::new(source);
    let start = parser.token_start();
    let mut items = Vec::new();
    while parser.peek() != super::lexer::TokenKind::Eof {
        let pos_before = parser.pos;
        items.push(parser.parse_item());
        if parser.pos == pos_before {
            parser.advance();
        }
    }
    let end = parser.prev_end();
    let program = Program {
        items,
        span: Span::new(start, end),
    };
    (program, parser.errors)
}

pub fn parse_script(
    source: &str,
) -> (
    Vec<super::ast::stmt::Stmt>,
    Vec<crate::diagnostics::ParseError>,
) {
    let mut parser = Parser::new(source);
    let mut stmts = Vec::new();
    while parser.peek() != super::lexer::TokenKind::Eof {
        let pos_before = parser.pos;
        stmts.push(parser.parse_stmt());
        if parser.pos == pos_before {
            parser.advance();
        }
    }
    (stmts, parser.errors)
}
