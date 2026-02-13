//! Build command - compile Atlas source to bytecode

use anyhow::{Context, Result};
use atlas_runtime::{
    bytecode::disassemble,
    Binder, Bytecode, Compiler, Lexer, Parser, TypeChecker,
};
use std::fs;
use std::path::Path;

/// Run the build command
///
/// Compiles an Atlas source file to bytecode (.atb).
/// If `disasm` is true, prints disassembled bytecode to stdout.
/// If `json_output` is true, diagnostics are printed in JSON format.
pub fn run(file_path: &str, disasm: bool, json_output: bool) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Compile to bytecode
    let bytecode = compile_source(&source, file_path, json_output)?;

    // Generate output path (.atl -> .atb)
    let output_path = Path::new(file_path).with_extension("atb");

    // Serialize and write bytecode
    let bytes = bytecode.to_bytes();
    fs::write(&output_path, bytes)
        .with_context(|| format!("Failed to write bytecode file: {:?}", output_path))?;

    println!("Compiled {} -> {:?}", file_path, output_path);

    // Optionally disassemble
    if disasm {
        println!("\n{}", disassemble(&bytecode));
    }

    Ok(())
}

/// Compile source code to bytecode
///
/// Performs full compilation pipeline: lex -> parse -> bind -> typecheck -> compile
/// If `json_output` is true, diagnostics are printed in JSON format.
fn compile_source(source: &str, _file_path: &str, json_output: bool) -> Result<Bytecode> {
    // Lex the source code
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        print_errors(&lex_diagnostics, json_output);
        return Err(anyhow::anyhow!("Lexer errors"));
    }

    // Parse tokens into AST
    let mut parser = Parser::new(tokens);
    let (ast, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        print_errors(&parse_diagnostics, json_output);
        return Err(anyhow::anyhow!("Parse errors"));
    }

    // Bind symbols
    let mut binder = Binder::new();
    let (symbol_table, bind_diagnostics) = binder.bind(&ast);

    if !bind_diagnostics.is_empty() {
        print_errors(&bind_diagnostics, json_output);
        return Err(anyhow::anyhow!("Binding errors"));
    }

    // Type check
    let mut typechecker = TypeChecker::new(&symbol_table);
    let typecheck_diagnostics = typechecker.check(&ast);

    if !typecheck_diagnostics.is_empty() {
        print_errors(&typecheck_diagnostics, json_output);
        return Err(anyhow::anyhow!("Type errors"));
    }

    // Compile to bytecode
    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&ast).map_err(|diagnostics| {
        print_errors(&diagnostics, json_output);
        anyhow::anyhow!("Compilation errors")
    })?;

    Ok(bytecode)
}

/// Print diagnostics in JSON or human-readable format
fn print_errors(diagnostics: &[atlas_runtime::Diagnostic], json_output: bool) {
    if json_output {
        // JSON format to stdout
        for diag in diagnostics {
            println!("{}", diag.to_json_string().unwrap());
        }
    } else {
        // Human-readable format to stderr
        for diag in diagnostics {
            eprintln!("{:?}", diag);
        }
    }
}
