use super::expr::{Expr, Pattern};
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr, Span),
    Let(LetDecl, Span),
    Item(Item, Span),
    Empty(Span),
}

#[derive(Debug, Clone)]
pub struct LetDecl {
    pub pattern: Pattern,
    pub mutable: bool,
    pub type_ann: Option<Box<Expr>>,
    pub init: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn(FnDecl, Span),
    Struct(StructDecl, Span),
    Enum(EnumDecl, Span),
    Trait(TraitDecl, Span),
    Impl(ImplBlock, Span),
    Use(UseDecl, Span),
    Mod(ModDecl, Span),
    Type(TypeAlias, Span),
    Static(StaticDecl, Span),
    Const(ConstItem, Span),
    Macro(MacroInvocation, Span),
    ExternCrate(String, Span),
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub params: Vec<FnParam>,
    pub return_type: Option<Box<Expr>>,
    pub body: Option<Block>,
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_unsafe: bool,
    pub is_extern: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnParam {
    pub pattern: Pattern,
    pub type_ann: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub bounds: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub fields: Vec<StructField>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub type_ann: Box<Expr>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub variants: Vec<EnumVariant>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TraitDecl {
    pub name: String,
    pub methods: Vec<FnDecl>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub type_name: Box<Expr>,
    pub methods: Vec<FnDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: UsePath,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum UsePath {
    Simple(String, Span),
    Nested(String, Vec<UsePath>, Span),
    Glob(String, Span),
    Self_(String, Span),
}

#[derive(Debug, Clone)]
pub struct ModDecl {
    pub name: String,
    pub items: Vec<Item>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub name: String,
    pub generics: Vec<GenericParam>,
    pub type_: Box<Expr>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct StaticDecl {
    pub name: String,
    pub mutable: bool,
    pub type_: Box<Expr>,
    pub init: Box<Expr>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ConstItem {
    pub name: String,
    pub type_: Option<Box<Expr>>,
    pub init: Box<Expr>,
    pub visibility: Visibility,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MacroInvocation {
    pub name: String,
    pub body: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Pub,
    PubCrate,
    PubSuper,
    PubIn(String),
    Private,
}
