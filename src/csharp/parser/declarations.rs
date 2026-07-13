use super::super::ast::expr::Expr;
use super::super::ast::stmt::*;
use super::state::Parser;

impl Parser {
    pub fn parse_operator_overload(&mut self) -> Decl {
        let _start = self.peek_token().span.start;
        Decl::Operator(
            OperatorDecl {
                op: String::new(),
                return_type: Box::new(Expr::Ident("void".to_string(), self.peek_token().span)),
                params: Vec::new(),
                body: None,
                span: self.peek_token().span,
            },
            self.peek_token().span,
        )
    }

    pub fn parse_constructor(&mut self) -> Decl {
        let _start = self.peek_token().span.start;
        Decl::Constructor(
            ConstructorDecl {
                params: Vec::new(),
                body: None,
                initializer: None,
                visibility: Visibility::None,
                is_static: false,
                span: self.peek_token().span,
            },
            self.peek_token().span,
        )
    }
}
