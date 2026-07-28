#![cfg(feature = "cli")]

use std::process::Command;
use std::{fs, path::PathBuf};

#[cfg(feature = "lang-js")]
#[test]
fn tsx_fixture_is_clean_through_the_cli() {
    let fixture = format!(
        "{}/tests/lang-test/javascript/src/app.tsx",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args(["parse", &fixture, "--json", "--ast"])
        .output()
        .expect("run ripex CLI");

    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid CLI JSON");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["language"], "typescript");
    assert_eq!(json["status"], "complete");
    assert!(json["effective_mode"]
        .as_str()
        .unwrap_or_default()
        .contains("typescript"));
    assert_eq!(json["errors"], serde_json::json!([]));
    assert_eq!(json["ast"]["kind"], "javascript_module");
}

#[cfg(feature = "lang-js")]
#[test]
fn javascript_comments_are_preserved_in_cli_json() {
    let path = std::env::temp_dir().join(format!("ripex-cli-comments-{}.mjs", std::process::id()));
    fs::write(
        &path,
        "#!/usr/bin/env node\n// leading\nconst answer = 42;\n/* trailing */",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args(["parse", path.to_str().unwrap(), "--json"])
        .output()
        .expect("run ripex CLI");
    let _ = fs::remove_file(&path);

    assert!(
        output.status.success(),
        "stderr: {}\nstdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid CLI JSON");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["status"], "complete");
    let comments = json["comments"].as_array().expect("comments array");
    assert_eq!(comments.len(), 3, "{comments:#?}");
    assert_eq!(comments[0]["kind"], "hashbang");
    assert_eq!(comments[1]["kind"], "line");
    assert_eq!(comments[2]["kind"], "block");
}

#[test]
fn compiler_check_json_reports_real_type_errors() {
    let path = std::env::temp_dir().join(format!("ripex-cli-check-{}.rs", std::process::id()));
    fs::write(&path, "fn value() -> i32 { \"wrong\" }").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args([
            "check",
            path.to_str().unwrap(),
            "--trusted-project",
            "--json",
        ])
        .output()
        .expect("run ripex compiler check");
    let _ = fs::remove_file(&path);

    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["language"], "rust");
    assert_eq!(json["status"], "failed");
    assert_eq!(json["stages"][0]["backend"], "rustc");
    assert!(!json["stages"][0]["diagnostics"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[test]
fn compiler_check_missing_toolchain_exits_two() {
    let path = std::env::temp_dir().join(format!("ripex-cli-toolchain-{}.rs", std::process::id()));
    fs::write(&path, "fn value() -> i32 { 1 }").unwrap();
    let missing = PathBuf::from("ripex-toolchain-that-does-not-exist");
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args([
            "check",
            path.to_str().unwrap(),
            "--trusted-project",
            "--toolchain",
            missing.to_str().unwrap(),
            "--json",
        ])
        .output()
        .expect("run ripex compiler check");
    let _ = fs::remove_file(&path);

    assert_eq!(output.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["status"], "unavailable");
}

#[test]
fn missing_file_json_is_a_single_error_envelope() {
    let path = std::env::temp_dir().join(format!(
        "ripex-cli-missing-{}-{}.rs",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args(["parse", path.to_str().unwrap(), "--json"])
        .output()
        .expect("run ripex CLI");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stderr.windows(6).any(|window| window == b"ripex:"));
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("missing-file JSON envelope");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["operation"], "parse");
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "file_not_found");
    assert!(!output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stdout).contains("ripex:"));
}

#[cfg(feature = "lang-js")]
#[test]
fn explicit_typescript_mode_overrides_javascript_extension() {
    let path = std::env::temp_dir().join(format!("ripex-cli-ts-on-js-{}.js", std::process::id()));
    fs::write(
        &path,
        "interface Generated { value: number }\nconst value: number = 1;",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args([
            "parse",
            path.to_str().unwrap(),
            "--lang",
            "typescript",
            "--json",
        ])
        .output()
        .expect("run ripex CLI");
    let _ = fs::remove_file(&path);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("forced TypeScript JSON");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["language"], "typescript");
    assert_eq!(json["status"], "complete");
    assert!(json["effective_mode"]
        .as_str()
        .unwrap_or_default()
        .contains("typescript"));
}

#[cfg(feature = "lang-js")]
#[test]
fn parse_json_omits_optional_ast_but_reports_status_and_mode() {
    let path = std::env::temp_dir().join(format!("ripex-cli-status-{}.js", std::process::id()));
    fs::write(&path, "const answer = 42;").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args(["parse", path.to_str().unwrap(), "--json"])
        .output()
        .expect("run ripex CLI");
    let _ = fs::remove_file(&path);

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("status JSON");
    assert_eq!(json["schema_version"], 2);
    assert_eq!(json["status"], "complete");
    assert_eq!(json["completeness"], "complete");
    assert!(json.get("ast").is_none());
    assert!(json["effective_mode"].as_str().is_some());
    assert_eq!(json["trust"]["trusted_project"], false);
}

#[cfg(feature = "lang-js")]
#[test]
fn over_limit_input_is_rejected_before_parse() {
    let path = std::env::temp_dir().join(format!("ripex-cli-too-large-{}.js", std::process::id()));
    let data = vec![b'a'; ripex::limits::MAX_INPUT_SIZE + 1];
    fs::write(&path, data).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ripex"))
        .args(["parse", path.to_str().unwrap(), "--json"])
        .output()
        .expect("run ripex CLI");
    let _ = fs::remove_file(&path);

    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("input-limit JSON");
    assert_eq!(json["error"]["code"], "input_too_large");
}
