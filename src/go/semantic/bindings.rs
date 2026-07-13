#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Const,
    Param,
    Function,
    Import,
    Type,
    Field,
}
