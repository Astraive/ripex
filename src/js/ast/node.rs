use crate::span::Span;

pub trait AstNode {
    fn span(&self) -> Span;
}
