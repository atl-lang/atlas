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
pub fn run(file_path: &str, disasm: bool) -> Result<()> {
    // Read source file
    let source = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file: {}", file_path))?;

    // Compile to bytecode
    let bytecode = compile_source(&source, file_path)?;

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
fn compile_source(source: &str, _file_path: &str) -> Result<Bytecode> {
    // Lex the source code
    let mut lexer = Lexer::new(source);
    let (tokens, lex_diagnostics) = lexer.tokenize();

    if !lex_diagnostics.is_empty() {
        return Err(anyhow::anyhow!(
            "Lexer errors:\n{}",
            format_diagnostics(&lex_diagnostics)
        ));
    }

    // Parse tokens into AST
    let mut parser = Parser::new(tokens);
    let (ast, parse_diagnostics) = parser.parse();

    if !parse_diagnostics.is_empty() {
        return Err(anyhow::anyhow!(
            "Parse errors:\n{}",
            format_diagnostics(&parse_diagnostics)
        ));
    }

    // Bind symbols
    let mut binder = Binder::new();
    let (symbol_table, bind_diagnostics) = binder.bind(&ast);

    if !bind_diagnostics.is_empty() {
        return Err(anyhow::anyhow!(
            "Binding errors:\n{}",
            format_diagnostics(&bind_diagnostics)
        ));
    }

    // Type check
    let mut typechecker = TypeChecker::new(&symbol_table);
    let typecheck_diagnostics = typechecker.check(&ast);

    if !typecheck_diagnostics.is_empty() {
        return Err(anyhow::anyhow!(
            "Type errors:\n{}",
            format_diagnostics(&typecheck_diagnostics)
        ));
    }

    // Compile to bytecode
    let mut compiler = Compiler::new();
    let bytecode = compiler
        .compile(&ast)
        .map_err(|diagnostics| {
            anyhow::anyhow!(
                "Compilation errors:\n{}",
                format_diagnostics(&diagnostics)
            )
        })?;

    Ok(bytecode)
}

/// Format diagnostics for display
fn format_diagnostics(diagnostics: &[atlas_runtime::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|d| format!("{:?}", d))
        .collect::<Vec<_>>()
        .join("\n")
}
