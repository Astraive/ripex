#![no_main]

use libfuzzer_sys::fuzz_target;
use ripex::{parser_for, parser_for_ext, LanguageParser};

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let parsers = [
        parser_for("javascript"),
        parser_for_ext("javascript", "js"),
        parser_for_ext("javascript", "jsx"),
        parser_for_ext("typescript", "ts"),
        parser_for_ext("typescript", "tsx"),
    ];

    for parser in parsers.into_iter().flatten() {
        let result = parser.parse(&source);
        let _ = parser.extract(&result);
    }
});
