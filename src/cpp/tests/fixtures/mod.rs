pub const USING_NAMESPACE: &str = "using namespace std;\n";
pub const FUNC_ADD: &str = "int add(int a, int b) { return a + b; }\n";
pub const CLASS_SIMPLE: &str = "class MyClass { public: int x; };\n";
pub const TEMPLATE_FUNC: &str = "template <typename T> void sort(T arr) { }\n";
pub const CLASS_DERIVED: &str = "class Derived : public Base { };\n";
pub const NAMESPACE: &str = "namespace ns { int x; }\n";
pub const FUNC_EXCEPTION: &str = "void test() { try { } catch (int e) { } }\n";

pub const ALL_FIXTURES: &[&str] = &[
    USING_NAMESPACE,
    FUNC_ADD,
    CLASS_SIMPLE,
    TEMPLATE_FUNC,
    CLASS_DERIVED,
    NAMESPACE,
    FUNC_EXCEPTION,
];
