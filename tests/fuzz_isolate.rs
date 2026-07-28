#![cfg(feature = "lang-python")]

//! Minimal test to isolate which parser/language causes the OOM crash.
use std::io::Write;

#[test]
fn isolate_python_def_pass() {
    let mut out = std::io::stdout();
    let _ = writeln!(&mut out, "Creating lexer...");
    let _ = out.flush();
    let lexer = ripex::python::lexer::Lexer::new("def f(): pass");
    let _ = writeln!(&mut out, "Tokenizing...");
    let _ = out.flush();
    let (tokens, errors) = lexer.tokenize();
    let _ = writeln!(
        &mut out,
        "Got {} tokens, {} errors",
        tokens.len(),
        errors.len()
    );
    let _ = out.flush();
    for (i, t) in tokens.iter().enumerate() {
        let _ = writeln!(&mut out, "  token[{}]: {:?} {:?}", i, t.kind, t.span);
        let _ = out.flush();
    }

    let _ = writeln!(&mut out, "Parsing...");
    let _ = out.flush();
    let (_program, errors) = ripex::python::parse_program("def f(): pass");
    let _ = writeln!(&mut out, "Parsed: {} errors", errors.len());
    let _ = out.flush();

    let _ = writeln!(&mut out, "Success!");
    let _ = out.flush();
}
