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
use std::path::Path;

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
            "c" | "h" => Some(Self::C),
            "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some(Self::Cpp),
            "cs" => Some(Self::CSharp),
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
}

pub trait LanguageParser: Send + Sync {
    fn language_id(&self) -> &'static str;
    fn extensions(&self) -> &'static [&'static str];
    fn parse(&self, source: &str) -> ParseResult;
    fn symbols(&self, result: &ParseResult) -> Vec<ParsedSymbol> {
        self.extract(result).symbols
    }
    fn imports(&self, result: &ParseResult) -> Vec<ParsedImport> {
        self.extract(result).imports
    }
    fn calls(&self, result: &ParseResult) -> Vec<ParsedCall> {
        self.extract(result).calls
    }
    fn variables(&self, result: &ParseResult) -> Vec<ParsedVariable> {
        self.extract(result).variables
    }
    fn extract(&self, result: &ParseResult) -> ExtractionResult;
}

pub struct ParseResult {
    pub source: String,
    pub errors: Vec<diagnostics::ParseError>,
    /// Source comments captured by parsers that support comment retention.
    pub comments: Vec<ParsedComment>,
    pub ast: Program,
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
}

/// Returns a parser for `language_id`, picking TypeScript-aware parsing
/// when `ext` indicates a TS file (`.ts`/`.tsx`/`.mts`/`.cts`).
pub fn parser_for_ext(language_id: &str, extension: &str) -> Option<Box<dyn LanguageParser>> {
    let language = Language::from_id(language_id)?;
    #[cfg(feature = "lang-js")]
    let ext = normalized_extension(extension);
    #[cfg(not(feature = "lang-js"))]
    let _ = extension;
    match language {
        #[cfg(feature = "lang-js")]
        Language::JavaScript | Language::TypeScript => {
            let effective_ext = if ext.is_empty() {
                match language {
                    Language::TypeScript => "ts",
                    Language::JavaScript => "js",
                    _ => unreachable!(),
                }
            } else {
                ext.as_str()
            };
            Some(Box::new(JavascriptParser::for_extension(effective_ext)))
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

pub fn parser_for(language_id: &str) -> Option<Box<dyn LanguageParser>> {
    parser_for_ext(language_id, "")
}

pub fn registry() -> HashMap<&'static str, Box<dyn LanguageParser>> {
    #[allow(unused_mut)]
    let mut map: HashMap<&'static str, Box<dyn LanguageParser>> = HashMap::new();
    #[cfg(feature = "lang-js")]
    map.insert("javascript", Box::new(JavascriptParser::new()));
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
            fn extensions(&self) -> &'static [&'static str] {
                $exts
            }
            fn parse(&self, source: &str) -> ParseResult {
                let (ast, errors) = $lang::parse_program(source);
                ParseResult {
                    source: source.to_string(),
                    errors,
                    comments: Vec::new(),
                    ast: Program::$variant(ast),
                }
            }
            fn extract(&self, result: &ParseResult) -> ExtractionResult {
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
impl JavascriptParser {
    /// Plain JavaScript (Script mode, no TypeScript syntax).
    pub fn new() -> Self {
        let mut options = js::config::ParserOptions::default();
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
impl Default for JavascriptParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "lang-js")]
impl LanguageParser for JavascriptParser {
    fn language_id(&self) -> &'static str {
        "javascript"
    }
    fn extensions(&self) -> &'static [&'static str] {
        &["js", "jsx", "ts", "tsx", "mjs", "cjs", "mts", "cts"]
    }
    fn parse(&self, source: &str) -> ParseResult {
        let (program, errors, arena, comments) =
            js::parser::parse_program_with_comments(source, &self.options);
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
        }
    }
    fn extract(&self, result: &ParseResult) -> ExtractionResult {
        #[allow(unreachable_patterns)]
        match &result.ast {
            Program::Js(program, arena) => js::facts::extract_facts(program, arena),
            _ => ExtractionResult::new(),
        }
    }
}
