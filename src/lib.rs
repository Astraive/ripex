//! Multi-language structural parsing, fact extraction, canonical generation,
//! and production-toolchain-backed compiler validation.

pub mod arena;
pub mod compiler;
pub mod diagnostics;
pub mod facts;
pub mod limits;
pub mod span;

#[cfg(feature = "lang-c")]
pub mod c;
#[cfg(feature = "lang-cpp")]
pub mod cpp;
#[cfg(feature = "lang-csharp")]
pub mod csharp;
#[cfg(feature = "lang-go")]
pub mod go;
#[cfg(feature = "lang-js")]
pub mod js;
#[cfg(feature = "lang-python")]
pub mod python;
#[cfg(feature = "lang-rust")]
pub mod rust;

use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use crate::diagnostics::DiagnosticCode;

pub use facts::*;

/// A language family understood by ripex.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
    Go,
    Rust,
    C,
    Cpp,
    CSharp,
    Java,
    Kotlin,
    Swift,
}

impl Language {
    pub const fn id(self) -> &'static str {
        match self {
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Python => "python",
            Self::Go => "go",
            Self::Rust => "rust",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::CSharp => "csharp",
            Self::Java => "java",
            Self::Kotlin => "kotlin",
            Self::Swift => "swift",
        }
    }
    pub fn from_id(id: &str) -> Option<Self> {
        match id.to_ascii_lowercase().as_str() {
            "javascript" | "js" | "jsx" => Some(Self::JavaScript),
            "typescript" | "ts" | "tsx" => Some(Self::TypeScript),
            "python" | "py" => Some(Self::Python),
            "go" => Some(Self::Go),
            "rust" | "rs" => Some(Self::Rust),
            "c" => Some(Self::C),
            "cpp" | "c++" | "cxx" => Some(Self::Cpp),
            "csharp" | "c#" | "cs" => Some(Self::CSharp),
            "java" => Some(Self::Java),
            "kotlin" | "kt" | "kts" => Some(Self::Kotlin),
            "swift" => Some(Self::Swift),
            _ => None,
        }
    }

    pub fn from_extension(extension: &str) -> Option<Self> {
        match normalized_extension(extension).as_str() {
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::JavaScript),
            "ts" | "tsx" | "mts" | "cts" => Some(Self::TypeScript),
            "py" | "pyi" => Some(Self::Python),
            "go" => Some(Self::Go),
            "rs" => Some(Self::Rust),
            // C and C++ both commonly use .h; automatic detection must not
            // silently choose one. Use an explicit language selector instead.
            "c" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some(Self::Cpp),
            "cs" => Some(Self::CSharp),
            "java" => Some(Self::Java),
            "kt" | "kts" => Some(Self::Kotlin),
            "swift" => Some(Self::Swift),
            _ => None,
        }
    }
}

fn normalized_extension(extension: &str) -> String {
    extension.trim_start_matches('.').to_ascii_lowercase()
}

/// Detect a language from a source path using the same rules as the CLI.
pub fn detect_language(path: impl AsRef<Path>) -> Option<Language> {
    let extension = path.as_ref().extension()?.to_str()?;
    Language::from_extension(extension)
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExtractionResult {
    pub symbols: Vec<ParsedSymbol>,
    pub imports: Vec<ParsedImport>,
    pub calls: Vec<ParsedCall>,
    pub variables: Vec<ParsedVariable>,
}

impl Default for ExtractionResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractionResult {
    pub fn new() -> Self {
        ExtractionResult {
            symbols: Vec::new(),
            imports: Vec::new(),
            calls: Vec::new(),
            variables: Vec::new(),
        }
    }

    /// Remove malformed call facts before exposing extraction results publicly.
    fn retain_valid_calls(&mut self) {
        self.calls
            .retain(|call| !call.callee_text.trim().is_empty());
    }
}

/// Completeness of a parser result and its semantic facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ParseStatus {
    /// Parsing completed without diagnostics.
    Complete,
    /// Parsing recovered from one or more syntax diagnostics.
    Recovered,
    /// A configured parser resource limit was reached.
    LimitExceeded,
    /// Parsing failed before a trustworthy program could be produced.
    Failed,
}

