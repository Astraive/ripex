pub const USING_SYSTEM: &str = "using System;\n";
pub const CLASS_SIMPLE: &str = "class Foo { }\n";
pub const INTERFACE: &str = "interface IBar { public void Baz(); }\n";
pub const ENUM: &str = "enum Color { Red, Green, Blue }\n";
pub const STRUCT_SIMPLE: &str = "struct Point { public int X; public int Y; }\n";

pub const ALL_FIXTURES: &[&str] = &[USING_SYSTEM, CLASS_SIMPLE, INTERFACE, ENUM, STRUCT_SIMPLE];
