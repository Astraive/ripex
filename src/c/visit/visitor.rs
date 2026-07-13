use crate::c::ast::*;

pub trait Visitor {
    fn visit_program(&mut self, _program: &Program) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
    fn visit_block(&mut self, _block: &Block) {}
    fn visit_func_decl(&mut self, _decl: &FuncDecl) {}
    fn visit_var_decl(&mut self, _decl: &VarDecl) {}
}
