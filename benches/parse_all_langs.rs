//! Throughput benchmarks for every language parser, exercised against real
//! files from the checked-in `tests/lang-test` corpus. Run with:
//!
//! ```sh
//! cargo bench --bench parse_all_langs
//! ```
//!
//! The goal these benchmarks serve: prove Ripex parses real-world source
//! faster than the tree-sitter fallback it replaces in Graxus, and track
//! per-language throughput regressions over time.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ripex::parser_for_ext;

// Real corpus samples (paths are relative to the ripex crate root).
const JS: &str = include_str!("../tests/lang-test/javascript/src/index.js");
const TS: &str = include_str!("../tests/lang-test/javascript/src/types.ts");
const PY: &str = include_str!("../tests/lang-test/python/src/main.py");
const RS: &str = include_str!("../tests/lang-test/rust/src/lib.rs");
const GO: &str = include_str!("../tests/lang-test/go/main.go");
const C: &str = include_str!("../tests/lang-test/c/utils/math.c");
const CPP: &str = include_str!("../tests/lang-test/cpp/utils/math.hpp");
const CS: &str = include_str!("../tests/lang-test/csharp/Utils/Strings.cs");

fn bench_lang(c: &mut Criterion, name: &str, ext: &str, src: &str) {
    let mut group = c.benchmark_group(format!("parse/{name}"));
    group.throughput(Throughput::Bytes(src.len() as u64));
    group.bench_with_input(BenchmarkId::from_parameter("parse"), &src, |b, src| {
        b.iter(|| {
            // `parser_for_ext` performs parser selection followed by lexing
            // and parsing, matching the public library entry point.
            parser_for_ext(name, ext).map(|p| p.parse(src));
        });
    });
    group.finish();
}

fn parse_all_langs(c: &mut Criterion) {
    bench_lang(c, "js", "js", JS);
    bench_lang(c, "ts", "ts", TS);
    bench_lang(c, "python", "py", PY);
    bench_lang(c, "rust", "rs", RS);
    bench_lang(c, "go", "go", GO);
    bench_lang(c, "c", "c", C);
    bench_lang(c, "cpp", "cpp", CPP);
    bench_lang(c, "csharp", "cs", CS);
}

criterion_group!(benches, parse_all_langs);
criterion_main!(benches);
