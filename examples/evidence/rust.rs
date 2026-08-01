use super::{EvidenceCase, ExpectedFacts};

// Oracle names follow the Rust fact extractor: use paths, declarations, call callees, and bindings.
const CASES: &[EvidenceCase] = &[EvidenceCase {
    id: "rust_struct_function_facts",
    language: "rust",
    extension: "rs",
    source: r#"use std::fmt::Display;

pub struct Point {
    x: i32,
    y: i32,
}

pub fn sum(point: Point) -> i32 {
    let total = point.x + point.y;
    helper(total)
}

fn helper(value: i32) -> i32 {
    value
}
"#,
    expected: ExpectedFacts {
        symbols: &["Point", "x", "y", "sum", "helper"],
        imports: &["std::fmt::Display"],
        calls: &["helper"],
        variables: &["point", "total", "value"],
    },
    malformed: &[
        "pub struct MissingBrace { field: i32",
        "fn missing_parameter(value: i32 -> i32 { value }",
    ],
}];

pub fn cases() -> &'static [EvidenceCase] {
    CASES
}
