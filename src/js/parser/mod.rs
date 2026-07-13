pub mod declarations;
pub mod expressions;
pub mod jsx;
pub mod modules;
pub mod patterns;
pub mod recovery;
pub mod state;
pub mod statements;
pub mod typescript;

pub use state::{Context, Parser};

use crate::arena::Arena;
use crate::diagnostics::{DiagnosticCode, ParseError};
use crate::js::ast::{Expr, Module, Program, Script, Stmt};
use crate::js::config::ParserOptions;
use crate::js::lexer::{Lexer, Token};

pub fn parse_program(
    source: &str,
    options: &ParserOptions,
) -> (Program, Vec<ParseError>, Arena<Expr>) {
    if source.len() > crate::limits::MAX_INPUT_SIZE {
        let err = ParseError::new(DiagnosticCode::InputTooLarge, crate::span::Span::ZERO);
        let ast = Arena::new();
        return (
            Program::Script(Script {
                span: crate::span::Span::ZERO,
                body: Vec::new(),
            }),
            vec![err],
            ast,
        );
    }
    let tokens = tokenize(source, options);
    let mut parser = Parser::new(tokens, options);
    let program = parser.parse_program();
    let errors = parser.errors.clone();
    let ast = parser.ast;
    (program, errors, ast)
}

pub fn parse_module(
    source: &str,
    options: &ParserOptions,
) -> (Module, Vec<ParseError>, Arena<Expr>) {
    let mut opts = options.clone();
    opts.source_type = crate::js::config::SourceType::Module;
    let tokens = tokenize(source, &opts);
    let mut parser = Parser::new(tokens, &opts);
    let module = parser.parse_module();
    let errors = parser.errors.clone();
    let ast = parser.ast;
    (module, errors, ast)
}

pub fn parse_script(
    source: &str,
    options: &ParserOptions,
) -> (Vec<Stmt>, Vec<ParseError>, Arena<Expr>) {
    let tokens = tokenize(source, options);
    let mut parser = Parser::new(tokens, options);
    let stmts = parser.parse_script();
    let errors = parser.errors.clone();
    let ast = parser.ast;
    (stmts, errors, ast)
}

fn tokenize(source: &str, options: &ParserOptions) -> Vec<Token> {
    let lexer = Lexer::with_jsx(source, options.features.jsx);
    lexer.collect()
}
