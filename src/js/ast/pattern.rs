use super::expr::{ExprRef, Ident, PropName, TypeAnn};
use super::node::AstNode;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Pat {
    Ident(BindingIdent),
    Object(ObjectPat),
    Array(ArrayPat),
    Rest(RestPat),
    Assign(AssignPat),
    Expr(ExprRef),
    Invalid(InvalidPat),
}

#[derive(Debug, Clone)]
pub struct BindingIdent {
    pub span: Span,
    pub id: Ident,
    pub type_ann: Option<TypeAnn>,
    pub optional: bool,
}

#[derive(Debug, Clone)]
pub struct ObjectPat {
    pub span: Span,
    pub props: Vec<ObjectPatProp>,
    pub rest: Option<Box<RestPat>>,
}

#[derive(Debug, Clone)]
pub enum ObjectPatProp {
    KeyValue(KeyValuePatProp),
    Shorthand(BindingIdent),
    Rest(RestPat),
}

#[derive(Debug, Clone)]
pub struct KeyValuePatProp {
    pub span: Span,
    pub key: PropName,
    pub value: Box<Pat>,
}

#[derive(Debug, Clone)]
pub struct ArrayPat {
    pub span: Span,
    pub elements: Vec<Option<Pat>>,
    pub rest: Option<Box<RestPat>>,
}

#[derive(Debug, Clone)]
pub struct RestPat {
    pub span: Span,
    pub arg: Box<Pat>,
}

#[derive(Debug, Clone)]
pub struct AssignPat {
    pub span: Span,
    pub left: Box<Pat>,
    pub right: ExprRef,
}

#[derive(Debug, Clone)]
pub struct InvalidPat {
    pub span: Span,
}

impl AstNode for BindingIdent {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ObjectPat {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for KeyValuePatProp {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ArrayPat {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for RestPat {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for AssignPat {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for InvalidPat {
    fn span(&self) -> Span {
        self.span
    }
}
