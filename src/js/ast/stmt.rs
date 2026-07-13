use super::decl::Decl;
use super::expr::ExprRef;
use super::expr::Ident;
use super::node::AstNode;
use super::pattern::Pat;
use crate::span::Span;

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Stmt {
    Block(BlockStmt),
    Empty(EmptyStmt),
    Expr(ExprStmt),
    If(IfStmt),
    Switch(SwitchStmt),
    For(ForStmt),
    ForIn(ForInStmt),
    ForOf(ForOfStmt),
    While(WhileStmt),
    DoWhile(DoWhileStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Return(ReturnStmt),
    Throw(ThrowStmt),
    Try(TryStmt),
    Debugger(DebuggerStmt),
    Labelled(LabelledStmt),
    Decl(Decl),
    With(WithStmt),
}

#[derive(Debug, Clone)]
pub struct BlockStmt {
    pub span: Span,
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct EmptyStmt {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub span: Span,
    pub expr: ExprRef,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub span: Span,
    pub test: ExprRef,
    pub consequent: Box<Stmt>,
    pub alternate: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct SwitchStmt {
    pub span: Span,
    pub discriminant: ExprRef,
    pub cases: Vec<SwitchCase>,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub span: Span,
    pub test: Option<ExprRef>,
    pub consequent: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub span: Span,
    pub init: Option<ForInit>,
    pub test: Option<ExprRef>,
    pub update: Option<ExprRef>,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub enum ForInit {
    Expr(ExprRef),
    Decl(Box<Decl>),
}

#[derive(Debug, Clone)]
pub struct ForInStmt {
    pub span: Span,
    pub left: ForInit,
    pub right: ExprRef,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ForOfStmt {
    pub span: Span,
    pub left: ForInit,
    pub right: ExprRef,
    pub body: Box<Stmt>,
    pub await_: bool,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub span: Span,
    pub test: ExprRef,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct DoWhileStmt {
    pub span: Span,
    pub body: Box<Stmt>,
    pub test: ExprRef,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub span: Span,
    pub label: Option<Ident>,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub span: Span,
    pub label: Option<Ident>,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub span: Span,
    pub arg: Option<ExprRef>,
}

#[derive(Debug, Clone)]
pub struct ThrowStmt {
    pub span: Span,
    pub arg: ExprRef,
}

#[derive(Debug, Clone)]
pub struct TryStmt {
    pub span: Span,
    pub block: BlockStmt,
    pub handler: Option<CatchClause>,
    pub finalizer: Option<BlockStmt>,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub span: Span,
    pub param: Option<Pat>,
    pub body: BlockStmt,
}

#[derive(Debug, Clone)]
pub struct DebuggerStmt {
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LabelledStmt {
    pub span: Span,
    pub label: Ident,
    pub body: Box<Stmt>,
}

#[derive(Debug, Clone)]
pub struct WithStmt {
    pub span: Span,
    pub object: ExprRef,
    pub body: Box<Stmt>,
}

impl AstNode for BlockStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for EmptyStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ExprStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for IfStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for SwitchStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for SwitchCase {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ForStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ForInStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ForOfStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for WhileStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for DoWhileStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for BreakStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ContinueStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ReturnStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for ThrowStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for TryStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for CatchClause {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for DebuggerStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for LabelledStmt {
    fn span(&self) -> Span {
        self.span
    }
}
impl AstNode for WithStmt {
    fn span(&self) -> Span {
        self.span
    }
}
