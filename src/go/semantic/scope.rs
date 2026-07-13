use crate::go::ast::*;
use crate::go::visit::visitor::Visitor;
use crate::go::visit::walk::*;
use crate::span::Span;

use super::bindings::BindingKind;
use super::symbols::SymbolTable;

#[derive(Debug, Clone)]
pub struct Scope {
    pub id: usize,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub kind: ScopeKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeKind {
    Global,
    Function,
    Block,
}

#[derive(Debug, Clone)]
pub struct ScopeTree {
    scopes: Vec<Scope>,
    symbols: SymbolTable,
}

impl ScopeTree {
    pub fn new() -> Self {
        ScopeTree {
            scopes: Vec::new(),
            symbols: SymbolTable::new(),
        }
    }

    pub fn build(program: &Program) -> Self {
        let mut builder = ScopeBuilder::new();
        walk_program(&mut builder, program);
        builder.tree
    }

    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }

    pub fn symbols(&self) -> &SymbolTable {
        &self.symbols
    }

    pub fn root(&self) -> Option<&Scope> {
        self.scopes.first()
    }
}

impl Default for ScopeTree {
    fn default() -> Self {
        ScopeTree::new()
    }
}

struct ScopeBuilder {
    tree: ScopeTree,
    current_scope: usize,
}

impl ScopeBuilder {
    fn new() -> Self {
        let mut tree = ScopeTree::new();
        tree.scopes.push(Scope {
            id: 0,
            parent: None,
            children: Vec::new(),
            kind: ScopeKind::Global,
        });
        ScopeBuilder {
            tree,
            current_scope: 0,
        }
    }

    fn enter_scope(&mut self, kind: ScopeKind) -> usize {
        let id = self.tree.scopes.len();
        self.tree.scopes.push(Scope {
            id,
            parent: Some(self.current_scope),
            children: Vec::new(),
            kind,
        });
        self.tree.scopes[self.current_scope].children.push(id);
        let prev = self.current_scope;
        self.current_scope = id;
        prev
    }

    fn leave_scope(&mut self, prev: usize) {
        self.current_scope = prev;
    }

    fn declare(&mut self, name: &str, kind: BindingKind, span: Span) {
        self.tree
            .symbols
            .insert(name.to_string(), kind, span, self.current_scope);
    }
}

impl Visitor for ScopeBuilder {
    fn visit_func_decl(&mut self, decl: &FuncDecl) {
        self.declare(&decl.name, BindingKind::Function, decl.span);
        let prev = self.enter_scope(ScopeKind::Function);
        for (name, _) in &decl.params {
            self.declare(name, BindingKind::Param, decl.span);
        }
        if let Some(ref body) = decl.body {
            self.visit_block(body);
        }
        self.leave_scope(prev);
    }

    fn visit_block(&mut self, block: &Block) {
        let prev = self.enter_scope(ScopeKind::Block);
        for s in &block.stmts {
            self.visit_stmt(s);
        }
        self.leave_scope(prev);
    }

    fn visit_var_decl(&mut self, decl: &VarDecl) {
        for name in &decl.names {
            self.declare(name, BindingKind::Var, decl.span);
        }
    }

    fn visit_const_decl(&mut self, decl: &ConstDecl) {
        for name in &decl.names {
            self.declare(name, BindingKind::Const, decl.span);
        }
    }

    fn visit_type_decl(&mut self, decl: &TypeDecl) {
        self.declare(&decl.name, BindingKind::Type, decl.span);
    }

    fn visit_import_decl(&mut self, decl: &ImportDecl) {
        if let Some(ref alias) = decl.alias {
            self.declare(alias, BindingKind::Import, decl.span);
        }
    }

    fn visit_assign_stmt(&mut self, stmt: &[Expr]) {
        for expr in stmt {
            self.visit_expr(expr);
        }
    }
}
