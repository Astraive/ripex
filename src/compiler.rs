//! Compiler-backed validation for source files and projects.
//!
//! ripex's parsers are structural. This module delegates language conformance
//! and type checking to production toolchains instead of pretending the
//! structural AST is a compiler semantic model.

use crate::{detect_language, Language};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CompilerCheckOptions {
    /// Override the primary compiler/static analyzer executable.
    pub toolchain: Option<PathBuf>,
    /// Language standard or target (for example `c23`, `c++23`, or `ES2022`).
    pub standard: Option<String>,
    /// Treat a file as part of its nearest project when a project manifest exists.
    pub project: bool,
    /// Arguments appended verbatim after ripex's correctness flags.
    pub extra_args: Vec<String>,
    /// Maximum runtime for each toolchain stage.
    pub timeout: Duration,
}

impl Default for CompilerCheckOptions {
    fn default() -> Self {
        Self {
            toolchain: None,
            standard: None,
            project: false,
            extra_args: Vec::new(),
            timeout: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CheckStatus {
    Passed,
    Failed,
    Unavailable,
    InvocationError,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum CheckStageKind {
    Syntax,
    TypeCheck,
    Compile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Note,
    Help,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompilerDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: Option<String>,
    pub message: String,
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub raw: String,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompilerStageResult {
    pub kind: CheckStageKind,
    pub backend: String,
    pub status: CheckStatus,
    pub command: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub diagnostics: Vec<CompilerDiagnostic>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompilerCheckReport {
    pub language: Language,
    pub path: PathBuf,
    pub status: CheckStatus,
    pub stages: Vec<CompilerStageResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerInvocation {
    pub kind: CheckStageKind,
    pub backend: String,
    pub candidates: Vec<Vec<String>>,
    pub args: Vec<String>,
    pub current_dir: PathBuf,
    pub cleanup: Vec<PathBuf>,
}

impl CompilerCheckReport {
    pub fn passed(&self) -> bool {
        self.status == CheckStatus::Passed
    }
}

/// Plan the exact external compiler invocations without executing them.
pub fn plan_compiler_check(
    path: impl AsRef<Path>,
    language: Option<Language>,
    options: &CompilerCheckOptions,
) -> io::Result<Vec<CompilerInvocation>> {
    let path = absolute_path(path.as_ref())?;
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("check target does not exist: {}", path.display()),
        ));
    }
    let language = language
        .or_else(|| detect_language(&path))
        .or_else(|| detect_project_language(&path))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "cannot detect language"))?;
    let cwd = if path.is_dir() {
        path.clone()
    } else {
        path.parent().unwrap_or(Path::new(".")).to_path_buf()
    };
    let target = path.to_string_lossy().into_owned();
    let override_candidate = options
        .toolchain
        .as_ref()
        .map(|value| vec![vec![value.to_string_lossy().into_owned()]]);
    let mut plans = Vec::new();

    match language {
        Language::C => {
            if let Some(project_plans) = native_compilation_database_plans(
                &path,
                Language::C,
                options,
                override_candidate.clone(),
            )? {
                plans.extend(project_plans);
            } else {
                let standard = options.standard.as_deref().unwrap_or("c17");
                for source in native_source_files(&path, &["c"], &["h"])? {
                    let mut args = vec![
                        format!("-std={standard}"),
                        "-pedantic-errors".into(),
                        "-fsyntax-only".into(),
                        source.to_string_lossy().into_owned(),
                    ];
                    args.extend(options.extra_args.clone());
                    plans.push(invocation(
                        CheckStageKind::Compile,
                        "c-compiler",
                        override_candidate
                            .clone()
                            .unwrap_or_else(|| candidates(&["gcc", "clang"])),
                        args,
                        cwd.clone(),
                    ));
                }
            }
        }
        Language::Cpp => {
            if let Some(project_plans) = native_compilation_database_plans(
                &path,
                Language::Cpp,
                options,
                override_candidate.clone(),
            )? {
                plans.extend(project_plans);
            } else {
                let standard = options.standard.as_deref().unwrap_or("c++20");
                for source in
                    native_source_files(&path, &["cpp", "cc", "cxx"], &["hpp", "hh", "hxx"])?
                {
                    let mut args = vec![
                        format!("-std={standard}"),
                        "-pedantic-errors".into(),
                        "-fsyntax-only".into(),
                        source.to_string_lossy().into_owned(),
                    ];
                    args.extend(options.extra_args.clone());
                    plans.push(invocation(
                        CheckStageKind::Compile,
                        "c++-compiler",
                        override_candidate
                            .clone()
                            .unwrap_or_else(|| candidates(&["g++", "clang++"])),
                        args,
                        cwd.clone(),
                    ));
                }
            }
        }
        Language::Rust => {
            if let Some(manifest) = project_file(&path, "Cargo.toml", options.project) {
                let target_dir = temporary_artifact("cargo-target");
                let mut args = vec![
                    "check".into(),
                    "--manifest-path".into(),
                    manifest.to_string_lossy().into_owned(),
                    "--message-format=short".into(),
                    "--target-dir".into(),
                    target_dir.to_string_lossy().into_owned(),
                ];
                args.extend(options.extra_args.clone());
                let mut plan = invocation(
                    CheckStageKind::TypeCheck,
                    "cargo",
                    override_candidate.unwrap_or_else(|| candidates(&["cargo"])),
                    args,
                    cwd,
                );
                plan.cleanup.push(target_dir);
                plans.push(plan);
            } else {
                if path.is_dir() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "a Rust directory check requires Cargo.toml",
                    ));
                }
                let edition = options.standard.as_deref().unwrap_or("2021");
                let output = temporary_artifact("rmeta");
                let mut args = vec![
                    "--crate-type=lib".into(),
                    format!("--edition={edition}"),
                    "--emit=metadata".into(),
                    "--error-format=short".into(),
                    "-o".into(),
                    output.to_string_lossy().into_owned(),
                    target,
                ];
                args.extend(options.extra_args.clone());
                let mut plan = invocation(
                    CheckStageKind::TypeCheck,
                    "rustc",
                    override_candidate.unwrap_or_else(|| candidates(&["rustc"])),
                    args,
                    cwd,
                );
                plan.cleanup.push(output);
                plans.push(plan);
            }
        }
        Language::Go => {
            let project_dir = project_directory(&path, "go.mod", options.project);
            let is_project = project_dir.is_some();
            let output = temporary_artifact(if cfg!(windows) { "exe" } else { "bin" });
            let (current_dir, mut args) = if let Some(dir) = project_dir {
                (dir, vec!["build".into(), "./...".into()])
            } else if path.is_dir() {
                (
                    path.clone(),
                    vec![
                        "build".into(),
                        "-o".into(),
                        output.to_string_lossy().into_owned(),
                        ".".into(),
                    ],
                )
            } else {
                (
                    cwd,
                    vec![
                        "build".into(),
                        "-o".into(),
                        output.to_string_lossy().into_owned(),
                        target,
                    ],
                )
            };
            args.extend(options.extra_args.clone());
            let mut plan = invocation(
                CheckStageKind::TypeCheck,
                "go",
                override_candidate.unwrap_or_else(|| candidates(&["go"])),
                args,
                current_dir,
            );
            if !is_project {
                plan.cleanup.push(output);
            }
            plans.push(plan);
        }
        Language::CSharp => {
            if let Some(project) = csharp_project(&path, options.project) {
                let output_dir = temporary_artifact("dotnet-output");
                let intermediate_dir = temporary_artifact("dotnet-intermediate");
                let mut args = vec![
                    "build".into(),
                    project.to_string_lossy().into_owned(),
                    "--nologo".into(),
                    format!("--property:BaseOutputPath={}", output_dir.display()),
                    format!(
                        "--property:BaseIntermediateOutputPath={}",
                        intermediate_dir.display()
                    ),
                ];
                if let Some(standard) = &options.standard {
                    args.push(format!("--property:LangVersion={standard}"));
                }
                args.extend(options.extra_args.clone());
                let mut plan = invocation(
                    CheckStageKind::TypeCheck,
                    "dotnet",
                    override_candidate.unwrap_or_else(|| candidates(&["dotnet"])),
                    args,
                    cwd,
                );
                plan.cleanup.extend([output_dir, intermediate_dir]);
                plans.push(plan);
            } else {
                let output = temporary_artifact("dll");
                let mut args = vec![
                    "/nologo".into(),
                    "/target:library".into(),
                    format!("/out:{}", output.display()),
                ];
                if let Some(standard) = &options.standard {
                    args.push(format!("/langversion:{standard}"));
                }
                args.extend(
                    source_files(&path, &["cs"])?
                        .iter()
                        .map(|file| file.to_string_lossy().into_owned()),
                );
                args.extend(options.extra_args.clone());
                let mut plan = invocation(
                    CheckStageKind::TypeCheck,
                    "csc",
                    override_candidate.unwrap_or_else(|| candidates(&["csc"])),
                    args,
                    cwd,
                );
                plan.cleanup.push(output);
                plans.push(plan);
            }
        }
        Language::TypeScript => {
            let mut args = vec!["--noEmit".into(), "--pretty".into(), "false".into()];
            if let Some(config) = project_file(&path, "tsconfig.json", options.project) {
                args.extend(["--project".into(), config.to_string_lossy().into_owned()]);
            } else {
                args.push("--strict".into());
                if matches!(path.extension().and_then(|v| v.to_str()), Some("tsx")) {
                    args.extend(["--jsx".into(), "react-jsx".into()]);
                }
                if let Some(target_standard) = &options.standard {
                    args.extend(["--target".into(), target_standard.clone()]);
                }
                args.extend(
                    source_files(&path, &["ts", "tsx", "mts", "cts"])?
                        .iter()
                        .map(|file| file.to_string_lossy().into_owned()),
                );
            }
            args.extend(options.extra_args.clone());
            let default = vec![vec!["tsc".into()], npx_typescript_candidate()];
            plans.push(invocation(
                CheckStageKind::TypeCheck,
                "typescript",
                override_candidate.unwrap_or(default),
                args,
                cwd,
            ));
        }
        Language::JavaScript => {
            let files = javascript_files(&path)?;
            for file in files
                .iter()
                .filter(|file| !matches!(file.extension().and_then(|v| v.to_str()), Some("jsx")))
            {
                plans.push(invocation(
                    CheckStageKind::Syntax,
                    "node",
                    candidates(&["node"]),
                    vec!["--check".into(), file.to_string_lossy().into_owned()],
                    cwd.clone(),
                ));
            }
            let mut args = vec!["--noEmit".into(), "--pretty".into(), "false".into()];
            if let Some(config) = project_file(&path, "jsconfig.json", options.project) {
                args.extend(["--project".into(), config.to_string_lossy().into_owned()]);
            } else {
                args.extend([
                    "--allowJs".into(),
                    "--checkJs".into(),
                    "--strict".into(),
                    "--jsx".into(),
                    "preserve".into(),
                ]);
                if let Some(target_standard) = &options.standard {
                    args.extend(["--target".into(), target_standard.clone()]);
                }
                args.extend(files.iter().map(|file| file.to_string_lossy().into_owned()));
            }
            args.extend(options.extra_args.clone());
            plans.push(invocation(
                CheckStageKind::TypeCheck,
                "typescript-checkjs",
                override_candidate
                    .unwrap_or_else(|| vec![vec!["tsc".into()], npx_typescript_candidate()]),
                args,
                cwd,
            ));
        }
        Language::Python => {
            for source in source_files(&path, &["py", "pyi"])? {
                plans.push(invocation(
                    CheckStageKind::Syntax,
                    "python",
                    python_candidates(),
                    vec![
                        "-c".into(),
                        "import pathlib,sys; p=pathlib.Path(sys.argv[1]); compile(p.read_bytes(), str(p), 'exec')".into(),
                        source.to_string_lossy().into_owned(),
                    ],
                    cwd.clone(),
                ));
            }
            let mut args = vec!["--strict".into(), target];
            args.extend(options.extra_args.clone());
            plans.push(invocation(
                CheckStageKind::TypeCheck,
                "mypy",
                override_candidate.unwrap_or_else(|| {
                    vec![
                        vec!["mypy".into()],
                        vec!["python3".into(), "-m".into(), "mypy".into()],
                        vec!["python".into(), "-m".into(), "mypy".into()],
                    ]
                }),
                args,
                cwd,
            ));
        }
    }
    Ok(plans)
}

