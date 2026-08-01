use super::{EvidenceCase, ExpectedFacts};

// Oracle intent: declarations, the import source, calls, and bindings are all explicit in the source.
pub fn cases() -> &'static [EvidenceCase] {
    &[EvidenceCase {
        id: "typescript_interface_function_constants",
        language: "typescript",
        extension: "ts",
        source: r#"import { normalize } from "./text";

export interface User {
  name: string;
}

export function greet(input: User): string {
  const text: string = normalize(input.name);
  return text;
}

export const defaultUser: User = { name: "Ada" };
const greeting: string = greet(defaultUser);
"#,
        expected: ExpectedFacts {
            symbols: &["User", "greet", "text", "defaultUser", "greeting"],
            imports: &["./text"],
            calls: &["normalize", "greet"],
            variables: &["input", "text", "defaultUser", "greeting"],
        },
        malformed: &[
            "export const = 1;",
            "export function broken(input: User): string { const value: string = ; return value; }",
        ],
    }]
}
