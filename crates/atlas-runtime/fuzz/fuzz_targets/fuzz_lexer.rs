#![no_main]

use libfuzzer_sys::fuzz_target;

use atlas_runtime::lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8 — the lexer operates on strings, not raw bytes.
    // Invalid UTF-8 is not an interesting attack surface for a source-code lexer.
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // The lexer must NEVER panic regardless of input.
    // It returns (Vec<Token>, Vec<Diagnostic>) — errors are reported as diagnostics.
    let mut lexer = Lexer::new(input);
    let (_tokens, _diagnostics) = lexer.tokenize();
});
