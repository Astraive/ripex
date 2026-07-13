use super::expr::Expr;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, Span),
    Decl(Decl, Span),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>, Span),
    Switch(Expr, Vec<CaseSection>, Span),
    While(Expr, Box<Stmt>, Span),
    Do(Box<Stmt>, Expr, Span),
    For(
        Option<Box<Stmt>>,
        Option<Expr>,
        Option<Box<Stmt>>,
        Box<Stmt>,
        Span,
    ),
    Foreach(String, Expr, Box<Stmt>, Span),
    Return(Option<Expr>, Span),
    YieldReturn(Expr, Span),
    YieldBreak(Span),
    Break(Span),
    Continue(Span),
    Goto(String, Span),
    GotoCase(Expr, Span),
    GotoDefault(Span),
    Throw(Option<Expr>, Span),
    Try(Box<Stmt>, Vec<CatchClause>, Option<Box<Stmt>>, Span),
    Checked(Box<Stmt>, Span),
    Unchecked(Box<Stmt>, Span),
    Lock(Expr, Box<Stmt>, Span),
    Using(Expr, Box<Stmt>, Span),
    Fixed(Expr, Box<Stmt>, Span),
    Unsafe(Box<Stmt>, Span),
    Block(Block, Span),
    Label(String, Span),
    LocalFunc(FuncDecl, Span),
    Empty(Span),
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Namespace(String, Vec<Decl>, Span),
    Class(ClassDecl, Span),
    Struct(StructDecl, Span),
    Interface(InterfaceDecl, Span),
    Record(ClassDecl, Span),
    Enum(EnumDecl, Span),
    Delegate(DelegateDecl, Span),
    Event(EventDecl, Span),
    Property(PropertyDecl, Span),
    Field(FieldDecl, Span),
    Method(FuncDecl, Span),
    Constructor(ConstructorDecl, Span),
    Destructor(DestructorDecl, Span),
    Operator(OperatorDecl, Span),
    Conversion(ConversionDecl, Span),
    Using(UsingDecl, Span),
    UsingStatic(String, Span),
    ExternAlias(String, Span),
}

#[derive(Debug, Clone)]
pub struct FuncDecl {
    pub name: String,
    pub return_type: Box<Expr>,
    pub params: Vec<ParamDecl>,
    pub body: Option<Block>,
    pub is_async: bool,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_abstract: bool,
    pub is_sealed: bool,
    pub is_unsafe: bool,
    pub is_extern: bool,
    pub is_partial: bool,
    pub visibility: Visibility,
    pub type_params: Vec<TypeParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ParamDecl {
    pub type_: Box<Expr>,
    pub name: String,
    pub is_ref: bool,
    pub is_out: bool,
    pub is_in: bool,
    pub is_params: bool,
    pub is_this: bool,
    pub default: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: String,
    pub constraints: Vec<TypeConstraint>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum TypeConstraint {
    Class(Span),
    Struct(Span),
    NotNull(Span),
    Unmanaged(Span),
    BaseType(Box<Expr>, Span),
    New(Span),
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub name: String,
    pub base: Option<Box<Expr>>,
    pub interfaces: Vec<Box<Expr>>,
    pub members: Vec<Decl>,
    pub is_static: bool,
    pub is_abstract: bool,
    pub is_sealed: bool,
    pub is_partial: bool,
    pub is_readonly: bool,
    pub visibility: Visibility,
    pub type_params: Vec<TypeParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub interfaces: Vec<Box<Expr>>,
    pub members: Vec<Decl>,
    pub is_readonly: bool,
    pub is_partial: bool,
    pub visibility: Visibility,
    pub type_params: Vec<TypeParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub bases: Vec<Box<Expr>>,
    pub members: Vec<Decl>,
    pub is_partial: bool,
    pub visibility: Visibility,
    pub type_params: Vec<TypeParam>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub members: Vec<EnumMember>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DelegateDecl {
    pub name: String,
    pub return_type: Box<Expr>,
    pub params: Vec<ParamDecl>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EventDecl {
    pub type_: Box<Expr>,
    pub name: String,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct PropertyDecl {
    pub type_: Box<Expr>,
    pub name: String,
    pub getter: Option<Box<Stmt>>,
    pub setter: Option<Box<Stmt>>,
    pub init: Option<Box<Expr>>,
    pub is_auto: bool,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub type_: Box<Expr>,
    pub name: String,
    pub init: Option<Expr>,
    pub is_const: bool,
    pub is_readonly: bool,
    pub is_static: bool,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstructorDecl {
    pub params: Vec<ParamDecl>,
    pub body: Option<Block>,
    pub initializer: Option<ConstructorInit>,
    pub visibility: Visibility,
    pub is_static: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ConstructorInit {
    Base(Vec<Expr>),
    This(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub struct DestructorDecl {
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct OperatorDecl {
    pub op: String,
    pub return_type: Box<Expr>,
    pub params: Vec<ParamDecl>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConversionDecl {
    pub is_explicit: bool,
    pub return_type: Box<Expr>,
    pub param: ParamDecl,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UsingDecl {
    pub namespace: String,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CaseSection {
    pub labels: Vec<CaseLabel>,
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum CaseLabel {
    Case(Expr, Span),
    Default(Span),
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub type_: Option<Box<Expr>>,
    pub name: Option<String>,
    pub when: Option<Box<Expr>>,
    pub body: Box<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    ProtectedInternal,
    PrivateProtected,
    None,
}
