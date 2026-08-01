use super::{EvidenceCase, ExpectedFacts};

// Oracle: the C extractor should report the include path, the inline struct and
// functions, the one initializer call, and every declared variable/field.
pub fn cases() -> &'static [EvidenceCase] {
    static CASES: &[EvidenceCase] = &[EvidenceCase {
        id: "c17_struct_call",
        language: "c",
        extension: "c",
        source: "#include <stddef.h>\n\
struct Point { int x; int y; } point;\n\
int sum_point(struct Point value) {\n\
    int total = value.x + value.y;\n\
    return total;\n\
}\n\
int main(void) {\n\
    int result = sum_point(point);\n\
    return result;\n\
}\n",
        expected: ExpectedFacts {
            symbols: &["Point", "sum_point", "main"],
            imports: &["stddef.h"],
            calls: &["sum_point"],
            variables: &["point", "x", "y", "value", "total", "result"],
        },
        malformed: &[
            "#include <stddef.h>\nstruct Broken { int x;\n",
            "int broken(int value) { return value;\n",
        ],
    }];
    CASES
}
