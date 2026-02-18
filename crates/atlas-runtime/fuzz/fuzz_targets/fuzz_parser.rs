#![no_main]

use libfuzzer_sys::fuzz_target;

use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;

fuzz_target!(|data: &[u8]| {
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Lex first — if the lexer doesn't panic, feed tokens to the parser.
    let mut lexer = Lexer::new(input);
    let (tokens, _lex_diagnostics) = lexer.tokenize();

    // The parser must NEVER panic regardless of the token stream.
    // It returns (Program, Vec<Diagnostic>) — malformed programs are reported, not crashed on.
    let mut parser = Parser::new(tokens);
    let (_program, _parse_diagnostics) = parser.parse();
});
