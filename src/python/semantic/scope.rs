use super::bindings::BindingKind;
use super::symbols::SymbolTable;
use crate::python::ast::*;
use crate::python::visit::visitor::Visitor;
use crate::python::visit::walk::*;
use crate::span::Span;

fn expr_span(expr: &Expr) -> Span {
    match expr {
        Expr::Literal(_, s)
        | Expr::Ident(_, s)
        | Expr::Attribute(_, _, s)
        | Expr::Subscript(_, _, s)
        | Expr::Slice(_, _, _, s)
        | Expr::Call(_, _, _, s)
        | Expr::Binary(_, _, _, s)
        | Expr::Unary(_, _, s)
        | Expr::IfElse(_, _, _, s)
        | Expr::Lambda(_, _, s)
        | Expr::List(_, s)
        | Expr::Tuple(_, s)
        | Expr::Dict(_, s)
        | Expr::Set(_, s)
        | Expr::ListComp(_, _, s)
        | Expr::SetComp(_, _, s)
        | Expr::DictComp(_, _, s)
        | Expr::Generator(_, _, s)
        | Expr::Await(_, s)
        | Expr::Yield(_, s)
        | Expr::YieldFrom(_, s)
        | Expr::Starred(_, s)
        | Expr::Walrus(_, _, s)
        | Expr::FString(_, s)
        | Expr::Compare(_, _, _, s)
        | Expr::Paren(_, s)
        | Expr::Match(_, _, s)
        | Expr::Ellipsis(s)
        | Expr::Error(s) => *s,
    }
}

fn stmt_span(stmt: &Stmt) -> Span {
    match stmt {
        Stmt::Expr(_, s)
        | Stmt::Assign(_, _, s)
        | Stmt::AugAssign(_, _, _, s)
        | Stmt::AnnAssign(_, _, _, s)
        | Stmt::If(_, _, _, s)
        | Stmt::While(_, _, _, s)
        | Stmt::For(_, _, _, _, s)
        | Stmt::With(_, _, s)
        | Stmt::Match(_, _, s)
        | Stmt::Return(_, s)
        | Stmt::Yield(_, s)
        | Stmt::Raise(_, _, s)
        | Stmt::Assert(_, _, s)
        | Stmt::Break(s)
        | Stmt::Continue(s)
        | Stmt::Pass(s)
        | Stmt::Delete(_, s)
        | Stmt::Global(_, s)
        | Stmt::Nonlocal(_, s)
        | Stmt::Import(_, s)
        | Stmt::ImportFrom(_, _, _, s)
        | Stmt::Try(_, _, _, _, s)
        | Stmt::FuncDef(_, s)
        | Stmt::ClassDef(_, s)
        | Stmt::Async(_, s)
        | Stmt::Block(_, s)
        | Stmt::Empty(s) => *s,
    }
}

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
    Class,
    Block,
    Comprehension,
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
    fn visit_func_def(&mut self, def: &FuncDef) {
        self.declare(&def.name, BindingKind::Function, def.span);
        let prev = self.enter_scope(ScopeKind::Function);
        for a in &def.args {
            self.declare(&a.name, BindingKind::Param, a.span);
        }
        for s in &def.body {
            self.visit_stmt(s);
        }
        self.leave_scope(prev);
    }

    fn visit_class_def(&mut self, def: &ClassDef) {
        self.declare(&def.name, BindingKind::Class, def.span);
        let prev = self.enter_scope(ScopeKind::Class);
        for s in &def.body {
            self.visit_stmt(s);
        }
        self.leave_scope(prev);
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::FuncDef(f, _) => self.visit_func_def(f),
            Stmt::ClassDef(c, _) => self.visit_class_def(c),
            Stmt::Assign(target, _, _) => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    self.declare(name, BindingKind::Var, stmt_span(stmt));
                }
                walk_stmt(self, stmt);
            }
            Stmt::For(target, _, body, else_, _) => {
                if let Expr::Ident(name, _) = target.as_ref() {
                    self.declare(name, BindingKind::For, expr_span(target));
                }
                let prev = self.enter_scope(ScopeKind::Block);
                for s in body {
                    self.visit_stmt(s);
                }
                if let Some(ref e) = else_ {
                    for s in e {
                        self.visit_stmt(s);
                    }
                }
                self.leave_scope(prev);
            }
            Stmt::With(_, body, _) => {
                let prev = self.enter_scope(ScopeKind::Block);
                for s in body {
                    self.visit_stmt(s);
                }
                self.leave_scope(prev);
            }
            Stmt::Try(body, handlers, _else_, _finalizer, _) => {
                let prev = self.enter_scope(ScopeKind::Block);
                for s in body {
                    self.visit_stmt(s);
                }
                self.leave_scope(prev);
                for h in handlers {
                    if let Some(ref name) = h.name {
                        self.declare(name, BindingKind::Except, h.span);
                    }
                }
            }
            Stmt::Import(aliases, _) => {
                for a in aliases {
                    let name = a.asname.as_ref().unwrap_or(&a.name);
                    self.declare(name, BindingKind::Import, a.span);
                }
            }
            Stmt::ImportFrom(_, aliases, _, _) => {
                for a in aliases {
                    let name = a.asname.as_ref().unwrap_or(&a.name);
                    self.declare(name, BindingKind::Import, a.span);
                }
            }
            Stmt::Global(names, _) => {
                for n in names {
                    self.declare(n, BindingKind::Global, stmt_span(stmt));
                }
            }
            Stmt::Nonlocal(names, _) => {
                for n in names {
                    self.declare(n, BindingKind::Nonlocal, stmt_span(stmt));
                }
            }
            _ => walk_stmt(self, stmt),
        }
    }
}