/// Run authoritative compiler/static-analysis checks for a file or project.
pub fn check_with_compiler(
    path: impl AsRef<Path>,
    language: Option<Language>,
    options: &CompilerCheckOptions,
) -> io::Result<CompilerCheckReport> {
    let path = absolute_path(path.as_ref())?;
    let language = language
        .or_else(|| detect_language(&path))
        .or_else(|| detect_project_language(&path))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "cannot detect language"))?;
    let plans = plan_compiler_check(&path, Some(language), options)?;
    let stages = plans
        .iter()
        .map(|plan| execute_invocation(plan, options.timeout))
        .collect::<Vec<_>>();
    let status = aggregate_status(&stages);
    Ok(CompilerCheckReport {
        language,
        path,
        status,
        stages,
    })
}

fn invocation(
    kind: CheckStageKind,
    backend: &str,
    candidates: Vec<Vec<String>>,
    args: Vec<String>,
    current_dir: PathBuf,
) -> CompilerInvocation {
    CompilerInvocation {
        kind,
        backend: backend.into(),
        candidates,
        args,
        current_dir,
        cleanup: Vec::new(),
    }
}

fn candidates(names: &[&str]) -> Vec<Vec<String>> {
    names.iter().map(|name| vec![(*name).into()]).collect()
}

