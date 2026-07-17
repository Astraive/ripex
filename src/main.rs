//! ripex CLI — parse source files with the hand-written ripex parsers and
//! emit extracted facts / AST summaries as JSON or human-readable text.

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
use clap::{Parser, Subcommand};

use ripex::{
    compiler::{check_with_compiler, CheckStatus, CompilerCheckOptions},
    detect_language, parser_for_ext, registry, ExtractionResult, Language, ParseResult, Program,
};

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
        file: String,
        /// Force a language id (e.g. javascript, python, go, rust, c, cpp, csharp).
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
        path: String,
        /// Force a language id when it cannot be detected from the path.
        #[arg(long, short)]
        lang: Option<String>,
        /// Search parent directories for a project manifest and check the project.
        #[arg(long)]
        project: bool,
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
        /// Emit the complete machine-readable compiler report.
        #[arg(long)]
        json: bool,
    },
    /// List the parsers registered in ripex.
    Ls,
}

struct CheckRequest {
    path: String,
    lang: Option<String>,
    project: bool,
    toolchain: Option<PathBuf>,
    standard: Option<String>,
    timeout: u64,
    extra_args: Vec<String>,
    json: bool,
}

fn run_check(request: CheckRequest) -> anyhow::Result<()> {
    let language = request
        .lang
        .as_deref()
        .map(|id| Language::from_id(id).ok_or_else(|| anyhow::anyhow!("unknown language: {id}")))
        .transpose()?;
    let options = CompilerCheckOptions {
        toolchain: request.toolchain,
        standard: request.standard,
        project: request.project,
        extra_args: request.extra_args,
        timeout: Duration::from_secs(request.timeout),
    };
    let report = check_with_compiler(&request.path, language, &options)
        .with_context(|| format!("could not compiler-check {}", request.path))?;

    if request.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&CompilerJsonOutput {
                schema_version: 1,
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
                println!("  - {}", diagnostic.raw);
            }
            if stage.diagnostics.is_empty() {
                for line in stage.stdout.lines().chain(stage.stderr.lines()) {
                    println!("  | {line}");
                }
            }
        }
    }

    match report.status {
        CheckStatus::Passed => Ok(()),
        CheckStatus::Failed => std::process::exit(1),
        CheckStatus::Unavailable | CheckStatus::InvocationError | CheckStatus::TimedOut => {
            std::process::exit(2)
        }
    }
}

#[derive(serde::Serialize)]
struct CompilerJsonOutput<'a> {
    schema_version: u32,
    #[serde(flatten)]
    report: &'a ripex::compiler::CompilerCheckReport,
}

fn run_parse(
    file: &str,
    lang: Option<&str>,
    json: bool,
    ast: bool,
    facts: bool,
) -> anyhow::Result<()> {
    let path = Path::new(file);
    let source =
        std::fs::read_to_string(path).with_context(|| format!("failed to read file: {file}"))?;

    let detected;
    let lang_id = match lang {
        Some(l) => l,
        None => {
            detected = detect_language(path).ok_or_else(|| {
                anyhow::anyhow!("could not detect language from extension of {file}; pass --lang")
            })?;
            detected.id()
        }
    };
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();
    let language =
        Language::from_id(lang_id).ok_or_else(|| anyhow::anyhow!("unknown language: {lang_id}"))?;

    let parser = parser_for_ext(lang_id, extension)
        .ok_or_else(|| anyhow::anyhow!("no parser registered for language: {lang_id}"))?;

    let result: ParseResult = parser.parse(&source);
    let extracted: ExtractionResult = parser.extract(&result);

    if json {
        let out = JsonOutput {
            schema_version: 1,
            language: language.id(),
            file,
            errors: &result.errors,
            comments: &result.comments,
            ast: if ast {
                Some(ast_summary(&result))
            } else {
                None
            },
            facts: &extracted,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
        if !result.errors.is_empty() {
            std::process::exit(1);
        }
        return Ok(());
    }

    println!("language: {lang_id}");
    println!("file:     {file}");
    println!("errors:   {}", result.errors.len());
    for e in &result.errors {
        println!("  - {e}");
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

    if facts || (!ast) {
        print_facts(&extracted);
    }

    if !result.errors.is_empty() {
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
    language: &'a str,
    file: &'a str,
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
    println!("ripex parsers ({}):", lang_ids.len());
    for id in lang_ids {
        let p = &reg[id];
        println!("  {}  ({})", id, p.extensions().join(", "));
    }
    Ok(())
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
    let res = match cli.command {
        Command::Parse {
            file,
            lang,
            json,
            ast,
            facts,
        } => run_parse(&file, lang.as_deref(), json, ast, facts),
        Command::Check {
            path,
            lang,
            project,
            toolchain,
            standard,
            timeout,
            extra_args,
            json,
        } => run_check(CheckRequest {
            path,
            lang,
            project,
            toolchain,
            standard,
            timeout,
            extra_args,
            json,
        }),
        Command::Ls => run_ls(),
    };
    if let Err(e) = res {
        eprintln!("ripex: {e:#}");
        std::process::exit(2);
    }
    Ok(())
}
