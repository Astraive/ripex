use crate::arena::Arena;
use crate::js::ast::*;
use crate::js::semantic::bindings::BindingKind;
use crate::js::semantic::symbols::SymbolTable;
use crate::js::visit::visitor::Visitor;
use crate::js::visit::walk::*;
use crate::span::Span;

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
    Module,
    Class,
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

    pub fn build(program: &Program, ast: &Arena<Expr>) -> Self {
        let mut builder = ScopeBuilder::new();
        walk_program(&mut builder, program, ast);
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
    fn visit_fn_decl(&mut self, decl: &FnDecl, ast: &Arena<Expr>) {
        self.declare(&decl.id.name, BindingKind::Function, decl.span);
        let prev = self.enter_scope(ScopeKind::Function);
        for param in &decl.params {
            self.visit_pat(param, ast);
        }
        if let Some(body) = &decl.body {
            self.visit_block_stmt(body, ast);
        }
        self.leave_scope(prev);
    }

    fn visit_fn_expr(&mut self, expr: &FnExpr, ast: &Arena<Expr>) {
        let prev = self.enter_scope(ScopeKind::Function);
        if let Some(id) = &expr.id {
            self.declare(&id.name, BindingKind::Function, expr.span);
        }
        for param in &expr.params {
            self.visit_pat(param, ast);
        }
        if let Some(body) = &expr.body {
            self.visit_block_stmt(body, ast);
        }
        self.leave_scope(prev);
    }

    fn visit_arrow_expr(&mut self, expr: &ArrowExpr, ast: &Arena<Expr>) {
        let prev = self.enter_scope(ScopeKind::Function);
        for param in &expr.params {
            self.visit_pat(param, ast);
        }
        match &expr.body {
            ArrowBody::Block(block) => self.visit_block_stmt(block, ast),
            ArrowBody::Expr(e) => self.visit_expr(*e, ast),
        }
        self.leave_scope(prev);
    }

    fn visit_class_decl(&mut self, decl: &ClassDecl, ast: &Arena<Expr>) {
        self.declare(&decl.id.name, BindingKind::Class, decl.span);
        let prev = self.enter_scope(ScopeKind::Class);
        if let Some(super_class) = &decl.super_class {
            self.visit_expr(*super_class, ast);
        }
        self.leave_scope(prev);
    }

    fn visit_class_expr(&mut self, expr: &ClassExpr, ast: &Arena<Expr>) {
        let prev = self.enter_scope(ScopeKind::Class);
        if let Some(super_class) = &expr.super_class {
            self.visit_expr(*super_class, ast);
        }
        self.leave_scope(prev);
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt, ast: &Arena<Expr>) {
        let prev = self.enter_scope(ScopeKind::Block);
        for s in &stmt.stmts {
            self.visit_stmt(s, ast);
        }
        self.leave_scope(prev);
    }

    fn visit_var_decl(&mut self, decl: &VarDecl, ast: &Arena<Expr>) {
        let kind = match decl.kind {
            VarKind::Var => BindingKind::Var,
            VarKind::Let => BindingKind::Let,
            VarKind::Const => BindingKind::Const,
            VarKind::Using => BindingKind::Let,
        };
        for d in &decl.decls {
            self.declare_from_pat(&d.name, kind, ast);
        }
    }

    fn visit_import_decl(&mut self, decl: &ImportDecl, _ast: &Arena<Expr>) {
        for spec in &decl.specifiers {
            match spec {
                ImportSpecifier::Named(n) => {
                    self.declare(&n.local.name, BindingKind::Import, n.span);
                }
                ImportSpecifier::Default(d) => {
                    self.declare(&d.local.name, BindingKind::Import, d.span);
                }
                ImportSpecifier::Namespace(ns) => {
                    self.declare(&ns.local.name, BindingKind::Import, ns.span);
                }
            }
        }
    }
}

impl ScopeBuilder {
    fn declare_from_pat(&mut self, pat: &Pat, kind: BindingKind, _ast: &Arena<Expr>) {
        match pat {
            Pat::Ident(bi) => {
                self.declare(&bi.id.name, kind, bi.span);
            }
            Pat::Array(ap) => {
                for e in ap.elements.iter().flatten() {
                    self.declare_from_pat(e, kind, _ast);
                }
                if let Some(rest) = &ap.rest {
                    self.declare_from_pat(&rest.arg, kind, _ast);
                }
            }
            Pat::Object(op) => {
                for prop in &op.props {
                    match prop {
                        ObjectPatProp::KeyValue(kv) => self.declare_from_pat(&kv.value, kind, _ast),
                        ObjectPatProp::Shorthand(id) => {
                            self.declare(&id.id.name, kind, id.span);
                        }
                        ObjectPatProp::Rest(rp) => self.declare_from_pat(&rp.arg, kind, _ast),
                    }
                }
                if let Some(rest) = &op.rest {
                    self.declare_from_pat(&rest.arg, kind, _ast);
                }
            }
            Pat::Rest(rp) => self.declare_from_pat(&rp.arg, kind, _ast),
            Pat::Assign(ap) => self.declare_from_pat(&ap.left, kind, _ast),
            _ => {}
        }
    }

    #[allow(dead_code)]
    fn visit_pat_binding(&mut self, pat: &Pat, kind: BindingKind, ast: &Arena<Expr>) {
        self.declare_from_pat(pat, kind, ast);
    }
}
