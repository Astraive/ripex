use super::expr::{Block, Expr};
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, Span),
    Decl(Decl, Span),
    Assign(Vec<Expr>, Vec<Expr>, Span),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>, Span),
    For(
        Option<Box<Stmt>>,
        Option<Expr>,
        Option<Box<Stmt>>,
        Box<Stmt>,
        Span,
    ),
    ForRange(Expr, String, Option<String>, Box<Stmt>, Span),
    Switch(Option<Expr>, Vec<CaseClause>, Span),
    Select(Vec<CaseClause>, Span),
    Return(Vec<Expr>, Span),
    Break(Option<String>, Span),
    Continue(Option<String>, Span),
    Defer(Expr, Span),
    Go(Expr, Span),
    Block(Block, Span),
    Empty(Span),
    Label(String, Box<Stmt>, Span),
    Goto(String, Span),
    Send(Expr, Expr, Span),
    Fallthrough(Span),
}

#[derive(Debug, Clone)]
pub enum Decl {
    Var(VarDecl, Span),
    Const(ConstDecl, Span),
    Type(TypeDecl, Span),
    Func(FuncDecl, Span),
    Import(ImportDecl, Span),
    Package(String, Span),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub names: Vec<String>,
    pub kind: Option<Box<Expr>>,
    pub values: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstDecl {
    pub names: Vec<String>,
    pub kind: Option<Box<Expr>>,
    pub values: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
    pub name: String,
    pub kind: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub receiver: Option<(String, String)>,
    pub params: Vec<(String, Box<Expr>)>,
    pub returns: Vec<Box<Expr>>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseClause {
    pub expr: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}
