#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Param,
    Function,
    Namespace,
    Class,
    Struct,
    Enum,
    Type,
    Template,
    Label,
}
