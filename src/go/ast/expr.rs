use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Bool(bool, Span),
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
    Nil(Span),
    Ident(String, Span),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Call(Box<Expr>, Vec<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    Selector(Box<Expr>, String, Span),
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>, Span),
    Array(Vec<Expr>, Span),
    StructLit(String, Vec<FieldInit>, Span),
    MapLit(Vec<(Expr, Expr)>, Span),
    FuncLit(Box<FuncType>, Box<Block>, Span),
    Paren(Box<Expr>, Span),
    TypeAssert(Box<Expr>, Box<Expr>, Span),
    CompositeLit(Box<Expr>, Vec<Expr>, Span),
}

#[derive(Debug, Clone)]
pub struct FuncType {
    pub params: Vec<(String, Box<Expr>)>,
    pub returns: Vec<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldInit {
    pub name: String,
    pub value: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<super::stmt::Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
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
    BitClear,
    Shl,
    Shr,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    ShlAssign,
    ShrAssign,
    BitClearAssign,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
    Ref,
    Receive,
    Plus,
}
