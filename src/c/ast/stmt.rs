use super::expr::Expr;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, Span),
    Decl(FuncDecl, Span),
    VarDecl(VarDecl, Span),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>, Span),
    Switch(Expr, Vec<CaseClause>, Span),
    While(Expr, Box<Stmt>, Span),
    Do(Box<Stmt>, Expr, Span),
    For(
        Option<Box<Stmt>>,
        Option<Expr>,
        Option<Box<Stmt>>,
        Box<Stmt>,
        Span,
    ),
    Return(Option<Expr>, Span),
    Break(Span),
    Continue(Span),
    Goto(String, Span),
    Label(String, Span),
    Block(Block, Span),
    Empty(Span),
    Preprocessor(PreprocDirective, Span),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub return_type: Box<Expr>,
    pub params: Vec<ParamDecl>,
    pub is_variadic: bool,
    pub is_knr: bool,
    pub body: Option<Block>,
    pub storage_class: Option<String>,
    pub is_inline: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub type_: Box<Expr>,
    pub name: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub type_: Box<Expr>,
    pub name: String,
    pub init: Option<Expr>,
    pub is_const: bool,
    pub storage_class: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseClause {
    pub expr: Option<Expr>,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum PreprocDirective {
    Include(String, Span),
    Define(String, Option<String>, Span),
    Undef(String, Span),
    Ifdef(String, Span),
    Ifndef(String, Span),
    If(String, Span),
    Else(Span),
    Elif(String, Span),
    Endif(Span),
    Error(String, Span),
    Pragma(String, Span),
    Line(String, Span),
}
