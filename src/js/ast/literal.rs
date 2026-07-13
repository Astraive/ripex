use super::expr::ExprRef;
use super::node::AstNode;
use crate::span::Span;

#[derive(Debug, Clone)]
pub enum Lit {
    Str(StrLit),
    Num(NumLit),
    Bool(BoolLit),
    Null(NullLit),
    BigInt(BigIntLit),
    RegExp(RegExpLit),
    Template(TemplateLit),
}

#[derive(Debug, Clone)]
pub struct StrLit {
    pub span: Span,
    pub value: String,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct NumLit {
    pub span: Span,
    pub value: f64,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct BoolLit {
    pub span: Span,
    pub value: bool,
}

#[derive(Debug, Clone)]
pub struct NullLit {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BigIntLit {
    pub span: Span,
    pub value: String,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct RegExpLit {
    pub span: Span,
    pub pattern: String,
    pub flags: String,
}

#[derive(Debug, Clone)]
pub struct TemplateLit {
    pub span: Span,
    pub quasis: Vec<TemplateElement>,
    pub expressions: Vec<ExprRef>,
}

#[derive(Debug, Clone)]
pub struct TemplateElement {
    pub span: Span,
    pub value: String,
    pub tail: bool,
}

impl AstNode for StrLit {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for NumLit {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for BoolLit {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for NullLit {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for BigIntLit {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for RegExpLit {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TemplateLit {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TemplateElement {
    fn span(&self) -> Span {
        self.span
    }
}
