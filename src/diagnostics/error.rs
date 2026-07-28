use crate::diagnostics::codes::DiagnosticCode;
use crate::span::Span;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParseError {
    pub code: DiagnosticCode,
    pub span: Span,
    pub message: String,
}

impl ParseError {
    pub fn new(code: DiagnosticCode, span: Span) -> Self {
        let message = code.message().to_string();
        ParseError {
            code,
            span,
            message,
        }
    }

    pub fn with_message(code: DiagnosticCode, span: Span, message: impl Into<String>) -> Self {
        ParseError {
            code,
            span,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.message == self.code.message() {
            write!(f, "{} at {}", self.code.message(), self.span)
        } else {
            write!(
                f,
                "{} at {}: {}",
                self.code.message(),
                self.span,
                self.message
            )
        }
    }
}

impl std::error::Error for ParseError {}
