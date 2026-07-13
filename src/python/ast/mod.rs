pub mod decl;
pub mod expr;
pub mod literal;
pub mod pattern;
pub mod stmt;

pub use decl::*;
pub use expr::*;
pub use literal::*;
pub use pattern::*;
pub use stmt::*;

use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Program {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}
