use super::decl::{ClassDef, FuncDef};
use super::expr::Expr;
use super::pattern::MatchCase;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, Span),
    Assign(Box<Expr>, Box<Expr>, Span),
    AugAssign(Box<Expr>, super::expr::BinaryOp, Box<Expr>, Span),
    AnnAssign(Box<Expr>, Box<Expr>, Option<Box<Expr>>, Span),
    If(Box<Expr>, Vec<Stmt>, Vec<Stmt>, Span),
    While(Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>, Span),
    For(Box<Expr>, Box<Expr>, Vec<Stmt>, Option<Vec<Stmt>>, Span),
    With(Vec<WithItem>, Vec<Stmt>, Span),
    Match(Box<Expr>, Vec<MatchCase>, Span),
    Return(Option<Expr>, Span),
    Yield(Option<Expr>, Span),
    Raise(Option<Expr>, Option<Expr>, Span),
    Assert(Expr, Option<Expr>, Span),
    Break(Span),
    Continue(Span),
    Pass(Span),
    Delete(Expr, Span),
    Global(Vec<String>, Span),
    Nonlocal(Vec<String>, Span),
    Import(Vec<Alias>, Span),
    ImportFrom(Option<String>, Vec<Alias>, usize, Span),
    Try(
        Vec<Stmt>,
        Vec<ExceptHandler>,
        Option<Vec<Stmt>>,
        Option<Vec<Stmt>>,
        Span,
    ),
    FuncDef(FuncDef, Span),
    ClassDef(ClassDef, Span),
    Async(Box<Stmt>, Span),
    Block(Vec<Stmt>, Span),
    Empty(Span),
}

#[derive(Debug, Clone)]
pub struct WithItem {
    pub context: Box<Expr>,
    pub target: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub asname: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExceptHandler {
    pub type_: Option<Box<Expr>>,
    pub name: Option<String>,
    pub body: Vec<Stmt>,
    pub span: Span,
}
