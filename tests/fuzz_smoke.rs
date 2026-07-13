//! Deterministic fuzz-style test: runs all parsers on random-ish byte strings
//! to check that none panic or overflow.
use ripex::registry;
use std::io::Write;

const SEEDS: &[&str] = &[
    "",
    " ",
    "x",
    "1",
    "\"hello\"",
    "()",
    "{}",
    "[]",
    "/* comment */",
    "// line comment",
    "x + y",
    "fn f() {}",
    "class X {}",
    "def f(): pass",
    "import os",
    "package main",
    "use std::io;",
    "#include <stdio.h>",
    "using namespace std;",
    "public class X {}",
    "0xDEAD",
    "3.14e10",
    "\"unterminated",
    "'single quote'",
    "`template ${expr}`",
    "<<<<<",
    "&&&&&",
    "|||||",
    "====",
    "!====",
    ">>>",
    "\x00\x01\x02",
    "\u{2000}\u{2000}\u{2000}\u{2000}\u{2000}\u{2000}\u{2000}\u{2000}\u{2000}\u{2000}",
    "a b c d e f g h i j k l m n o p",
    "(((())()))",
    "{[({})]}",
    "; ; ; ; ; ; ; ;",
    "...",
    "=>",
    "->",
    "??",
    "?.",
    "||",
    "&&",
    "a::b",
    "+= -= *= /= %= &= |= ^= <<= >>=",
    "for(;;){break;continue;}",
    "if(x){y}else{z}",
    "try{x}catch(e){}finally{}",
    "switch(x){case 1:break;default:break;}",
    "while(1){break;}",
    "do{x}while(1);",
];

#[test]
fn fuzz_smoke_all_languages() {
    let _ = std::io::stdout().lock();

    for (i, &input) in SEEDS.iter().enumerate() {
        let _ = std::io::stdout().write(
            format!(
                "seed #{i} ({:?})...\n",
                input.chars().take(40).collect::<String>()
            )
            .as_bytes(),
        );
        let _ = std::io::stdout().flush();
        let parsers = registry();
        for (lang, parser) in &parsers {
            let _ = std::io::stdout().write(format!("  {lang}...\n").as_bytes());
            let _ = std::io::stdout().flush();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let r = parser.parse(input);
                let _ = parser.extract(&r);
            }));
            assert!(
                result.is_ok(),
                "panic in {lang} parser on seed #{i} (len={}): {:?}",
                input.len(),
                input.chars().take(80).collect::<String>()
            );
        }
    }
}

#[test]
fn fuzz_smoke_random_bytes() {
    let parsers = registry();
    let mut rng = 42u64;

    for n in 0..200 {
        // Generate a "random" byte string of varying length
        let len = (rng % 64) as usize;
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let b = (rng >> 40) as u8;
            bytes.push(b);
        }
        let input = String::from_utf8_lossy(&bytes);

        for (lang, parser) in &parsers {
            let _ = std::io::stdout().write(format!("random #{n} {lang}...\n").as_bytes());
            let _ = std::io::stdout().flush();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let r = parser.parse(&input);
                let _ = parser.extract(&r);
            }));
            assert!(
                result.is_ok(),
                "panic in {lang} parser on random n={n} rng={rng}: len={}",
                input.len()
            );
        }
    }
}
