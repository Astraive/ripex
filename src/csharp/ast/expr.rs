use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64, Span),
    UInt(u64, Span),
    Long(i64, Span),
    ULong(u64, Span),
    Float(f64, Span),
    Double(f64, Span),
    Decimal(f64, Span),
    String(String, Span),
    Char(char, Span),
    Bool(bool, Span),
    Null(Span),
    Ident(String, Span),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Span),
    Unary(UnaryOp, Box<Expr>, Span),
    Call(Box<Expr>, Vec<Expr>, Span),
    Index(Box<Expr>, Box<Expr>, Span),
    Member(Box<Expr>, String, Span),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    NullCoalesce(Box<Expr>, Box<Expr>, Span),
    NullConditional(Box<Expr>, String, Span),
    Lambda(LambdaExpr, Span),
    AnonymousMethod(Vec<Expr>, Box<super::stmt::Block>, Span),
    ObjectInit(String, Vec<MemberInit>, Span),
    CollectionInit(Vec<Expr>, Span),
    Array(Vec<Expr>, Span),
    New(Box<Expr>, Vec<Expr>, Span),
    Typeof(Box<Expr>, Span),
    Nameof(Box<Expr>, Span),
    Sizeof(Box<Expr>, Span),
    Default(Box<Expr>, Span),
    Await(Box<Expr>, Span),
    Paren(Box<Expr>, Span),
    Assign(Box<Expr>, Box<Expr>, Span),
    IsPattern(Box<Expr>, Box<Expr>, Span),
    SwitchExpr(Box<Expr>, Vec<SwitchArm>, Span),
    InterpolatedString(Vec<InterpolatedPart>, Span),
    Throw(Box<Expr>, Span),
    Error(Span),
}

#[derive(Debug, Clone)]
pub struct LambdaExpr {
    pub params: Vec<LambdaParam>,
    pub body: LambdaBody,
    pub is_async: bool,
    pub is_static: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LambdaParam {
    pub name: String,
    pub type_: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum LambdaBody {
    Expr(Box<Expr>),
    Block(Box<super::stmt::Block>),
}

#[derive(Debug, Clone)]
pub struct MemberInit {
    pub name: String,
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SwitchArm {
    pub pattern: Box<Expr>,
    pub when: Option<Box<Expr>>,
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum InterpolatedPart {
    Literal(String, Span),
    Expr(Box<Expr>, Span),
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
    NullCoalesce,
    Is,
    As,
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
    Await,
    IndexFromEnd,
}