#[cfg(windows)]
fn python_candidates() -> Vec<Vec<String>> {
    candidates(&["python", "python3"])
}

#[cfg(windows)]
fn npx_typescript_candidate() -> Vec<String> {
    vec!["npx.cmd".into(), "--no-install".into(), "tsc".into()]
}

#[cfg(not(windows))]
fn npx_typescript_candidate() -> Vec<String> {
    vec!["npx".into(), "--no-install".into(), "tsc".into()]
}

#[cfg(not(windows))]
fn python_candidates() -> Vec<Vec<String>> {
    candidates(&["python3", "python"])
}

fn execute_invocation(plan: &CompilerInvocation, timeout: Duration) -> CompilerStageResult {
    let mut last_not_found = None;
    for candidate in &plan.candidates {
        let Some((executable, prefix)) = candidate.split_first() else {
            continue;
        };
        let command = candidate
            .iter()
            .chain(plan.args.iter())
            .cloned()
            .collect::<Vec<_>>();
        let mut process = Command::new(executable);
        process
            .args(prefix)
            .args(&plan.args)
            .current_dir(&plan.current_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        match run_process(process, timeout) {
            Ok(capture) => {
                let stdout = String::from_utf8_lossy(&capture.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&capture.stderr).into_owned();
                if capture.status.is_some_and(|status| !status.success())
                    && output_indicates_unavailable(&stdout, &stderr)
                {
                    last_not_found = Some(
                        stdout
                            .lines()
                            .chain(stderr.lines())
                            .find(|line| !line.trim().is_empty())
                            .unwrap_or("toolchain is unavailable")
                            .to_string(),
                    );
                    cleanup_artifacts(plan);
                    continue;
                }
                let status = if capture.timed_out {
                    CheckStatus::TimedOut
                } else if capture.status.is_some_and(|status| status.success()) {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                };
                let diagnostics = parse_diagnostics(&stdout, &stderr);
                cleanup_artifacts(plan);
                return CompilerStageResult {
                    kind: plan.kind,
                    backend: plan.backend.clone(),
                    status,
                    command,
                    exit_code: capture.status.and_then(|status| status.code()),
                    stdout,
                    stderr,
                    diagnostics,
                };
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                last_not_found = Some(error.to_string());
            }
            Err(error) => {
                cleanup_artifacts(plan);
                return CompilerStageResult {
                    kind: plan.kind,
                    backend: plan.backend.clone(),
                    status: CheckStatus::InvocationError,
                    command,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: error.to_string(),
                    diagnostics: Vec::new(),
                };
            }
        }
    }
    cleanup_artifacts(plan);
    CompilerStageResult {
        kind: plan.kind,
        backend: plan.backend.clone(),
        status: CheckStatus::Unavailable,
        command: plan.candidates.first().cloned().unwrap_or_default(),
        exit_code: None,
        stdout: String::new(),
        stderr: last_not_found.unwrap_or_else(|| "no toolchain candidate configured".into()),
        diagnostics: Vec::new(),
    }
}

fn output_indicates_unavailable(stdout: &str, stderr: &str) -> bool {
    let output = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    output.contains("this is not the tsc command you are looking for")
        || output.contains("could not determine executable to run")
        || output.contains("python was not found")
}

struct ProcessCapture {
    status: Option<ExitStatus>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    timed_out: bool,
}

fn run_process(mut command: Command, timeout: Duration) -> io::Result<ProcessCapture> {
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let stdout_reader = thread::spawn(move || read_stream(stdout));
    let stderr_reader = thread::spawn(move || read_stream(stderr));
    let started = Instant::now();
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (Some(status), false);
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            break (Some(child.wait()?), true);
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| io::Error::other("stdout reader thread panicked"))??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| io::Error::other("stderr reader thread panicked"))??;
    Ok(ProcessCapture {
        status,
        stdout,
        stderr,
        timed_out,
    })
}

