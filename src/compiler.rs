//! Compiler-backed validation for source files and projects.
//!
//! ripex's parsers are structural. This module delegates language conformance
//! and type checking to production toolchains instead of pretending the
//! structural AST is a compiler semantic model.

use crate::{detect_language, Language};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

const DEFAULT_MAX_OUTPUT_BYTES: usize = 1024 * 1024;
const DEFAULT_MAX_TOTAL_OUTPUT_BYTES: usize = 2 * DEFAULT_MAX_OUTPUT_BYTES;
const MAX_DISCOVERY_DEPTH: usize = 64;
const MAX_DISCOVERY_FILES: usize = 10_000;
const MAX_DISCOVERY_PATH_BYTES: usize = 32 * 1024;
const MAX_COMPILATION_DATABASE_BYTES: usize = 16 * 1024 * 1024;
const MAX_DISCOVERY_TIME: Duration = Duration::from_secs(5);
const HARD_MAX_OUTPUT_BYTES: usize = 64 * 1024 * 1024;
const HARD_MAX_TOTAL_OUTPUT_BYTES: usize = 128 * 1024 * 1024;
const PROCESS_DRAIN_GRACE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone)]
pub struct CompilerCheckOptions {
    /// Override the primary compiler/static analyzer executable.
    pub toolchain: Option<PathBuf>,
    /// Language standard or target (for example `c23`, `c++23`, or `ES2022`).
    pub standard: Option<String>,
    /// Treat a file as part of its nearest project when a project manifest exists.
    pub project: bool,
    /// Permit execution of project/build metadata and compilation databases.
    pub trusted_project: bool,
    /// Arguments appended verbatim after ripex's correctness flags.
    ///
    /// Raw arguments are intentionally rejected unless both `trusted_project` and
    /// `allow_unsafe_args` are enabled.
    pub extra_args: Vec<String>,
    /// Explicitly permit raw arguments that may alter compiler side effects.
    pub allow_unsafe_args: bool,
    /// Maximum captured bytes from either compiler output stream.
    pub max_output_bytes: usize,
    /// Maximum captured bytes across stdout and stderr combined.
    pub max_total_output_bytes: usize,
    /// Maximum runtime for each toolchain stage.
    pub timeout: Duration,
}

