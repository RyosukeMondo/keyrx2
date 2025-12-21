#![no_main]

use libfuzzer_sys::fuzz_target;
use keyrx_compiler::parser::Parser;
use std::path::Path;

fuzz_target!(|data: &[u8]| {
    // Convert arbitrary bytes to UTF-8 string, ignoring invalid sequences
    if let Ok(script) = std::str::from_utf8(data) {
        // Create a new parser instance for each fuzz iteration
        let mut parser = Parser::new();

        // Use a dummy path for fuzzing
        let dummy_path = Path::new("fuzz_input.rhai");

        // Try to parse the script
        // The parser should never panic - it should either succeed or return an error
        let _ = parser.parse_string(script, dummy_path);

        // If we get here without panicking, the parser handled the input gracefully
    }
});
