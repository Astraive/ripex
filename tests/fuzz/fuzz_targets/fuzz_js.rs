#![no_main]

use libfuzzer_sys::fuzz_target;
use ripex::js;
use ripex::js::config::ParserOptions;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let opts = ParserOptions::default();
    let (_program, _errors, _arena) = js::parser::parse_program(&source, &opts);
});
