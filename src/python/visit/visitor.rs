use crate::python::ast::*;

pub trait Visitor {
    fn visit_program(&mut self, _program: &Program) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
    fn visit_func_def(&mut self, _def: &FuncDef) {}
    fn visit_class_def(&mut self, _def: &ClassDef) {}
    fn visit_pattern(&mut self, _pat: &Pattern) {}
}
