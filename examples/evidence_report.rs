//! Generate the Ripex 0.3 parser evidence report.
//!
//! Run with `cargo run --example evidence_report --all-features -- docs/BENCHMARKS.md`.
//! The report is measured from the checked-in corpus and curated per-language
//! gold cases; it does not infer correctness from Ripex output alone.

mod evidence;

use evidence::{all_cases, allocation_baseline, peak_allocation_since, EvidenceCase};
use ripex::{parser_for_ext, ExtractionResult};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tree_sitter::Parser as TreeSitterParser;

const CASE_TIMEOUT: Duration = Duration::from_secs(2);
const BENCH_ITERATIONS: usize = 10;
const LANGUAGES: [&str; 8] = [
    "c",
    "cpp",
    "csharp",
    "go",
    "javascript",
    "python",
    "rust",
    "typescript",
];

#[derive(Clone)]
struct CorpusFile {
    language: &'static str,
    extension: String,
    source: String,
}

#[derive(Default, Clone)]
struct Execution {
    complete: bool,
    errors: usize,
    symbols: Vec<String>,
    imports: Vec<String>,
    calls: Vec<String>,
    variables: Vec<String>,
    panicked: bool,
    timed_out: bool,
}

#[derive(Default)]
struct CorpusStats {
    files: usize,
    bytes: usize,
    complete: usize,
    errors: usize,
    panics: usize,
    hangs: usize,
}

#[derive(Default)]
struct AccuracyStats {
    cases: usize,
    gold: usize,
    predicted: usize,
    true_positive: usize,
}

#[derive(Default)]
struct MalformedStats {
    inputs: usize,
    diagnostics: usize,
    panics: usize,
    hangs: usize,
}

#[derive(Default)]
struct BenchStats {
    bytes: usize,
    iterations: usize,
    elapsed: Duration,
    peak_allocated_bytes: usize,
    tree_sitter_elapsed: Duration,
    tree_sitter_peak_allocated_bytes: usize,
}

fn language_for_extension(extension: &str) -> Option<&'static str> {
    match extension {
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some("cpp"),
        "cs" => Some("csharp"),
        "go" => Some("go"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "py" | "pyi" => Some("python"),
        "rs" => Some("rust"),
        "ts" | "tsx" | "mts" | "cts" => Some("typescript"),
        _ => None,
    }
}

fn collect_files(root: &Path, files: &mut Vec<CorpusFile>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|name| name.to_str());
            if !matches!(
                name,
                Some("node_modules" | "target" | ".git" | "dist" | "build")
            ) {
                collect_files(&path, files);
            }
            continue;
        }
        let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
            continue;
        };
        let extension = extension.to_ascii_lowercase();
        let Some(language) = language_for_extension(&extension) else {
            continue;
        };
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        files.push(CorpusFile {
            language,
            extension,
            source,
        });
    }
}

fn run_source(language: &'static str, extension: &str, source: &str) -> Execution {
    let extension = extension.to_owned();
    let source = source.to_owned();
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Some(parser) = parser_for_ext(language, &extension) else {
                return Execution {
                    errors: 1,
                    ..Execution::default()
                };
            };
            let parsed = parser.parse(&source);
            let errors = parsed.errors.len();
            let complete = errors == 0 && parsed.is_complete();
            let facts = parser
                .extract_best_effort(&parsed)
                .unwrap_or_else(|_| ExtractionResult::new());
            Execution {
                complete,
                errors,
                symbols: facts.symbols.into_iter().map(|fact| fact.name).collect(),
                imports: facts.imports.into_iter().map(|fact| fact.source).collect(),
                calls: facts
                    .calls
                    .into_iter()
                    .map(|fact| fact.callee_text)
                    .collect(),
                variables: facts.variables.into_iter().map(|fact| fact.name).collect(),
                ..Execution::default()
            }
        }));
        let execution = result.unwrap_or_else(|_| Execution {
            panicked: true,
            ..Execution::default()
        });
        let _ = sender.send(execution);
    });

    match receiver.recv_timeout(CASE_TIMEOUT) {
        Ok(execution) => execution,
        Err(mpsc::RecvTimeoutError::Timeout) => Execution {
            timed_out: true,
            ..Execution::default()
        },
        Err(mpsc::RecvTimeoutError::Disconnected) => Execution {
            panicked: true,
            ..Execution::default()
        },
    }
}