impl Default for CompilerCheckOptions {
    fn default() -> Self {
        Self {
            toolchain: None,
            standard: None,
            project: false,
            trusted_project: false,
            extra_args: Vec::new(),
            allow_unsafe_args: false,
            max_output_bytes: DEFAULT_MAX_OUTPUT_BYTES,
            max_total_output_bytes: DEFAULT_MAX_TOTAL_OUTPUT_BYTES,
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
    OutputLimit,
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
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub output_truncated: bool,
    pub diagnostics: Vec<CompilerDiagnostic>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompilerCheckReport {
    pub language: Language,
    pub path: PathBuf,
    pub status: CheckStatus,
    /// Whether project/build metadata was explicitly trusted for this check.
    pub trusted_project: bool,
    pub output_truncated: bool,
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
    if !options.extra_args.is_empty()
        && (!options.trusted_project || !options.allow_unsafe_args)
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "raw compiler arguments require trusted_project and allow_unsafe_args",
        ));
    }
    if requires_trusted_execution(&path, options) && !options.trusted_project {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "project, build, and compilation-database checks require trusted_project",
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
                let target_dir = temporary_artifact("cargo-target")?;
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
                let output = temporary_artifact("rmeta")?;
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
            let output = temporary_artifact(if cfg!(windows) { "exe" } else { "bin" })?;
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
            if let Some(project) = csharp_project(&path, options.project)? {
                let output_dir = temporary_artifact("dotnet-output")?;
                let intermediate_dir = temporary_artifact("dotnet-intermediate")?;
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
                let output = temporary_artifact("dll")?;
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
        .map(|plan| {
            execute_invocation(
                plan,
                options.timeout,
                options.max_output_bytes,
                options.max_total_output_bytes,
            )
        })
        .collect::<Vec<_>>();
    let status = aggregate_status(&stages);
    let output_truncated = stages.iter().any(|stage| stage.output_truncated);
    Ok(CompilerCheckReport {
        language,
        path,
        status,
        trusted_project: options.trusted_project,
        output_truncated,
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

fn execute_invocation(
    plan: &CompilerInvocation,
    timeout: Duration,
    max_output_bytes: usize,
    max_total_output_bytes: usize,
) -> CompilerStageResult {
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
        match run_process(process, timeout, max_output_bytes, max_total_output_bytes) {
            Ok(capture) => {
                let stdout = String::from_utf8_lossy(&capture.stdout).into_owned();
                let stderr = String::from_utf8_lossy(&capture.stderr).into_owned();
                if !capture.output_limited
                    && capture.status.is_some_and(|status| !status.success())
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
                let status = if capture.output_limited {
                    CheckStatus::OutputLimit
                } else if capture.timed_out {
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
                    stdout_truncated: capture.stdout_truncated,
                    stderr_truncated: capture.stderr_truncated,
                    output_truncated: capture.output_limited,
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
                    stdout_truncated: false,
                    stderr_truncated: false,
                    output_truncated: false,
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
        stdout_truncated: false,
        stderr_truncated: false,
        output_truncated: false,
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
    stdout_truncated: bool,
    stderr_truncated: bool,
    timed_out: bool,
    output_limited: bool,
}

#[derive(Debug)]
struct StreamCapture {
    bytes: Vec<u8>,
    truncated: bool,
}

#[derive(Clone, Copy)]
enum StreamKind {
    Stdout,
    Stderr,
}

fn run_process(
    mut command: Command,
    timeout: Duration,
    max_output_bytes: usize,
    max_total_output_bytes: usize,
) -> io::Result<ProcessCapture> {
    let max_output_bytes = max_output_bytes.min(HARD_MAX_OUTPUT_BYTES);
    let max_total_output_bytes = max_total_output_bytes.min(HARD_MAX_TOTAL_OUTPUT_BYTES);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().expect("stdout was piped");
    let stderr = child.stderr.take().expect("stderr was piped");
    let (sender, receiver) = mpsc::channel();
    spawn_reader(stdout, max_output_bytes, StreamKind::Stdout, &sender);
    spawn_reader(stderr, max_output_bytes, StreamKind::Stderr, &sender);

    let started = Instant::now();
    let mut status = None;
    let mut timed_out = false;
    let mut output_limited = false;
    let mut stdout_capture = None;
    let mut stderr_capture = None;
    let mut reader_error = None;
    let mut stopping = false;
    let mut drain_deadline = None;

    loop {
        drain_reader_results(
            &receiver,
            &mut stdout_capture,
            &mut stderr_capture,
            &mut reader_error,
            &mut output_limited,
        );
        if status.is_none() {
            status = child.try_wait()?;
        }
        if !stopping {
            if output_limited {
                terminate_child(&mut child);
                stopping = true;
                drain_deadline = Some(Instant::now() + PROCESS_DRAIN_GRACE);
            } else if status.is_some() {
                stopping = true;
                drain_deadline = Some(Instant::now() + PROCESS_DRAIN_GRACE);
            } else if started.elapsed() >= timeout {
                terminate_child(&mut child);
                timed_out = true;
                stopping = true;
                drain_deadline = Some(Instant::now() + PROCESS_DRAIN_GRACE);
            }
        }
        if stopping {
            if status.is_none() {
                status = child.try_wait()?;
            }
            if status.is_some() && stdout_capture.is_some() && stderr_capture.is_some() {
                break;
            }
            if drain_deadline.is_some_and(|deadline| Instant::now() >= deadline) {
                break;
            }
        }
        thread::sleep(Duration::from_millis(5));
    }
    drain_reader_results(
        &receiver,
        &mut stdout_capture,
        &mut stderr_capture,
        &mut reader_error,
        &mut output_limited,
    );
    if let Some(error) = reader_error {
        return Err(error);
    }
    let mut stdout = stdout_capture.unwrap_or(StreamCapture {
        bytes: Vec::new(),
        truncated: true,
    });
    let mut stderr = stderr_capture.unwrap_or(StreamCapture {
        bytes: Vec::new(),
        truncated: true,
    });
    let total_cap = max_total_output_bytes;
    if stdout.bytes.len().saturating_add(stderr.bytes.len()) > total_cap {
        let stdout_len = stdout.bytes.len();
        let stdout_limit = stdout_len.min(total_cap);
        stdout.bytes.truncate(stdout_limit);
        let remaining = total_cap.saturating_sub(stdout.bytes.len());
        let stderr_len = stderr.bytes.len();
        stderr.bytes.truncate(remaining);
        stdout.truncated |= stdout.bytes.len() < stdout_len;
        stderr.truncated |= stderr.bytes.len() < stderr_len;
        output_limited = true;
    }
    output_limited |= stdout.truncated || stderr.truncated;
    Ok(ProcessCapture {
        status,
        stdout: stdout.bytes,
        stderr: stderr.bytes,
        stdout_truncated: stdout.truncated,
        stderr_truncated: stderr.truncated,
        timed_out,
        output_limited,
    })
}

fn spawn_reader(
    stream: impl Read + Send + 'static,
    cap: usize,
    kind: StreamKind,
    sender: &Sender<(StreamKind, io::Result<StreamCapture>)>,
) {
    let sender = sender.clone();
    thread::spawn(move || {
        let result = read_stream(stream, cap);
        let _ = sender.send((kind, result));
    });
}

fn drain_reader_results(
    receiver: &Receiver<(StreamKind, io::Result<StreamCapture>)>,
    stdout: &mut Option<StreamCapture>,
    stderr: &mut Option<StreamCapture>,
    reader_error: &mut Option<io::Error>,
    output_limited: &mut bool,
) {
    while let Ok((kind, result)) = receiver.try_recv() {
        match result {
            Ok(capture) => {
                *output_limited |= capture.truncated;
                match kind {
                    StreamKind::Stdout => *stdout = Some(capture),
                    StreamKind::Stderr => *stderr = Some(capture),
                }
            }
            Err(error) => *reader_error = Some(error),
        }
    }
}

fn terminate_child(child: &mut Child) {
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        unsafe {
            // The child is placed in its own process group before spawning.
            unsafe extern "C" {
                fn kill(pid: i32, signal: i32) -> i32;
            }
            let _ = kill(-pid, 9);
        }
    }
    let _ = child.kill();
}

fn read_stream(mut stream: impl Read, cap: usize) -> io::Result<StreamCapture> {
    let mut bytes = Vec::with_capacity(cap.min(8192));
    let mut buffer = [0_u8; 8192];
    while bytes.len() < cap {
        let chunk = (cap - bytes.len()).min(buffer.len());
        let read = stream.read(&mut buffer[..chunk])?;
        if read == 0 {
            return Ok(StreamCapture {
                bytes,
                truncated: false,
            });
        }
        bytes.extend_from_slice(&buffer[..read]);
    }
    let mut probe = [0_u8; 1];
    let read = stream.read(&mut probe)?;
    if read == 0 {
        Ok(StreamCapture {
            bytes,
            truncated: false,
        })
    } else {
        Ok(StreamCapture {
            bytes,
            truncated: true,
        })
    }
}

fn aggregate_status(stages: &[CompilerStageResult]) -> CheckStatus {
    for status in [
        CheckStatus::OutputLimit,
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

fn read_bounded_file(path: &Path, max_bytes: usize) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut bytes = Vec::new();
    file.by_ref()
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("file {} exceeds its {} byte budget", path.display(), max_bytes),
        ));
    }
    String::from_utf8(bytes).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file {} is not valid UTF-8: {error}", path.display()),
        )
    })
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
    let contents = read_bounded_file(&database, MAX_COMPILATION_DATABASE_BYTES)?;
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
    let target = if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("compile_commands.json"))
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
    if path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("compile_commands.json"))
    {
        return Some(path.to_path_buf());
    }
    if !enabled && !path.is_dir() {
        return None;
    }
    let start = if path.is_dir() { path } else { path.parent()? };
    start
        .ancestors()
        .take(MAX_DISCOVERY_DEPTH + 1)
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
    let extensions = extensions
        .iter()
        .map(|extension| extension.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let started = Instant::now();
    let mut pending = vec![(path.to_path_buf(), 0_usize)];
    let mut visited_files = 0_usize;
    while let Some((directory, depth)) = pending.pop() {
        if started.elapsed() > MAX_DISCOVERY_TIME {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "source discovery exceeded its time budget",
            ));
        }
        if depth > MAX_DISCOVERY_DEPTH {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "source discovery exceeded its depth budget",
            ));
        }
        let mut children = fs::read_dir(&directory)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        children.sort();
        for child in children {
            if child.to_string_lossy().len() > MAX_DISCOVERY_PATH_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "source discovery encountered a path over its budget",
                ));
            }
            let metadata = fs::symlink_metadata(&child)?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                let name = child.file_name().and_then(|value| value.to_str());
                if !matches!(name, Some("node_modules" | "target" | ".git")) {
                    pending.push((child, depth + 1));
                }
                continue;
            }
            visited_files = visited_files.saturating_add(1);
            if visited_files > MAX_DISCOVERY_FILES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "source discovery exceeded its file budget",
                ));
            }
            let extension = child
                .extension()
                .and_then(|value| value.to_str())
                .map(str::to_ascii_lowercase);
            if extension
                .as_deref()
                .is_some_and(|extension| extensions.iter().any(|item| item == extension))
            {
                output.push(child);
            }
        }
    }
    Ok(())
}

