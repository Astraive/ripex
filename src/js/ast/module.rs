use super::decl::Decl;
use super::expr::{ExprRef, Ident};
use super::literal::StrLit;
use super::node::AstNode;
use super::stmt::Stmt;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Module {
    pub span: Span,
    pub body: Vec<ModuleItem>,
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum ModuleItem {
    Stmt(Stmt),
    Decl(Decl),
    Import(ImportDecl),
    Export(ExportDecl),
}

#[derive(Debug, Clone)]
pub struct ImportDecl {
    pub span: Span,
    pub specifiers: Vec<ImportSpecifier>,
    pub source: StrLit,
    /// `import type ...` applies to every specifier in this declaration.
    pub is_type_only: bool,
    pub assertions: Vec<ImportAttribute>,
}

#[derive(Debug, Clone)]
pub struct ImportAttribute {
    pub span: Span,
    pub key: ImportAttributeKey,
    pub value: StrLit,
}

#[derive(Debug, Clone)]
pub enum ImportAttributeKey {
    Ident(Ident),
    StrLit(StrLit),
}

#[derive(Debug, Clone)]
pub enum ImportSpecifier {
    Default(ImportDefault),
    Named(ImportNamed),
    Namespace(ImportNamespace),
}

#[derive(Debug, Clone)]
pub struct ImportDefault {
    pub span: Span,
    pub local: Ident,
}

#[derive(Debug, Clone)]
pub struct ImportNamed {
    pub span: Span,
    pub imported: Ident,
    pub local: Ident,
    /// `import { type Foo } from "..."` is type-only for this specifier.
    pub is_type_only: bool,
}

#[derive(Debug, Clone)]
pub struct ImportNamespace {
    pub span: Span,
    pub local: Ident,
}

#[derive(Debug, Clone)]
pub enum ExportDecl {
    Named(ExportNamed),
    Default(ExportDefault),
    All(ExportAll),
}

#[derive(Debug, Clone)]
pub struct ExportNamed {
    pub span: Span,
    pub specifiers: Vec<ExportSpecifier>,
    pub source: Option<StrLit>,
    pub decl: Option<Box<Decl>>,
    /// `export type { Foo } from "..."` applies to every specifier.
    pub is_type_only: bool,
}

#[derive(Debug, Clone)]
pub struct ExportSpecifier {
    pub span: Span,
    pub local: Ident,
    pub exported: Ident,
    /// `export { type Foo } from "..."` is type-only for this specifier.
    pub is_type_only: bool,
}

#[derive(Debug, Clone)]
pub struct ExportDefault {
    pub span: Span,
    pub decl: ExprRef,
    pub has_assign: bool,
}

#[derive(Debug, Clone)]
pub struct ExportAll {
    pub span: Span,
    pub source: StrLit,
    /// `export type * from "..."` is a type-only re-export.
    pub is_type_only: bool,
}

impl AstNode for Module {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ImportDecl {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ImportAttribute {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ImportDefault {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ImportNamed {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ImportNamespace {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ExportNamed {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ExportSpecifier {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ExportDefault {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ExportAll {
    fn span(&self) -> Span {
        self.span
    }
}
