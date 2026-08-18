use std::convert::TryFrom;
use std::path::Path;

use pyo3::exceptions::{PyOverflowError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

// ---------------------------------------------------------------------------
// Native Python Classes and Data Structures
// ---------------------------------------------------------------------------

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pos {
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}

#[pymethods]
impl Pos {
    #[new]
    #[pyo3(signature = (offset=0, line=1, column=0))]
    pub fn new(offset: u32, line: u32, column: u32) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Pos(offset={}, line={}, column={})",
            self.offset, self.line, self.column
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("offset", self.offset)?;
        dict.set_item("line", self.line)?;
        dict.set_item("column", self.column)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

#[pymethods]
impl Span {
    #[new]
    pub fn new(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    fn __repr__(&self) -> String {
        format!("Span(start={:?}, end={:?})", self.start, self.end)
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("start", self.start.to_dict(py)?)?;
        dict.set_item("end", self.end.to_dict(py)?)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub message: String,
    pub span: Span,
}

#[pymethods]
impl Diagnostic {
    #[new]
    pub fn new(code: String, message: String, span: Span) -> Self {
        Self {
            code,
            message,
            span,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Diagnostic(code='{}', message='{}', span={:?})",
            self.code, self.message, self.span
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("code", &self.code)?;
        dict.set_item("message", &self.message)?;
        dict.set_item("span", self.span.to_dict(py)?)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub kind: String,
    pub text: String,
    pub span: Span,
}

#[pymethods]
impl Comment {
    #[new]
    pub fn new(kind: String, text: String, span: Span) -> Self {
        Self { kind, text, span }
    }

    fn __repr__(&self) -> String {
        format!(
            "Comment(kind='{}', text='{}', span={:?})",
            self.kind, self.text, self.span
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("text", &self.text)?;
        dict.set_item("span", self.span.to_dict(py)?)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstSummary {
    pub kind: String,
    pub top_level_nodes: u32,
    pub expression_nodes: Option<u32>,
}

#[pymethods]
impl AstSummary {
    #[new]
    #[pyo3(signature = (kind, top_level_nodes, expression_nodes=None))]
    pub fn new(kind: String, top_level_nodes: u32, expression_nodes: Option<u32>) -> Self {
        Self {
            kind,
            top_level_nodes,
            expression_nodes,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "AstSummary(kind='{}', top_level_nodes={}, expression_nodes={:?})",
            self.kind, self.top_level_nodes, self.expression_nodes
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("top_level_nodes", self.top_level_nodes)?;
        dict.set_item("expression_nodes", self.expression_nodes)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeKind {
    pub kind: String,
    pub name: Option<String>,
    pub items: Vec<TypeKind>,
}

#[pymethods]
impl TypeKind {
    #[new]
    #[pyo3(signature = (kind, name=None, items=Vec::new()))]
    pub fn new(kind: String, name: Option<String>, items: Vec<TypeKind>) -> Self {
        Self { kind, name, items }
    }

    fn __repr__(&self) -> String {
        format!(
            "TypeKind(kind='{}', name={:?}, items_count={})",
            self.kind,
            self.name,
            self.items.len()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("name", &self.name)?;
        let items: PyResult<Vec<Bound<'py, PyDict>>> =
            self.items.iter().map(|t| t.to_dict(py)).collect();
        dict.set_item("items", items?)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedParam {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default_value: Option<String>,
}

#[pymethods]
impl ParsedParam {
    #[new]
    #[pyo3(signature = (name, type_annotation=None, default_value=None))]
    pub fn new(
        name: String,
        type_annotation: Option<String>,
        default_value: Option<String>,
    ) -> Self {
        Self {
            name,
            type_annotation,
            default_value,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ParsedParam(name='{}', type_annotation={:?}, default_value={:?})",
            self.name, self.type_annotation, self.default_value
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("name", &self.name)?;
        dict.set_item("type_annotation", &self.type_annotation)?;
        dict.set_item("default_value", &self.default_value)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSymbol {
    pub kind: String,
    pub name: String,
    pub exported: bool,
    pub visibility: String,
    pub line_start: u32,
    pub line_end: u32,
    pub signature: String,
    pub is_test: bool,
    pub is_async: bool,
    pub return_type: Option<String>,
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_abstract: bool,
    pub is_static: bool,
    pub is_constexpr: bool,
    pub is_final: bool,
    pub storage_class: String,
    pub template_params: Vec<String>,
    pub attributes: Vec<String>,
    pub base_classes: Vec<String>,
    pub type_kind: TypeKind,
    pub doc_string: Option<String>,
    pub params: Vec<ParsedParam>,
}

#[pymethods]
impl ParsedSymbol {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (
        kind, name, exported=false, visibility="public".to_string(), line_start=1, line_end=1,
        signature="".to_string(), is_test=false, is_async=false, return_type=None,
        is_constructor=false, is_destructor=false, is_virtual=false, is_override=false,
        is_abstract=false, is_static=false, is_constexpr=false, is_final=false,
        storage_class="local".to_string(), template_params=Vec::new(), attributes=Vec::new(),
        base_classes=Vec::new(), type_kind=TypeKind::new("unknown".to_string(), None, Vec::new()),
        doc_string=None, params=Vec::new()
    ))]
    pub fn new(
        kind: String,
        name: String,
        exported: bool,
        visibility: String,
        line_start: u32,
        line_end: u32,
        signature: String,
        is_test: bool,
        is_async: bool,
        return_type: Option<String>,
        is_constructor: bool,
        is_destructor: bool,
        is_virtual: bool,
        is_override: bool,
        is_abstract: bool,
        is_static: bool,
        is_constexpr: bool,
        is_final: bool,
        storage_class: String,
        template_params: Vec<String>,
        attributes: Vec<String>,
        base_classes: Vec<String>,
        type_kind: TypeKind,
        doc_string: Option<String>,
        params: Vec<ParsedParam>,
    ) -> Self {
        Self {
            kind,
            name,
            exported,
            visibility,
            line_start,
            line_end,
            signature,
            is_test,
            is_async,
            return_type,
            is_constructor,
            is_destructor,
            is_virtual,
            is_override,
            is_abstract,
            is_static,
            is_constexpr,
            is_final,
            storage_class,
            template_params,
            attributes,
            base_classes,
            type_kind,
            doc_string,
            params,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ParsedSymbol(name='{}', kind='{}', exported={}, visibility='{}', lines={}-{})",
            self.name, self.kind, self.exported, self.visibility, self.line_start, self.line_end
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("name", &self.name)?;
        dict.set_item("exported", self.exported)?;
        dict.set_item("visibility", &self.visibility)?;
        dict.set_item("line_start", self.line_start)?;
        dict.set_item("line_end", self.line_end)?;
        dict.set_item("signature", &self.signature)?;
        dict.set_item("is_test", self.is_test)?;
        dict.set_item("is_async", self.is_async)?;
        dict.set_item("return_type", &self.return_type)?;
        dict.set_item("is_constructor", self.is_constructor)?;
        dict.set_item("is_destructor", self.is_destructor)?;
        dict.set_item("is_virtual", self.is_virtual)?;
        dict.set_item("is_override", self.is_override)?;
        dict.set_item("is_abstract", self.is_abstract)?;
        dict.set_item("is_static", self.is_static)?;
        dict.set_item("is_constexpr", self.is_constexpr)?;
        dict.set_item("is_final", self.is_final)?;
        dict.set_item("storage_class", &self.storage_class)?;
        dict.set_item("template_params", &self.template_params)?;
        dict.set_item("attributes", &self.attributes)?;
        dict.set_item("base_classes", &self.base_classes)?;
        dict.set_item("type_kind", self.type_kind.to_dict(py)?)?;
        dict.set_item("doc_string", &self.doc_string)?;
        let params: PyResult<Vec<Bound<'py, PyDict>>> =
            self.params.iter().map(|p| p.to_dict(py)).collect();
        dict.set_item("params", params?)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSpecifier {
    pub imported: String,
    pub local: String,
    pub kind: String,
}

#[pymethods]
impl ImportSpecifier {
    #[new]
    pub fn new(imported: String, local: String, kind: String) -> Self {
        Self {
            imported,
            local,
            kind,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ImportSpecifier(imported='{}', local='{}', kind='{}')",
            self.imported, self.local, self.kind
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("imported", &self.imported)?;
        dict.set_item("local", &self.local)?;
        dict.set_item("kind", &self.kind)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedImport {
    pub kind: String,
    pub source: String,
    pub local_name: Option<String>,
    pub imported_name: Option<String>,
    pub line: u32,
    pub is_type_only: bool,
    pub is_reexport: bool,
    pub specifiers: Vec<ImportSpecifier>,
    pub is_star_import: bool,
    pub module_path: Vec<String>,
}

#[pymethods]
impl ParsedImport {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (
        kind, source, local_name=None, imported_name=None, line=1, is_type_only=false,
        is_reexport=false, specifiers=Vec::new(), is_star_import=false, module_path=Vec::new()
    ))]
    pub fn new(
        kind: String,
        source: String,
        local_name: Option<String>,
        imported_name: Option<String>,
        line: u32,
        is_type_only: bool,
        is_reexport: bool,
        specifiers: Vec<ImportSpecifier>,
        is_star_import: bool,
        module_path: Vec<String>,
    ) -> Self {
        Self {
            kind,
            source,
            local_name,
            imported_name,
            line,
            is_type_only,
            is_reexport,
            specifiers,
            is_star_import,
            module_path,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ParsedImport(source='{}', kind='{}', line={}, specifiers_count={})",
            self.source,
            self.kind,
            self.line,
            self.specifiers.len()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("source", &self.source)?;
        dict.set_item("local_name", &self.local_name)?;
        dict.set_item("imported_name", &self.imported_name)?;
        dict.set_item("line", self.line)?;
        dict.set_item("is_type_only", self.is_type_only)?;
        dict.set_item("is_reexport", self.is_reexport)?;
        let specs: PyResult<Vec<Bound<'py, PyDict>>> =
            self.specifiers.iter().map(|s| s.to_dict(py)).collect();
        dict.set_item("specifiers", specs?)?;
        dict.set_item("is_star_import", self.is_star_import)?;
        dict.set_item("module_path", &self.module_path)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCall {
    pub kind: String,
    pub callee_text: String,
    pub object: Option<String>,
    pub line: u32,
    pub column: u32,
    pub is_await: bool,
    pub is_optional: bool,
    pub type_args: Vec<TypeKind>,
}

#[pymethods]
impl ParsedCall {
    #[new]
    #[pyo3(signature = (kind, callee_text, object=None, line=1, column=0, is_await=false, is_optional=false, type_args=Vec::new()))]
    pub fn new(
        kind: String,
        callee_text: String,
        object: Option<String>,
        line: u32,
        column: u32,
        is_await: bool,
        is_optional: bool,
        type_args: Vec<TypeKind>,
    ) -> Self {
        Self {
            kind,
            callee_text,
            object,
            line,
            column,
            is_await,
            is_optional,
            type_args,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ParsedCall(callee_text='{}', kind='{}', line={}, column={})",
            self.callee_text, self.kind, self.line, self.column
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("callee_text", &self.callee_text)?;
        dict.set_item("object", &self.object)?;
        dict.set_item("line", self.line)?;
        dict.set_item("column", self.column)?;
        dict.set_item("is_await", self.is_await)?;
        dict.set_item("is_optional", self.is_optional)?;
        let args: PyResult<Vec<Bound<'py, PyDict>>> =
            self.type_args.iter().map(|t| t.to_dict(py)).collect();
        dict.set_item("type_args", args?)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageSite {
    pub line: u32,
    pub column: u32,
    pub usage_kind: String,
}

#[pymethods]
impl UsageSite {
    #[new]
    pub fn new(line: u32, column: u32, usage_kind: String) -> Self {
        Self {
            line,
            column,
            usage_kind,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "UsageSite(usage_kind='{}', line={}, column={})",
            self.usage_kind, self.line, self.column
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("line", self.line)?;
        dict.set_item("column", self.column)?;
        dict.set_item("usage_kind", &self.usage_kind)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedVariable {
    pub name: String,
    pub kind: String,
    pub type_annotation: Option<String>,
    pub is_mutable: bool,
    pub line_def: u32,
    pub scope_symbol: Option<String>,
    pub scope_start: u32,
    pub scope_end: u32,
    pub usage_sites: Vec<UsageSite>,
    pub storage_class: String,
    pub type_kind: TypeKind,
    pub is_constructor: bool,
    pub is_destructor: bool,
    pub is_imported: bool,
    pub is_static: bool,
    pub is_constexpr: bool,
    pub is_thread_local: bool,
    pub is_extern: bool,
}

#[pymethods]
impl ParsedVariable {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (
        name, kind, type_annotation=None, is_mutable=false, line_def=1, scope_symbol=None,
        scope_start=1, scope_end=1, usage_sites=Vec::new(), storage_class="local".to_string(),
        type_kind=TypeKind::new("unknown".to_string(), None, Vec::new()), is_constructor=false,
        is_destructor=false, is_imported=false, is_static=false, is_constexpr=false,
        is_thread_local=false, is_extern=false
    ))]
    pub fn new(
        name: String,
        kind: String,
        type_annotation: Option<String>,
        is_mutable: bool,
        line_def: u32,
        scope_symbol: Option<String>,
        scope_start: u32,
        scope_end: u32,
        usage_sites: Vec<UsageSite>,
        storage_class: String,
        type_kind: TypeKind,
        is_constructor: bool,
        is_destructor: bool,
        is_imported: bool,
        is_static: bool,
        is_constexpr: bool,
        is_thread_local: bool,
        is_extern: bool,
    ) -> Self {
        Self {
            name,
            kind,
            type_annotation,
            is_mutable,
            line_def,
            scope_symbol,
            scope_start,
            scope_end,
            usage_sites,
            storage_class,
            type_kind,
            is_constructor,
            is_destructor,
            is_imported,
            is_static,
            is_constexpr,
            is_thread_local,
            is_extern,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ParsedVariable(name='{}', kind='{}', is_mutable={}, line_def={})",
            self.name, self.kind, self.is_mutable, self.line_def
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("name", &self.name)?;
        dict.set_item("kind", &self.kind)?;
        dict.set_item("type_annotation", &self.type_annotation)?;
        dict.set_item("is_mutable", self.is_mutable)?;
        dict.set_item("line_def", self.line_def)?;
        dict.set_item("scope_symbol", &self.scope_symbol)?;
        dict.set_item("scope_start", self.scope_start)?;
        dict.set_item("scope_end", self.scope_end)?;
        let usages: PyResult<Vec<Bound<'py, PyDict>>> =
            self.usage_sites.iter().map(|u| u.to_dict(py)).collect();
        dict.set_item("usage_sites", usages?)?;
        dict.set_item("storage_class", &self.storage_class)?;
        dict.set_item("type_kind", self.type_kind.to_dict(py)?)?;
        dict.set_item("is_constructor", self.is_constructor)?;
        dict.set_item("is_destructor", self.is_destructor)?;
        dict.set_item("is_imported", self.is_imported)?;
        dict.set_item("is_static", self.is_static)?;
        dict.set_item("is_constexpr", self.is_constexpr)?;
        dict.set_item("is_thread_local", self.is_thread_local)?;
        dict.set_item("is_extern", self.is_extern)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Facts {
    pub symbols: Vec<ParsedSymbol>,
    pub imports: Vec<ParsedImport>,
    pub calls: Vec<ParsedCall>,
    pub variables: Vec<ParsedVariable>,
}

#[pymethods]
impl Facts {
    #[new]
    #[pyo3(signature = (symbols=Vec::new(), imports=Vec::new(), calls=Vec::new(), variables=Vec::new()))]
    pub fn new(
        symbols: Vec<ParsedSymbol>,
        imports: Vec<ParsedImport>,
        calls: Vec<ParsedCall>,
        variables: Vec<ParsedVariable>,
    ) -> Self {
        Self {
            symbols,
            imports,
            calls,
            variables,
        }
    }

    pub fn find_symbols_by_name(&self, name: &str) -> Vec<ParsedSymbol> {
        self.symbols
            .iter()
            .filter(|s| s.name.contains(name))
            .cloned()
            .collect()
    }

    pub fn find_calls_by_callee(&self, callee: &str) -> Vec<ParsedCall> {
        self.calls
            .iter()
            .filter(|c| c.callee_text.contains(callee))
            .cloned()
            .collect()
    }

    pub fn find_imports_by_source(&self, source: &str) -> Vec<ParsedImport> {
        self.imports
            .iter()
            .filter(|i| i.source.contains(source))
            .cloned()
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "Facts(symbols_count={}, imports_count={}, calls_count={}, variables_count={})",
            self.symbols.len(),
            self.imports.len(),
            self.calls.len(),
            self.variables.len()
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        let symbols: PyResult<Vec<Bound<'py, PyDict>>> =
            self.symbols.iter().map(|s| s.to_dict(py)).collect();
        dict.set_item("symbols", symbols?)?;
        let imports: PyResult<Vec<Bound<'py, PyDict>>> =
            self.imports.iter().map(|i| i.to_dict(py)).collect();
        dict.set_item("imports", imports?)?;
        let calls: PyResult<Vec<Bound<'py, PyDict>>> =
            self.calls.iter().map(|c| c.to_dict(py)).collect();
        dict.set_item("calls", calls?)?;
        let variables: PyResult<Vec<Bound<'py, PyDict>>> =
            self.variables.iter().map(|v| v.to_dict(py)).collect();
        dict.set_item("variables", variables?)?;
        Ok(dict)
    }
}

#[pyclass(module = "ripex._ripex", get_all, set_all, eq)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutput {
    pub language: String,
    pub status: String,
    pub completeness: bool,
    pub truncated: bool,
    pub effective_mode: String,
    pub facts: Facts,
    pub diagnostics: Vec<Diagnostic>,
    pub comments: Vec<Comment>,
    pub ast_summary: Option<AstSummary>,
}

#[pymethods]
impl ParseOutput {
    #[allow(clippy::too_many_arguments)]
    #[new]
    #[pyo3(signature = (language, status, completeness, truncated, effective_mode, facts, diagnostics=Vec::new(), comments=Vec::new(), ast_summary=None))]
    pub fn new(
        language: String,
        status: String,
        completeness: bool,
        truncated: bool,
        effective_mode: String,
        facts: Facts,
        diagnostics: Vec<Diagnostic>,
        comments: Vec<Comment>,
        ast_summary: Option<AstSummary>,
    ) -> Self {
        Self {
            language,
            status,
            completeness,
            truncated,
            effective_mode,
            facts,
            diagnostics,
            comments,
            ast_summary,
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ParseOutput(language='{}', status='{}', completeness={}, truncated={}, effective_mode='{}')",
            self.language, self.status, self.completeness, self.truncated, self.effective_mode
        )
    }

    fn __str__(&self) -> String {
        self.__repr__()
    }

    pub fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("language", &self.language)?;
        dict.set_item("status", &self.status)?;
        dict.set_item("completeness", self.completeness)?;
        dict.set_item("truncated", self.truncated)?;
        dict.set_item("effective_mode", &self.effective_mode)?;
        dict.set_item("facts", self.facts.to_dict(py)?)?;
        let diagnostics: PyResult<Vec<Bound<'py, PyDict>>> =
            self.diagnostics.iter().map(|d| d.to_dict(py)).collect();
        dict.set_item("diagnostics", diagnostics?)?;
        let comments: PyResult<Vec<Bound<'py, PyDict>>> =
            self.comments.iter().map(|c| c.to_dict(py)).collect();
        dict.set_item("comments", comments?)?;
        match &self.ast_summary {
            Some(a) => dict.set_item("ast_summary", a.to_dict(py)?)?,
            None => dict.set_item("ast_summary", py.None())?,
        }
        Ok(dict)
    }
}

// ---------------------------------------------------------------------------
// Conversion and Parsing Helpers
// ---------------------------------------------------------------------------

fn checked_u32(value: usize, context: &str) -> PyResult<u32> {
    u32::try_from(value)
        .map_err(|_| PyOverflowError::new_err(format!("conversion_overflow: {context} exceeds u32")))
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

fn comment_kind(kind: ripex::facts::CommentKind) -> &'static str {
    match kind {
        ripex::facts::CommentKind::Line => "line",
        ripex::facts::CommentKind::Block => "block",
        ripex::facts::CommentKind::Hashbang => "hashbang",
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

fn type_kind(value: &ripex::facts::TypeKind) -> PyResult<TypeKind> {
    use ripex::facts::TypeKind as RootTypeKind;
    let (kind, name, items) = match value {
        RootTypeKind::Simple(name) => ("simple", Some(name.clone()), Vec::new()),
        RootTypeKind::Generic(name, values) => (
            "generic",
            Some(name.clone()),
            values.iter().map(type_kind).collect::<PyResult<Vec<_>>>()?,
        ),
        RootTypeKind::Array(inner) => ("array", None, vec![type_kind(inner)?]),
        RootTypeKind::Slice(inner) => ("slice", None, vec![type_kind(inner)?]),
        RootTypeKind::Pointer(inner) => ("pointer", None, vec![type_kind(inner)?]),
        RootTypeKind::Reference(inner) => ("reference", None, vec![type_kind(inner)?]),
        RootTypeKind::MutRef(inner) => ("mut_ref", None, vec![type_kind(inner)?]),
        RootTypeKind::FnPtr(args, result) => {
            let mut values = args.iter().map(type_kind).collect::<PyResult<Vec<_>>>()?;
            values.push(type_kind(result)?);
            ("fn_ptr", None, values)
        }
        RootTypeKind::Template(name, values) => (
            "template",
            Some(name.clone()),
            values.iter().map(type_kind).collect::<PyResult<Vec<_>>>()?,
        ),
        RootTypeKind::Tuple(values) => (
            "tuple",
            None,
            values.iter().map(type_kind).collect::<PyResult<Vec<_>>>()?,
        ),
        RootTypeKind::Union(values) => (
            "union",
            None,
            values.iter().map(type_kind).collect::<PyResult<Vec<_>>>()?,
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

fn span(value: ripex::span::Span) -> PyResult<Span> {
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

fn parsed_symbol(value: ripex::facts::ParsedSymbol) -> PyResult<ParsedSymbol> {
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

fn parsed_import(value: ripex::facts::ParsedImport) -> PyResult<ParsedImport> {
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

fn parsed_call(value: ripex::facts::ParsedCall) -> PyResult<ParsedCall> {
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
            .collect::<PyResult<Vec<_>>>()?,
    })
}

fn usage_site(value: ripex::facts::UsageSite) -> PyResult<UsageSite> {
    Ok(UsageSite {
        line: checked_u32(value.line, "usage.line")?,
        column: checked_u32(value.column, "usage.column")?,
        usage_kind: usage_kind(value.usage_kind).to_string(),
    })
}

fn parsed_variable(value: ripex::facts::ParsedVariable) -> PyResult<ParsedVariable> {
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
            .collect::<PyResult<Vec<_>>>()?,
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

fn map_facts(value: ripex::ExtractionResult) -> PyResult<Facts> {
    Ok(Facts {
        symbols: value
            .symbols
            .into_iter()
            .map(parsed_symbol)
            .collect::<PyResult<Vec<_>>>()?,
        imports: value
            .imports
            .into_iter()
            .map(parsed_import)
            .collect::<PyResult<Vec<_>>>()?,
        calls: value
            .calls
            .into_iter()
            .map(parsed_call)
            .collect::<PyResult<Vec<_>>>()?,
        variables: value
            .variables
            .into_iter()
            .map(parsed_variable)
            .collect::<PyResult<Vec<_>>>()?,
    })
}

fn ast_summary(result: &ripex::ParseResult) -> PyResult<AstSummary> {
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
        #[cfg(feature = "lang-python")]
        Program::Python(program) => ("python_module", program.statements.len(), None),
        #[cfg(feature = "lang-go")]
        Program::Go(program) => ("go_file", program.declarations.len(), None),
        #[cfg(feature = "lang-rust")]
        Program::Rust(program) => ("rust_file", program.items.len(), None),
        #[cfg(feature = "lang-c")]
        Program::C(program) => ("c_translation_unit", program.declarations.len(), None),
        #[cfg(feature = "lang-cpp")]
        Program::Cpp(program) => ("cpp_translation_unit", program.declarations.len(), None),
        #[cfg(feature = "lang-csharp")]
        Program::CSharp(program) => ("csharp_compilation_unit", program.declarations.len(), None),
        #[allow(unreachable_patterns)]
        _ => ("unknown", 0, None),
    };

    Ok(AstSummary {
        kind: kind.to_string(),
        top_level_nodes: checked_u32(top_level_nodes, "top_level_nodes")?,
        expression_nodes: expression_nodes
            .map(|value| checked_u32(value, "expression_nodes"))
            .transpose()?,
    })
}

fn map_diagnostic(value: &ripex::diagnostics::ParseError) -> PyResult<Diagnostic> {
    Ok(Diagnostic {
        code: diagnostic_code(value.code).to_string(),
        message: value.message.clone(),
        span: span(value.span)?,
    })
}

fn map_comment(value: &ripex::facts::ParsedComment) -> PyResult<Comment> {
    Ok(Comment {
        kind: comment_kind(value.kind).to_string(),
        text: value.text.clone(),
        span: span(value.span)?,
    })
}

fn parse_status_str(value: ripex::ParseStatus) -> &'static str {
    match value {
        ripex::ParseStatus::Complete => "complete",
        ripex::ParseStatus::Recovered => "recovered",
        ripex::ParseStatus::LimitExceeded => "limit_exceeded",
        ripex::ParseStatus::Failed => "failed",
        _ => "unknown",
    }
}

fn extension_from_filename(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
}

fn resolve_selection(
    language: Option<&str>,
    filename: Option<&str>,
    extension: Option<&str>,
) -> PyResult<(ripex::Language, Option<String>)> {
    let resolved_extension = extension
        .map(|ext| ext.trim_start_matches('.').to_ascii_lowercase())
        .or_else(|| filename.and_then(extension_from_filename));

    let resolved_language = match language {
        Some(name) => ripex::Language::from_id(name).ok_or_else(|| {
            PyValueError::new_err(format!("unsupported or unknown language: '{name}'"))
        })?,
        None => {
            let detected = filename
                .and_then(ripex::detect_language)
                .or_else(|| {
                    resolved_extension
                        .as_deref()
                        .and_then(ripex::Language::from_extension)
                });
            detected.ok_or_else(|| {
                PyValueError::new_err(
                    "unable to infer language: supply an explicit 'language', 'filename', or 'extension'",
                )
            })?
        }
    };

    Ok((resolved_language, resolved_extension))
}

// ---------------------------------------------------------------------------
// Module Functions
// ---------------------------------------------------------------------------

#[pyfunction]
#[pyo3(signature = (source, language=None, filename=None, extension=None, include_ast_summary=false))]
pub fn parse_sync(
    source: String,
    language: Option<&str>,
    filename: Option<&str>,
    extension: Option<&str>,
    include_ast_summary: bool,
) -> PyResult<ParseOutput> {
    let (target_language, resolved_extension) = resolve_selection(language, filename, extension)?;

    let parser = ripex::parser_for_language(target_language, resolved_extension.as_deref())
        .ok_or_else(|| {
            PyValueError::new_err(format!(
                "language '{}' is not enabled or available in this build",
                target_language.id()
            ))
        })?;

    let result = parser.parse(&source);
    let raw_facts = parser
        .extract_best_effort(&result)
        .unwrap_or_else(|_| ripex::ExtractionResult::new());
    let facts = map_facts(raw_facts)?;

    let diagnostics = result
        .errors
        .iter()
        .map(map_diagnostic)
        .collect::<PyResult<Vec<Diagnostic>>>()?;
    let comments = result
        .comments
        .iter()
        .map(map_comment)
        .collect::<PyResult<Vec<Comment>>>()?;

    let summary = if include_ast_summary {
        Some(ast_summary(&result)?)
    } else {
        None
    };

    let status = parse_status_str(result.status).to_string();
    let completeness = result.status.is_complete();
    let truncated = matches!(result.status, ripex::ParseStatus::LimitExceeded);

    Ok(ParseOutput {
        language: target_language.id().to_string(),
        status,
        completeness,
        truncated,
        effective_mode: result.effective_parser_mode().to_string(),
        facts,
        diagnostics,
        comments,
        ast_summary: summary,
    })
}

#[pyfunction]
pub fn detect_language(filename: String) -> Option<String> {
    ripex::detect_language(filename).map(|lang| lang.id().to_string())
}

#[pyfunction]
pub fn supported_languages() -> Vec<String> {
    vec![
        "c".to_string(),
        "cpp".to_string(),
        "csharp".to_string(),
        "go".to_string(),
        "javascript".to_string(),
        "python".to_string(),
        "rust".to_string(),
        "typescript".to_string(),
    ]
}

// ---------------------------------------------------------------------------
// PyO3 Module Definition
// ---------------------------------------------------------------------------

#[pymodule]
fn _ripex(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Pos>()?;
    m.add_class::<Span>()?;
    m.add_class::<Diagnostic>()?;
    m.add_class::<Comment>()?;
    m.add_class::<AstSummary>()?;
    m.add_class::<TypeKind>()?;
    m.add_class::<ParsedParam>()?;
    m.add_class::<ParsedSymbol>()?;
    m.add_class::<ImportSpecifier>()?;
    m.add_class::<ParsedImport>()?;
    m.add_class::<ParsedCall>()?;
    m.add_class::<UsageSite>()?;
    m.add_class::<ParsedVariable>()?;
    m.add_class::<Facts>()?;
    m.add_class::<ParseOutput>()?;

    m.add_function(wrap_pyfunction!(parse_sync, m)?)?;
    m.add_function(wrap_pyfunction!(detect_language, m)?)?;
    m.add_function(wrap_pyfunction!(supported_languages, m)?)?;

    Ok(())
}