fn project_file(path: &Path, name: &str, enabled: bool) -> Option<PathBuf> {
    if path
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(name))
    {
        return Some(path.to_path_buf());
    }
    if !enabled && !path.is_dir() {
        return None;
    }
    let start = if path.is_dir() { path } else { path.parent()? };
    start
        .ancestors()
        .take(MAX_DISCOVERY_DEPTH + 1)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
}

fn project_directory(path: &Path, manifest: &str, enabled: bool) -> Option<PathBuf> {
    project_file(path, manifest, enabled).and_then(|file| file.parent().map(Path::to_path_buf))
}

fn csharp_project(path: &Path, enabled: bool) -> io::Result<Option<PathBuf>> {
    if is_csharp_project_file(path) {
        return Ok(Some(path.to_path_buf()));
    }
    if !enabled && !path.is_dir() {
        return Ok(None);
    }
    let Some(start) = (if path.is_dir() { Some(path) } else { path.parent() }) else {
        return Ok(None);
    };
    for directory in start.ancestors().take(MAX_DISCOVERY_DEPTH + 1) {
        let mut projects = fs::read_dir(directory)?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|candidate| is_csharp_project_file(candidate))
            .collect::<Vec<_>>();
        projects.sort();
        if projects.len() > 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "ambiguous C# project selection in {}",
                    directory.display()
                ),
            ));
        }
        if let Some(project) = projects.pop() {
            return Ok(Some(project));
        }
    }
    Ok(None)
}

