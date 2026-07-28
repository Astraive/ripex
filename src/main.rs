//! ripex CLI — parse source files with the hand-written ripex parsers and
//! emit extracted facts / AST summaries as JSON or human-readable text.

use std::error::Error as StdError;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[allow(unused_imports)]
use ripex::{
    compiler::{check_with_compiler, CheckStatus, CompilerCheckOptions},
    detect_language, parser_for_language, registry, ExtractionResult, Language, ParseResult,
    ParseStatus, Program,
};

const JSON_SCHEMA_VERSION: u32 = 2;

#[derive(Debug)]
struct CliFailure {
    code: &'static str,
    category: &'static str,
    message: String,
    exit_code: i32,
}

impl CliFailure {
    fn new(
        code: &'static str,
        category: &'static str,
        message: impl Into<String>,
        exit_code: i32,
    ) -> Self {
        Self {
            code,
            category,
            message: message.into(),
            exit_code,
        }
    }
}

impl std::fmt::Display for CliFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for CliFailure {}

#[derive(Parser)]
#[command(
    name = "ripex",
    version,
    about = "multi-language parser and structural fact extractor",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse a source file and print extracted facts / AST.
    Parse {
        /// Path to the source file.
        file: PathBuf,
        /// Force a language id (e.g. javascript, typescript, python, go, rust, c, cpp, csharp).
        /// If omitted, the language is detected from the file extension.
        #[arg(long, short)]
        lang: Option<String>,
        /// Emit machine-readable JSON (facts + optional AST summary + errors).
        #[arg(long)]
        json: bool,
        /// Include the AST shape summary (structural, not a full dump).
        #[arg(long)]
        ast: bool,
        /// Include the extracted symbols / imports / calls / variables.
        #[arg(long)]
        facts: bool,
    },
    /// Validate a file or project with the language's production compiler/toolchain.
    Check {
        /// Source file, project directory, or project manifest to check.
        path: PathBuf,
        /// Force a language id when it cannot be detected from the path.
        #[arg(long, short)]
        lang: Option<String>,
        /// Search parent directories for a project manifest and check the project.
        #[arg(long)]
        project: bool,
        /// Explicitly trust this project and permit compiler execution.
        #[arg(long)]
        trusted_project: bool,
        /// Override the primary compiler or static analyzer executable.
        #[arg(long)]
        toolchain: Option<PathBuf>,
        /// Language standard/target, such as c23, c++23, or ES2022.
        #[arg(long)]
        standard: Option<String>,
        /// Maximum seconds allowed for each compiler stage.
        #[arg(long, default_value_t = 120)]
        timeout: u64,
        /// Append an argument to the compiler invocation; repeat as needed.
        #[arg(long = "arg", allow_hyphen_values = true)]
        extra_args: Vec<String>,
        /// Permit the raw `--arg` passthrough for this explicitly trusted check.
        #[arg(long)]
        allow_unsafe_args: bool,
        /// Emit the complete machine-readable compiler report.
        #[arg(long)]
        json: bool,
    },
    /// List the parsers registered in ripex.
    Ls,
}

struct CheckRequest {
    path: PathBuf,
    lang: Option<String>,
    project: bool,
    trusted_project: bool,
    toolchain: Option<PathBuf>,
    standard: Option<String>,
    timeout: u64,
    extra_args: Vec<String>,
    allow_unsafe_args: bool,
    json: bool,
}

#[derive(serde::Serialize)]
struct TrustMetadata {
    trusted_project: bool,
    sandboxed: bool,
}

fn parse_status_id(status: &ParseStatus) -> &'static str {
    match status {
        ParseStatus::Complete => "complete",
        ParseStatus::Recovered => "recovered",
        ParseStatus::LimitExceeded => "limit_exceeded",
        ParseStatus::Failed => "failed",
        _ => "failed",
    }
}

fn parse_completeness(status: &ParseStatus) -> &'static str {
    parse_status_id(status)
}