impl ParseStatus {
    pub const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }

    pub const fn is_incomplete(self) -> bool {
        !self.is_complete()
    }
}

#[allow(dead_code)]
fn status_from_errors(errors: &[diagnostics::ParseError]) -> ParseStatus {
    if errors.is_empty() {
        ParseStatus::Complete
    } else if errors.iter().any(|error| {
        matches!(
            error.code,
            DiagnosticCode::InputTooLarge
                | DiagnosticCode::TokenLimitExceeded
                | DiagnosticCode::MaxRecursionExceeded
        )
    }) {
        ParseStatus::LimitExceeded
    } else {
        ParseStatus::Recovered
    }
}

/// Error returned when facts are requested from an incompatible or incomplete
/// parse result.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExtractionError {
    /// The result was produced by a different language parser or mode.
    ParserMismatch {
        expected_language: Language,
        actual_language: Language,
        expected_mode: String,
        actual_mode: String,
    },
    /// Strict extraction rejects recovered or otherwise incomplete results.
    IncompleteParse { status: ParseStatus },
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParserMismatch {
                expected_language,
                actual_language,
                expected_mode,
                actual_mode,
            } => write!(
                f,
                "parser mismatch: expected {} ({expected_mode}), got {} ({actual_mode})",
                expected_language.id(),
                actual_language.id(),
            ),
            Self::IncompleteParse { status } => {
                write!(f, "cannot perform strict extraction from {status:?} parse")
            }
        }
    }
}

impl std::error::Error for ExtractionError {}

/// Parser implementation for one language and effective parser mode.
pub trait LanguageParser: Send + Sync {
    fn language_id(&self) -> &'static str;
    fn language(&self) -> Language;
    fn parser_mode(&self) -> &'static str {
        "default"
    }
    fn extensions(&self) -> &'static [&'static str];
    fn parse(&self, source: &str) -> ParseResult;
    fn extract_unchecked(&self, result: &ParseResult) -> ExtractionResult;

    fn validate_result(&self, result: &ParseResult) -> Result<(), ExtractionError> {
        if result.language != self.language() || result.parser_mode != self.parser_mode() {
            return Err(ExtractionError::ParserMismatch {
                expected_language: self.language(),
                actual_language: result.language,
                expected_mode: self.parser_mode().to_string(),
                actual_mode: result.parser_mode.clone(),
            });
        }
        Ok(())
    }

    /// Extract facts only from a complete parse result.
    fn extract(&self, result: &ParseResult) -> Result<ExtractionResult, ExtractionError> {
        self.validate_result(result)?;
        if !result.status.is_complete() {
            return Err(ExtractionError::IncompleteParse {
                status: result.status,
            });
        }
        let mut extracted = self.extract_unchecked(result);
        extracted.retain_valid_calls();
        Ok(extracted)
    }

    /// Explicitly opt into facts recovered from an incomplete parse.
    fn extract_best_effort(
        &self,
        result: &ParseResult,
    ) -> Result<ExtractionResult, ExtractionError> {
        self.validate_result(result)?;
        let mut extracted = self.extract_unchecked(result);
        extracted.retain_valid_calls();
        Ok(extracted)
    }

    fn symbols(&self, result: &ParseResult) -> Result<Vec<ParsedSymbol>, ExtractionError> {
        Ok(self.extract(result)?.symbols)
    }
    fn imports(&self, result: &ParseResult) -> Result<Vec<ParsedImport>, ExtractionError> {
        Ok(self.extract(result)?.imports)
    }
    fn calls(&self, result: &ParseResult) -> Result<Vec<ParsedCall>, ExtractionError> {
        Ok(self.extract(result)?.calls)
    }
    fn variables(&self, result: &ParseResult) -> Result<Vec<ParsedVariable>, ExtractionError> {
        Ok(self.extract(result)?.variables)
    }
}

