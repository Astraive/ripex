// Go source-code fixtures for parser tests.

pub const IMPORT_SINGLE: &str = "package main\nimport \"fmt\"\n";
pub const IMPORT_MULTI: &str = "package main\nimport (\n\"fmt\"\n\"os\"\n)\n";
pub const FUNC_EMPTY: &str = "func foo() {}";
pub const FUNC_PARAMS_RETURN: &str = "func add(a int, b int) int { return a + b }";
pub const FUNC_VARIADIC: &str = "func sum(nums ...int) int { return 0 }";
pub const TYPE_STRUCT: &str = "type Point struct {\nx int\n}";
pub const TYPE_INTERFACE: &str = "type Stringer interface {\nString() string\n}";
pub const TYPE_ALIAS: &str = "type MyInt int";
pub const METHOD_RECEIVER: &str = "func (t Type) Method() {}";
pub const IF_ELSE: &str = "func f() { if (true) {} else {} }";
pub const FOR_LOOP: &str = "func f() { for (;;) {} }";
pub const SWITCH: &str = "func f() { switch (x) { case 1: break } }";
pub const DEFER: &str = "func f() { defer cleanup() }";
pub const GOROUTINE: &str = "func f() { go work() }";
pub const VAR_DECL: &str = "var x = 42";
pub const CONST_DECL: &str = "const y = 42";
pub const EXPORTED: &str = "func foo() {}\nfunc Foo() {}";

pub const ALL_FIXTURES: &[(&str, &str)] = &[
    ("import_single", IMPORT_SINGLE),
    ("import_multi", IMPORT_MULTI),
    ("func_empty", FUNC_EMPTY),
    ("func_params_return", FUNC_PARAMS_RETURN),
    ("func_variadic", FUNC_VARIADIC),
    ("type_struct", TYPE_STRUCT),
    ("type_interface", TYPE_INTERFACE),
    ("type_alias", TYPE_ALIAS),
    ("method_receiver", METHOD_RECEIVER),
    ("if_else", IF_ELSE),
    ("for_loop", FOR_LOOP),
    ("switch", SWITCH),
    ("defer", DEFER),
    ("goroutine", GOROUTINE),
    ("var_decl", VAR_DECL),
    ("const_decl", CONST_DECL),
    ("exported", EXPORTED),
];