fn read_stream(mut stream: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    stream.read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn aggregate_status(stages: &[CompilerStageResult]) -> CheckStatus {
    for status in [
        CheckStatus::TimedOut,
        CheckStatus::InvocationError,
        CheckStatus::Failed,
        CheckStatus::Unavailable,
    ] {
        if stages.iter().any(|stage| stage.status == status) {
            return status;
        }
    }
    CheckStatus::Passed
}

fn parse_diagnostics(stdout: &str, stderr: &str) -> Vec<CompilerDiagnostic> {
    stdout
        .lines()
        .chain(stderr.lines())
        .filter_map(parse_diagnostic_line)
        .collect()
}

fn parse_diagnostic_line(line: &str) -> Option<CompilerDiagnostic> {
    let (severity, marker, offset) = [
        (DiagnosticSeverity::Error, "error", 0),
        (DiagnosticSeverity::Warning, "warning", 0),
        (DiagnosticSeverity::Note, "note", 0),
        (DiagnosticSeverity::Help, "help", 0),
    ]
    .into_iter()
    .find_map(|(severity, word, offset)| {
        line.to_ascii_lowercase()
            .find(&format!("{word}:"))
            .map(|position| (severity, word, position + offset))
            .or_else(|| {
                line.to_ascii_lowercase()
                    .find(&format!("{word} "))
                    .map(|position| (severity, word, position + offset))
            })
    })?;
    let prefix = line[..offset].trim_end_matches([' ', ':']);
    let message_start = offset + marker.len();
    let mut message = line[message_start..]
        .trim_start_matches([' ', ':'])
        .to_string();
    let code = message
        .split_whitespace()
        .next()
        .filter(|word| word.starts_with("TS") || word.starts_with("CS") || word.starts_with('E'))
        .map(|word| word.trim_end_matches(':').to_string());
    if let Some(code) = &code {
        message = message
            .strip_prefix(code)
            .unwrap_or(&message)
            .trim_start_matches([' ', ':'])
            .to_string();
    }
    let (file, line_number, column) = parse_location(prefix);
    Some(CompilerDiagnostic {
        severity,
        code,
        message,
        file,
        line: line_number,
        column,
        raw: line.to_string(),
    })
}

fn parse_location(prefix: &str) -> (Option<String>, Option<u32>, Option<u32>) {
    if let Some(open) = prefix.rfind('(') {
        if let Some(location) = prefix.strip_suffix(')') {
            let numbers = &location[open + 1..];
            let mut parts = numbers.split(',');
            if let Some(line) = parts.next().and_then(|value| value.parse().ok()) {
                return (
                    Some(prefix[..open].to_string()),
                    Some(line),
                    parts.next().and_then(|value| value.parse().ok()),
                );
            }
        }
    }
    let mut parts = prefix.rsplitn(3, ':');
    let last = parts.next();
    let second = parts.next();
    let rest = parts.next();
    match (
        last.and_then(|value| value.parse::<u32>().ok()),
        second.and_then(|value| value.parse::<u32>().ok()),
        rest,
    ) {
        (Some(column), Some(line), Some(file)) => {
            (Some(file.to_string()), Some(line), Some(column))
        }
        (Some(line), _, _) => {
            let file = prefix.rsplit_once(':').map(|(file, _)| file.to_string());
            (file, Some(line), None)
        }
        _ => (None, None, None),
    }
}

/// Prefer a compilation database for native projects. It is the only portable
/// way to retain the include paths, defines, targets, and generated headers a
/// real C/C++ build uses, so each entry is converted into a no-output semantic
/// check rather than reconstructed from a directory walk.
fn native_compilation_database_plans(
    path: &Path,
    language: Language,
    options: &CompilerCheckOptions,
    override_candidate: Option<Vec<Vec<String>>>,
) -> io::Result<Option<Vec<CompilerInvocation>>> {
    let Some(database) = compilation_database(path, options.project) else {
        return Ok(None);
    };
    let contents = fs::read_to_string(&database)?;
    let entries: serde_json::Value = serde_json::from_str(&contents).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "invalid compilation database {}: {error}",
                database.display()
            ),
        )
    })?;
    let entries = entries.as_array().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "compilation database {} must contain an array",
                database.display()
            ),
        )
    })?;
    let database_dir = database.parent().unwrap_or(Path::new("."));
    let target = if path.file_name().and_then(|name| name.to_str()) == Some("compile_commands.json")
    {
        database_dir
    } else {
        path
    };
    let target = normalized_path(target);
    let mut plans = Vec::new();

    for (index, entry) in entries.iter().enumerate() {
        let entry = entry.as_object().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("compilation database entry {index} is not an object"),
            )
        })?;
        let directory = entry
            .get("directory")
            .and_then(serde_json::Value::as_str)
            .map(|directory| resolve_database_path(database_dir, directory))
            .unwrap_or_else(|| database_dir.to_path_buf());
        let file = entry
            .get("file")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("compilation database entry {index} has no string file"),
                )
            })?;
        let source = resolve_database_path(&directory, file);
        if !native_source_matches_language(&source, language)
            || !source_is_selected(&source, &target)
        {
            continue;
        }
        let command = compilation_database_command(entry, index)?;
        let (configured_candidate, configured_args) = split_compiler_command(command);
        if configured_candidate.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("compilation database entry {index} has an empty command"),
            ));
        }
        let is_msvc = configured_candidate
            .iter()
            .any(|part| is_msvc_compiler(part));
        let mut args = sanitize_native_compiler_args(configured_args, is_msvc);
        args.push(if is_msvc {
            "/Zs".into()
        } else {
            "-fsyntax-only".into()
        });
        if let Some(standard) = &options.standard {
            args.push(native_standard_flag(language, standard, is_msvc)?);
        }
        args.extend(options.extra_args.clone());
        plans.push(invocation(
            CheckStageKind::Compile,
            if language == Language::C {
                "c-compile-commands"
            } else {
                "c++-compile-commands"
            },
            override_candidate
                .clone()
                .unwrap_or_else(|| vec![configured_candidate]),
            args,
            directory,
        ));
    }

    if plans.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "compilation database {} has no {} entry for {}",
                database.display(),
                language.id(),
                path.display()
            ),
        ));
    }
    Ok(Some(plans))
}

