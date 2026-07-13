use crate::rust::ast::*;

pub trait Visitor {
    fn visit_program(&mut self, _program: &Program) {}
    fn visit_stmt(&mut self, _stmt: &Stmt) {}
    fn visit_expr(&mut self, _expr: &Expr) {}
    fn visit_block(&mut self, _block: &Block) {}
    fn visit_item(&mut self, _item: &Item) {}
    fn visit_fn_decl(&mut self, _decl: &FnDecl) {}
    fn visit_struct_decl(&mut self, _decl: &StructDecl) {}
    fn visit_enum_decl(&mut self, _decl: &EnumDecl) {}
    fn visit_trait_decl(&mut self, _decl: &TraitDecl) {}
    fn visit_pattern(&mut self, _pat: &Pattern) {}
}
