use crate::python::ast::*;

pub trait Pass {
    fn pass_expr(&mut self, _expr: &mut Expr) {}
    fn pass_stmt(&mut self, _stmt: &mut Stmt) {}
    fn pass_program(&mut self, program: &mut Program) {
        for stmt in &mut program.stmts {
            self.pass_stmt(stmt);
        }
    }
}

pub fn apply_pass(pass: &mut dyn Pass, program: &mut Program) {
    pass.pass_program(program);
}
