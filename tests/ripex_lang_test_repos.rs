//! End-to-end corpus test for ripex.
//!
//! Walks every source file in `tests/lang-test/`, parsing + extracting facts with
//! a per-file timeout so a single misbehaving file can never wedge the whole
//! suite. Reports per-language coverage (files / ok / parse-errors / panics /
//! hangs / facts).
//!
//! This is a regression gate for crashes, hangs, and diagnostic budgets.
//!
//! Known gap features deliberately included so the run surfaces them:
//! - JS: class static blocks, decorators, `export * as ns`, top-level await
//! - non-JS: Go method receivers, Rust macro bodies, C++ lambda/template bodies,
//!   f-string expressions, enum members, etc.

use ripex::{parser_for_ext, ExtractionResult, ParseResult};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;

const PER_FILE_TIMEOUT_MS: u64 = 5_000;

/// Resolve the ripex end-to-end corpus. It lives inside the `tests/` dir as
/// `tests/lang-test/`. Fall back to an absolute path so the test is runnable
/// from CI or a different cwd.
fn corpus_root() -> PathBuf {
    let relative = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("lang-test");
    if relative.exists() {
        return relative;
    }
    PathBuf::from("E:/astraive/repo/ripex/tests/lang-test")
}

fn lang_for_ext(ext: &str) -> Option<&'static str> {
    match ext {
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "ts" | "tsx" | "mts" | "cts" => Some("typescript"),
        "py" => Some("python"),
        "go" => Some("go"),
        "rs" => Some("rust"),
        "c" | "h" => Some("c"),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some("cpp"),
        "cs" => Some("csharp"),
        _ => None,
    }
}

#[derive(Default, Clone)]
struct LangStat {
    files: usize,
    ok: usize,
    errors: usize,
    panics: usize,
    hangs: usize,
    facts: usize,
}

#[derive(Clone, Copy)]
enum Outcome {
    Ok,
    Err,
    Panic,
    Hang,
}

struct FileResult {
    rel: String,
    lang: String,
    outcome: Outcome,
    error_count: usize,
    facts: usize,
}

fn walkdir(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    stack.push(p);
                } else {
                    out.push(p);
                }
            }
        }
    }
    out
}

/// Parse + extract one file inside a watchdog thread. Returns the outcome plus
/// error/fact counts. A hang is bounded by `PER_FILE_TIMEOUT_MS`.
fn try_file(path: &Path, lang_static: &'static str, ext: &str) -> Option<FileResult> {
    let src = std::fs::read_to_string(path).ok()?;
    if src.len() > 3_000_000 {
        return None; // defensive: above the documented input-size panic threshold
    }

    let (tx, rx) = mpsc::channel();
    let lang = lang_static.to_string();
    let lang_for_file = lang.clone();
    let ext = ext.to_string();
    std::thread::spawn(move || {
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let parser = match parser_for_ext(&lang, &ext) {
                Some(p) => p,
                None => return,
            };
            let pr: ParseResult = parser.parse(&src);
            let ex: ExtractionResult = parser.extract(&pr);
            let facts = ex.symbols.len() + ex.imports.len() + ex.calls.len() + ex.variables.len();
            let _ = tx.send((pr.errors.len(), facts));
        }));
    });

    let full_rel = path.to_string_lossy().replace('\\', "/");

    match rx.recv_timeout(Duration::from_millis(PER_FILE_TIMEOUT_MS)) {
        Ok((error_count, facts)) => {
            let outcome = if error_count == 0 {
                Outcome::Ok
            } else {
                Outcome::Err
            };
            Some(FileResult {
                rel: full_rel,
                lang: lang_for_file.clone(),
                outcome,
                error_count,
                facts,
            })
        }
        Err(mpsc::RecvTimeoutError::Timeout) => Some(FileResult {
            rel: full_rel,
            lang: lang_for_file.clone(),
            outcome: Outcome::Hang,
            error_count: 0,
            facts: 0,
        }),
        Err(mpsc::RecvTimeoutError::Disconnected) => Some(FileResult {
            rel: full_rel,
            lang: lang_for_file.clone(),
            outcome: Outcome::Panic,
            error_count: 0,
            facts: 0,
        }),
    }
}

