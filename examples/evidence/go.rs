use super::{EvidenceCase, ExpectedFacts};

// Oracle intent: keep this case small enough to verify package imports, exported
// declarations, selector calls, and both parameter and local-variable facts by hand.
const GO_BASICS: EvidenceCase = EvidenceCase {
    id: "go-basics",
    language: "go",
    extension: "go",
    source: r#"package main

import "fmt"

type Person struct {
    Name string
}

func Greet(name string) string {
    message := "hello " + name
    fmt.Println(message)
    return message
}
"#,
    expected: ExpectedFacts {
        symbols: &["Person", "Greet"],
        imports: &["\"fmt\""],
        calls: &["Println"],
        variables: &["name", "message"],
    },
    malformed: &[
        "package main\nfunc broken( {",
        "package main\nimport \"fmt\"\nfunc broken() { fmt.Println(",
    ],
};

pub fn cases() -> &'static [EvidenceCase] {
    &[GO_BASICS]
}
