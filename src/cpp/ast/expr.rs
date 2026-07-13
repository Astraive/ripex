use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64, Span),
    UInt(u64, Span),
    Float(f64, Span),
    String(String, Span),
    Char(char, Span),
    Bool(bool, Span),
    NullPtr(Span),
    Ident(String, Span),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Call(Box<Expr>, Vec<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    Member(Box<Expr>, String, Span),
    Arrow(Box<Expr>, String, Span),
    Deref(Box<Expr>, Span),
    Ref(Box<Expr>, Span),
    Cast(Box<Expr>, Box<Expr>, Span),
    DynamicCast(Box<Expr>, Box<Expr>, Span),
    StaticCast(Box<Expr>, Box<Expr>, Span),
    ConstCast(Box<Expr>, Box<Expr>, Span),
    ReinterpretCast(Box<Expr>, Box<Expr>, Span),
    Sizeof(Box<Expr>, Span),
    Alignof(Box<Expr>, Span),
    Typeid(Box<Expr>, Span),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    Comma(Vec<Expr>, Span),
    Lambda(LambdaExpr, Span),
    New(Box<Expr>, Vec<Expr>, Span),
    Delete(Box<Expr>, Span),
    This(Span),
    Paren(Box<Expr>, Span),
    Assign(Box<Expr>, Box<Expr>, Span),
    Template(Box<Expr>, Vec<Expr>, Span),
    BraceInit(Vec<Expr>, Span),
    Error(Span),
}

#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub captures: Vec<LambdaCapture>,
    pub params: Vec<ParamDecl>,
    pub return_type: Option<Box<Expr>>,
    pub body: Box<super::stmt::Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LambdaCapture {
    pub by_ref: bool,
    pub name: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub type_: Box<Expr>,
    pub name: Option<String>,
    pub default: Option<Box<Expr>>,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Deref,
    Ref,
    Plus,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}
