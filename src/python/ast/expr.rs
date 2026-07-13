use super::literal::Literal;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal, Span),
    Ident(String, Span),
    Attribute(Box<Expr>, String, Span),
    Subscript(Box<Expr>, Box<Expr>, Span),
    Slice(
        Option<Box<Expr>>,
        Option<Box<Expr>>,
        Option<Box<Expr>>,
        Span,
    ),
    Call(Box<Expr>, Vec<Expr>, Vec<Keyword>, Span),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    IfElse(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    Lambda(Vec<String>, Box<Expr>, Span),
    List(Vec<Expr>, Span),
    Tuple(Vec<Expr>, Span),
    Dict(Vec<(Expr, Expr)>, Span),
    Set(Vec<Expr>, Span),
    ListComp(Box<Expr>, Vec<Comprehension>, Span),
    SetComp(Box<Expr>, Vec<Comprehension>, Span),
    DictComp(Box<Expr>, Vec<Comprehension>, Span),
    Generator(Box<Expr>, Vec<Comprehension>, Span),
    Await(Box<Expr>, Span),
    Yield(Option<Box<Expr>>, Span),
    YieldFrom(Box<Expr>, Span),
    Starred(Box<Expr>, Span),
    Walrus(Box<Expr>, Box<Expr>, Span),
    FString(Vec<FStringPart>, Span),
    Compare(Box<Expr>, Vec<CmpOp>, Vec<Box<Expr>>, Span),
    Paren(Box<Expr>, Span),
    Ellipsis(Span),
    Match(Box<Expr>, Vec<super::pattern::MatchCase>, Span),
    Error(Span),
}

#[derive(Debug, Clone)]
pub struct Keyword {
    pub name: Option<String>,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Comprehension {
    pub target: Box<Expr>,
    pub iter: Box<Expr>,
    pub ifs: Vec<Box<Expr>>,
    pub is_async: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum FStringPart {
    Text(String, Span),
    Expr(Box<Expr>, Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Mod,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    And,
    Or,
    MatMult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Pos,
    Not,
    Invert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Is,
    IsNot,
    In,
    NotIn,
}
