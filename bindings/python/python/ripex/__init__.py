"""
Native multi-language structural parsing and fact extraction for Python.
"""

from __future__ import annotations

import asyncio
from enum import Enum
from typing import Any, Dict, List, Optional, Union

from ._ripex import (
    AstSummary,
    Comment,
    Diagnostic,
    Facts,
    ImportSpecifier,
    ParseOutput,
    ParsedCall,
    ParsedImport,
    ParsedParam,
    ParsedSymbol,
    ParsedVariable,
    Pos,
    Span,
    TypeKind,
    UsageSite,
    detect_language as _detect_language,
    parse_sync as _parse_sync,
    supported_languages as _supported_languages,
)

__version__ = "0.3.1"


class Language(str, Enum):
    JAVASCRIPT = "javascript"
    TYPESCRIPT = "typescript"
    PYTHON = "python"
    GO = "go"
    RUST = "rust"
    C = "c"
    CPP = "cpp"
    CSHARP = "csharp"


class ParseStatus(str, Enum):
    COMPLETE = "complete"
    RECOVERED = "recovered"
    LIMIT_EXCEEDED = "limit_exceeded"
    FAILED = "failed"


class CommentKind(str, Enum):
    LINE = "line"
    BLOCK = "block"
    HASHBANG = "hashbang"


def parse_sync(
    source: str,
    *,
    language: Optional[Union[str, Language]] = None,
    filename: Optional[str] = None,
    extension: Optional[str] = None,
    include_ast_summary: bool = False,
) -> ParseOutput:
    """
    Parse source code synchronously and extract structural facts.

    :param source: The source code string to parse.
    :param language: Explicit language identifier (e.g. 'python', 'javascript', 'rust').
    :param filename: Optional filename used for automatic language detection.
    :param extension: Optional file extension used for language or mode selection.
    :param include_ast_summary: Whether to compute and include AST node counts.
    :return: ParseOutput containing status, completeness, facts, diagnostics, comments.
    """
    lang_str = (
        str(language.value if isinstance(language, Language) else language)
        if language is not None
        else None
    )
    return _parse_sync(
        source,
        language=lang_str,
        filename=filename,
        extension=extension,
        include_ast_summary=include_ast_summary,
    )


async def parse(
    source: str,
    *,
    language: Optional[Union[str, Language]] = None,
    filename: Optional[str] = None,
    extension: Optional[str] = None,
    include_ast_summary: bool = False,
) -> ParseOutput:
    """
    Parse source code asynchronously in a worker thread and extract structural facts.

    :param source: The source code string to parse.
    :param language: Explicit language identifier (e.g. 'python', 'javascript', 'rust').
    :param filename: Optional filename used for automatic language detection.
    :param extension: Optional file extension used for language or mode selection.
    :param include_ast_summary: Whether to compute and include AST node counts.
    :return: ParseOutput containing status, completeness, facts, diagnostics, comments.
    """
    return await asyncio.to_thread(
        parse_sync,
        source,
        language=language,
        filename=filename,
        extension=extension,
        include_ast_summary=include_ast_summary,
    )


def detect_language(filename: str) -> Optional[str]:
    """
    Detect the canonical language for a given filename.

    :param filename: The path or filename to inspect.
    :return: Canonical language identifier or None if unrecognized.
    """
    return _detect_language(filename)


def supported_languages() -> List[str]:
    """
    Return the list of all supported canonical language identifiers.
    """
    return _supported_languages()


__all__ = [
    "__version__",
    "Pos",
    "Span",
    "Diagnostic",
    "Comment",
    "AstSummary",
    "TypeKind",
    "ParsedParam",
    "ParsedSymbol",
    "ImportSpecifier",
    "ParsedImport",
    "ParsedCall",
    "UsageSite",
    "ParsedVariable",
    "Facts",
    "ParseOutput",
    "Language",
    "ParseStatus",
    "CommentKind",
    "parse",
    "parse_sync",
    "detect_language",
    "supported_languages",
]
