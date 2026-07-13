use crate::diagnostics::error::ParseError;
use crate::diagnostics::Severity;

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub error: ParseError,
}

impl Diagnostic {
    pub fn new(severity: Severity, error: ParseError) -> Self {
        Diagnostic { severity, error }
    }

    pub fn error(error: ParseError) -> Self {
        Diagnostic::new(Severity::Error, error)
    }

    pub fn warning(error: ParseError) -> Self {
        Diagnostic::new(Severity::Warning, error)
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Info => "info",
            Severity::Hint => "hint",
        };
        write!(f, "[{}] {}", level, self.error)
    }
}

#[derive(Debug, Clone)]
pub struct DiagnosticReporter {
    diagnostics: Vec<Diagnostic>,
}

impl DiagnosticReporter {
    pub fn new() -> Self {
        DiagnosticReporter {
            diagnostics: Vec::new(),
        }
    }

    pub fn report(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn report_error(&mut self, error: ParseError) {
        self.report(Diagnostic::error(error));
    }

    pub fn report_warning(&mut self, error: ParseError) {
        self.report(Diagnostic::warning(error));
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity.is_error())
    }

    pub fn errors(&self) -> impl Iterator<Item = &Diagnostic> {
        self.diagnostics.iter()
    }

    pub fn into_inner(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn extend(&mut self, other: impl IntoIterator<Item = Diagnostic>) {
        self.diagnostics.extend(other);
    }
}

impl Default for DiagnosticReporter {
    fn default() -> Self {
        DiagnosticReporter::new()
    }
}

impl IntoIterator for DiagnosticReporter {
    type Item = Diagnostic;
    type IntoIter = std::vec::IntoIter<Diagnostic>;

    fn into_iter(self) -> Self::IntoIter {
        self.diagnostics.into_iter()
    }
}
