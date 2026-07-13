//! Crude bisection probe to locate the JS statement that aborts the process.
//! Run: cargo test -p ripex --test js_crash_probe -- --nocapture --include-ignored

#[test]
#[ignore]
fn probe_index_js() {
    let root = if let Ok(p) = std::env::var("RIPEX_LANG_TEST_ROOT") {
        p
    } else {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../graxus-lang-test").to_string()
    };
    let path = std::path::Path::new(&root).join("javascript/src/index.js");
    let src = std::fs::read_to_string(&path).unwrap();
    let parser = ripex::parser_for("javascript").unwrap();

    // Parse progressively: prefix of N lines.
    let lines: Vec<&str> = src.lines().collect();
    for n in 1..=lines.len() {
        let prefix: String = lines[..n].join("\n") + "\n";
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let r = parser.parse(&prefix);
            let e = parser.extract(&r);
            (r.errors.len(), e.symbols.len())
        }));
        match r {
            Ok((errs, syms)) => {
                eprintln!(
                    "lines {}/{} -> ok errs={} syms={}",
                    n,
                    lines.len(),
                    errs,
                    syms
                );
            }
            Err(_) => {
                eprintln!("PANIC at line {}: {}", n, lines[n - 1]);
                // print previous few lines for context
                for (k, line) in lines.iter().enumerate().take(n).skip(n.saturating_sub(3)) {
                    eprintln!("   prev[{}]: {}", k + 1, line);
                }
                return;
            }
        }
    }
    eprintln!("completed full file without panic");
}
