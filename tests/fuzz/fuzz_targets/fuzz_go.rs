#![no_main]

use libfuzzer_sys::fuzz_target;
use ripex::parser_for;

fuzz_target!(|data: &[u8]| {
    if data.len() > ripex::limits::MAX_INPUT_SIZE {
        return;
    }
    let source = String::from_utf8_lossy(data);
    if let Some(parser) = parser_for("go") {
        let result = parser.parse(&source);
        let _ = parser.extract(&result);
        let _ = parser.extract_best_effort(&result);
    }
});