/// Result of parsing source text, branded with its producing parser.
pub struct ParseResult {
    pub source: String,
    pub errors: Vec<diagnostics::ParseError>,
    /// Source comments captured by parsers that support comment retention.
    pub comments: Vec<ParsedComment>,
    pub ast: Program,
    /// Completeness and resource status of this parse.
    pub status: ParseStatus,
    /// Language grammar that produced this result.
    pub language: Language,
    /// Effective parser mode (for example `typescript-jsx-module`).
    pub parser_mode: String,
}

impl ParseResult {
    pub const fn is_complete(&self) -> bool {
        self.status.is_complete()
    }

    pub const fn producer_language(&self) -> Language {
        self.language
    }

    pub fn effective_parser_mode(&self) -> &str {
        &self.parser_mode
    }
}

pub enum Program {
    #[cfg(feature = "lang-js")]
    Js(js::ast::Program, js::ast::ExprArena),
    #[cfg(feature = "lang-python")]
    Python(python::Program),
    #[cfg(feature = "lang-go")]
    Go(go::Program),
    #[cfg(feature = "lang-rust")]
    Rust(rust::Program),
    #[cfg(feature = "lang-c")]
    C(c::Program),
    #[cfg(feature = "lang-cpp")]
    Cpp(cpp::Program),
    #[cfg(feature = "lang-csharp")]
    CSharp(csharp::Program),
    #[cfg(not(any(
        feature = "lang-js",
        feature = "lang-python",
        feature = "lang-go",
        feature = "lang-rust",
        feature = "lang-c",
        feature = "lang-cpp",
        feature = "lang-csharp"
    )))]
    None,
}