fn is_csharp_project_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("csproj" | "sln")
    )
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
            .any(|entry| is_csharp_project_file(&entry.path()))
        {
            return Some(Language::CSharp);
        }
        return None;
    }
    let name = path.file_name()?.to_str()?;
    if name.eq_ignore_ascii_case("Cargo.toml") {
        return Some(Language::Rust);
    }
    if name.eq_ignore_ascii_case("go.mod") {
        return Some(Language::Go);
    }
    if name.eq_ignore_ascii_case("tsconfig.json") {
        return Some(Language::TypeScript);
    }
    if is_csharp_project_file(path) {
        Some(Language::CSharp)
    } else {
        None
    }
}

fn requires_trusted_execution(path: &Path, options: &CompilerCheckOptions) -> bool {
    if path.is_dir() || options.project {
        return true;
    }
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    name.eq_ignore_ascii_case("compile_commands.json")
        || name.eq_ignore_ascii_case("Cargo.toml")
        || name.eq_ignore_ascii_case("go.mod")
        || name.eq_ignore_ascii_case("tsconfig.json")
        || name.eq_ignore_ascii_case("jsconfig.json")
        || is_csharp_project_file(path)
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
        if let Some(parent) = path.parent() {
            let owned = parent
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("ripex-compiler-"));
            if owned {
                let _ = fs::remove_dir_all(parent);
            }
        }
    }
}

fn temporary_artifact(extension: &str) -> io::Result<PathBuf> {
    let root = std::env::temp_dir();
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    for attempt in 0_u32..100 {
        let directory = root.join(format!("ripex-compiler-{seed:032x}-{attempt:02x}"));
        match fs::create_dir(&directory) {
            Ok(()) => return Ok(directory.join(format!("artifact.{extension}"))),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not create an exclusive compiler temporary directory",
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
        let root = temporary_artifact("compile-commands-test").unwrap();
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

        let plans = plan_compiler_check(
            &root,
            Some(Language::C),
            &CompilerCheckOptions {
                trusted_project: true,
                ..Default::default()
            },
        )
        .unwrap();
        let source_plans = plan_compiler_check(
            &source,
            Some(Language::C),
            &CompilerCheckOptions {
                project: true,
                trusted_project: true,
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
    #[test]
    fn process_output_is_capped_and_status_is_distinct() {
        let mut command = Command::new(if cfg!(windows) { "cmd" } else { "sh" });
        #[cfg(windows)]
        command.args(["/C", "for /L %i in (1,1,10000) do @echo output"]);
        #[cfg(not(windows))]
        command.args(["-c", "yes output"]);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let capture = run_process(command, Duration::from_secs(5), 128, 256).unwrap();
        assert!(capture.output_limited);
        assert!(capture.stdout.len() <= 128);
        assert!(capture.stdout_truncated || capture.stderr_truncated);
    }

    #[test]
    fn timeout_cleanup_returns_within_grace_period() {
        let mut command = Command::new(if cfg!(windows) { "cmd" } else { "sh" });
        #[cfg(windows)]
        command.args(["/C", "ping -n 20 127.0.0.1 > nul"]);
        #[cfg(not(windows))]
        command.args(["-c", "sleep 20"]);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        let started = Instant::now();
        let capture = run_process(command, Duration::from_millis(20), 128, 256).unwrap();
        assert!(capture.timed_out);
        assert!(started.elapsed() < Duration::from_secs(2));
    }

    #[test]
    fn temporary_artifact_uses_an_owned_non_pid_directory() {
        let artifact = temporary_artifact("test").unwrap();
        let parent = artifact.parent().unwrap();
        let name = parent.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("ripex-compiler-"));
        assert!(!name.contains(&std::process::id().to_string()));
        let _ = fs::remove_dir_all(parent);
    }
}