fn run_check(request: CheckRequest) -> anyhow::Result<()> {
    if !request.trusted_project {
        return Err(anyhow::Error::new(CliFailure::new(
            "trust_required",
            "policy",
            "compiler checks execute project/toolchain code; pass --trusted-project explicitly",
            2,
        )));
    }
    if !request.extra_args.is_empty() && !request.allow_unsafe_args {
        return Err(anyhow::Error::new(CliFailure::new(
            "unsafe_args_required",
            "policy",
            "raw compiler arguments require --allow-unsafe-args",
            2,
        )));
    }

    let language = request
        .lang
        .as_deref()
        .map(|id| {
            Language::from_id(id).ok_or_else(|| {
                anyhow::Error::new(CliFailure::new(
                    "unknown_language",
                    "selection",
                    format!("unknown language: {id}"),
                    2,
                ))
            })
        })
        .transpose()?;
    let report = check_with_compiler(
        &request.path,
        language,
        &CompilerCheckOptions {
            toolchain: request.toolchain,
            standard: request.standard,
            project: request.project,
            trusted_project: request.trusted_project,
            allow_unsafe_args: request.allow_unsafe_args,
            extra_args: request.extra_args,
            max_output_bytes: 1024 * 1024,
            max_total_output_bytes: 2 * 1024 * 1024,
            timeout: Duration::from_secs(request.timeout),
        },
    )
    .map_err(|error| {
        let code = match error.kind() {
            io::ErrorKind::NotFound => "check_target_missing",
            io::ErrorKind::InvalidInput => "check_planning_invalid",
            _ => "check_planning_failed",
        };
        anyhow::Error::new(CliFailure::new(
            code,
            "check_planning",
            format!(
                "could not compiler-check {}: {error}",
                request.path.display()
            ),
            2,
        ))
    })?;

    if request.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&CompilerJsonOutput {
                schema_version: JSON_SCHEMA_VERSION,
                operation: "check",
                completeness: "not_applicable",
                effective_mode: None,
                trust: TrustMetadata {
                    trusted_project: request.trusted_project,
                    sandboxed: false,
                },
                report: &report,
            })?
        );
    } else {
        println!("language: {}", report.language.id());
        println!("path:     {}", report.path.display());
        println!("status:   {:?}", report.status);
        for stage in &report.stages {
            println!("\n[{:?}] {}: {:?}", stage.kind, stage.backend, stage.status);
            println!("  command: {}", stage.command.join(" "));
            if let Some(code) = stage.exit_code {
                println!("  exit:    {code}");
            }
            for diagnostic in &stage.diagnostics {
                eprintln!("  - {}", diagnostic.raw);
            }
            if stage.diagnostics.is_empty() {
                for line in stage.stdout.lines().chain(stage.stderr.lines()) {
                    eprintln!("  | {line}");
                }
            }
        }
    }

    match report.status {
        CheckStatus::Passed => Ok(()),
        CheckStatus::Failed => std::process::exit(1),
        CheckStatus::Unavailable
        | CheckStatus::InvocationError
        | CheckStatus::TimedOut
        | CheckStatus::OutputLimit => std::process::exit(2),
    }
}

#[derive(serde::Serialize)]
struct CompilerJsonOutput<'a> {
    schema_version: u32,
    operation: &'static str,
    completeness: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    effective_mode: Option<&'static str>,
    trust: TrustMetadata,
    #[serde(flatten)]
    report: &'a ripex::compiler::CompilerCheckReport,
}

#[derive(serde::Serialize)]
struct JsonErrorOutput {
    schema_version: u32,
    operation: &'static str,
    status: &'static str,
    completeness: &'static str,
    effective_mode: Option<String>,
    trust: TrustMetadata,
    error: JsonError,
}

#[derive(serde::Serialize)]
struct JsonError {
    code: &'static str,
    category: &'static str,
    message: String,
    exit_code: i32,
}

