use std::convert::TryFrom;
use std::path::Path;

use napi::{Error, Result};
use napi_derive::napi;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[napi(string_enum)]
pub enum Language {
    #[napi(value = "javascript")]
    JavaScript,
    #[napi(value = "typescript")]
    TypeScript,
    #[napi(value = "python")]
    Python,
    #[napi(value = "go")]
    Go,
    #[napi(value = "rust")]
    Rust,
    #[napi(value = "c")]
    C,
    #[napi(value = "cpp")]
    Cpp,
    #[napi(value = "csharp")]
    CSharp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[napi(string_enum)]
pub enum Status {
    #[napi(value = "complete")]
    Complete,
    #[napi(value = "recovered")]
    Recovered,
    #[napi(value = "limit_exceeded")]
    LimitExceeded,
    #[napi(value = "failed")]
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[napi(string_enum)]
pub enum CommentKind {
    #[napi(value = "line")]
    Line,
    #[napi(value = "block")]
    Block,
    #[napi(value = "hashbang")]
    Hashbang,
}

#[napi(object)]
pub struct ParseOptions {
    pub language: Option<String>,
    pub filename: Option<String>,
    pub extension: Option<String>,
    #[napi(js_name = "includeAstSummary")]
    pub include_ast_summary: Option<bool>,
}

#[napi(object)]
pub struct Pos {
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}

#[napi(object)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

#[napi(object)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
}

#[napi(object)]
pub struct Comment {
    pub kind: CommentKind,
    pub text: String,
    pub span: Span,
}

#[napi(object, use_nullable = true)]
pub struct AstSummary {
    pub kind: String,
    #[napi(js_name = "topLevelNodes")]
    pub top_level_nodes: u32,
    #[napi(js_name = "expressionNodes")]
    pub expression_nodes: Option<u32>,
}

#[napi(object, use_nullable = true)]
pub struct ParsedParam {
    pub name: String,
    #[napi(js_name = "typeAnnotation")]
    pub type_annotation: Option<String>,
    #[napi(js_name = "defaultValue")]
    pub default_value: Option<String>,
}

#[napi(object, use_nullable = true)]
pub struct TypeKind {
    pub kind: String,
    pub name: Option<String>,
    pub items: Vec<TypeKind>,
}

#[napi(object, use_nullable = true)]
pub struct ParsedSymbol {
    pub kind: String,
    pub name: String,
    pub exported: bool,
    pub visibility: String,
    #[napi(js_name = "lineStart")]
    pub line_start: u32,
    #[napi(js_name = "lineEnd")]
    pub line_end: u32,
    pub signature: String,
    #[napi(js_name = "isTest")]
    pub is_test: bool,
    #[napi(js_name = "isAsync")]
    pub is_async: bool,
    #[napi(js_name = "returnType")]
    pub return_type: Option<String>,
    #[napi(js_name = "isConstructor")]
    pub is_constructor: bool,
    #[napi(js_name = "isDestructor")]
    pub is_destructor: bool,
    #[napi(js_name = "isVirtual")]
    pub is_virtual: bool,
    #[napi(js_name = "isOverride")]
    pub is_override: bool,
    #[napi(js_name = "isAbstract")]
    pub is_abstract: bool,
    #[napi(js_name = "isStatic")]
    pub is_static: bool,
    #[napi(js_name = "isConstexpr")]
    pub is_constexpr: bool,
    #[napi(js_name = "isFinal")]
    pub is_final: bool,
    #[napi(js_name = "storageClass")]
    pub storage_class: String,
    #[napi(js_name = "templateParams")]
    pub template_params: Vec<String>,
    pub attributes: Vec<String>,
    #[napi(js_name = "baseClasses")]
    pub base_classes: Vec<String>,
    #[napi(js_name = "typeKind")]
    pub type_kind: TypeKind,
    #[napi(js_name = "docString")]
    pub doc_string: Option<String>,
    pub params: Vec<ParsedParam>,
}

#[napi(object)]
pub struct ImportSpecifier {
    pub imported: String,
    pub local: String,
    pub kind: String,
}

#[napi(object, use_nullable = true)]
pub struct ParsedImport {
    pub kind: String,
    pub source: String,
    #[napi(js_name = "localName")]
    pub local_name: Option<String>,
    #[napi(js_name = "importedName")]
    pub imported_name: Option<String>,
    pub line: u32,
    #[napi(js_name = "isTypeOnly")]
    pub is_type_only: bool,
    #[napi(js_name = "isReexport")]
    pub is_reexport: bool,
    pub specifiers: Vec<ImportSpecifier>,
    #[napi(js_name = "isStarImport")]
    pub is_star_import: bool,
    #[napi(js_name = "modulePath")]
    pub module_path: Vec<String>,
}

#[napi(object, use_nullable = true)]
pub struct ParsedCall {
    pub kind: String,
    #[napi(js_name = "calleeText")]
    pub callee_text: String,
    pub object: Option<String>,
    pub line: u32,
    pub column: u32,
    #[napi(js_name = "isAwait")]
    pub is_await: bool,
    #[napi(js_name = "isOptional")]
    pub is_optional: bool,
    #[napi(js_name = "typeArgs")]
    pub type_args: Vec<TypeKind>,
}

#[napi(object)]
pub struct UsageSite {
    pub line: u32,
    pub column: u32,
    #[napi(js_name = "usageKind")]
    pub usage_kind: String,
}

#[napi(object, use_nullable = true)]
pub struct ParsedVariable {
    pub name: String,
    pub kind: String,
    #[napi(js_name = "typeAnnotation")]
    pub type_annotation: Option<String>,
    #[napi(js_name = "isMutable")]
    pub is_mutable: bool,
    #[napi(js_name = "lineDef")]
    pub line_def: u32,
    #[napi(js_name = "scopeSymbol")]
    pub scope_symbol: Option<String>,
    #[napi(js_name = "scopeStart")]
    pub scope_start: u32,
    #[napi(js_name = "scopeEnd")]
    pub scope_end: u32,
    #[napi(js_name = "usageSites")]
    pub usage_sites: Vec<UsageSite>,
    #[napi(js_name = "storageClass")]
    pub storage_class: String,
    #[napi(js_name = "typeKind")]
    pub type_kind: TypeKind,
    #[napi(js_name = "isConstructor")]
    pub is_constructor: bool,
    #[napi(js_name = "isDestructor")]
    pub is_destructor: bool,
    #[napi(js_name = "isImported")]
    pub is_imported: bool,
    #[napi(js_name = "isStatic")]
    pub is_static: bool,
    #[napi(js_name = "isConstexpr")]
    pub is_constexpr: bool,
    #[napi(js_name = "isThreadLocal")]
    pub is_thread_local: bool,
    #[napi(js_name = "isExtern")]
    pub is_extern: bool,
}

#[napi(object)]
pub struct Facts {
    pub symbols: Vec<ParsedSymbol>,
    pub imports: Vec<ParsedImport>,
    pub calls: Vec<ParsedCall>,
    pub variables: Vec<ParsedVariable>,
}

#[napi(object, use_nullable = true)]
pub struct ParseOutput {
    pub language: Language,
    pub status: Status,
    pub completeness: bool,
    pub truncated: bool,
    #[napi(js_name = "effectiveMode")]
    pub effective_mode: String,
    pub diagnostics: Vec<Diagnostic>,
    pub comments: Vec<Comment>,
    #[napi(js_name = "astSummary")]
    pub ast_summary: Option<AstSummary>,
    pub facts: Facts,
}

fn conversion_error(context: &str) -> Error {
    Error::from_reason(format!("conversion_overflow: {context} exceeds u32"))
}

fn checked_u32(value: usize, context: &str) -> Result<u32> {
    u32::try_from(value).map_err(|_| conversion_error(context))
}

fn diagnostic_code(code: ripex::diagnostics::DiagnosticCode) -> &'static str {
    use ripex::diagnostics::DiagnosticCode::*;
    match code {
        UnexpectedToken => "unexpected_token",
        UnterminatedString => "unterminated_string",
        UnterminatedComment => "unterminated_comment",
        UnterminatedTemplate => "unterminated_template",
        InvalidEscape => "invalid_escape",
        InvalidNumber => "invalid_number",
        InvalidRegex => "invalid_regex",
        InvalidLHS => "invalid_lhs",
        InvalidAssignment => "invalid_assignment",
        DuplicateBinding => "duplicate_binding",
        UndeclaredBinding => "undeclared_binding",
        StrictModeViolation => "strict_mode_violation",
        IllegalReturn => "illegal_return",
        IllegalBreak => "illegal_break",
        IllegalContinue => "illegal_continue",
        IllegalAwait => "illegal_await",
        IllegalYield => "illegal_yield",
        IllegalSuper => "illegal_super",
        IllegalNewTarget => "illegal_new_target",
        UnexpectedReserved => "unexpected_reserved",
        UnterminatedArrow => "unterminated_arrow",
        MissingParamName => "missing_param_name",
        DuplicateParam => "duplicate_param",
        TooManyArgs => "too_many_args",
        SyntaxError => "syntax_error",
        NotImplemented => "not_implemented",
        InvalidTypeAnnotation => "invalid_type_annotation",
        InvalidDecorator => "invalid_decorator",
        InvalidImport => "invalid_import",
        InvalidExport => "invalid_export",
        InvalidJSX => "invalid_jsx",
        UnterminatedJSX => "unterminated_jsx",
        InputTooLarge => "input_too_large",
        TokenLimitExceeded => "token_limit_exceeded",
        MaxRecursionExceeded => "max_recursion_exceeded",
    }
}

