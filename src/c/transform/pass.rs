use crate::c::ast::*;

pub trait Pass {
    fn pass_expr(&mut self, _expr: &mut Expr) {}
    fn pass_stmt(&mut self, _stmt: &mut Stmt) {}
    fn pass_program(&mut self, program: &mut Program) {
        for stmt in &mut program.decls {
            self.pass_stmt(stmt);
        }
    }
}