fn compilation_database(path: &Path, enabled: bool) -> Option<PathBuf> {
    if path.file_name().and_then(|name| name.to_str()) == Some("compile_commands.json") {
        return Some(path.to_path_buf());
    }
    if !enabled && !path.is_dir() {
        return None;
    }
    let start = if path.is_dir() { path } else { path.parent()? };
    start
        .ancestors()
        .map(|directory| directory.join("compile_commands.json"))
        .find(|candidate| candidate.is_file())
}

fn compilation_database_command(
    entry: &serde_json::Map<String, serde_json::Value>,
    index: usize,
) -> io::Result<Vec<String>> {
    if let Some(arguments) = entry.get("arguments") {
        let arguments = arguments.as_array().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("compilation database entry {index} has non-array arguments"),
            )
        })?;
        return arguments
            .iter()
            .map(|argument| {
                argument.as_str().map(str::to_owned).ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "compilation database entry {index} has a non-string command argument"
                        ),
                    )
                })
            })
            .collect();
    }
    let command = entry
        .get("command")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("compilation database entry {index} has no command or arguments"),
            )
        })?;
    split_command_line(command).map_err(|message| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid command in compilation database entry {index}: {message}"),
        )
    })
}

fn split_compiler_command(command: Vec<String>) -> (Vec<String>, Vec<String>) {
    let compiler = command
        .iter()
        .position(|part| is_known_compiler(part))
        .unwrap_or(0);
    (
        command[..=compiler].to_vec(),
        command[compiler + 1..].to_vec(),
    )
}