fn comment_kind(kind: ripex::facts::CommentKind) -> CommentKind {
    match kind {
        ripex::facts::CommentKind::Line => CommentKind::Line,
        ripex::facts::CommentKind::Block => CommentKind::Block,
        ripex::facts::CommentKind::Hashbang => CommentKind::Hashbang,
    }
}

fn visibility(value: ripex::facts::Visibility) -> &'static str {
    match value {
        ripex::facts::Visibility::Public => "public",
        ripex::facts::Visibility::Private => "private",
        ripex::facts::Visibility::Protected => "protected",
        ripex::facts::Visibility::Internal => "internal",
        ripex::facts::Visibility::Unknown => "unknown",
    }
}

fn symbol_kind(value: ripex::facts::SymbolKind) -> &'static str {
    match value {
        ripex::facts::SymbolKind::Function => "function",
        ripex::facts::SymbolKind::Class => "class",
        ripex::facts::SymbolKind::Struct => "struct",
        ripex::facts::SymbolKind::Trait => "trait",
        ripex::facts::SymbolKind::Interface => "interface",
        ripex::facts::SymbolKind::Method => "method",
        ripex::facts::SymbolKind::Module => "module",
        ripex::facts::SymbolKind::Constant => "constant",
        ripex::facts::SymbolKind::Enum => "enum",
        ripex::facts::SymbolKind::Type => "type",
        ripex::facts::SymbolKind::Variable => "variable",
        ripex::facts::SymbolKind::Constructor => "constructor",
        ripex::facts::SymbolKind::Destructor => "destructor",
        ripex::facts::SymbolKind::Getter => "getter",
        ripex::facts::SymbolKind::Setter => "setter",
        ripex::facts::SymbolKind::Property => "property",
        ripex::facts::SymbolKind::Event => "event",
        ripex::facts::SymbolKind::Delegate => "delegate",
        ripex::facts::SymbolKind::Namespace => "namespace",
    }
}

