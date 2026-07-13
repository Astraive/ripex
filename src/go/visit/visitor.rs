use crate::go::ast::*;

pub trait Visitor {
    fn visit_program(&mut self, _program: &Program) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
    fn visit_block(&mut self, _block: &Block) {}
    fn visit_decl(&mut self, _decl: &Decl) {}
    fn visit_func_decl(&mut self, _decl: &FuncDecl) {}
    fn visit_var_decl(&mut self, _decl: &VarDecl) {}
    fn visit_const_decl(&mut self, _decl: &ConstDecl) {}
    fn visit_type_decl(&mut self, _decl: &TypeDecl) {}
    fn visit_import_decl(&mut self, _decl: &ImportDecl) {}
    fn visit_assign_stmt(&mut self, _targets: &[Expr]) {}
}
