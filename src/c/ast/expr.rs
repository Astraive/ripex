use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64, Span),
    UInt(u64, Span),
    Float(f64, Span),
    String(String, Span),
    Char(char, Span),
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
    Sizeof(Box<Expr>, Span),
    Alignof(Box<Expr>, Span),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    Comma(Vec<Expr>, Span),
    StmtExpr(Vec<super::stmt::Stmt>, Span),
    Paren(Box<Expr>, Span),
    Assign(Box<Expr>, Box<Expr>, Span),
    StringConcat(Vec<String>, Span),
    DeclSpec(DeclSpec, Span),
    Error(Span),
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

#[derive(Debug, Clone)]
pub enum DeclSpec {
    Void,
    Char,
    Short,
    Int,
    Long,
    Float,
    Double,
    Signed,
    Unsigned,
    Struct(String, Option<Vec<StructField>>),
    Union(String, Option<Vec<StructField>>),
    Enum(String, Option<Vec<EnumConstant>>),
    Typedef(Box<Expr>, String),
    TypeName(String),
    Const,
    Volatile,
    Restrict,
    Extern,
    Static,
    Register,
    Inline,
    Atomic,
    Auto,
    ThreadLocal,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub type_: Box<Expr>,
    pub name: String,
    pub bitfield: Option<usize>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumConstant {
    pub name: String,
    pub value: Option<Box<Expr>>,
    pub span: Span,
}
