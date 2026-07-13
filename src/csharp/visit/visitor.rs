use crate::csharp::ast::*;

pub trait Visitor {
    fn visit_program(&mut self, _program: &Program) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
    fn visit_block(&mut self, _block: &Block) {}
    fn visit_decl(&mut self, _decl: &Decl) {}
    fn visit_func_decl(&mut self, _decl: &FuncDecl) {}
    fn visit_class_decl(&mut self, _decl: &ClassDecl) {}
    fn visit_struct_decl(&mut self, _decl: &StructDecl) {}
    fn visit_interface_decl(&mut self, _decl: &InterfaceDecl) {}
    fn visit_enum_decl(&mut self, _decl: &EnumDecl) {}
    fn visit_field_decl(&mut self, _decl: &FieldDecl) {}
    fn visit_property_decl(&mut self, _decl: &PropertyDecl) {}
    fn visit_namespace_decl(&mut self, _name: &str, _members: &[Decl]) {}
}
