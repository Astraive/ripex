use super::bindings::BindingKind;
use super::symbols::SymbolTable;
use crate::rust::ast::*;
use crate::rust::visit::visitor::Visitor;
use crate::rust::visit::walk::*;
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
    Trait,
    Impl,
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
        let mut b = ScopeBuilder::new();
        walk_program(&mut b, program);
        b.tree
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
    fn visit_fn_decl(&mut self, decl: &FnDecl) {
        self.declare(&decl.name, BindingKind::Function, decl.span);
        let prev = self.enter_scope(ScopeKind::Function);
        for p in &decl.params {
            self.visit_pattern(&p.pattern);
        }
        if let Some(ref body) = decl.body {
            self.visit_block(body);
        }
        self.leave_scope(prev);
    }
    fn visit_struct_decl(&mut self, decl: &StructDecl) {
        self.declare(&decl.name, BindingKind::Struct, decl.span);
    }
    fn visit_enum_decl(&mut self, decl: &EnumDecl) {
        self.declare(&decl.name, BindingKind::Enum, decl.span);
    }
    fn visit_trait_decl(&mut self, decl: &TraitDecl) {
        self.declare(&decl.name, BindingKind::Trait, decl.span);
        let prev = self.enter_scope(ScopeKind::Trait);
        for m in &decl.methods {
            self.visit_fn_decl(m);
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
    fn visit_pattern(&mut self, pat: &Pattern) {
        if let Pattern::Ident(name, _) = pat {
            self.declare(name, BindingKind::Param, Span::ZERO);
        }
        walk_pattern(self, pat);
    }
    fn visit_stmt(&mut self, stmt: &Stmt) {
        if let Stmt::Let(l, _) = stmt {
            if let Pattern::Ident(name, _) = &l.pattern {
                self.declare(name, BindingKind::Let, l.span);
            }
        }
        walk_stmt(self, stmt);
    }
}
