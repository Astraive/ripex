use crate::cpp::ast::*;

pub trait Pass {
    fn pass_expr(&mut self, _expr: &mut Expr) {}
    fn pass_stmt(&mut self, _stmt: &mut Stmt) {}
    fn pass_decl(&mut self, _decl: &mut Decl) {}
    fn pass_program(&mut self, program: &mut Program) {
        for decl in &mut program.decls {
            self.pass_decl(decl);
        }
    }
}