fn import_kind(value: ripex::facts::ImportKind) -> &'static str {
    match value {
        ripex::facts::ImportKind::NamedImport => "named_import",
        ripex::facts::ImportKind::NamespaceImport => "namespace_import",
        ripex::facts::ImportKind::DefaultImport => "default_import",
        ripex::facts::ImportKind::SideEffectImport => "side_effect_import",
        ripex::facts::ImportKind::RustUse => "rust_use",
        ripex::facts::ImportKind::GoImport => "go_import",
        ripex::facts::ImportKind::FromImport => "from_import",
        ripex::facts::ImportKind::PythonImport => "python_import",
        ripex::facts::ImportKind::ReExport => "re_export",
        ripex::facts::ImportKind::TypeImport => "type_import",
        ripex::facts::ImportKind::TypeReExport => "type_re_export",
        ripex::facts::ImportKind::DynamicImport => "dynamic_import",
    }
}

fn import_specifier_kind(value: ripex::facts::ImportSpecifierKind) -> &'static str {
    match value {
        ripex::facts::ImportSpecifierKind::Named => "named",
        ripex::facts::ImportSpecifierKind::Default => "default",
        ripex::facts::ImportSpecifierKind::Namespace => "namespace",
        ripex::facts::ImportSpecifierKind::SideEffect => "side_effect",
        ripex::facts::ImportSpecifierKind::Type => "type",
    }
}