/// Select a parser by an explicit language and an optional compatible
/// extension. The language is authoritative: a `.ts` extension cannot turn
/// an explicitly selected JavaScript parser into TypeScript (and vice versa).
pub fn parser_for_language(
    language: Language,
    extension: Option<&str>,
) -> Option<Box<dyn LanguageParser>> {
    #[cfg(feature = "lang-js")]
    let extension = extension.map(normalized_extension);
    #[cfg(not(feature = "lang-js"))]
    let _ = extension;
    match language {
        #[cfg(feature = "lang-js")]
        Language::JavaScript => {
            let parser = match extension.as_deref() {
                Some("js" | "mjs" | "ts" | "mts" | "cts") => JavascriptParser::module(),
                Some("jsx" | "tsx") => JavascriptParser::with_jsx(),
                Some("cjs") | None => JavascriptParser::new(),
                // Unknown extensions cannot change the explicit JavaScript
                // grammar or source mode.
                Some(_) => JavascriptParser::new(),
            };
            Some(Box::new(parser))
        }
        #[cfg(feature = "lang-js")]
        Language::TypeScript => {
            let parser = match extension.as_deref() {
                Some("tsx") | Some("jsx") => JavascriptParser::with_typescript_jsx(),
                // An explicit TypeScript selection remains authoritative even
                // when the physical file uses a JavaScript extension.
                _ => JavascriptParser::with_typescript(),
            };
            Some(Box::new(parser))
        }
        #[cfg(feature = "lang-python")]
        Language::Python => Some(Box::new(PythonParser::new())),
        #[cfg(feature = "lang-go")]
        Language::Go => Some(Box::new(GoParser::new())),
        #[cfg(feature = "lang-rust")]
        Language::Rust => Some(Box::new(RustParser::new())),
        #[cfg(feature = "lang-c")]
        Language::C => Some(Box::new(CParser::new())),
        #[cfg(feature = "lang-cpp")]
        Language::Cpp => Some(Box::new(CppParser::new())),
        #[cfg(feature = "lang-csharp")]
        Language::CSharp => Some(Box::new(CSharpParser::new())),
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

/// Convenience selector for callers that already hold a physical extension.
pub fn parser_for_language_ext(
    language: Language,
    extension: &str,
) -> Option<Box<dyn LanguageParser>> {
    parser_for_language(
        language,
        (!extension.trim().is_empty()).then_some(extension),
    )
}

/// Returns a parser for `language_id` with an optional compatible extension.
pub fn parser_for_ext(language_id: &str, extension: &str) -> Option<Box<dyn LanguageParser>> {
    let language = Language::from_id(language_id)?;
    parser_for_language(
        language,
        (!extension.trim().is_empty()).then_some(extension),
    )
}

pub fn parser_for(language_id: &str) -> Option<Box<dyn LanguageParser>> {
    let language = Language::from_id(language_id)?;
    parser_for_language(language, None)
}

pub fn registry() -> HashMap<&'static str, Box<dyn LanguageParser>> {
    #[allow(unused_mut)]
    let mut map: HashMap<&'static str, Box<dyn LanguageParser>> = HashMap::new();
    #[cfg(feature = "lang-js")]
    map.insert("javascript", Box::new(JavascriptParser::new()));
    #[cfg(feature = "lang-js")]
    map.insert("typescript", Box::new(JavascriptParser::with_typescript()));
    #[cfg(feature = "lang-python")]
    map.insert("python", Box::new(PythonParser::new()));
    #[cfg(feature = "lang-go")]
    map.insert("go", Box::new(GoParser::new()));
    #[cfg(feature = "lang-rust")]
    map.insert("rust", Box::new(RustParser::new()));
    #[cfg(feature = "lang-c")]
    map.insert("c", Box::new(CParser::new()));
    #[cfg(feature = "lang-cpp")]
    map.insert("cpp", Box::new(CppParser::new()));
    #[cfg(feature = "lang-csharp")]
    map.insert("csharp", Box::new(CSharpParser::new()));
    map
}

#[cfg(any(
    feature = "lang-python",
    feature = "lang-go",
    feature = "lang-rust",
    feature = "lang-c",
    feature = "lang-cpp",
    feature = "lang-csharp"
))]
macro_rules! lang_parser_impl {
    ($name:ident, $lang:ident, $variant:ident, $id:expr_2021, $exts:expr_2021) => {
        #[derive(Default)]
        pub struct $name;
        impl $name {
            pub fn new() -> Self {
                Self::default()
            }
        }
        impl LanguageParser for $name {
            fn language_id(&self) -> &'static str {
                $id
            }
            fn language(&self) -> Language {
                Language::$variant
            }
            fn extensions(&self) -> &'static [&'static str] {
                $exts
            }
            fn parse(&self, source: &str) -> ParseResult {
                let (ast, errors) = $lang::parse_program(source);
                let status = status_from_errors(&errors);
                ParseResult {
                    source: source.to_string(),
                    errors,
                    comments: Vec::new(),
                    ast: Program::$variant(ast),
                    status,
                    language: Language::$variant,
                    parser_mode: "default".to_string(),
                }
            }
            fn extract_unchecked(&self, result: &ParseResult) -> ExtractionResult {
                #[allow(unreachable_patterns)]
                match &result.ast {
                    Program::$variant(ast) => $lang::facts::extract_facts(ast),
                    _ => ExtractionResult::new(),
                }
            }
        }
    };
}

#[cfg(feature = "lang-python")]
lang_parser_impl!(PythonParser, python, Python, "python", &["py", "pyi"]);
#[cfg(feature = "lang-go")]
lang_parser_impl!(GoParser, go, Go, "go", &["go"]);
#[cfg(feature = "lang-rust")]
lang_parser_impl!(RustParser, rust, Rust, "rust", &["rs"]);
#[cfg(feature = "lang-c")]
lang_parser_impl!(CParser, c, C, "c", &["c", "h"]);
#[cfg(feature = "lang-cpp")]
lang_parser_impl!(
    CppParser,
    cpp,
    Cpp,
    "cpp",
    &["cpp", "hpp", "cc", "cxx", "hh", "hxx"]
);
#[cfg(feature = "lang-csharp")]
lang_parser_impl!(CSharpParser, csharp, CSharp, "csharp", &["cs"]);

#[cfg(feature = "lang-js")]
// JS/TS has special arena handling
pub struct JavascriptParser {
    options: js::config::ParserOptions,
}

