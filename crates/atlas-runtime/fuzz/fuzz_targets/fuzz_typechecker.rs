#![no_main]

use libfuzzer_sys::fuzz_target;

use atlas_runtime::binder::Binder;
use atlas_runtime::lexer::Lexer;
use atlas_runtime::parser::Parser;
use atlas_runtime::typechecker::TypeChecker;

fuzz_target!(|data: &[u8]| {
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // Full frontend pipeline: lex → parse → bind → typecheck.
    // Every stage must handle arbitrary input without panicking.

    let mut lexer = Lexer::new(input);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    // If lexing produced errors, still try to parse — the parser should handle bad token streams.
    // But if lexing produced zero tokens, there's nothing useful for downstream stages.
    if tokens.is_empty() {
        return;
    }

    let mut parser = Parser::new(tokens);
    let (program, parse_diagnostics) = parser.parse();

    // If there are lex or parse errors, the AST may be incomplete but the binder
    // and typechecker must still not panic on it.
    if !lex_diagnostics.is_empty() || !parse_diagnostics.is_empty() {
        // Still run binding and typechecking on the (possibly malformed) AST.
        // A production compiler must not crash on partially-parsed input.
    }

    let mut binder = Binder::new();
    let (mut symbol_table, _bind_diagnostics) = binder.bind(&program);

    let mut type_checker = TypeChecker::new(&mut symbol_table);
    let _type_diagnostics = type_checker.check(&program);
});
