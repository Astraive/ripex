#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Let,
    Const,
    Static,
    Param,
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Type,
    Macro,
    Self_,
}