fn read_source(path: &Path) -> anyhow::Result<String> {
    let file = File::open(path).map_err(|error| {
        let code = if error.kind() == io::ErrorKind::NotFound {
            "file_not_found"
        } else {
            "read_failed"
        };
        anyhow::Error::new(CliFailure::new(
            code,
            "read",
            format!("failed to open {}: {error}", path.display()),
            2,
        ))
    })?;
    let metadata = file.metadata().map_err(|error| {
        anyhow::Error::new(CliFailure::new(
            "read_failed",
            "read",
            format!("failed to inspect {}: {error}", path.display()),
            2,
        ))
    })?;
    if !metadata.file_type().is_file() {
        return Err(anyhow::Error::new(CliFailure::new(
            "non_regular_file",
            "read",
            format!("{} is not a regular file", path.display()),
            2,
        )));
    }

    let mut bytes = Vec::new();
    let mut bounded = file.take((ripex::limits::MAX_INPUT_SIZE + 1) as u64);
    bounded.read_to_end(&mut bytes).map_err(|error| {
        anyhow::Error::new(CliFailure::new(
            "read_failed",
            "read",
            format!("failed to read {}: {error}", path.display()),
            2,
        ))
    })?;
    if bytes.len() > ripex::limits::MAX_INPUT_SIZE {
        return Err(anyhow::Error::new(CliFailure::new(
            "input_too_large",
            "validation",
            format!(
                "{} exceeds the {} byte parser input limit",
                path.display(),
                ripex::limits::MAX_INPUT_SIZE
            ),
            1,
        )));
    }
    String::from_utf8(bytes).map_err(|error| {
        anyhow::Error::new(CliFailure::new(
            "invalid_utf8",
            "validation",
            format!(
                "{} is not valid UTF-8 at byte {}",
                path.display(),
                error.utf8_error().valid_up_to()
            ),
            1,
        ))
    })
}