fn is_known_compiler(part: &str) -> bool {
    let name = Path::new(part)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(part)
        .trim_end_matches(".exe")
        .to_ascii_lowercase();
    matches!(
        name.as_str(),
        "cc" | "c++" | "gcc" | "g++" | "clang" | "clang++" | "clang-cl" | "cl"
    ) || name.starts_with("clang-")
        || name.starts_with("gcc-")
        || name.starts_with("g++-")
}

fn is_msvc_compiler(part: &str) -> bool {
    Path::new(part)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(part)
        .trim_end_matches(".exe")
        .eq_ignore_ascii_case("cl")
        || Path::new(part)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(part)
            .trim_end_matches(".exe")
            .eq_ignore_ascii_case("clang-cl")
}

fn sanitize_native_compiler_args(arguments: Vec<String>, msvc: bool) -> Vec<String> {
    let mut sanitized = Vec::with_capacity(arguments.len());
    let mut skip_next = false;
    for argument in arguments {
        if skip_next {
            skip_next = false;
            continue;
        }
        if (!msvc && matches!(argument.as_str(), "-c" | "-o" | "-MF" | "-MT" | "-MQ"))
            || (msvc && matches!(argument.as_str(), "/c" | "/Fo" | "/Fd" | "/Fe" | "/Fi"))
        {
            skip_next = matches!(
                argument.as_str(),
                "-o" | "-MF" | "-MT" | "-MQ" | "/Fo" | "/Fd" | "/Fe" | "/Fi"
            );
            continue;
        }
        if (!msvc
            && (matches!(argument.as_str(), "-MD" | "-MMD" | "-MP" | "-fsyntax-only")
                || (argument.starts_with("-o") && argument.len() > 2)))
            || (msvc
                && (argument.eq_ignore_ascii_case("/Zs")
                    || ["/Fo", "/Fd", "/Fe", "/Fi"]
                        .iter()
                        .any(|flag| argument.starts_with(flag) && argument.len() > flag.len())))
        {
            continue;
        }
        sanitized.push(argument);
    }
    sanitized
}

fn native_standard_flag(language: Language, standard: &str, msvc: bool) -> io::Result<String> {
    if !msvc {
        return Ok(format!("-std={standard}"));
    }
    let standard = standard.to_ascii_lowercase();
    match (language, standard.as_str()) {
        (Language::C, "c11" | "c17") => Ok(format!("/std:{standard}")),
        (Language::C, "c23") => Ok("/std:clatest".into()),
        (Language::Cpp, "c++14" | "c++17" | "c++20") => Ok(format!("/std:{standard}")),
        (Language::Cpp, "c++23") => Ok("/std:c++latest".into()),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "MSVC cannot select requested {} standard: {standard}",
                language.id()
            ),
        )),
    }
}

