#![no_main]

use libfuzzer_sys::fuzz_target;
use ripex::python;

fuzz_target!(|data: &[u8]| {
    let source = String::from_utf8_lossy(data);
    let (_program, _errors) = python::parser::parse_program(&source);
});
