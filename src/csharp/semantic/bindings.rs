#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Var,
    Param,
    Method,
    Class,
    Struct,
    Interface,
    Enum,
    Property,
    Event,
    Delegate,
    Namespace,
    Type,
}
