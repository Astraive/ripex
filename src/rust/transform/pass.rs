use crate::rust::ast::*;

pub trait Pass {
    fn pass_expr(&mut self, _expr: &mut Expr) {}
    fn pass_stmt(&mut self, _stmt: &mut Stmt) {}
    fn pass_item(&mut self, _item: &mut Item) {}
    fn pass_program(&mut self, program: &mut Program) {
        for item in &mut program.items {
            self.pass_item(item);
        }
    }
}
