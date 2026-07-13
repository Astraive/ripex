pub const FUNC_ADD: &str = "int add(int a, int b) { return a + b; }\n";
pub const STRUCT_VAR: &str = "struct Point p;\n";
pub const FUNC_IF: &str = "void test(int x) { if (x) { return; } else { return; } }\n";
pub const FUNC_FOR: &str = "int test(int a) { if (a) { return a; } return 0; }\n";
pub const FUNC_WHILE: &str = "void test() { while (1) { break; } }\n";
pub const PTR_DECL: &str = "int* ptr;\n";
pub const EXTERN_FUNC: &str = "extern void abort();\n";

pub const ALL_FIXTURES: &[&str] = &[
    FUNC_ADD,
    STRUCT_VAR,
    FUNC_IF,
    FUNC_FOR,
    FUNC_WHILE,
    PTR_DECL,
    EXTERN_FUNC,
];
