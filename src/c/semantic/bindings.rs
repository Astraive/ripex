#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Param,
    Function,
    Struct,
    Enum,
    Union,
    Typedef,
    Label,
}
