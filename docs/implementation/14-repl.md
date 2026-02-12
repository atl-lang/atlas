# REPL Core Architecture

UI-agnostic REPL core with persistent state.

## REPL Core Structure

```rust
// repl.rs
use crate::*;

pub struct ReplCore {
    interpreter: Interpreter,
    symbol_table: SymbolTable,
}

pub struct ReplResult {
    pub value: Option<Value>,
    pub diagnostics: Vec<Diagnostic>,
    pub stdout: String,
}

impl ReplCore {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn eval(&mut self, input: &str) -> ReplResult {
        let mut diagnostics = Vec::new();

        // Lex
        let mut lexer = Lexer::new(input.to_string());
        let (tokens, lex_diags) = lexer.tokenize();
        diagnostics.extend(lex_diags);

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Parse
        let mut parser = Parser::new(tokens);
        let (ast, parse_diags) = parser.parse();
        diagnostics.extend(parse_diags);

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Bind
        let mut binder = Binder::new();
        let (symbols, bind_diags) = binder.bind(&ast);
        diagnostics.extend(bind_diags);

        // Merge new symbols into existing table
        // (simplified - actual implementation merges properly)
        self.symbol_table = symbols;

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Typecheck
        let mut typechecker = TypeChecker::new(&self.symbol_table);
        let typecheck_diags = typechecker.check(&ast);
        diagnostics.extend(typecheck_diags);

        if !diagnostics.is_empty() {
            return ReplResult {
                value: None,
                diagnostics,
                stdout: String::new(),
            };
        }

        // Evaluate
        match self.interpreter.eval(&ast) {
            Ok(value) => ReplResult {
                value: Some(value),
                diagnostics,
                stdout: String::new(),  // TODO: Capture stdout
            },
            Err(e) => {
                diagnostics.push(Diagnostic::error(
                    "AT0100",
                    &format!("Runtime error: {:?}", e),
                    DUMMY_SPAN,
                ));
                ReplResult {
                    value: None,
                    diagnostics,
                    stdout: String::new(),
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.interpreter = Interpreter::new();
        self.symbol_table = SymbolTable::new();
    }
}
```

## REPL UI (CLI Layer)

```rust
// atlas-cli/src/commands/repl.rs
use rustyline::Editor;
use atlas_runtime::*;

pub fn run_repl() -> anyhow::Result<()> {
    let mut rl = Editor::<()>::new()?;
    let mut repl = ReplCore::new();

    println!("Atlas v0.1 REPL");
    println!("Type expressions or statements, or :quit to exit");

    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line.trim() == ":quit" || line.trim() == ":q" {
                    break;
                }

                if line.trim() == ":reset" {
                    repl.reset();
                    println!("REPL state reset");
                    continue;
                }

                if line.trim().is_empty() {
                    continue;
                }

                rl.add_history_entry(&line)?;

                let result = repl.eval(&line);

                // Display diagnostics
                if !result.diagnostics.is_empty() {
                    for diag in &result.diagnostics {
                        println!("{}", diag.to_human(&line));
                    }
                }

                // Display value (if expression)
                if result.diagnostics.is_empty() {
                    if let Some(value) = result.value {
                        if !matches!(value, Value::Null) {
                            println!("{}", value.to_string());
                        }
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
                break;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    Ok(())
}
```

## State Persistence

- **Symbol table:** Persists across inputs
- **Interpreter environment:** Persists variables and functions
- **History:** Managed by rustyline

## REPL Commands

- `:quit` or `:q` - Exit REPL
- `:reset` - Clear all state
- `:help` - Show help (future)

## Key Design Decisions

- **Core is UI-agnostic:** Can be used by CLI, TUI, or web frontend
- **Full pipeline every time:** Lex, parse, bind, typecheck, eval
- **State merging:** New definitions merge into existing symbol table
- **Error display:** Diagnostics shown immediately
- **Expression results:** Non-null values are printed automatically