#[test]
fn end_to_end_ripex_lang_test() {
    let root = corpus_root();
    assert!(root.exists(), "corpus not found at {}", root.display());

    let mut stat: BTreeMap<String, LangStat> = BTreeMap::new();
    let mut files: Vec<FileResult> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    let mut total_panics = 0;
    let mut total_hangs = 0;

    for path in walkdir(&root) {
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e,
            None => continue,
        };
        let lang = match lang_for_ext(ext) {
            Some(l) => l,
            None => {
                skipped.push(path.to_string_lossy().into_owned());
                continue;
            }
        };
        if let Some(fr) = try_file(&path, lang, ext) {
            let s = stat.entry(lang.to_string()).or_default();
            s.files += 1;
            s.facts += fr.facts;
            match fr.outcome {
                Outcome::Ok => s.ok += 1,
                Outcome::Err => s.errors += fr.error_count,
                Outcome::Panic => {
                    s.panics += 1;
                    total_panics += 1;
                }
                Outcome::Hang => {
                    s.hangs += 1;
                    total_hangs += 1;
                }
            }
            files.push(fr);
        }
    }

    // ---- Report ----
    println!("\n=== ripex end-to-end corpus report ===");
    println!("corpus: {}", root.display());
    println!(
        "{:<11} {:>5} {:>4} {:>7} {:>6} {:>5} {:>6}",
        "lang", "files", "ok", "errors", "panics", "hangs", "facts"
    );
    println!("{}", "-".repeat(46));
    let mut files_total = 0;
    let mut ok_total = 0;
    let mut errors_total = 0;
    let mut facts_total = 0;
    for (lang, s) in &stat {
        println!(
            "{:<11} {:>5} {:>4} {:>7} {:>6} {:>5} {:>6}",
            lang, s.files, s.ok, s.errors, s.panics, s.hangs, s.facts
        );
        files_total += s.files;
        ok_total += s.ok;
        errors_total += s.errors;
        facts_total += s.facts;
    }
    println!("{}", "-".repeat(46));
    println!(
        "{:<11} {:>5} {:>4} {:>7} {:>6} {:>5} {:>6}",
        "TOTAL", files_total, ok_total, errors_total, total_panics, total_hangs, facts_total
    );
    if !skipped.is_empty() {
        println!("\nskipped (no ripex parser): {} files", skipped.len());
    }

    if total_hangs > 0 {
        println!(
            "\n--- HANGING files ({}): these infinite-loop in ripex ---",
            total_hangs
        );
        for f in files.iter().filter(|f| matches!(f.outcome, Outcome::Hang)) {
            println!("  [{}] {}", f.lang, f.rel);
        }
    }
    if total_panics > 0 {
        println!(
            "\n--- PANICKING files ({}): ripex crashed ---",
            total_panics
        );
        for f in files.iter().filter(|f| matches!(f.outcome, Outcome::Panic)) {
            println!("  [{}] {}", f.lang, f.rel);
        }
    }

    println!("\npanics={} hangs={}", total_panics, total_hangs);

    // ---- Assertions (regression gate) ----
    assert!(
        !files.is_empty(),
        "no corpus files were parsed — corpus path or parsers misconfigured"
    );
    // Hard requirement 1: ripex must never panic (crash) on real-world source.
    assert_eq!(
        total_panics, 0,
        "ripex panicked on {} file(s)",
        total_panics
    );

    // Hard requirement 2: ripex must never infinite-loop (hang) on real-world
    // source. All four originally-hanging files (Rust macro/generics, TS
    // interface members, C++ template params) were fixed; any hang is now a
    // regression and fails the suite.
    assert_eq!(total_hangs, 0, "ripex hung on {} file(s)", total_hangs);

    // Facts must actually be extracted across the corpus.
    assert!(
        facts_total > 0,
        "no facts extracted at all — extraction pipeline is broken"
    );

    // Languages that currently cover the complete checked-in corpus must stay
    // clean. Other parsers have explicit non-increasing budgets so coverage
    // work cannot silently regress while their remaining syntax is completed.
    let error_budgets = [
        ("c", 36usize),
        ("cpp", 104),
        ("csharp", 105),
        ("go", 0),
        ("javascript", 0),
        ("python", 0),
        ("rust", 0),
        ("typescript", 20),
    ];
    for (language, budget) in error_budgets {
        let actual = stat.get(language).map_or(0, |value| value.errors);
        assert!(
            actual <= budget,
            "{language} emitted {actual} diagnostics; regression budget is {budget}"
        );
    }
}
