use super::bindings::BindingKind;
use crate::span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: BindingKind,
    pub span: Span,
    pub scope_id: usize,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: HashMap<String, Vec<Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }
    pub fn insert(&mut self, name: String, kind: BindingKind, span: Span, scope_id: usize) {
        self.symbols.entry(name.clone()).or_default().push(Symbol {
            name,
            kind,
            span,
            scope_id,
        });
    }
    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name).and_then(|s| s.last())
    }
    pub fn lookup_all(&self, name: &str) -> Vec<&Symbol> {
        self.symbols
            .get(name)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
    pub fn symbols(&self) -> impl Iterator<Item = &Symbol> {
        self.symbols.values().flat_map(|v| v.iter())
    }
    pub fn clear(&mut self) {
        self.symbols.clear();
    }
    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}
