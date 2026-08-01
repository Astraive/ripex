use super::{EvidenceCase, ExpectedFacts};

// Curated syntax exercises imports, class/function symbols, calls, and assignments;
// malformed variants intentionally omit required Python punctuation.
// Expectations are an independent syntax-grounded oracle for the report.
pub fn cases() -> &'static [EvidenceCase] {
    &[EvidenceCase {
        id: "python_core_symbols_and_facts",
        language: "python",
        extension: "py",
        source: r#"import math
from pathlib import Path

class Greeter:
    def greet(greeter, name):
        message = name
        return message

def make_message(label):
    helper = Greeter()
    return helper.greet(label)

result = make_message("Ada")
"#,
        expected: ExpectedFacts {
            symbols: &["Greeter", "greet", "make_message"],
            imports: &["math", "pathlib"],
            calls: &["Greeter", "greet", "make_message"],
            variables: &["greeter", "name", "message", "label", "helper", "result"],
        },
        malformed: &[
            "def broken(value)\n    return value\n",
            "class Broken\n    pass\n",
        ],
    }]
}
