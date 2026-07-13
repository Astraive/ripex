use super::module::Module;
use super::node::AstNode;
use super::stmt::Stmt;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Program {
    Script(Script),
    Module(Module),
}

#[derive(Debug, Clone)]
pub struct Script {
    pub span: Span,
    pub body: Vec<Stmt>,
}

impl AstNode for Script {
    fn span(&self) -> Span {
        self.span
    }
}
