use super::{EvidenceCase, ExpectedFacts};

// The module source keeps import/export, symbol, call, and binding facts explicit;
// malformed inputs exercise recovery without changing the valid oracle.
pub fn cases() -> &'static [EvidenceCase] {
    &[EvidenceCase {
        id: "javascript-module-bindings",
        language: "javascript",
        extension: "mjs",
        source: r#"import { formatName } from "./format.js";
export const prefix = "Hello";
export function greet(name) {
  const message = `${prefix}, ${name}`;
  return message;
}
const output = greet("Ada");
"#,
        expected: ExpectedFacts {
            symbols: &["prefix", "greet", "message", "output"],
            imports: &["./format.js"],
            calls: &["greet"],
            variables: &["prefix", "name", "message", "output"],
        },
        malformed: &[
            "import { formatName from \"./format.js\";",
            "export function broken(name) { return name;",
        ],
    }]
}
