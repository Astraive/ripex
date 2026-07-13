use crate::arena::Arena;
use crate::js::ast::{Expr, Program};

use super::scope::ScopeTree;

pub struct Resolver;

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    pub fn new() -> Self {
        Resolver
    }

    pub fn resolve(program: &Program, ast: &Arena<Expr>) -> ScopeTree {
        ScopeTree::build(program, ast)
    }
}