fn split_command_line(command: &str) -> Result<Vec<String>, &'static str> {
    let mut arguments = Vec::new();
    let mut argument = String::new();
    let mut quote = None;
    let mut started = false;
    let mut characters = command.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\\' && quote != Some('\'') {
            let next = characters.peek().copied();
            if matches!(next, Some(next) if next.is_whitespace() || matches!(next, '\'' | '"')) {
                argument.push(characters.next().expect("peeked character exists"));
                started = true;
                continue;
            }
            argument.push(character);
            started = true;
            continue;
        }
        if matches!(character, '\'' | '"') {
            if quote == Some(character) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(character);
            } else {
                argument.push(character);
            }
            started = true;
            continue;
        }
        if character.is_whitespace() && quote.is_none() {
            if started {
                arguments.push(std::mem::take(&mut argument));
                started = false;
            }
            continue;
        }
        argument.push(character);
        started = true;
    }
    if quote.is_some() {
        return Err("unterminated quote");
    }
    if started {
        arguments.push(argument);
    }
    Ok(arguments)
}

fn native_source_files(path: &Path, units: &[&str], headers: &[&str]) -> io::Result<Vec<PathBuf>> {
    if path.is_file() {
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase);
        if extension
            .as_deref()
            .is_none_or(|extension| !units.contains(&extension) && !headers.contains(&extension))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "check target is not a supported native source or header file",
            ));
        }
        return Ok(vec![path.to_path_buf()]);
    }
    source_files(path, units)
}

fn native_source_matches_language(path: &Path, language: Language) -> bool {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    match language {
        Language::C => extension.as_deref() == Some("c"),
        Language::Cpp => matches!(extension.as_deref(), Some("cpp" | "cc" | "cxx")),
        _ => false,
    }
}

fn resolve_database_path(directory: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        directory.join(path)
    };
    normalized_path(&resolved)
}

fn normalized_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn source_is_selected(source: &Path, target: &Path) -> bool {
    let source = normalized_path(source);
    if target.is_dir() {
        source.starts_with(target)
    } else {
        source == target
    }
}

fn javascript_files(path: &Path) -> io::Result<Vec<PathBuf>> {
    source_files(path, &["js", "jsx", "mjs", "cjs"])
}

fn source_files(path: &Path, extensions: &[&str]) -> io::Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut files = Vec::new();
    collect_files(path, extensions, &mut files)?;
    if files.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "check target contains no matching source files",
        ));
    }
    files.sort();
    Ok(files)
}

fn collect_files(path: &Path, extensions: &[&str], output: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            let name = child.file_name().and_then(|value| value.to_str());
            if !matches!(name, Some("node_modules" | "target" | ".git")) {
                collect_files(&child, extensions, output)?;
            }
        } else if child
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extensions.contains(&extension))
        {
            output.push(child);
        }
    }
    Ok(())
}

fn project_file(path: &Path, name: &str, enabled: bool) -> Option<PathBuf> {
    if path.file_name().and_then(|value| value.to_str()) == Some(name) {
        return Some(path.to_path_buf());
    }
    if !enabled && !path.is_dir() {
        return None;
    }
    let start = if path.is_dir() { path } else { path.parent()? };
    start
        .ancestors()
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
}

fn project_directory(path: &Path, manifest: &str, enabled: bool) -> Option<PathBuf> {
    project_file(path, manifest, enabled).and_then(|file| file.parent().map(Path::to_path_buf))
}

fn csharp_project(path: &Path, enabled: bool) -> Option<PathBuf> {
    if matches!(
        path.extension().and_then(|value| value.to_str()),
        Some("csproj" | "sln")
    ) {
        return Some(path.to_path_buf());
    }
    if !enabled && !path.is_dir() {
        return None;
    }
    let start = if path.is_dir() { path } else { path.parent()? };
    for directory in start.ancestors() {
        if let Ok(entries) = fs::read_dir(directory) {
            if let Some(project) = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|candidate| {
                    matches!(
                        candidate.extension().and_then(|value| value.to_str()),
                        Some("csproj" | "sln")
                    )
                })
            {
                return Some(project);
            }
        }
    }
    None
}

