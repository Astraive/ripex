pub mod declarations;
pub mod expressions;
pub mod recovery;
pub mod state;
pub mod statements;

pub use state::Parser;

use super::ast::{Program, Stmt};
use crate::span::Span;

pub fn parse_program(source: &str) -> (Program, Vec<crate::diagnostics::ParseError>) {
    let mut parser = Parser::new(source);
    let start = parser.token_start();
    let stmts = parser.parse_compilation_unit();
    let mut decls = Vec::new();
    for stmt in stmts {
        if let Stmt::Decl(d, _) = stmt {
            decls.push(d);
        }
    }
    let program = Program {
        decls,
        span: Span::new(start, parser.prev_end()),
    };
    (program, parser.errors)
}

pub fn parse_script(source: &str) -> (Vec<Stmt>, Vec<crate::diagnostics::ParseError>) {
    let mut parser = Parser::new(source);
    let decls = parser.parse_compilation_unit();
    (decls, parser.errors)
}
