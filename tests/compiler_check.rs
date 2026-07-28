use ripex::compiler::{
    check_with_compiler, plan_compiler_check, CheckStatus, CompilerCheckOptions,
};
use ripex::Language;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempSource {
    dir: PathBuf,
    path: PathBuf,
}

impl TempSource {
    fn rust(name: &str, source: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = (0_u32..100)
            .map(|attempt| {
                std::env::temp_dir().join(format!("ripex-check-{nonce:x}-{attempt:02x}"))
            })
            .find(|candidate| fs::create_dir(candidate).is_ok())
            .expect("could not create an exclusive test directory");
        let path = dir.join(format!("{name}.rs"));
        fs::write(&path, source).unwrap();
        Self { dir, path }
    }
}

impl Drop for TempSource {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn rustc_accepts_valid_typed_source() {
    let source = TempSource::rust("valid", "pub fn add(a: i32, b: i32) -> i32 { a + b }");
    let report = check_with_compiler(&source.path, None, &Default::default()).unwrap();
    assert_eq!(report.status, CheckStatus::Passed, "{report:#?}");
    assert_eq!(report.stages.len(), 1);
    assert_eq!(report.stages[0].backend, "rustc");
    assert!(report.stages[0]
        .command
        .contains(&"--edition=2021".to_string()));
}

#[test]
fn rustc_rejects_a_real_type_error() {
    let source = TempSource::rust("invalid", "pub fn value() -> i32 { \"not an integer\" }");
    let report = check_with_compiler(&source.path, None, &Default::default()).unwrap();
    assert_eq!(report.status, CheckStatus::Failed, "{report:#?}");
    assert!(
        !report.stages[0].diagnostics.is_empty(),
        "compiler output was not normalized: {report:#?}"
    );
}

#[test]
fn cargo_accepts_the_compiler_conformance_fixture() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test/rust");
    let options = CompilerCheckOptions {
        trusted_project: true,
        ..Default::default()
    };
    let report = check_with_compiler(&project, None, &options).unwrap();
    assert_eq!(report.status, CheckStatus::Passed, "{report:#?}");
    assert_eq!(report.stages.len(), 1);
    assert_eq!(report.stages[0].backend, "cargo");
}

#[test]
fn missing_explicit_toolchain_is_never_treated_as_success() {
    let source = TempSource::rust("valid", "pub fn value() -> i32 { 1 }");
    let options = CompilerCheckOptions {
        toolchain: Some(PathBuf::from("ripex-toolchain-that-does-not-exist")),
        ..Default::default()
    };
    let report = check_with_compiler(&source.path, Some(Language::Rust), &options).unwrap();
    assert_eq!(report.status, CheckStatus::Unavailable);
    assert_eq!(report.stages[0].status, CheckStatus::Unavailable);
}

#[test]
fn every_language_has_a_compiler_plan() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test");
    let cases = [
        (Language::C, root.join("c/main.c")),
        (Language::Cpp, root.join("cpp/main.cpp")),
        (Language::CSharp, root.join("csharp/Program.cs")),
        (Language::Go, root.join("go/main.go")),
        (Language::JavaScript, root.join("javascript/src/index.js")),
        (Language::TypeScript, root.join("javascript/src/types.ts")),
        (Language::Python, root.join("python/src/main.py")),
        (Language::Rust, root.join("rust/src/lib.rs")),
    ];
    for (language, path) in cases {
        let plans = plan_compiler_check(path, Some(language), &Default::default()).unwrap();
        assert!(!plans.is_empty(), "missing {language:?} compiler plan");
        assert!(plans.iter().all(|plan| !plan.candidates.is_empty()));
    }
}

#[test]
fn untrusted_project_checks_fail_closed() {
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test/rust");
    let error = plan_compiler_check(&project, Some(Language::Rust), &Default::default())
        .expect_err("project execution must require explicit trust");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
}

#[test]
fn raw_extra_args_require_explicit_unsafe_gate() {
    let source = TempSource::rust("args", "pub fn value() -> i32 { 1 }");
    let options = CompilerCheckOptions {
        extra_args: vec!["--emit=llvm-ir".into()],
        ..Default::default()
    };
    let error = plan_compiler_check(&source.path, Some(Language::Rust), &options)
        .expect_err("raw args must fail closed");
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);

    let options = CompilerCheckOptions {
        trusted_project: true,
        allow_unsafe_args: true,
        extra_args: vec!["--emit=llvm-ir".into()],
        ..Default::default()
    };
    let plans = plan_compiler_check(&source.path, Some(Language::Rust), &options).unwrap();
    assert!(plans[0].args.contains(&"--emit=llvm-ir".to_string()));
}

#[test]
fn source_discovery_normalizes_extensions_and_is_deterministic() {
    let source = TempSource::rust("discovery", "pub fn value() -> i32 { 1 }");
    let c_source = source.dir.join("UPPER.C");
    fs::write(&c_source, "int value(void) { return 1; }\n").unwrap();
    let options = CompilerCheckOptions {
        trusted_project: true,
        ..Default::default()
    };
    let first = plan_compiler_check(&source.dir, Some(Language::C), &options).unwrap();
    let second = plan_compiler_check(&source.dir, Some(Language::C), &options).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), 1);
    assert!(first[0].args.iter().any(|arg| arg.ends_with("UPPER.C")));
}
