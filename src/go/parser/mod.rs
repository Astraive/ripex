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
    let mut decls = Vec::new();
    while parser.peek() != super::lexer::TokenKind::Eof {
        let pos_before = parser.pos;
        let stmt = parser.parse_stmt_recovery();
        if parser.pos == pos_before {
            break;
        }
        if let super::ast::stmt::Stmt::Decl(d, _) = stmt {
            decls.push(d);
        }
    }
    let end = parser.prev_end();
    let program = Program {
        decls,
        span: Span::new(start, end),
    };
    (program, parser.errors)
}

pub fn parse_script(source: &str) -> (Vec<super::ast::Stmt>, Vec<crate::diagnostics::ParseError>) {
    let mut parser = Parser::new(source);
    let mut stmts = Vec::new();
    while parser.peek() != super::lexer::TokenKind::Eof {
        stmts.push(parser.parse_stmt_recovery());
    }
    (stmts, parser.errors)
}
