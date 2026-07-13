use ripex::parser_for_ext;
use std::path::Path;
fn try_one(path: &str) -> bool {
    let p = Path::new(path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (lang, lexr) = match ext {
        "rs" => ("rust", ext),
        "py" => ("python", ext),
        "ts" | "tsx" => ("typescript", ext),
        "js" | "jsx" => ("javascript", ext),
        "go" => ("go", ext),
        "c" => ("c", ext),
        "cpp" | "cc" | "hpp" => ("cpp", ext),
        "cs" => ("csharp", ext),
        _ => return true,
    };
    let src = match std::fs::read_to_string(p) {
        Ok(s) => s,
        Err(_) => return true,
    };
    if src.len() > 3_000_000 {
        return true;
    }
    let parser = match parser_for_ext(lang, lexr) {
        Some(p) => p,
        None => return true,
    };
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| parser.parse(&src)));
    match r {
        Ok(_) => true,
        Err(_) => {
            println!("PANIC {}", path);
            false
        }
    }
}
fn main() {
    let mut bad = Vec::new();
    for d in [
        "E:/astraive/repo/graxus/cli",
        "E:/astraive/repo/graxus/crates",
    ] {
        for e in std::fs::read_dir(d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                continue;
            }
            if !try_one(p.to_str().unwrap()) {
                bad.push(p);
            }
        }
    }
    println!("bad files: {}", bad.len());
}
