#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CommentKind {
    Line,
    Block,
    Hashbang,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedComment {
    pub kind: CommentKind,
    pub text: String,
    pub span: crate::span::Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SymbolKind {
    Function,
    Class,
    Struct,
    Trait,
    Interface,
    Method,
    Module,
    Constant,
    Enum,
    Type,
    Variable,
    Constructor,
    Destructor,
    Getter,
    Setter,
    Property,
    Event,
    Delegate,
    Namespace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ImportKind {
    NamedImport,
    NamespaceImport,
    DefaultImport,
    SideEffectImport,
    RustUse,
    GoImport,
    FromImport,
    PythonImport,
    ReExport,
    TypeImport,
    TypeReExport,
    DynamicImport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CallKind {
    FunctionCall,
    MethodCall,
    ConstructorCall,
    PathCall,
    SelectorCall,
    DestructorCall,
    SuperCall,
    DelegateCall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum VarKind {
    Let,
    Const,
    Var,
    Parameter,
    ForLoop,
    Pattern,
    Static,
    Global,
    ThreadLocal,
    Extern,
    Register,
    Auto,
    Field,
    Property,
    EnumMember,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum UsageKind {
    Read,
    Write,
    Move,
    Borrow,
    BorrowMut,
    PassedAsArg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum StorageClass {
    Local,
    Global,
    Static,
    Extern,
    Register,
    ThreadLocal,
    Auto,
    Parameter,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum TypeKind {
    Simple(String),
    Generic(String, Vec<TypeKind>),
    Array(Box<TypeKind>),
    Slice(Box<TypeKind>),
    Pointer(Box<TypeKind>),
    Reference(Box<TypeKind>),
    MutRef(Box<TypeKind>),
    FnPtr(Vec<TypeKind>, Box<TypeKind>),
    Template(String, Vec<TypeKind>),
    Tuple(Vec<TypeKind>),
    Union(Vec<TypeKind>),
    Optional(Box<TypeKind>),
    Inferred,
    Void,
    Never,
    Unknown,
}

impl TypeKind {
    pub fn simple(name: impl Into<String>) -> Self {
        TypeKind::Simple(name.into())
    }
    pub fn is_inferred(&self) -> bool {
        matches!(self, TypeKind::Inferred)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportSpecifier {
    pub imported: String,
    pub local: String,
    pub kind: ImportSpecifierKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ImportSpecifierKind {
    Named,
    Default,
    Namespace,
    SideEffect,
    Type,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedParam {
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub type_annotation: Option<String>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedSymbol {
    pub kind: SymbolKind,
    pub name: String,
    pub exported: bool,
    pub visibility: Visibility,
    pub line_start: usize,
    pub line_end: usize,
    pub signature: String,
    pub is_test: bool,
    pub is_async: bool,
    pub return_type: Option<String>,
    // NEW: constructor/destructor
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_abstract: bool,
    pub is_static: bool,
    pub is_constexpr: bool,
    pub is_final: bool,
    // NEW: storage & templates
    pub storage_class: StorageClass,
    pub template_params: Vec<String>,
    pub attributes: Vec<String>,
    pub base_classes: Vec<String>,
    // NEW: type info
    pub type_kind: TypeKind,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Option::is_none"))]
    pub doc_string: Option<String>,
    #[cfg_attr(feature = "serde", serde(default, skip_serializing_if = "Vec::is_empty"))]
    pub params: Vec<ParsedParam>,
}

impl ParsedSymbol {
    pub fn builder(kind: SymbolKind, name: impl Into<String>) -> SymbolBuilder {
        SymbolBuilder {
            inner: ParsedSymbol {
                kind,
                name: name.into(),
                exported: false,
                visibility: Visibility::Unknown,
                line_start: 0,
                line_end: 0,
                signature: String::new(),
                is_test: false,
                is_async: false,
                return_type: None,
                is_constructor: false,
                is_destructor: false,
                is_virtual: false,
                is_override: false,
                is_abstract: false,
                is_static: false,
                is_constexpr: false,
                is_final: false,
                storage_class: StorageClass::Unknown,
                template_params: Vec::new(),
                attributes: Vec::new(),
                base_classes: Vec::new(),
                type_kind: TypeKind::Unknown,
                doc_string: None,
                params: Vec::new(),
            },
        }
    }
}

pub struct SymbolBuilder {
    inner: ParsedSymbol,
}

impl SymbolBuilder {
    pub fn exported(mut self, v: bool) -> Self {
        self.inner.exported = v;
        self
    }
    pub fn visibility(mut self, v: Visibility) -> Self {
        self.inner.visibility = v;
        self
    }
    pub fn lines(mut self, start: usize, end: usize) -> Self {
        self.inner.line_start = start;
        self.inner.line_end = end;
        self
    }
    pub fn signature(mut self, s: impl Into<String>) -> Self {
        self.inner.signature = s.into();
        self
    }
    pub fn is_test(mut self, v: bool) -> Self {
        self.inner.is_test = v;
        self
    }
    pub fn is_async(mut self, v: bool) -> Self {
        self.inner.is_async = v;
        self
    }
    pub fn return_type(mut self, t: Option<String>) -> Self {
        self.inner.return_type = t;
        self
    }
    pub fn constructor(mut self, v: bool) -> Self {
        self.inner.is_constructor = v;
        self
    }
    pub fn destructor(mut self, v: bool) -> Self {
        self.inner.is_destructor = v;
        self
    }
    pub fn virtual_(mut self, v: bool) -> Self {
        self.inner.is_virtual = v;
        self
    }
    pub fn override_(mut self, v: bool) -> Self {
        self.inner.is_override = v;
        self
    }
    pub fn abstract_(mut self, v: bool) -> Self {
        self.inner.is_abstract = v;
        self
    }
    pub fn static_(mut self, v: bool) -> Self {
        self.inner.is_static = v;
        self
    }
    pub fn storage(mut self, s: StorageClass) -> Self {
        self.inner.storage_class = s;
        self
    }
    pub fn type_kind(mut self, t: TypeKind) -> Self {
        self.inner.type_kind = t;
        self
    }
    pub fn doc_string(mut self, d: Option<String>) -> Self {
        self.inner.doc_string = d;
        self
    }
    pub fn params(mut self, p: Vec<ParsedParam>) -> Self {
        self.inner.params = p;
        self
    }
    pub fn attributes(mut self, attrs: Vec<String>) -> Self {
        self.inner.attributes = attrs;
        self
    }
    pub fn build(self) -> ParsedSymbol {
        self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedImport {
    pub kind: ImportKind,
    pub source: String,
    pub local_name: Option<String>,
    pub imported_name: Option<String>,
    pub line: usize,
    // NEW: strong tracking
    pub is_type_only: bool,
    pub is_reexport: bool,
    pub specifiers: Vec<ImportSpecifier>,
    pub is_star_import: bool,
    pub module_path: Vec<String>,
}

impl ParsedImport {
    pub fn builder(kind: ImportKind, source: impl Into<String>) -> ImportBuilder {
        ImportBuilder {
            inner: ParsedImport {
                kind,
                source: source.into(),
                local_name: None,
                imported_name: None,
                line: 0,
                is_type_only: false,
                is_reexport: false,
                specifiers: Vec::new(),
                is_star_import: false,
                module_path: Vec::new(),
            },
        }
    }
}

pub struct ImportBuilder {
    inner: ParsedImport,
}

impl ImportBuilder {
    pub fn local(mut self, name: impl Into<String>) -> Self {
        self.inner.local_name = Some(name.into());
        self
    }
    pub fn imported(mut self, name: impl Into<String>) -> Self {
        self.inner.imported_name = Some(name.into());
        self
    }
    pub fn line(mut self, l: usize) -> Self {
        self.inner.line = l;
        self
    }
    pub fn type_only(mut self, v: bool) -> Self {
        self.inner.is_type_only = v;
        self
    }
    pub fn reexport(mut self, v: bool) -> Self {
        self.inner.is_reexport = v;
        self
    }
    pub fn star(mut self, v: bool) -> Self {
        self.inner.is_star_import = v;
        self
    }
    pub fn specifiers(mut self, s: Vec<ImportSpecifier>) -> Self {
        self.inner.specifiers = s;
        self
    }
    pub fn build(self) -> ParsedImport {
        self.inner
    }
}

/// Error returned when a call fact has no semantic callee name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyCalleeError;

impl std::fmt::Display for EmptyCalleeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("call callee name must not be empty")
    }
}

impl std::error::Error for EmptyCalleeError {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedCall {
    pub kind: CallKind,
    pub callee_text: String,
    pub object: Option<String>,
    pub line: usize,
    pub column: usize,
    pub is_await: bool,
    pub is_optional: bool,
    pub type_args: Vec<TypeKind>,
}

impl ParsedCall {
    pub fn builder(kind: CallKind, callee: impl Into<String>) -> CallBuilder {
        CallBuilder {
            inner: ParsedCall {
                kind,
                callee_text: callee.into(),
                object: None,
                line: 0,
                column: 0,
                is_await: false,
                is_optional: false,
                type_args: Vec::new(),
            },
        }
    }
}

pub struct CallBuilder {
    inner: ParsedCall,
}

impl CallBuilder {
    pub fn object(mut self, o: impl Into<String>) -> Self {
        self.inner.object = Some(o.into());
        self
    }
    pub fn pos(mut self, line: usize, col: usize) -> Self {
        self.inner.line = line;
        self.inner.column = col;
        self
    }
    pub fn await_(mut self, v: bool) -> Self {
        self.inner.is_await = v;
        self
    }
    pub fn optional(mut self, v: bool) -> Self {
        self.inner.is_optional = v;
        self
    }
    pub fn type_args(mut self, args: Vec<TypeKind>) -> Self {
        self.inner.type_args = args;
        self
    }
    /// Validate and build the call fact.
    pub fn try_build(self) -> Result<ParsedCall, EmptyCalleeError> {
        if self.inner.callee_text.trim().is_empty() {
            Err(EmptyCalleeError)
        } else {
            Ok(self.inner)
        }
    }

    /// Build a call fact, panicking instead of exposing an empty semantic name.
    ///
    /// Extractors should normally use [`Self::try_build`]. This infallible
    /// compatibility method remains for existing internal builders and makes
    /// malformed call construction fail immediately rather than fabricating a
    /// name.
    pub fn build(self) -> ParsedCall {
        self.try_build()
            .expect("call callee name must not be empty")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UsageSite {
    pub line: usize,
    pub column: usize,
    pub usage_kind: UsageKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedVariable {
    pub name: String,
    pub kind: VarKind,
    pub type_annotation: Option<String>,
    pub is_mutable: bool,
    pub line_def: usize,
    pub scope_symbol: Option<String>,
    pub scope_start: usize,
    pub scope_end: usize,
    pub usage_sites: Vec<UsageSite>,
    // NEW
    pub storage_class: StorageClass,
    pub type_kind: TypeKind,
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_imported: bool,
    pub is_static: bool,
    pub is_constexpr: bool,
    pub is_thread_local: bool,
    pub is_extern: bool,
}

impl ParsedVariable {
    pub fn builder(name: impl Into<String>, kind: VarKind) -> VarBuilder {
        VarBuilder {
            inner: ParsedVariable {
                name: name.into(),
                kind,
                type_annotation: None,
                is_mutable: false,
                line_def: 0,
                scope_symbol: None,
                scope_start: 0,
                scope_end: 0,
                usage_sites: Vec::new(),
                storage_class: StorageClass::Unknown,
                type_kind: TypeKind::Unknown,
                is_constructor: false,
                is_destructor: false,
                is_imported: false,
                is_static: false,
                is_constexpr: false,
                is_thread_local: false,
                is_extern: false,
            },
        }
    }
}

pub struct VarBuilder {
    inner: ParsedVariable,
}

impl VarBuilder {
    pub fn type_ann(mut self, t: Option<String>) -> Self {
        self.inner.type_annotation = t;
        self
    }
    pub fn mutable(mut self, v: bool) -> Self {
        self.inner.is_mutable = v;
        self
    }
    pub fn line(mut self, l: usize) -> Self {
        self.inner.line_def = l;
        self
    }
    pub fn scope(mut self, symbol: Option<String>, start: usize, end: usize) -> Self {
        self.inner.scope_symbol = symbol;
        self.inner.scope_start = start;
        self.inner.scope_end = end;
        self
    }
    pub fn storage(mut self, s: StorageClass) -> Self {
        self.inner.storage_class = s;
        self
    }
    pub fn type_kind(mut self, t: TypeKind) -> Self {
        self.inner.type_kind = t;
        self
    }
    pub fn imported(mut self, v: bool) -> Self {
        self.inner.is_imported = v;
        self
    }
    pub fn build(self) -> ParsedVariable {
        self.inner
    }
}