fn run_parse(
    path: &Path,
    lang: Option<&str>,
    json: bool,
    ast: bool,
    facts: bool,
) -> anyhow::Result<()> {
    let source = read_source(path)?;
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let language = match lang {
        Some(id) => Language::from_id(id).ok_or_else(|| {
            anyhow::Error::new(CliFailure::new(
                "unknown_language",
                "selection",
                format!("unknown language: {id}"),
                2,
            ))
        })?,
        None => detect_language(path).ok_or_else(|| {
            anyhow::Error::new(CliFailure::new(
                "language_detection_failed",
                "detection",
                format!(
                    "could not detect language from extension of {}; pass --lang",
                    path.display()
                ),
                2,
            ))
        })?,
    };
    let parser = parser_for_language(language, (!extension.is_empty()).then_some(extension))
        .ok_or_else(|| {
            anyhow::Error::new(CliFailure::new(
                "parser_unavailable",
                "selection",
                format!("no parser registered for language: {}", language.id()),
                2,
            ))
        })?;

    let result: ParseResult = parser.parse(&source);
    let extracted: ExtractionResult = match parser.extract(&result) {
        Ok(extracted) => extracted,
        Err(error) if matches!(&result.status, ParseStatus::Recovered) => {
            parser.extract_best_effort(&result).map_err(|best_effort_error| {
                anyhow::Error::new(CliFailure::new(
                    "extraction_failed",
                    "parse",
                    format!(
                        "strict extraction failed ({error:?}); best-effort extraction failed ({best_effort_error:?})"
                    ),
                    1,
                ))
            })?
        }
        Err(error) => {
            return Err(anyhow::Error::new(CliFailure::new(
                "extraction_failed",
                "parse",
                format!("could not extract facts: {error:?}"),
                1,
            )))
        }
    };
    let status = parse_status_id(&result.status);
    let complete = matches!(&result.status, ParseStatus::Complete) && result.errors.is_empty();

    if json {
        let out = JsonOutput {
            schema_version: JSON_SCHEMA_VERSION,
            operation: "parse",
            language: result.language.id(),
            file: path,
            status,
            completeness: parse_completeness(&result.status),
            effective_mode: &result.parser_mode,
            truncated: matches!(&result.status, ParseStatus::LimitExceeded),
            trust: TrustMetadata {
                trusted_project: false,
                sandboxed: false,
            },
            errors: &result.errors,
            comments: &result.comments,
            ast: ast.then(|| ast_summary(&result)),
            facts: &extracted,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        if !complete {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("language: {}", result.language.id());
    println!("file:     {}", path.display());
    println!("status:   {status}");
    println!("mode:     {}", result.parser_mode);
    println!("errors:   {}", result.errors.len());
    for error in &result.errors {
        eprintln!("  - {error}");
    }

    if ast {
        println!("\nAST:");
        let summary = ast_summary(&result);
        println!("  kind: {}", summary.kind);
        println!("  top-level nodes: {}", summary.top_level_nodes);
        if let Some(count) = summary.expression_nodes {
            println!("  expression nodes: {count}");
        }
    }

    if facts || !ast {
        print_facts(&extracted);
    }

    if !complete {
        std::process::exit(1);
    }
    Ok(())
}

fn ast_summary(result: &ParseResult) -> AstSummary {
    match &result.ast {
        #[cfg(feature = "lang-js")]
        Program::Js(program, arena) => match program {
            ripex::js::ast::Program::Script(script) => AstSummary {
                kind: "javascript_script",
                top_level_nodes: script.body.len(),
                expression_nodes: Some(arena.len()),
            },
            ripex::js::ast::Program::Module(module) => AstSummary {
                kind: "javascript_module",
                top_level_nodes: module.body.len(),
                expression_nodes: Some(arena.len()),
            },
        },
        #[cfg(feature = "lang-python")]
        Program::Python(program) => AstSummary {
            kind: "python",
            top_level_nodes: program.stmts.len(),
            expression_nodes: None,
        },
        #[cfg(feature = "lang-go")]
        Program::Go(program) => AstSummary {
            kind: "go",
            top_level_nodes: program.decls.len(),
            expression_nodes: None,
        },
        #[cfg(feature = "lang-rust")]
        Program::Rust(program) => AstSummary {
            kind: "rust",
            top_level_nodes: program.items.len(),
            expression_nodes: None,
        },
        #[cfg(feature = "lang-c")]
        Program::C(program) => AstSummary {
            kind: "c",
            top_level_nodes: program.decls.len(),
            expression_nodes: None,
        },
        #[cfg(feature = "lang-cpp")]
        Program::Cpp(program) => AstSummary {
            kind: "cpp",
            top_level_nodes: program.decls.len(),
            expression_nodes: None,
        },
        #[cfg(feature = "lang-csharp")]
        Program::CSharp(program) => AstSummary {
            kind: "csharp",
            top_level_nodes: program.decls.len(),
            expression_nodes: None,
        },
        #[allow(unreachable_patterns)]
        _ => AstSummary {
            kind: "unknown",
            top_level_nodes: 0,
            expression_nodes: None,
        },
    }
}

#[derive(serde::Serialize)]
struct JsonOutput<'a> {
    schema_version: u32,
    operation: &'static str,
    language: &'static str,
    file: &'a Path,
    status: &'static str,
    completeness: &'static str,
    effective_mode: &'a str,
    truncated: bool,
    trust: TrustMetadata,
    errors: &'a [ripex::diagnostics::ParseError],
    comments: &'a [ripex::ParsedComment],
    #[serde(skip_serializing_if = "Option::is_none")]
    ast: Option<AstSummary>,
    facts: &'a ExtractionResult,
}

#[derive(serde::Serialize)]
struct AstSummary {
    kind: &'static str,
    top_level_nodes: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    expression_nodes: Option<usize>,
}

fn print_facts(ex: &ExtractionResult) {
    println!(
        "\nsymbols:   {}  imports: {}  calls: {}  variables: {}",
        ex.symbols.len(),
        ex.imports.len(),
        ex.calls.len(),
        ex.variables.len()
    );
    if !ex.symbols.is_empty() {
        println!("  symbols:");
        for s in &ex.symbols {
            println!(
                "    - [{:?}] {} @ {}:{}",
                s.kind, s.name, s.line_start, s.line_end
            );
        }
    }
    if !ex.imports.is_empty() {
        println!("  imports:");
        for i in &ex.imports {
            let label = i
                .imported_name
                .as_deref()
                .or(i.local_name.as_deref())
                .unwrap_or("?");
            println!("    - {label} <- {}", i.source);
        }
    }
    if !ex.calls.is_empty() {
        println!("  calls:");
        for c in &ex.calls {
            println!("    - {} ({:?})", c.callee_text, c.kind);
        }
    }
    if !ex.variables.is_empty() {
        println!("  variables:");
        for v in &ex.variables {
            println!("    - {} [{:?}]", v.name, v.kind);
        }
    }
}
fn run_ls() -> anyhow::Result<()> {
    let reg = registry();
    let mut lang_ids: Vec<&str> = reg.keys().copied().collect();
    lang_ids.sort_unstable();
    let has_typescript_mode = parser_for_language(Language::TypeScript, None).is_some();
    println!(
        "ripex parsers ({}):",
        lang_ids.len() + usize::from(has_typescript_mode && !lang_ids.contains(&"typescript"))
    );
    for id in lang_ids {
        let p = &reg[id];
        println!("  {}  ({})", id, p.extensions().join(", "));
    }
    if has_typescript_mode && !reg.contains_key("typescript") {
        println!("  typescript  (ts, tsx, mts, cts)");
    }
    Ok(())
}

fn emit_json_error(operation: &'static str, error: &anyhow::Error) -> i32 {
    let (code, category, exit_code) = error
        .downcast_ref::<CliFailure>()
        .map(|failure| (failure.code, failure.category, failure.exit_code))
        .unwrap_or(("execution_failed", "execution", 2));
    let payload = JsonErrorOutput {
        schema_version: JSON_SCHEMA_VERSION,
        operation,
        status: "error",
        completeness: "failed",
        effective_mode: None,
        trust: TrustMetadata {
            trusted_project: false,
            sandboxed: false,
        },
        error: JsonError {
            code,
            category,
            message: error.to_string(),
            exit_code,
        },
    };
    if let Ok(serialized) = serde_json::to_string_pretty(&payload) {
        println!("{serialized}");
    }
    exit_code
}

fn main() -> anyhow::Result<()> {
    // These hand-written recursive-descent parsers overflow the default 1 MiB
    // Windows thread stack on non-trivial inputs. Run the work on a larger
    // stack, mirroring the graxus CLI's workaround.
    let handle = std::thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(run)
        .context("failed to spawn ripex worker thread")?;
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("ripex worker thread panicked"))?
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (operation, json, result) = match cli.command {
        Command::Parse {
            file,
            lang,
            json,
            ast,
            facts,
        } => (
            "parse",
            json,
            run_parse(&file, lang.as_deref(), json, ast, facts),
        ),
        Command::Check {
            path,
            lang,
            project,
            trusted_project,
            toolchain,
            standard,
            timeout,
            extra_args,
            allow_unsafe_args,
            json,
        } => (
            "check",
            json,
            run_check(CheckRequest {
                path,
                lang,
                project,
                trusted_project,
                toolchain,
                standard,
                timeout,
                extra_args,
                allow_unsafe_args,
                json,
            }),
        ),
        Command::Ls => ("ls", false, run_ls()),
    };
    if let Err(error) = result {
        let exit_code = if json {
            emit_json_error(operation, &error)
        } else {
            error
                .downcast_ref::<CliFailure>()
                .map(|failure| failure.exit_code)
                .unwrap_or(2)
        };
        eprintln!("ripex: {error:#}");
        std::process::exit(exit_code);
    }
    Ok(())
}
