use super::expr::Expr;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, Span),
    Decl(Decl, Span),
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
    RangeFor(Box<Stmt>, Expr, Box<Stmt>, Span),
    Return(Option<Expr>, Span),
    Break(Span),
    Continue(Span),
    Goto(String, Span),
    Label(String, Span),
    Try(Box<Stmt>, Vec<CatchClause>, Option<Box<Stmt>>, Span),
    Throw(Option<Expr>, Span),
    Block(Block, Span),
    Empty(Span),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Func(FuncDecl, Span),
    Var(VarDecl, Span),
    Namespace(String, Vec<Decl>, Span),
    Using(String, Span),
    UsingNamespace(String, Span),
    Template(TemplateDecl, Span),
    Class(ClassDecl, Span),
    Struct(StructDecl, Span),
    Enum(EnumDecl, Span),
    Typedef(TypedefDecl, Span),
    TypeAlias(String, Box<Expr>, Span),
    StaticAssert(Expr, String, Span),
    Asm(String, Span),
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub return_type: Box<Expr>,
    pub params: Vec<super::expr::ParamDecl>,
    pub is_variadic: bool,
    pub body: Option<Block>,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_const: bool,
    pub is_pure: bool,
    pub is_constexpr: bool,
    pub is_inline: bool,
    pub is_explicit: bool,
    pub is_static: bool,
    pub is_friend: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub type_: Box<Expr>,
    pub name: String,
    pub init: Option<Expr>,
    pub is_const: bool,
    pub is_constexpr: bool,
    pub is_static: bool,
    pub is_extern: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TemplateDecl {
    pub params: Vec<TemplateParam>,
    pub decl: Box<Decl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TemplateParam {
    Type(String, Span),
    Value(Box<Expr>, String, Option<Box<Expr>>, Span),
    Template(String, Span),
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub bases: Vec<BaseSpec>,
    pub members: Vec<ClassMember>,
    pub is_final: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BaseSpec {
    pub name: String,
    pub access: AccessSpec,
    pub is_virtual: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ClassMember {
    Decl(Decl, Span),
    Access(AccessSpec, Span),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessSpec {
    Public,
    Private,
    Protected,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub members: Vec<StructMember>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructMember {
    pub type_: Box<Expr>,
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub values: Vec<EnumValue>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypedefDecl {
    pub name: String,
    pub type_: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseClause {
    pub expr: Option<Expr>,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub type_: Box<Expr>,
    pub name: Option<String>,
    pub body: Box<Stmt>,
    pub span: Span,
}
