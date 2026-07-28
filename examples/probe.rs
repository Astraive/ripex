use ripex::parser_for_ext;

fn main() {
    for (lang, ext, src) in [
        ("rust", "rs", "pub fn main() {}\nstruct Foo { x: i32 }\n"),
        ("python", "py", "from os import path\nimport sys\n"),
        (
            "typescript",
            "ts",
            "interface Foo { bar: number; }\nexport const x: number = 1;\n",
        ),
    ] {
        let p = parser_for_ext(lang, ext).expect("parser");
        let r = p.parse(src);
        println!(
            "[{lang}] errors={:?}",
            r.errors.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
        let f = p.extract(&r).expect("fixture should parse completely");
        println!(
            "[{lang}] symbols={} imports={} calls={} vars={}",
            f.symbols.len(),
            f.imports.len(),
            f.calls.len(),
            f.variables.len()
        );
        for s in &f.symbols {
            println!("   sym {} {:?} @{}", s.name, s.kind, s.line_start);
        }
    }
}
