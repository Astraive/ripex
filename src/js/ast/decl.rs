use super::expr::{ClassMember, ExprRef, Ident, PropName, TypeAnn};
use super::node::AstNode;
use super::pattern::Pat;
use super::stmt::{BlockStmt, Stmt};
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Decl {
    Var(VarDecl),
    Fn(FnDecl),
    Class(ClassDecl),
    TsInterface(TsInterfaceDecl),
    TsTypeAlias(TsTypeAliasDecl),
    TsEnum(TsEnumDecl),
    TsModule(TsModuleDecl),
}

#[derive(Debug, Clone)]
pub struct VarDecl {
    pub span: Span,
    pub kind: VarKind,
    pub decls: Vec<VarDeclarator>,
    pub await_: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum VarKind {
    Var,
    Let,
    Const,
    Using,
}

#[derive(Debug, Clone)]
pub struct VarDeclarator {
    pub span: Span,
    pub name: Pat,
    pub init: Option<ExprRef>,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub span: Span,
    pub id: Ident,
    pub params: Vec<Pat>,
    pub body: Option<BlockStmt>,
    pub generator: bool,
    pub async_: bool,
    pub declare: bool,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
    pub span: Span,
    pub id: Ident,
    pub super_class: Option<ExprRef>,
    pub body: Vec<ClassMember>,
    pub declare: bool,
    pub abstract_: bool,
    pub decorators: Vec<Decorator>,
}

#[derive(Debug, Clone)]
pub struct TsInterfaceDecl {
    pub span: Span,
    pub id: Ident,
    pub extends: Vec<TypeAnn>,
    pub body: Vec<TsInterfaceBody>,
}

#[derive(Debug, Clone)]
pub struct TsTypeAliasDecl {
    pub span: Span,
    pub id: Ident,
    pub type_ann: TypeAnn,
}

#[derive(Debug, Clone)]
pub struct TsEnumDecl {
    pub span: Span,
    pub id: Ident,
    pub members: Vec<TsEnumMember>,
    pub is_const: bool,
}

#[derive(Debug, Clone)]
pub struct TsEnumMember {
    pub span: Span,
    pub id: Ident,
    pub init: Option<ExprRef>,
}

#[derive(Debug, Clone)]
pub struct TsModuleDecl {
    pub span: Span,
    pub id: Ident,
    pub body: Vec<Stmt>,
    pub is_namespace: bool,
}

#[derive(Debug, Clone)]
pub struct TsInterfaceBody {
    pub span: Span,
    pub key: PropName,
    pub value: TypeAnn,
    pub optional: bool,
    pub readonly: bool,
}

#[derive(Debug, Clone)]
pub struct Decorator {
    pub span: Span,
    /// The decorator expression, e.g. `@sealed` -> `sealed`, `@log("x")` -> `log("x")`.
    pub expr: ExprRef,
}

impl AstNode for VarDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for VarDeclarator {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for FnDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ClassDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TsInterfaceDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TsTypeAliasDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TsEnumDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TsEnumMember {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TsModuleDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TsInterfaceBody {
    fn span(&self) -> Span {
        self.span
    }
}
