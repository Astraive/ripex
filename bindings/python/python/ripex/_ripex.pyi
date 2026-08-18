from typing import Any, Dict, List, Optional

class Pos:
    offset: int
    line: int
    column: int
    def __init__(self, offset: int = 0, line: int = 1, column: int = 0) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class Span:
    start: Pos
    end: Pos
    def __init__(self, start: Pos, end: Pos) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class Diagnostic:
    code: str
    message: str
    span: Span
    def __init__(self, code: str, message: str, span: Span) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class Comment:
    kind: str
    text: str
    span: Span
    def __init__(self, kind: str, text: str, span: Span) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class AstSummary:
    kind: str
    top_level_nodes: int
    expression_nodes: Optional[int]
    def __init__(
        self,
        kind: str,
        top_level_nodes: int,
        expression_nodes: Optional[int] = None,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class TypeKind:
    kind: str
    name: Optional[str]
    items: List[TypeKind]
    def __init__(
        self,
        kind: str,
        name: Optional[str] = None,
        items: Optional[List[TypeKind]] = None,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ParsedParam:
    name: str
    type_annotation: Optional[str]
    default_value: Optional[str]
    def __init__(
        self,
        name: str,
        type_annotation: Optional[str] = None,
        default_value: Optional[str] = None,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ParsedSymbol:
    kind: str
    name: str
    exported: bool
    visibility: str
    line_start: int
    line_end: int
    signature: str
    is_test: bool
    is_async: bool
    return_type: Optional[str]
    is_constructor: bool
    is_destructor: bool
    is_virtual: bool
    is_override: bool
    is_abstract: bool
    is_static: bool
    is_constexpr: bool
    is_final: bool
    storage_class: str
    template_params: List[str]
    attributes: List[str]
    base_classes: List[str]
    type_kind: TypeKind
    doc_string: Optional[str]
    params: List[ParsedParam]
    def __init__(
        self,
        kind: str,
        name: str,
        exported: bool = False,
        visibility: str = "public",
        line_start: int = 1,
        line_end: int = 1,
        signature: str = "",
        is_test: bool = False,
        is_async: bool = False,
        return_type: Optional[str] = None,
        is_constructor: bool = False,
        is_destructor: bool = False,
        is_virtual: bool = False,
        is_override: bool = False,
        is_abstract: bool = False,
        is_static: bool = False,
        is_constexpr: bool = False,
        is_final: bool = False,
        storage_class: str = "local",
        template_params: Optional[List[str]] = None,
        attributes: Optional[List[str]] = None,
        base_classes: Optional[List[str]] = None,
        type_kind: Optional[TypeKind] = None,
        doc_string: Optional[str] = None,
        params: Optional[List[ParsedParam]] = None,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ImportSpecifier:
    imported: str
    local: str
    kind: str
    def __init__(self, imported: str, local: str, kind: str) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ParsedImport:
    kind: str
    source: str
    local_name: Optional[str]
    imported_name: Optional[str]
    line: int
    is_type_only: bool
    is_reexport: bool
    specifiers: List[ImportSpecifier]
    is_star_import: bool
    module_path: List[str]
    def __init__(
        self,
        kind: str,
        source: str,
        local_name: Optional[str] = None,
        imported_name: Optional[str] = None,
        line: int = 1,
        is_type_only: bool = False,
        is_reexport: bool = False,
        specifiers: Optional[List[ImportSpecifier]] = None,
        is_star_import: bool = False,
        module_path: Optional[List[str]] = None,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ParsedCall:
    kind: str
    callee_text: str
    object: Optional[str]
    line: int
    column: int
    is_await: bool
    is_optional: bool
    type_args: List[TypeKind]
    def __init__(
        self,
        kind: str,
        callee_text: str,
        object: Optional[str] = None,
        line: int = 1,
        column: int = 0,
        is_await: bool = False,
        is_optional: bool = False,
        type_args: Optional[List[TypeKind]] = None,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class UsageSite:
    line: int
    column: int
    usage_kind: str
    def __init__(self, line: int, column: int, usage_kind: str) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ParsedVariable:
    name: str
    kind: str
    type_annotation: Optional[str]
    is_mutable: bool
    line_def: int
    scope_symbol: Optional[str]
    scope_start: int
    scope_end: int
    usage_sites: List[UsageSite]
    storage_class: str
    type_kind: TypeKind
    is_constructor: bool
    is_destructor: bool
    is_imported: bool
    is_static: bool
    is_constexpr: bool
    is_thread_local: bool
    is_extern: bool
    def __init__(
        self,
        name: str,
        kind: str,
        type_annotation: Optional[str] = None,
        is_mutable: bool = False,
        line_def: int = 1,
        scope_symbol: Optional[str] = None,
        scope_start: int = 1,
        scope_end: int = 1,
        usage_sites: Optional[List[UsageSite]] = None,
        storage_class: str = "local",
        type_kind: Optional[TypeKind] = None,
        is_constructor: bool = False,
        is_destructor: bool = False,
        is_imported: bool = False,
        is_static: bool = False,
        is_constexpr: bool = False,
        is_thread_local: bool = False,
        is_extern: bool = False,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

class Facts:
    symbols: List[ParsedSymbol]
    imports: List[ParsedImport]
    calls: List[ParsedCall]
    variables: List[ParsedVariable]
    def __init__(
        self,
        symbols: Optional[List[ParsedSymbol]] = None,
        imports: Optional[List[ParsedImport]] = None,
        calls: Optional[List[ParsedCall]] = None,
        variables: Optional[List[ParsedVariable]] = None,
    ) -> None: ...
    def find_symbols_by_name(self, name: str) -> List[ParsedSymbol]: ...
    def find_calls_by_callee(self, callee: str) -> List[ParsedCall]: ...
    def find_imports_by_source(self, source: str) -> List[ParsedImport]: ...
    def to_dict(self) -> Dict[str, Any]: ...

class ParseOutput:
    language: str
    status: str
    completeness: bool
    truncated: bool
    effective_mode: str
    facts: Facts
    diagnostics: List[Diagnostic]
    comments: List[Comment]
    ast_summary: Optional[AstSummary]
    def __init__(
        self,
        language: str,
        status: str,
        completeness: bool,
        truncated: bool,
        effective_mode: str,
        facts: Facts,
        diagnostics: Optional[List[Diagnostic]] = None,
        comments: Optional[List[Comment]] = None,
        ast_summary: Optional[AstSummary] = None,
    ) -> None: ...
    def to_dict(self) -> Dict[str, Any]: ...

def parse_sync(
    source: str,
    language: Optional[str] = None,
    filename: Optional[str] = None,
    extension: Optional[str] = None,
    include_ast_summary: bool = False,
) -> ParseOutput: ...
def detect_language(filename: str) -> Optional[str]: ...
def supported_languages() -> List[str]: ...
