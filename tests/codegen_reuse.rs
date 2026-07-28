//! Code generators are stateful for efficiency, but each `generate` call must
//! produce output for only the supplied program.

fn is_generated_or_dependency_dir(path: &std::path::Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("node_modules" | "target" | ".git" | ".next" | "dist" | "build")
    )
}

#[cfg(feature = "lang-c")]
#[test]
fn c_codegen_can_be_reused() {
    let (program, errors) = ripex::c::parse_program("int main() { return 0; }");
    assert!(errors.is_empty());
    let mut codegen = ripex::c::codegen::Codegen::new();
    let first = codegen
        .generate(&program)
        .expect("canonical C generation");
    assert_eq!(
        codegen
            .generate(&program)
            .expect("canonical C generation"),
        first
    );
}

#[cfg(feature = "lang-c")]
#[test]
fn c_corpus_codegen_round_trips() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test/c");
    let mut stack = vec![root];
    let mut attempted = 0usize;
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if is_generated_or_dependency_dir(&path) {
                continue;
            }
            stack.extend(std::fs::read_dir(path).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        if !matches!(path.extension().and_then(|e| e.to_str()), Some("c" | "h")) {
            continue;
        }
        attempted += 1;
        let source = std::fs::read_to_string(&path).unwrap();
        let (program, errors) = ripex::c::parse_program(&source);
        assert!(
            errors.is_empty(),
            "initial parse failed for {}",
            path.display()
        );
        let generated = match ripex::c::codegen::Codegen::new().generate(&program) {
            Ok(generated) => generated,
            Err(error) => {
                assert!(
                    matches!(
                        error,
                        ripex::c::codegen::GenerationError::UnsupportedNode(_)
                    ),
                    "unexpected C generation error for {}: {error:?}",
                    path.display()
                );
                continue;
            }
        };
        let (_, errors) = ripex::c::parse_program(&generated);
        assert!(
            errors.is_empty(),
            "generated C failed for {}: {errors:?}\n{generated}",
            path.display()
        );
    }
    assert!(attempted > 0, "C corpus was empty");
}

#[cfg(feature = "lang-cpp")]
#[test]
fn cpp_codegen_can_be_reused() {
    let (program, errors) = ripex::cpp::parse_program("int main() { return 0; }");
    assert!(errors.is_empty());
    let mut codegen = ripex::cpp::codegen::Codegen::new();
    let first = codegen
        .generate(&program)
        .expect("canonical C++ generation");
    assert_eq!(
        codegen
            .generate(&program)
            .expect("canonical C++ generation"),
        first
    );
}

#[cfg(feature = "lang-cpp")]
#[test]
fn cpp_corpus_codegen_round_trips() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test/cpp");
    let mut stack = vec![root];
    let mut attempted = 0usize;
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if is_generated_or_dependency_dir(&path) {
                continue;
            }
            stack.extend(std::fs::read_dir(path).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        if !matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("cpp" | "cc" | "cxx" | "hpp" | "hxx")
        ) {
            continue;
        }
        attempted += 1;
        let source = std::fs::read_to_string(&path).unwrap();
        let (program, errors) = ripex::cpp::parse_program(&source);
        assert!(
            errors.is_empty(),
            "initial parse failed for {}",
            path.display()
        );
        let generated = match ripex::cpp::codegen::Codegen::new().generate(&program) {
            Ok(generated) => generated,
            Err(error) => {
                assert!(
                    matches!(
                        error,
                        ripex::cpp::codegen::GenerationError::UnsupportedNode(_)
                    ),
                    "unexpected C++ generation error for {}: {error:?}",
                    path.display()
                );
                continue;
            }
        };
        assert!(
            !generated.trim().is_empty(),
            "C++ generator dropped {}",
            path.display()
        );
        let (_, errors) = ripex::cpp::parse_program(&generated);
        assert!(
            errors.is_empty(),
            "generated C++ failed for {}: {errors:?}\n{generated}",
            path.display()
        );
    }
    assert!(attempted > 0, "C++ corpus was empty");
}

#[cfg(feature = "lang-csharp")]
#[test]
fn csharp_codegen_can_be_reused() {
    let (program, errors) = ripex::csharp::parse_program("class App { }");
    assert!(errors.is_empty());
    let mut codegen = ripex::csharp::codegen::Codegen::new();
    let first = codegen
        .generate(&program)
        .expect("canonical C# generation");
    assert_eq!(
        codegen
            .generate(&program)
            .expect("canonical C# generation"),
        first
    );
}

#[cfg(feature = "lang-csharp")]
#[test]
fn csharp_corpus_codegen_round_trips() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test/csharp");
    let mut stack = vec![root];
    let mut attempted = 0usize;
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if is_generated_or_dependency_dir(&path) {
                continue;
            }
            stack.extend(std::fs::read_dir(path).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("cs") {
            continue;
        }
        attempted += 1;
        let source = std::fs::read_to_string(&path).unwrap();
        let (program, errors) = ripex::csharp::parse_program(&source);
        assert!(
            errors.is_empty(),
            "initial parse failed for {}",
            path.display()
        );
        let generated = match ripex::csharp::codegen::Codegen::new().generate(&program) {
            Ok(generated) => generated,
            Err(error) => {
                assert!(
                    matches!(
                        error,
                        ripex::csharp::codegen::GenerationError::UnsupportedNode(_)
                    ),
                    "unexpected C# generation error for {}: {error:?}",
                    path.display()
                );
                continue;
            }
        };
        assert!(
            !generated.trim().is_empty(),
            "C# generator dropped {}",
            path.display()
        );
        let (_, errors) = ripex::csharp::parse_program(&generated);
        assert!(
            errors.is_empty(),
            "generated C# failed for {}: {errors:?}\n{generated}",
            path.display()
        );
    }
    assert!(attempted > 0, "C# corpus was empty");
}

