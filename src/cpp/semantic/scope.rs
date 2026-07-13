use crate::cpp::ast::*;
use crate::cpp::visit::visitor::Visitor;
use crate::cpp::visit::walk::*;
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
    Namespace,
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
        for param in &decl.params {
            if let Some(ref name) = param.name {
                self.declare(name, BindingKind::Param, param.span);
            }
        }
        if let Some(ref body) = decl.body {
            self.visit_block(body);
        }
        self.leave_scope(prev);
    }
    fn visit_var_decl(&mut self, decl: &VarDecl) {
        self.declare(&decl.name, BindingKind::Var, decl.span);
    }
    fn visit_class_decl(&mut self, decl: &ClassDecl) {
        self.declare(&decl.name, BindingKind::Class, decl.span);
        let prev = self.enter_scope(ScopeKind::Class);
        for m in &decl.members {
            if let ClassMember::Decl(d, _) = m {
                self.visit_decl(d);
            }
        }
        self.leave_scope(prev);
    }
    fn visit_struct_decl(&mut self, decl: &StructDecl) {
        self.declare(&decl.name, BindingKind::Struct, decl.span);
    }
    fn visit_enum_decl(&mut self, decl: &EnumDecl) {
        self.declare(&decl.name, BindingKind::Enum, decl.span);
    }
    fn visit_namespace_decl(&mut self, name: &str, members: &[Decl]) {
        self.declare(name, BindingKind::Namespace, Span::ZERO);
        let prev = self.enter_scope(ScopeKind::Namespace);
        for d in members {
            self.visit_decl(d);
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
}
