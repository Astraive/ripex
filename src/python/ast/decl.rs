use super::expr::Expr;
use super::stmt::Stmt;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub name: String,
    pub args: Vec<Arg>,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
    pub returns: Option<Box<Expr>>,
    pub is_async: bool,
    pub is_generator: bool,
    pub defaults: Vec<Expr>,
    pub kw_defaults: Vec<Expr>,
    pub vararg: Option<Box<Arg>>,
    pub kwarg: Option<Box<Arg>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: String,
    pub type_ann: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ClassDef {
    pub name: String,
    pub bases: Vec<Expr>,
    pub keywords: Vec<super::expr::Keyword>,
    pub body: Vec<Stmt>,
    pub decorators: Vec<Expr>,
    pub span: Span,
}
