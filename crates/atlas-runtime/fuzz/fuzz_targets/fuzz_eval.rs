#![no_main]

use libfuzzer_sys::fuzz_target;

use atlas_runtime::Atlas;

fuzz_target!(|data: &[u8]| {
    let input = match std::str::from_utf8(data) {
        Ok(s) => s,
        Err(_) => return,
    };

    // The full Atlas pipeline: lex → parse → bind → typecheck → interpret.
    // This is the most critical fuzz target — it exercises every component.
    //
    // Contract: Atlas::eval() must return Ok(Value) or Err(Vec<Diagnostic>).
    // It must NEVER panic. A host application embedding Atlas cannot tolerate crashes
    // from untrusted input.
    //
    // We use the default (deny-all) security context so the fuzzer cannot trigger
    // filesystem, network, or process operations. This is both safe and realistic —
    // an embedding host would restrict permissions.

    let runtime = Atlas::new();
    let _result = runtime.eval(input);
});
