#![no_main]

use libfuzzer_sys::fuzz_target;
use ripex::registry;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let source = String::from_utf8_lossy(data);
    let parsers = registry();
    for (_id, parser) in parsers {
        let result = parser.parse(&source);
        let _ = parser.extract(&result);
        let _ = parser.extract_best_effort(&result);
    }
});