fn call_kind(value: ripex::facts::CallKind) -> &'static str {
    match value {
        ripex::facts::CallKind::FunctionCall => "function_call",
        ripex::facts::CallKind::MethodCall => "method_call",
        ripex::facts::CallKind::ConstructorCall => "constructor_call",
        ripex::facts::CallKind::PathCall => "path_call",
        ripex::facts::CallKind::SelectorCall => "selector_call",
        ripex::facts::CallKind::DestructorCall => "destructor_call",
        ripex::facts::CallKind::SuperCall => "super_call",
        ripex::facts::CallKind::DelegateCall => "delegate_call",
    }
}

fn var_kind(value: ripex::facts::VarKind) -> &'static str {
    match value {
        ripex::facts::VarKind::Let => "let",
        ripex::facts::VarKind::Const => "const",
        ripex::facts::VarKind::Var => "var",
        ripex::facts::VarKind::Parameter => "parameter",
        ripex::facts::VarKind::ForLoop => "for_loop",
        ripex::facts::VarKind::Pattern => "pattern",
        ripex::facts::VarKind::Static => "static",
        ripex::facts::VarKind::Global => "global",
        ripex::facts::VarKind::ThreadLocal => "thread_local",
        ripex::facts::VarKind::Extern => "extern",
        ripex::facts::VarKind::Register => "register",
        ripex::facts::VarKind::Auto => "auto",
        ripex::facts::VarKind::Field => "field",
        ripex::facts::VarKind::Property => "property",
        ripex::facts::VarKind::EnumMember => "enum_member",
    }
}

fn usage_kind(value: ripex::facts::UsageKind) -> &'static str {
    match value {
        ripex::facts::UsageKind::Read => "read",
        ripex::facts::UsageKind::Write => "write",
        ripex::facts::UsageKind::Move => "move",
        ripex::facts::UsageKind::Borrow => "borrow",
        ripex::facts::UsageKind::BorrowMut => "borrow_mut",
        ripex::facts::UsageKind::PassedAsArg => "passed_as_arg",
    }
}

fn storage_class(value: ripex::facts::StorageClass) -> &'static str {
    match value {
        ripex::facts::StorageClass::Local => "local",
        ripex::facts::StorageClass::Global => "global",
        ripex::facts::StorageClass::Static => "static",
        ripex::facts::StorageClass::Extern => "extern",
        ripex::facts::StorageClass::Register => "register",
        ripex::facts::StorageClass::ThreadLocal => "thread_local",
        ripex::facts::StorageClass::Auto => "auto",
        ripex::facts::StorageClass::Parameter => "parameter",
        ripex::facts::StorageClass::Unknown => "unknown",
    }
}

fn type_kind(value: &ripex::facts::TypeKind) -> Result<TypeKind> {
    use ripex::facts::TypeKind as RootTypeKind;
    let (kind, name, items) = match value {
        RootTypeKind::Simple(name) => ("simple", Some(name.clone()), Vec::new()),
        RootTypeKind::Generic(name, values) => (
            "generic",
            Some(name.clone()),
            values.iter().map(type_kind).collect::<Result<Vec<_>>>()?,
        ),
        RootTypeKind::Array(inner) => ("array", None, vec![type_kind(inner)?]),
        RootTypeKind::Slice(inner) => ("slice", None, vec![type_kind(inner)?]),
        RootTypeKind::Pointer(inner) => ("pointer", None, vec![type_kind(inner)?]),
        RootTypeKind::Reference(inner) => ("reference", None, vec![type_kind(inner)?]),
        RootTypeKind::MutRef(inner) => ("mut_ref", None, vec![type_kind(inner)?]),
        RootTypeKind::FnPtr(args, result) => {
            let mut values = args.iter().map(type_kind).collect::<Result<Vec<_>>>()?;
            values.push(type_kind(result)?);
            ("fn_ptr", None, values)
        }
        RootTypeKind::Template(name, values) => (
            "template",
            Some(name.clone()),
            values.iter().map(type_kind).collect::<Result<Vec<_>>>()?,
        ),
        RootTypeKind::Tuple(values) => (
            "tuple",
            None,
            values.iter().map(type_kind).collect::<Result<Vec<_>>>()?,
        ),
        RootTypeKind::Union(values) => (
            "union",
            None,
            values.iter().map(type_kind).collect::<Result<Vec<_>>>()?,
        ),
        RootTypeKind::Optional(inner) => ("optional", None, vec![type_kind(inner)?]),
        RootTypeKind::Inferred => ("inferred", None, Vec::new()),
        RootTypeKind::Void => ("void", None, Vec::new()),
        RootTypeKind::Never => ("never", None, Vec::new()),
        RootTypeKind::Unknown => ("unknown", None, Vec::new()),
    };
    Ok(TypeKind {
        kind: kind.to_string(),
        name,
        items,
    })
}

