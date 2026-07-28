#![cfg(any(
    feature = "lang-c",
    feature = "lang-cpp",
    feature = "lang-csharp",
    feature = "lang-go",
    feature = "lang-js",
    feature = "lang-python",
    feature = "lang-rust",
))]

//! Regression tests: inputs larger than `MAX_INPUT_SIZE` (1 MB) must not
//! panic. `Parser::new` yields an empty token stream for such inputs, and the
//! per-language `parse_program` must handle the empty stream gracefully
//! (returning an empty program plus an `InputTooLarge` diagnostic) instead of
//! unwrapping `tokens.last()` on an empty vec.

#[cfg(feature = "lang-c")]
use ripex::c::parser::parse_program as c_parse;
#[cfg(feature = "lang-cpp")]
use ripex::cpp::parser::parse_program as cpp_parse;
#[cfg(feature = "lang-csharp")]
use ripex::csharp::parser::parse_program as csharp_parse;
#[cfg(feature = "lang-go")]
use ripex::go::parser::parse_program as go_parse;
#[cfg(feature = "lang-python")]
use ripex::python::parser::parse_program as python_parse;
#[cfg(feature = "lang-rust")]
use ripex::rust::parser::parse_program as rust_parse;

fn big_input(size: usize) -> String {
    // A syntactically valid-but-huge repeat, well over the 1 MB cap.
    let unit = "fn main() { let x = 1; }\n";
    let reps = size / unit.len() + 1;
    unit.repeat(reps)
}

#[cfg(feature = "lang-python")]
#[test]
fn oversized_python_does_not_panic() {
    let src = big_input(2 * 1024 * 1024);
    let _ = python_parse(&src);
}

#[cfg(feature = "lang-rust")]
#[test]
fn oversized_rust_does_not_panic() {
    let src = big_input(2 * 1024 * 1024);
    let _ = rust_parse(&src);
}

#[cfg(feature = "lang-go")]
#[test]
fn oversized_go_does_not_panic() {
    let src = big_input(2 * 1024 * 1024);
    let _ = go_parse(&src);
}

#[cfg(feature = "lang-c")]
#[test]
fn oversized_c_does_not_panic() {
    let src = big_input(2 * 1024 * 1024);
    let _ = c_parse(&src);
}

#[cfg(feature = "lang-cpp")]
#[test]
fn oversized_cpp_does_not_panic() {
    let src = big_input(2 * 1024 * 1024);
    let _ = cpp_parse(&src);
}

#[cfg(feature = "lang-csharp")]
#[test]
fn oversized_csharp_does_not_panic() {
    let src = big_input(2 * 1024 * 1024);
    let _ = csharp_parse(&src);
}
#[cfg(feature = "lang-js")]
#[test]
fn oversized_javascript_does_not_panic() {
    let src = big_input(2 * 1024 * 1024);
    let options = ripex::js::config::ParserOptions::default();
    let (_, errors, _) = ripex::js::parser::parse_script(&src, &options);
    assert!(errors
        .iter()
        .any(|error| error.code == ripex::diagnostics::DiagnosticCode::InputTooLarge));
}
