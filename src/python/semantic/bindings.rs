#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Global,
    Nonlocal,
    Param,
    Function,
    Class,
    Import,
    For,
    Except,
    Comprehension,
}