fn detect_project_language(path: &Path) -> Option<Language> {
    if path.is_dir() {
        if path.join("Cargo.toml").is_file() {
            return Some(Language::Rust);
        }
        if path.join("go.mod").is_file() {
            return Some(Language::Go);
        }
        if path.join("tsconfig.json").is_file() {
            return Some(Language::TypeScript);
        }
        if fs::read_dir(path)
            .ok()?
            .filter_map(Result::ok)
            .any(|entry| {
                matches!(
                    entry.path().extension().and_then(|value| value.to_str()),
                    Some("csproj" | "sln")
                )
            })
        {
            return Some(Language::CSharp);
        }
        return None;
    }
    let name = path.file_name()?.to_str()?;
    match name {
        "Cargo.toml" => Some(Language::Rust),
        "go.mod" => Some(Language::Go),
        "tsconfig.json" => Some(Language::TypeScript),
        _ if matches!(path.extension()?.to_str()?, "csproj" | "sln") => Some(Language::CSharp),
        _ => None,
    }
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn cleanup_artifacts(plan: &CompilerInvocation) {
    for path in &plan.cleanup {
        if path.is_dir() {
            let _ = fs::remove_dir_all(path);
        } else {
            let _ = fs::remove_file(path);
        }
    }
}

fn temporary_artifact(extension: &str) -> PathBuf {
    static NEXT_ARTIFACT: AtomicU64 = AtomicU64::new(0);
    let sequence = NEXT_ARTIFACT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "ripex-compiler-{}-{sequence}.{extension}",
        std::process::id()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_strict_c_check() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test/c/main.c");
        let plans = plan_compiler_check(path, Some(Language::C), &Default::default()).unwrap();
        assert_eq!(plans.len(), 1);
        assert!(plans[0].args.contains(&"-std=c17".to_string()));
        assert!(plans[0].args.contains(&"-pedantic-errors".to_string()));
        assert!(plans[0].args.contains(&"-fsyntax-only".to_string()));
    }

    #[test]
    fn compile_commands_preserve_project_specific_compiler_arguments() {
        let root = temporary_artifact("compile-commands-test");
        fs::create_dir_all(&root).unwrap();
        let source = root.join("main.c");
        fs::write(&source, "int main(void) { return 0; }").unwrap();
        let database = serde_json::json!([{
            "directory": root,
            "file": "main.c",
            "arguments": [
                "clang",
                "-Igenerated/include",
                "-DPROJECT_SETTING=1",
                "-c",
                "main.c",
                "-o",
                "main.o",
                "-MMD",
                "-MF",
                "main.d"
            ]
        }]);
        fs::write(
            root.join("compile_commands.json"),
            serde_json::to_string(&database).unwrap(),
        )
        .unwrap();

        let plans = plan_compiler_check(&root, Some(Language::C), &Default::default()).unwrap();
        let source_plans = plan_compiler_check(
            &source,
            Some(Language::C),
            &CompilerCheckOptions {
                project: true,
                ..Default::default()
            },
        )
        .unwrap();
        let _ = fs::remove_dir_all(&root);

        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].backend, "c-compile-commands");
        assert_eq!(source_plans.len(), 1);
        assert_eq!(source_plans[0].backend, "c-compile-commands");
        assert_eq!(plans[0].candidates, vec![vec![String::from("clang")]]);
        assert!(plans[0].args.contains(&"-Igenerated/include".to_string()));
        assert!(plans[0].args.contains(&"-DPROJECT_SETTING=1".to_string()));
        assert!(plans[0].args.contains(&"-fsyntax-only".to_string()));
        assert!(!plans[0].args.contains(&"-c".to_string()));
        assert!(!plans[0].args.contains(&"main.o".to_string()));
        assert!(!plans[0].args.contains(&"main.d".to_string()));
    }

    #[test]
    fn command_strings_support_quoted_compilation_database_arguments() {
        let command = split_command_line("clang -I\"generated include\" -c 'main file.c'").unwrap();
        assert_eq!(
            command,
            vec!["clang", "-Igenerated include", "-c", "main file.c"]
        );

        let command = split_command_line(
            "\"C:\\Program Files\\LLVM\\bin\\clang.exe\" /I\"C:\\generated files\" /c main.c",
        )
        .unwrap();
        assert_eq!(
            command,
            vec![
                "C:\\Program Files\\LLVM\\bin\\clang.exe",
                "/IC:\\generated files",
                "/c",
                "main.c"
            ]
        );
    }

    #[test]
    fn parses_gcc_and_csc_diagnostic_locations() {
        let gcc = parse_diagnostic_line("src/main.c:12:7: error: unknown type name 'X'").unwrap();
        assert_eq!(gcc.file.as_deref(), Some("src/main.c"));
        assert_eq!(gcc.line, Some(12));
        assert_eq!(gcc.column, Some(7));
        let csc = parse_diagnostic_line("Program.cs(4,2): error CS1002: ; expected").unwrap();
        assert_eq!(csc.code.as_deref(), Some("CS1002"));
        assert_eq!(csc.line, Some(4));
        assert_eq!(csc.column, Some(2));
    }

    #[test]
    fn recognizes_package_manager_toolchain_placeholders() {
        assert!(output_indicates_unavailable(
            "This is not the tsc command you are looking for",
            ""
        ));
    }
}
