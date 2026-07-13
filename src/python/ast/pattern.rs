use super::expr::Expr;
use crate::span::Span;

#[derive(Debug, Clone)]
pub struct MatchCase {
    pub pattern: Box<Pattern>,
    pub guard: Option<Box<Expr>>,
    pub body: Vec<super::stmt::Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Wildcard(Span),
    Value(String, Span),
    Literal(Box<Expr>, Span),
    Capture(String, Span),
    Sequence(Vec<Pattern>, Span),
    Mapping(Vec<(Pattern, Pattern)>, Option<Box<Pattern>>, Span),
    Class(String, Vec<Pattern>, Vec<(String, Pattern)>, Span),
    Or(Vec<Pattern>, Span),
    As(Box<Pattern>, String, Span),
    Guard(Box<Pattern>, Box<Expr>, Span),
    Group(Box<Pattern>, Span),
}
