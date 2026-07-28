#![no_main]

use libfuzzer_sys::fuzz_target;
use ripex::{parser_for, LanguageParser};

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    if let Some(parser) = parser_for("go") {
        let result = parser.parse(&source);
        let _ = parser.extract(&result);
    }
});
