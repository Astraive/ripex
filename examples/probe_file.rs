use ripex::parser_for_ext;
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let p = std::path::Path::new(&path);
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
        _ => return,
    };
    let src = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return,
    };
    let parser = match parser_for_ext(lang, lexr) {
        Some(p) => p,
        None => return,
    };
    let parsed = parser.parse(&src);
    let _facts = parser.extract(&parsed);
    println!("OK {} (errs={})", path, parsed.errors.len());
}