fn set(values: &[String]) -> BTreeSet<&str> {
    values.iter().map(String::as_str).collect()
}

fn score(expected: &[&str], actual: &[String], stats: &mut AccuracyStats) {
    let expected: BTreeSet<&str> = expected.iter().copied().collect();
    let actual = set(actual);
    stats.gold += expected.len();
    stats.predicted += actual.len();
    stats.true_positive += expected.intersection(&actual).count();
}

fn mismatch(expected: &[&str], actual: &[String]) -> (Vec<String>, Vec<String>) {
    let expected_set: BTreeSet<&str> = expected.iter().copied().collect();
    let actual_set = set(actual);
    let missing = expected_set
        .difference(&actual_set)
        .map(|value| (*value).to_owned())
        .collect();
    let extra = actual_set
        .difference(&expected_set)
        .map(|value| (*value).to_owned())
        .collect();
    (missing, extra)
}

fn accuracy_for_case(case: &EvidenceCase, execution: &Execution) -> AccuracyStats {
    let mut stats = AccuracyStats {
        cases: 1,
        ..AccuracyStats::default()
    };
    score(case.expected.symbols, &execution.symbols, &mut stats);
    score(case.expected.imports, &execution.imports, &mut stats);
    score(case.expected.calls, &execution.calls, &mut stats);
    score(case.expected.variables, &execution.variables, &mut stats);
    stats
}

