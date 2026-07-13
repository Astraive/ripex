#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Let,
    Const,
    Function,
    Class,
    Import,
    Param,
}

impl BindingKind {
    pub fn is_block_scoped(&self) -> bool {
        matches!(
            self,
            BindingKind::Let | BindingKind::Const | BindingKind::Class | BindingKind::Import
        )
    }

    pub fn is_function_scoped(&self) -> bool {
        matches!(self, BindingKind::Var)
    }
}