fn span(value: ripex::span::Span) -> Result<Span> {
    Ok(Span {
        start: Pos {
            offset: checked_u32(value.start.offset, "span.start.offset")?,
            line: checked_u32(value.start.line, "span.start.line")?,
            column: checked_u32(value.start.column, "span.start.column")?,
        },
        end: Pos {
            offset: checked_u32(value.end.offset, "span.end.offset")?,
            line: checked_u32(value.end.line, "span.end.line")?,
            column: checked_u32(value.end.column, "span.end.column")?,
        },
    })
}

fn parsed_param(value: &ripex::facts::ParsedParam) -> ParsedParam {
    ParsedParam {
        name: value.name.clone(),
        type_annotation: value.type_annotation.clone(),
        default_value: value.default_value.clone(),
    }
}

fn parsed_symbol(value: ripex::facts::ParsedSymbol) -> Result<ParsedSymbol> {
    Ok(ParsedSymbol {
        kind: symbol_kind(value.kind).to_string(),
        name: value.name,
        exported: value.exported,
        visibility: visibility(value.visibility).to_string(),
        line_start: checked_u32(value.line_start, "symbol.line_start")?,
        line_end: checked_u32(value.line_end, "symbol.line_end")?,
        signature: value.signature,
        is_test: value.is_test,
        is_async: value.is_async,
        return_type: value.return_type,
        is_constructor: value.is_constructor,
        is_destructor: value.is_destructor,
        is_virtual: value.is_virtual,
        is_override: value.is_override,
        is_abstract: value.is_abstract,
        is_static: value.is_static,
        is_constexpr: value.is_constexpr,
        is_final: value.is_final,
        storage_class: storage_class(value.storage_class).to_string(),
        template_params: value.template_params,
        attributes: value.attributes,
        base_classes: value.base_classes,
        type_kind: type_kind(&value.type_kind)?,
        doc_string: value.doc_string,
        params: value.params.iter().map(parsed_param).collect(),
    })
}

fn import_specifier(value: ripex::facts::ImportSpecifier) -> ImportSpecifier {
    ImportSpecifier {
        imported: value.imported,
        local: value.local,
        kind: import_specifier_kind(value.kind).to_string(),
    }
}

fn parsed_import(value: ripex::facts::ParsedImport) -> Result<ParsedImport> {
    Ok(ParsedImport {
        kind: import_kind(value.kind).to_string(),
        source: value.source,
        local_name: value.local_name,
        imported_name: value.imported_name,
        line: checked_u32(value.line, "import.line")?,
        is_type_only: value.is_type_only,
        is_reexport: value.is_reexport,
        specifiers: value.specifiers.into_iter().map(import_specifier).collect(),
        is_star_import: value.is_star_import,
        module_path: value.module_path,
    })
}

