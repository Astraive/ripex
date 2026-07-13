//! Code generators are stateful for efficiency, but each `generate` call must
//! produce output for only the supplied program.

#[cfg(feature = "lang-c")]
#[test]
fn c_codegen_can_be_reused() {
    let (program, errors) = ripex::c::parse_program("int main() { return 0; }");
    assert!(errors.is_empty());
    let mut codegen = ripex::c::codegen::Codegen::new();
    let first = codegen.generate(&program);
    assert_eq!(codegen.generate(&program), first);
}

#[cfg(feature = "lang-cpp")]
#[test]
fn cpp_codegen_can_be_reused() {
    let (program, errors) = ripex::cpp::parse_program("int main() { return 0; }");
    assert!(errors.is_empty());
    let mut codegen = ripex::cpp::codegen::Codegen::new();
    let first = codegen.generate(&program);
    assert_eq!(codegen.generate(&program), first);
}

#[cfg(feature = "lang-csharp")]
#[test]
fn csharp_codegen_can_be_reused() {
    let (program, errors) = ripex::csharp::parse_program("class App { }");
    assert!(errors.is_empty());
    let mut codegen = ripex::csharp::codegen::Codegen::new();
    let first = codegen.generate(&program);
    assert_eq!(codegen.generate(&program), first);
}

#[cfg(feature = "lang-go")]
#[test]
fn go_codegen_can_be_reused() {
    let (program, errors) = ripex::go::parse_program("package main\nfunc main() {}");
    assert!(errors.is_empty());
    let mut codegen = ripex::go::codegen::Codegen::new();
    let first = codegen.generate(&program);
    assert_eq!(codegen.generate(&program), first);
}

#[cfg(feature = "lang-python")]
#[test]
fn python_codegen_can_be_reused() {
    let (program, errors) = ripex::python::parse_program("value = 1\n");
    assert!(errors.is_empty());
    let mut codegen = ripex::python::codegen::Codegen::new();
    let first = codegen.generate(&program);
    assert_eq!(codegen.generate(&program), first);
}

#[cfg(feature = "lang-rust")]
#[test]
fn rust_codegen_can_be_reused() {
    let (program, errors) = ripex::rust::parse_program("fn main() {}");
    assert!(errors.is_empty());
    let mut codegen = ripex::rust::codegen::Codegen::new();
    let first = codegen.generate(&program);
    assert_eq!(codegen.generate(&program), first);
}
