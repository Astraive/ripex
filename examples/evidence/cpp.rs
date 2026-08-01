use super::{EvidenceCase, ExpectedFacts};

/// The oracle lists the exact names/sources/callees/variables emitted for this
/// compact C++20 namespace, class, method call, and declaration sample.
pub fn cases() -> &'static [EvidenceCase] {
    &[EvidenceCase {
        id: "cpp20_namespace_class_call",
        language: "cpp",
        extension: "cpp",
        source: r#"#include <vector>
namespace telemetry {
class Counter {
public:
    int value;
    int add(int amount) { return amount; }
};
constexpr int default_seed = 2;
int run(int seed) {
    Counter counter;
    int total = counter.add(seed);
    return total;
}
}
"#,
        expected: ExpectedFacts {
            symbols: &["telemetry", "Counter", "add", "run"],
            imports: &["vector"],
            calls: &["add"],
            variables: &[
                "value",
                "amount",
                "default_seed",
                "seed",
                "counter",
                "total",
            ],
        },
        malformed: &[
            "#include <vector\nint main() { return 0;",
            "class Broken { int value;\nint run() { return 1; }",
            "int compute( { return 1; }",
        ],
    }]
}
