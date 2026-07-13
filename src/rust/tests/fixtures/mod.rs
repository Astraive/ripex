#![allow(dead_code)] // snippet library; not every fixture is used by rust tests

pub const IMPORT_SIMPLE: &str = "use std::collections::HashMap;";

pub const IMPORT_NESTED: &str = "use std::io::{self, Read};";

pub const FN_EMPTY: &str = "fn foo() {}";

pub const FN_PARAMS: &str = "fn add(a: i32, b: i32) -> i32 { a + b }";

pub const FN_ASYNC: &str = "async fn fetch() -> String { String::new() }";

pub const FN_GENERIC: &str = "fn identity<T>(x: T) -> T { x }";

pub const STRUCT_SIMPLE: &str = "struct Point { x: MyInt, y: MyInt }";

pub const ENUM_SIMPLE: &str = "enum MyEnum { A(MyType), B }";

pub const TRAIT_DECL: &str = "trait Display { fn fmt(&self) -> String; }";

pub const IMPL_BLOCK: &str = "struct Foo {}\nimpl Foo { fn bar(&self) {} }";

pub const CONSTRUCTOR: &str = "struct Vec2 { x: MyFloat, y: MyFloat }\nimpl Vec2 { fn new(x: MyFloat, y: MyFloat) -> Self { Self { x, y } } }";

pub const ASSOCIATED_FN: &str =
    "struct Counter { count: MyInt }\nimpl Counter { fn zero() -> Self { Self { count: 0 } } }";

pub const IF_ELSE: &str = "fn check(x: MyInt) -> MyBool { if x > 0 { true } else { false } }";

pub const FOR_LOOP: &str =
    "fn sum(xs: MyVec) -> MyInt { let mut total = 0; for x in xs { total += x; } total }";

pub const WHILE_LOOP: &str = "fn countdown(n: MyInt) { let mut i = n; while i > 0 { i -= 1; } }";

pub const LOOP_INF: &str = "fn forever() { loop { break; } }";

pub const MATCH_EXPR: &str =
    "fn describe(x: MyInt) -> MyStr { match x { 0 => \"zero\", _ => \"other\" } }";

pub const LET_VAR: &str = "fn f() { let x = 42; }";

pub const LET_MUT: &str = "fn f() { let mut y = 10; y += 1; }";

pub const CONST_VAR: &str = "const MAX_SIZE: MyUsize = 1024;";

pub const STATIC_VAR: &str = "static APP_NAME: MyStr = \"graxus\";";

pub const STATIC_MUT: &str = "static mut COUNTER: MyInt = 0;";

pub const MACRO_VEC: &str = "fn make() -> MyVec { vec![1, 2, 3] }";

pub const MACRO_PRINTLN: &str = "fn greet() { println!(\"goodbye\"); }";

pub const ATTR_DERIVE: &str = "#[derive(Debug)]\nstruct Foo { a: MyInt }";

pub const ATTR_TEST: &str = "#[test]\nfn it_works() { assert_eq!(2 + 2, 4); }";

pub const ALL_FIXTURES: &[&str] = &[
    IMPORT_SIMPLE,
    IMPORT_NESTED,
    FN_EMPTY,
    FN_PARAMS,
    FN_ASYNC,
    FN_GENERIC,
    STRUCT_SIMPLE,
    ENUM_SIMPLE,
    TRAIT_DECL,
    IMPL_BLOCK,
    CONSTRUCTOR,
    ASSOCIATED_FN,
    IF_ELSE,
    FOR_LOOP,
    WHILE_LOOP,
    LOOP_INF,
    MATCH_EXPR,
    LET_VAR,
    LET_MUT,
    CONST_VAR,
    STATIC_VAR,
    STATIC_MUT,
    MACRO_VEC,
    MACRO_PRINTLN,
    ATTR_DERIVE,
    ATTR_TEST,
];