fn parsed_call(value: ripex::facts::ParsedCall) -> Result<ParsedCall> {
    Ok(ParsedCall {
        kind: call_kind(value.kind).to_string(),
        callee_text: value.callee_text,
        object: value.object,
        line: checked_u32(value.line, "call.line")?,
        column: checked_u32(value.column, "call.column")?,
        is_await: value.is_await,
        is_optional: value.is_optional,
        type_args: value
            .type_args
            .iter()
            .map(type_kind)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn usage_site(value: ripex::facts::UsageSite) -> Result<UsageSite> {
    Ok(UsageSite {
        line: checked_u32(value.line, "usage.line")?,
        column: checked_u32(value.column, "usage.column")?,
        usage_kind: usage_kind(value.usage_kind).to_string(),
    })
}

fn parsed_variable(value: ripex::facts::ParsedVariable) -> Result<ParsedVariable> {
    Ok(ParsedVariable {
        name: value.name,
        kind: var_kind(value.kind).to_string(),
        type_annotation: value.type_annotation,
        is_mutable: value.is_mutable,
        line_def: checked_u32(value.line_def, "variable.line_def")?,
        scope_symbol: value.scope_symbol,
        scope_start: checked_u32(value.scope_start, "variable.scope_start")?,
        scope_end: checked_u32(value.scope_end, "variable.scope_end")?,
        usage_sites: value
            .usage_sites
            .into_iter()
            .map(usage_site)
            .collect::<Result<Vec<_>>>()?,
        storage_class: storage_class(value.storage_class).to_string(),
        type_kind: type_kind(&value.type_kind)?,
        is_constructor: value.is_constructor,
        is_destructor: value.is_destructor,
        is_imported: value.is_imported,
        is_static: value.is_static,
        is_constexpr: value.is_constexpr,
        is_thread_local: value.is_thread_local,
        is_extern: value.is_extern,
    })
}

fn map_facts(value: ripex::ExtractionResult) -> Result<Facts> {
    Ok(Facts {
        symbols: value
            .symbols
            .into_iter()
            .map(parsed_symbol)
            .collect::<Result<Vec<_>>>()?,
        imports: value
            .imports
            .into_iter()
            .map(parsed_import)
            .collect::<Result<Vec<_>>>()?,
        calls: value
            .calls
            .into_iter()
            .map(parsed_call)
            .collect::<Result<Vec<_>>>()?,
        variables: value
            .variables
            .into_iter()
            .map(parsed_variable)
            .collect::<Result<Vec<_>>>()?,
    })
}

fn ast_summary(result: &ripex::ParseResult) -> Result<AstSummary> {
    use ripex::Program;
    let (kind, top_level_nodes, expression_nodes) = match &result.ast {
        Program::Js(program, arena) => match program {
            ripex::js::ast::Program::Script(script) => {
                ("javascript_script", script.body.len(), Some(arena.len()))
            }
            ripex::js::ast::Program::Module(module) => {
                ("javascript_module", module.body.len(), Some(arena.len()))
            }
        },
        Program::Python(program) => ("python", program.stmts.len(), None),
        Program::Go(program) => ("go", program.decls.len(), None),
        Program::Rust(program) => ("rust", program.items.len(), None),
        Program::C(program) => ("c", program.decls.len(), None),
        Program::Cpp(program) => ("cpp", program.decls.len(), None),
        Program::CSharp(program) => ("csharp", program.decls.len(), None),
        #[allow(unreachable_patterns)]
        _ => ("unknown", 0, None),
    };
    Ok(AstSummary {
        kind: kind.to_string(),
        top_level_nodes: checked_u32(top_level_nodes, "ast.top_level_nodes")?,
        expression_nodes: expression_nodes
            .map(|count| checked_u32(count, "ast.expression_nodes"))
            .transpose()?,
    })
}

fn map_diagnostic(value: &ripex::diagnostics::ParseError) -> Result<Diagnostic> {
    Ok(Diagnostic {
        code: diagnostic_code(value.code).to_string(),
        message: value.message.clone(),
        span: span(value.span)?,
    })
}

fn map_comment(value: &ripex::facts::ParsedComment) -> Result<Comment> {
    Ok(Comment {
        kind: comment_kind(value.kind),
        text: value.text.clone(),
        span: span(value.span)?,
    })
}

fn parse_status(value: ripex::ParseStatus) -> Status {
    match value {
        ripex::ParseStatus::Complete => Status::Complete,
        ripex::ParseStatus::Recovered => Status::Recovered,
        ripex::ParseStatus::LimitExceeded => Status::LimitExceeded,
        ripex::ParseStatus::Failed => Status::Failed,
        _ => Status::Failed,
    }
}

fn parse_language(value: ripex::Language) -> Result<Language> {
    match value {
        ripex::Language::JavaScript => Ok(Language::JavaScript),
        ripex::Language::TypeScript => Ok(Language::TypeScript),
        ripex::Language::Python => Ok(Language::Python),
        ripex::Language::Go => Ok(Language::Go),
        ripex::Language::Rust => Ok(Language::Rust),
        ripex::Language::C => Ok(Language::C),
        ripex::Language::Cpp => Ok(Language::Cpp),
        ripex::Language::CSharp => Ok(Language::CSharp),
        unsupported => Err(invalid_options(format!(
            "parser_unavailable: no public language mapping for {}",
            unsupported.id()
        ))),
    }
}

fn extension_from_filename(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .map(str::to_owned)
}

fn invalid_options(message: impl Into<String>) -> Error {
    Error::from_reason(format!("invalid_options: {}", message.into()))
}

fn resolve_selection(options: &ParseOptions) -> Result<(ripex::Language, Option<String>)> {
    if let Some(language_id) = options.language.as_deref() {
        let language_id = language_id.trim();
        let language = ripex::Language::from_id(language_id)
            .ok_or_else(|| invalid_options(format!("unknown_language: {language_id}")))?;
        let extension = options
            .extension
            .as_deref()
            .filter(|extension| !extension.trim().is_empty())
            .map(str::to_owned)
            .or_else(|| {
                options
                    .filename
                    .as_deref()
                    .and_then(extension_from_filename)
            });
        return Ok((language, extension));
    }

    if let Some(filename) = options.filename.as_deref() {
        let language = ripex::detect_language(filename).ok_or_else(|| {
            invalid_options(format!(
                "unsupported_extension: could not detect a parser for filename {filename}"
            ))
        })?;
        return Ok((language, extension_from_filename(filename)));
    }

    if let Some(extension) = options.extension.as_deref() {
        let extension = extension.trim();
        let language = ripex::Language::from_extension(extension)
            .ok_or_else(|| invalid_options(format!("unsupported_extension: {extension}")))?;
        return Ok((language, Some(extension.to_owned())));
    }

    Err(invalid_options(
        "missing_language_selector: provide language, filename, or extension",
    ))
}

fn parse_internal(source: String, options: Option<ParseOptions>) -> Result<ParseOutput> {
    let options = options.unwrap_or(ParseOptions {
        language: None,
        filename: None,
        extension: None,
        include_ast_summary: None,
    });
    let include_ast_summary = options.include_ast_summary.unwrap_or(false);
    let (language, extension) = resolve_selection(&options)?;
    let parser = ripex::parser_for_language(language, extension.as_deref()).ok_or_else(|| {
        invalid_options(format!(
            "parser_unavailable: no parser registered for language {}",
            language.id()
        ))
    })?;
    let result = parser.parse(&source);
    let extracted = if result.status.is_complete() {
        parser.extract(&result)
    } else {
        parser.extract_best_effort(&result)
    }
    .map_err(|error| Error::from_reason(format!("extraction_failed: {error}")))?;
    let facts = map_facts(extracted)?;
    let diagnostics = result
        .errors
        .iter()
        .map(map_diagnostic)
        .collect::<Result<Vec<_>>>()?;
    let comments = result
        .comments
        .iter()
        .map(map_comment)
        .collect::<Result<Vec<_>>>()?;
    let ast_summary = if include_ast_summary {
        Some(ast_summary(&result)?)
    } else {
        None
    };
    Ok(ParseOutput {
        language: parse_language(result.language)?,
        status: parse_status(result.status),
        completeness: result.is_complete(),
        truncated: matches!(result.status, ripex::ParseStatus::LimitExceeded),
        effective_mode: result.parser_mode,
        diagnostics,
        comments,
        ast_summary,
        facts,
    })
}

#[napi(js_name = "parseSync")]
pub fn parse_sync(source: String, options: Option<ParseOptions>) -> Result<ParseOutput> {
    parse_internal(source, options)
}

#[napi]
pub async fn parse(source: String, options: Option<ParseOptions>) -> Result<ParseOutput> {
    napi::tokio::task::spawn_blocking(move || parse_internal(source, options))
        .await
        .map_err(|_| Error::from_reason("worker_failed: parser worker failed"))?
}

#[napi(js_name = "detectLanguage")]
pub fn detect_language(filename: String) -> Option<Language> {
    ripex::detect_language(filename).and_then(|language| parse_language(language).ok())
}

#[napi(js_name = "supportedLanguages")]
pub fn supported_languages() -> Vec<Language> {
    let mut language_ids: Vec<String> = ripex::registry().into_keys().map(str::to_owned).collect();
    language_ids.sort_unstable();
    language_ids
        .into_iter()
        .filter_map(|language_id| ripex::Language::from_id(&language_id))
        .filter_map(|language| parse_language(language).ok())
        .collect()
}
