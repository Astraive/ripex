pub mod codes;
pub mod error;
pub mod reporter;

pub use codes::DiagnosticCode;
pub use error::ParseError;
pub use reporter::{Diagnostic, DiagnosticReporter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl Severity {
    pub fn is_error(&self) -> bool {
        matches!(self, Severity::Error)
    }
}
