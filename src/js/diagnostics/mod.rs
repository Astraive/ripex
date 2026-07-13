pub use crate::diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticReporter, ParseError, Severity,
};

pub type JsDiagnosticCode = DiagnosticCode;
pub type JsError = ParseError;
