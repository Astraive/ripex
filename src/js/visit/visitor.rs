use crate::arena::Arena;
use crate::js::ast::*;

pub trait Visitor {
    fn visit_program(&mut self, _program: &Program, _ast: &Arena<Expr>) {}
    fn visit_script(&mut self, _script: &Script, _ast: &Arena<Expr>) {}
    fn visit_module(&mut self, _module: &Module, _ast: &Arena<Expr>) {}
    fn visit_module_item(&mut self, _item: &ModuleItem, _ast: &Arena<Expr>) {}
    fn visit_import_decl(&mut self, _decl: &ImportDecl, _ast: &Arena<Expr>) {}
    fn visit_export_decl(&mut self, _decl: &ExportDecl, _ast: &Arena<Expr>) {}
    fn visit_stmt(&mut self, _stmt: &Stmt, _ast: &Arena<Expr>) {}
    fn visit_block_stmt(&mut self, _stmt: &BlockStmt, _ast: &Arena<Expr>) {}
    fn visit_if_stmt(&mut self, _stmt: &IfStmt, _ast: &Arena<Expr>) {}
    fn visit_while_stmt(&mut self, _stmt: &WhileStmt, _ast: &Arena<Expr>) {}
    fn visit_do_while_stmt(&mut self, _stmt: &DoWhileStmt, _ast: &Arena<Expr>) {}
    fn visit_for_stmt(&mut self, _stmt: &ForStmt, _ast: &Arena<Expr>) {}
    fn visit_for_in_stmt(&mut self, _stmt: &ForInStmt, _ast: &Arena<Expr>) {}
    fn visit_for_of_stmt(&mut self, _stmt: &ForOfStmt, _ast: &Arena<Expr>) {}
    fn visit_return_stmt(&mut self, _stmt: &ReturnStmt, _ast: &Arena<Expr>) {}
    fn visit_break_stmt(&mut self, _stmt: &BreakStmt, _ast: &Arena<Expr>) {}
    fn visit_continue_stmt(&mut self, _stmt: &ContinueStmt, _ast: &Arena<Expr>) {}
    fn visit_switch_stmt(&mut self, _stmt: &SwitchStmt, _ast: &Arena<Expr>) {}
    fn visit_throw_stmt(&mut self, _stmt: &ThrowStmt, _ast: &Arena<Expr>) {}
    fn visit_try_stmt(&mut self, _stmt: &TryStmt, _ast: &Arena<Expr>) {}
    fn visit_labelled_stmt(&mut self, _stmt: &LabelledStmt, _ast: &Arena<Expr>) {}
    fn visit_with_stmt(&mut self, _stmt: &WithStmt, _ast: &Arena<Expr>) {}
    fn visit_debugger_stmt(&mut self, _stmt: &DebuggerStmt, _ast: &Arena<Expr>) {}
    fn visit_empty_stmt(&mut self, _stmt: &EmptyStmt, _ast: &Arena<Expr>) {}
    fn visit_decl(&mut self, _decl: &Decl, _ast: &Arena<Expr>) {}
    fn visit_var_decl(&mut self, _decl: &VarDecl, _ast: &Arena<Expr>) {}
    fn visit_fn_decl(&mut self, _decl: &FnDecl, _ast: &Arena<Expr>) {}
    fn visit_class_decl(&mut self, _decl: &ClassDecl, _ast: &Arena<Expr>) {}
    fn visit_expr(&mut self, _expr_ref: ExprRef, _ast: &Arena<Expr>) {}
    fn visit_ident(&mut self, _ident: &Ident, _ast: &Arena<Expr>) {}
    fn visit_lit(&mut self, _lit: &Lit, _ast: &Arena<Expr>) {}
    fn visit_unary_expr(&mut self, _expr: &UnaryExpr, _ast: &Arena<Expr>) {}
    fn visit_binary_expr(&mut self, _expr: &BinaryExpr, _ast: &Arena<Expr>) {}
    fn visit_cond_expr(&mut self, _expr: &ConditionalExpr, _ast: &Arena<Expr>) {}
    fn visit_call_expr(&mut self, _expr: &CallExpr, _ast: &Arena<Expr>) {}
    fn visit_new_expr(&mut self, _expr: &NewExpr, _ast: &Arena<Expr>) {}
    fn visit_member_expr(&mut self, _expr: &MemberExpr, _ast: &Arena<Expr>) {}
    fn visit_array_expr(&mut self, _expr: &ArrayExpr, _ast: &Arena<Expr>) {}
    fn visit_object_expr(&mut self, _expr: &ObjectExpr, _ast: &Arena<Expr>) {}
    fn visit_fn_expr(&mut self, _expr: &FnExpr, _ast: &Arena<Expr>) {}
    fn visit_arrow_expr(&mut self, _expr: &ArrowExpr, _ast: &Arena<Expr>) {}
    fn visit_class_expr(&mut self, _expr: &ClassExpr, _ast: &Arena<Expr>) {}
    fn visit_assign_expr(&mut self, _expr: &AssignmentExpr, _ast: &Arena<Expr>) {}
    fn visit_update_expr(&mut self, _expr: &UpdateExpr, _ast: &Arena<Expr>) {}
    fn visit_yield_expr(&mut self, _expr: &YieldExpr, _ast: &Arena<Expr>) {}
    fn visit_await_expr(&mut self, _expr: &AwaitExpr, _ast: &Arena<Expr>) {}
    fn visit_pat(&mut self, _pat: &Pat, _ast: &Arena<Expr>) {}
    fn visit_binding_ident(&mut self, _ident: &BindingIdent, _ast: &Arena<Expr>) {}
}
