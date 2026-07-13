use crate::arena::NodeId;
use crate::span::Span;
use super::expr::Expr;
use super::stmt::Stmt;

pub trait AstNode {
    fn span(&self) -> Span;
    fn children(&self) -> Vec<NodeId>;
}

impl AstNode for Expr {
    fn span(&self) -> Span { Span::ZERO }
    fn children(&self) -> Vec<NodeId> { Vec::new() }
}

impl AstNode for Stmt {
    fn span(&self) -> Span { Span::ZERO }
    fn children(&self) -> Vec<NodeId> { Vec::new() }
}
