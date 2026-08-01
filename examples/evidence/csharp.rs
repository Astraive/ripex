use super::{EvidenceCase, ExpectedFacts};

// Oracle: names, using source, call callee, and variable names come directly from the C# AST facts.
pub fn cases() -> &'static [EvidenceCase] {
    &[EvidenceCase {
        id: "csharp-using-class-method-call-vars",
        language: "csharp",
        extension: "cs",
        source: r#"using System;

public class Greeter
{
    public string Greet(string name)
    {
        var message = Format(name);
        return message;
    }

    private string Format(string text) => text;
}
"#,
        expected: ExpectedFacts {
            symbols: &["Greeter", "Greeter.Greet", "name", "Greeter.Format", "text"],
            imports: &["System"],
            calls: &["Format"],
            variables: &["name", "message", "text"],
        },
        malformed: &[
            "public class Broken {",
            "public class Broken { public void Run( { }",
        ],
    }]
}