#[cfg(feature = "lang-js")]
impl Default for JavascriptParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "lang-js")]
impl JavascriptParser {
    /// Plain JavaScript (Script mode, no TypeScript syntax).
    pub fn new() -> Self {
        let mut options = js::config::ParserOptions::default();
        options.features.import_attributes = true;
        JavascriptParser { options }
    }
    /// JavaScript module parsing without JSX or TypeScript syntax.
    pub fn module() -> Self {
        let mut options = js::config::ParserOptions::module();
        options.features.import_attributes = true;
        JavascriptParser { options }
    }

    /// TypeScript-capable parser (Module mode with TS + JSX features enabled).
    /// Use for `.ts`/`.tsx`/`.mts`/`.cts` files.
    pub fn with_typescript() -> Self {
        let mut options = js::config::ParserOptions::module();
        js::config::ParserPlugins::typescript().apply(&mut options);
        JavascriptParser { options }
    }

    /// JavaScript module parsing with JSX enabled.
    pub fn with_jsx() -> Self {
        let mut options = js::config::ParserOptions::module();
        js::config::ParserPlugins::all_js().apply(&mut options);
        JavascriptParser { options }
    }

    /// TypeScript module parsing with JSX enabled.
    pub fn with_typescript_jsx() -> Self {
        let mut options = js::config::ParserOptions::module();
        js::config::ParserPlugins::all_ts().apply(&mut options);
        JavascriptParser { options }
    }

    /// Returns a parser suitable for the given file extension.
    pub fn for_extension(ext: &str) -> Self {
        match normalized_extension(ext).as_str() {
            "tsx" => Self::with_typescript_jsx(),
            "ts" | "mts" | "cts" => Self::with_typescript(),
            "jsx" => Self::with_jsx(),
            "js" | "mjs" => {
                let mut options = js::config::ParserOptions::module();
                options.features.import_attributes = true;
                Self { options }
            }
            _ => Self::new(),
        }
    }
}

#[cfg(feature = "lang-js")]
impl LanguageParser for JavascriptParser {
    fn language_id(&self) -> &'static str {
        if self.options.features.typescript {
            "typescript"
        } else {
            "javascript"
        }
    }
    fn language(&self) -> Language {
        if self.options.features.typescript {
            Language::TypeScript
        } else {
            Language::JavaScript
        }
    }
    fn parser_mode(&self) -> &'static str {
        match (
            self.options.features.typescript,
            self.options.features.jsx,
            self.options.is_module(),
        ) {
            (true, true, true) => "typescript-jsx-module",
            (true, false, true) => "typescript-module",
            (true, true, false) => "typescript-jsx-script",
            (true, false, false) => "typescript-script",
            (false, true, true) => "javascript-jsx-module",
            (false, false, true) => "javascript-module",
            (false, true, false) => "javascript-jsx-script",
            (false, false, false) => "javascript-script",
        }
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"]
    }
    fn parse(&self, source: &str) -> ParseResult {
        let (program, errors, arena, comments) =
            js::parser::parse_program_with_comments(source, &self.options);
        let status = status_from_errors(&errors);
        ParseResult {
            source: source.to_string(),
            errors,
            comments: comments
                .into_iter()
                .map(|comment| ParsedComment {
                    kind: if comment.text.starts_with("#!") {
                        CommentKind::Hashbang
                    } else if comment.multi_line {
                        CommentKind::Block
                    } else {
                        CommentKind::Line
                    },
                    text: comment.text,
                    span: comment.span,
                })
                .collect(),
            ast: Program::Js(program, arena),
            status,
            language: self.language(),
            parser_mode: self.parser_mode().to_string(),
        }
    }
    fn extract_unchecked(&self, result: &ParseResult) -> ExtractionResult {
        #[allow(unreachable_patterns)]
        match &result.ast {
            Program::Js(program, arena) => js::facts::extract_facts(program, arena),
            _ => ExtractionResult::new(),
        }
    }
}
