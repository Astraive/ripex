pub mod expr;
pub mod stmt;

pub use expr::*;
pub use stmt::*;

use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Program {
    pub decls: Vec<Decl>,
    pub span: Span,
}
