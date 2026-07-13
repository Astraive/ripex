use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Bool(bool, Span),
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
    Char(char, Span),
    Ident(String, Span),
    Path(Vec<String>, Span),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Call(Box<Expr>, Vec<Expr>, Span),
    MethodCall(Box<Expr>, String, Vec<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    Field(Box<Expr>, String, Span),
    Tuple(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    Struct(String, Vec<FieldInit>, Option<Box<Expr>>, Span),
    Closure(Vec<Pattern>, Box<Expr>, Span),
    Block(Box<super::stmt::Block>, Span),
    If(Box<Expr>, Box<super::stmt::Block>, Option<Box<Expr>>, Span),
    Match(Box<Expr>, Vec<MatchArm>, Span),
    While(Box<Expr>, Box<super::stmt::Block>, Span),
    Loop(Box<super::stmt::Block>, Span),
    For(Box<Pattern>, Box<Expr>, Box<super::stmt::Block>, Span),
    Return(Option<Box<Expr>>, Span),
    Break(Option<Box<Expr>>, Span),
    Continue(Span),
    Paren(Box<Expr>, Span),
    Async(Box<Expr>, Span),
    Await(Box<Expr>, Span),
    Ref(Box<Expr>, bool, Span),
    Deref(Box<Expr>, Span),
    Cast(Box<Expr>, Box<Expr>, Span),
    Error(Span),
}

#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: String,
    pub value: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub patterns: Vec<Pattern>,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard(Span),
    Ident(String, Span),
    Lit(Box<Expr>, Span),
    Tuple(Vec<Pattern>, Span),
    Struct(String, Vec<FieldPattern>, Span),
    Range(Box<Pattern>, Box<Pattern>, Span),
    Or(Vec<Pattern>, Span),
    Ref(Box<Pattern>, bool, Span),
    Slice(Vec<Pattern>, Span),
    Rest(Span),
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Box<Pattern>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
    Range,
    RangeInclusive,
    Pipe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
    Ref,
    RefMut,
}
