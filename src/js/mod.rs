//! JavaScript/TypeScript parser module with AST, diagnostics, transforms, codegen,
//! and basic semantic analysis.

pub mod ast;
pub mod codegen;
pub mod config;
pub mod diagnostics;
pub mod facts;
pub mod lexer;
pub mod parser;
pub mod semantic;
pub mod syntax;
pub mod transform;
pub mod visit;

pub use ast::*;
pub use config::{EcmaVersion, ParserOptions, ParserPlugins, SourceType};
pub use diagnostics::{Diagnostic, DiagnosticCode, DiagnosticReporter, ParseError, Severity};
pub use parser::{parse_module, parse_program, parse_script};

#[cfg(test)]
pub mod tests;