fn merge_accuracy(into: &mut AccuracyStats, other: AccuracyStats) {
    into.cases += other.cases;
    into.gold += other.gold;
    into.predicted += other.predicted;
    into.true_positive += other.true_positive;
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        100.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

fn tree_sitter_language(language: &str, extension: &str) -> Option<tree_sitter::Language> {
    Some(match language {
        "c" => tree_sitter_c::LANGUAGE.into(),
        "cpp" => tree_sitter_cpp::LANGUAGE.into(),
        "csharp" => tree_sitter_c_sharp::LANGUAGE.into(),
        "go" => tree_sitter_go::LANGUAGE.into(),
        "javascript" => tree_sitter_javascript::LANGUAGE.into(),
        "python" => tree_sitter_python::LANGUAGE.into(),
        "rust" => tree_sitter_rust::LANGUAGE.into(),
        "typescript" if extension == "tsx" => tree_sitter_typescript::LANGUAGE_TSX.into(),
        "typescript" => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        _ => return None,
    })
}

fn tree_sitter_parses(language: &str, extension: &str, source: &str) -> bool {
    let Some(language) = tree_sitter_language(language, extension) else {
        return false;
    };
    let mut parser = TreeSitterParser::new();
    if parser.set_language(&language).is_err() {
        return false;
    }
    parser
        .parse(source, None)
        .map(|tree| !tree.root_node().has_error())
        .unwrap_or(false)
}

fn benchmark(language: &str, files: &[CorpusFile]) -> BenchStats {
    let bytes = files.iter().map(|file| file.source.len()).sum::<usize>();
    let baseline = allocation_baseline();
    let start = Instant::now();
    for _ in 0..BENCH_ITERATIONS {
        for file in files {
            if let Some(parser) = parser_for_ext(language, &file.extension) {
                let parsed = parser.parse(&file.source);
                let _ = parser.extract_best_effort(&parsed);
            }
        }
    }
    let elapsed = start.elapsed();
    let peak_allocated_bytes = peak_allocation_since(baseline);

    let tree_baseline = allocation_baseline();
    let tree_start = Instant::now();
    for _ in 0..BENCH_ITERATIONS {
        for file in files {
            let _ = tree_sitter_parses(language, &file.extension, &file.source);
        }
    }
    let tree_sitter_elapsed = tree_start.elapsed();
    let tree_sitter_peak_allocated_bytes = peak_allocation_since(tree_baseline);

    BenchStats {
        bytes,
        iterations: BENCH_ITERATIONS,
        elapsed,
        peak_allocated_bytes,
        tree_sitter_elapsed,
        tree_sitter_peak_allocated_bytes,
    }
}

fn corpus_stats(files: &[CorpusFile]) -> CorpusStats {
    let mut stats = CorpusStats {
        files: files.len(),
        bytes: files.iter().map(|file| file.source.len()).sum(),
        ..CorpusStats::default()
    };
    for file in files {
        let execution = run_source(file.language, &file.extension, &file.source);
        stats.errors += execution.errors;
        if execution.complete {
            stats.complete += 1;
        }
        stats.panics += usize::from(execution.panicked);
        stats.hangs += usize::from(execution.timed_out);
    }
    stats
}

fn malformed_stats(cases: &[&'static EvidenceCase]) -> MalformedStats {
    let mut stats = MalformedStats::default();
    for case in cases {
        for source in case.malformed {
            stats.inputs += 1;
            let execution = run_source(case.language, case.extension, source);
            stats.diagnostics += usize::from(execution.errors > 0 || !execution.complete);
            stats.panics += usize::from(execution.panicked);
            stats.hangs += usize::from(execution.timed_out);
        }
    }
    stats
}

fn fmt_duration(duration: Duration) -> String {
    format!("{:.3}", duration.as_secs_f64() * 1000.0)
}

fn report() -> String {
    let corpus_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/lang-test");
    let mut corpus = Vec::new();
    collect_files(&corpus_root, &mut corpus);
    let cases = all_cases();
    let mut by_language: BTreeMap<&str, Vec<CorpusFile>> = BTreeMap::new();
    for file in corpus {
        by_language.entry(file.language).or_default().push(file);
    }

    let mut output = String::new();
    output.push_str("# Ripex 0.3 parser evidence\n\n");
    output.push_str("This report is generated from checked-in source and curated gold cases.\n");
    output.push_str("It is evidence for the published structural contract, not a claim of compiler-level semantic equivalence.\n\n");
    output.push_str(&format!(
        "Environment: `{}/{}`; corpus source: `tests/lang-test`; benchmark iterations: `{}`.\n\n",
        std::env::consts::OS,
        std::env::consts::ARCH,
        BENCH_ITERATIONS
    ));
    output.push_str("## Corpus coverage\n\n");
    output.push_str("| Language | Corpus files | Bytes | Complete parses | Parse success | Diagnostics | Panics | Hangs | Tree-sitter clean parses |\n|---|---:|---:|---:|---:|---:|---:|---:|---:|\n");
    for language in LANGUAGES {
        let files = by_language.get(language).cloned().unwrap_or_default();
        let stats = corpus_stats(&files);
        let tree_clean = files
            .iter()
            .filter(|file| tree_sitter_parses(language, &file.extension, &file.source))
            .count();
        output.push_str(&format!(
            "| `{language}` | {} | {} | {} | {:.2}% | {} | {} | {} | {}/{} |\n",
            stats.files,
            stats.bytes,
            stats.complete,
            percent(stats.complete, stats.files),
            stats.errors,
            stats.panics,
            stats.hangs,
            tree_clean,
            stats.files
        ));
    }

    output.push_str("\n## Curated fact accuracy\n\n");
    output.push_str("Gold facts are independently listed in `examples/evidence/<language>.rs`; matching uses exact names, import sources, call names, and variable names.\n\n");
    output.push_str("| Language | Case IDs | Cases | Gold facts | Predicted facts | True positives | Precision | Recall |\n|---|---|---:|---:|---:|---:|---:|---:|\n");
    for language in LANGUAGES {
        let language_cases: Vec<_> = cases
            .iter()
            .copied()
            .filter(|case| case.language == language)
            .collect();
        let mut stats = AccuracyStats::default();
        for case in &language_cases {
            merge_accuracy(
                &mut stats,
                accuracy_for_case(
                    case,
                    &run_source(case.language, case.extension, case.source),
                ),
            );
        }
        let case_ids = language_cases
            .iter()
            .map(|case| case.id)
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "| `{language}` | `{case_ids}` | {} | {} | {} | {} | {:.2}% | {:.2}% |\n",
            stats.cases,
            stats.gold,
            stats.predicted,
            stats.true_positive,
            percent(stats.true_positive, stats.predicted),
            percent(stats.true_positive, stats.gold)
        ));
    }

    output.push_str("\n### Gold mismatches\n\n");
    let mut mismatch_count = 0;
    for case in &cases {
        let execution = run_source(case.language, case.extension, case.source);
        let checks: [(&str, &[&str], &[String]); 4] = [
            ("symbols", case.expected.symbols, &execution.symbols),
            ("imports", case.expected.imports, &execution.imports),
            ("calls", case.expected.calls, &execution.calls),
            ("variables", case.expected.variables, &execution.variables),
        ];
        for (category, expected, actual) in checks {
            let (missing, extra) = mismatch(expected, actual);
            if !missing.is_empty() || !extra.is_empty() {
                mismatch_count += 1;
                output.push_str(&format!("- `{}` `{category}`:", case.id));
                if !missing.is_empty() {
                    output.push_str(&format!(" missing `{}`", missing.join("`, `")));
                }
                if !extra.is_empty() {
                    output.push_str(&format!(" extra `{}`", extra.join("`, `")));
                }
                output.push('\n');
            }
        }
    }
    if mismatch_count == 0 {
        output.push_str("No gold mismatches detected.\n");
    }
    output.push('\n');

    let malformed = malformed_stats(&cases);
    output.push_str("\n## Malformed-input behavior\n\n");
    output.push_str(&format!(
        "Ripex exercised **{}** curated malformed inputs: **{}** produced a diagnostic or incomplete status, **{}** panicked, and **{}** exceeded the two-second watchdog.\n\n",
        malformed.inputs, malformed.diagnostics, malformed.panics, malformed.hangs
    ));
    output.push_str("Each language module owns at least two malformed variants; malformed input is measured separately from the valid fact oracle.\n\n");

    output.push_str("## Throughput and allocation evidence\n\n");
    output.push_str("Throughput includes parse plus best-effort fact extraction. Ripex memory is peak allocator bytes observed during the measured loop. Tree-sitter parsing uses native allocations that this Rust allocator probe cannot observe, so its allocator column is reported as zero and must not be interpreted as total memory. This keeps the measurement cross-platform and explicit. Tree-sitter is parse-only.\n\n");
    output.push_str("| Language | Corpus bytes | Ripex MB/s | Ripex peak alloc bytes | Tree-sitter MB/s | Tree-sitter observed Rust alloc bytes |\n|---|---:|---:|---:|---:|---:|\n");
    for language in LANGUAGES {
        let files = by_language.get(language).cloned().unwrap_or_default();
        let stats = benchmark(language, &files);
        let ripex_mb_s = stats.bytes as f64 * stats.iterations as f64
            / stats.elapsed.as_secs_f64().max(f64::MIN_POSITIVE)
            / 1_000_000.0;
        let tree_mb_s = stats.bytes as f64 * stats.iterations as f64
            / stats
                .tree_sitter_elapsed
                .as_secs_f64()
                .max(f64::MIN_POSITIVE)
            / 1_000_000.0;
        output.push_str(&format!(
            "| `{language}` | {} | {:.2} | {} | {:.2} | {} |\n",
            stats.bytes,
            ripex_mb_s,
            stats.peak_allocated_bytes,
            tree_mb_s,
            stats.tree_sitter_peak_allocated_bytes
        ));
        eprintln!(
            "evidence {language}: ripex={}ms tree-sitter={}ms",
            fmt_duration(stats.elapsed),
            fmt_duration(stats.tree_sitter_elapsed)
        );
    }

    output.push_str("\n## Interpretation and limits\n\n");
    output.push_str("- Corpus measurements use the repository's checked-in language-test sources; the table records their exact file and byte counts.\n");
    output.push_str("- Precision/recall applies to the curated gold cases, not to every fact in every corpus file. Expanding gold coverage is the path to stronger accuracy claims.\n");
    output.push_str("- Tree-sitter comparison measures parser acceptance and parse throughput; it is not treated as an independent semantic oracle.\n");
    output.push_str("- Compiler conformance remains a separate gate because structural parsing cannot establish type checking, linking, macro, SDK, or project semantics.\n");
    output
}

fn main() -> std::io::Result<()> {
    let rendered = report();
    if let Some(path) = std::env::args_os().nth(1) {
        fs::write(path, rendered)
    } else {
        print!("{rendered}");
        Ok(())
    }
}