#[cfg(feature = "lang-go")]
#[test]
fn go_codegen_can_be_reused() {
    let (program, errors) = ripex::go::parse_program("package main\nfunc main() {}");
    assert!(errors.is_empty());
    let mut codegen = ripex::go::codegen::Codegen::new();
    let first = codegen
        .generate(&program)
        .expect("canonical Go generation");
    assert_eq!(
        codegen
            .generate(&program)
            .expect("canonical Go generation"),
        first
    );
}

#[cfg(feature = "lang-go")]
#[test]
fn go_corpus_codegen_round_trips() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("lang-test")
        .join("go");
    let mut stack = vec![root];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if is_generated_or_dependency_dir(&path) {
                continue;
            }
            stack.extend(
                std::fs::read_dir(path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("go") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let (program, errors) = ripex::go::parse_program(&source);
        assert!(
            errors.is_empty(),
            "initial parse failed for {}",
            path.display()
        );
        let generated = ripex::go::codegen::Codegen::new()
            .generate(&program)
            .expect("canonical Go generation");
        let (_, errors) = ripex::go::parse_program(&generated);
        assert!(
            errors.is_empty(),
            "generated Go failed for {}: {errors:?}\n{generated}",
            path.display()
        );
    }
}

#[cfg(feature = "lang-python")]
#[test]
fn python_codegen_can_be_reused() {
    let (program, errors) = ripex::python::parse_program("value = 1\n");
    assert!(errors.is_empty());
    let mut codegen = ripex::python::codegen::Codegen::new();
    let first = codegen
        .generate(&program)
        .expect("canonical Python generation");
    assert_eq!(
        codegen
            .generate(&program)
            .expect("canonical Python generation"),
        first
    );
}

#[cfg(feature = "lang-python")]
#[test]
fn python_corpus_codegen_round_trips() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("lang-test")
        .join("python");
    let mut stack = vec![root];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if is_generated_or_dependency_dir(&path) {
                continue;
            }
            stack.extend(
                std::fs::read_dir(path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("py") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let (program, errors) = ripex::python::parse_program(&source);
        assert!(
            errors.is_empty(),
            "initial parse failed for {}",
            path.display()
        );
        let generated = ripex::python::codegen::Codegen::new()
            .generate(&program)
            .expect("canonical Python generation");
        let (_, errors) = ripex::python::parse_program(&generated);
        assert!(
            errors.is_empty(),
            "generated Python failed for {}: {errors:?}\n{generated}",
            path.display()
        );
    }
}

#[cfg(feature = "lang-rust")]
#[test]
fn rust_codegen_can_be_reused() {
    let (program, errors) = ripex::rust::parse_program("fn main() {}");
    assert!(errors.is_empty());
    let mut codegen = ripex::rust::codegen::Codegen::new();
    let first = codegen
        .generate(&program)
        .expect("canonical Rust generation");
    assert_eq!(
        codegen
            .generate(&program)
            .expect("canonical Rust generation"),
        first
    );
}

#[cfg(feature = "lang-rust")]
#[test]
fn rust_corpus_codegen_round_trips() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("lang-test")
        .join("rust");
    let mut stack = vec![root];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if is_generated_or_dependency_dir(&path) {
                continue;
            }
            stack.extend(
                std::fs::read_dir(path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let (program, errors) = ripex::rust::parse_program(&source);
        assert!(
            errors.is_empty(),
            "initial parse failed for {}",
            path.display()
        );
        let generated = ripex::rust::codegen::Codegen::new()
            .generate(&program)
            .expect("canonical Rust generation");
        let (_, errors) = ripex::rust::parse_program(&generated);
        assert!(
            errors.is_empty(),
            "generated Rust failed for {}: {errors:?}\n{generated}",
            path.display()
        );
    }
}

#[cfg(feature = "lang-js")]
#[test]
fn javascript_typescript_corpus_codegen_round_trips() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test/javascript");
    let mut stack = vec![root];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if is_generated_or_dependency_dir(&path) {
                continue;
            }
            stack.extend(std::fs::read_dir(path).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();
        if !matches!(
            ext,
            "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" | "mts" | "cts"
        ) {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        let mut options = ripex::js::config::ParserOptions::module();
        match ext {
            "jsx" => ripex::js::config::ParserPlugins::all_js().apply(&mut options),
            "ts" | "mts" | "cts" => {
                ripex::js::config::ParserPlugins::typescript().apply(&mut options)
            }
            "tsx" => ripex::js::config::ParserPlugins::all_ts().apply(&mut options),
            _ => {}
        }
        let (program, errors, mut arena) = ripex::js::parser::parse_program(&source, &options);
        assert!(
            errors.is_empty(),
            "initial parse failed for {}",
            path.display()
        );
        let generated = ripex::js::codegen::Printer::new()
            .print_program(&program, &mut arena)
            .expect("canonical JavaScript generation");
        let (_, errors, _) = ripex::js::parser::parse_program(&generated, &options);
        assert!(
            errors.is_empty(),
            "generated JS/TS failed for {}: {errors:?}\n{generated}",
            path.display()
        );
    }
}
