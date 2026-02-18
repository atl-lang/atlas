# Phase 20b: For-In Loops - Semantic Analysis (Binder + TypeChecker)

## 🎯 Scope: Binder + TypeChecker Desugaring

**What THIS phase does:** Implement binder logic and typechecker desugaring for for-in loops
**Depends on:** Phase-20a complete (frontend parses for-in)
**Estimated time:** 3-4 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** Phase-20a complete (for-in parsing works)
**BLOCKS:** Phases 20c, 20d

**Verify dependency:**
```bash
cargo test -p atlas-runtime test_parse_for_in -- --exact
# All parsing tests must pass
```

---

## Objective

Implement semantic analysis for for-in loops:
1. **Binder:** Add loop variable to scope, bind iterable and body
2. **TypeChecker:** Validate iterable type, desugar to traditional for loop

**Desugaring strategy:**
```atlas
// Source
for item in arr {
    print(item);
}

// Desugared to
{
    let __iter = arr;
    for (let __i = 0; __i < len(__iter); __i = __i + 1) {
        let item = __iter[__i];
        print(item);
    }
}
```

**Why desugar?**
- Reuses existing for loop implementation
- No new runtime semantics needed
- Type checking happens on desugared form
- Interpreter/VM just execute traditional for loops

---

## Design

**Binder changes:**
- Create new scope for for-in body
- Add loop variable to that scope
- Bind iterable expression
- Bind body statements

**TypeChecker changes:**
- Validate iterable is Array type
- Generate unique variable names (__iter, __i)
- Construct desugared AST (Block with for loop)
- Return desugared form (Stmt::ForIn → Stmt::Block with Stmt::For inside)

---

## Files

**Update:** `crates/atlas-runtime/src/binder.rs` (~30 lines)
**Update:** `crates/atlas-runtime/src/typechecker/mod.rs` (~80 lines)
**Create:** `crates/atlas-runtime/tests/test_for_in_semantic.rs` (~150 lines)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime
# Must pass
cargo test -p atlas-runtime test_parse_for_in -- --exact
# Parsing tests must pass
```

---

### GATE 0: Study Existing For Loop Semantics

**Read binder for traditional for:**
```bash
grep -A 20 "Stmt::For {" crates/atlas-runtime/src/binder.rs
```

**Read typechecker for traditional for:**
```bash
grep -A 30 "Stmt::For {" crates/atlas-runtime/src/typechecker/mod.rs
```

**Understand:**
- How scope is created for loop
- How init/cond/update are checked
- We'll reuse this for desugared form

**Acceptance:**
- ✅ Understand existing for loop semantics
- ✅ Ready to implement for-in

---

### GATE 1: Implement Binder for For-In

**File:** `crates/atlas-runtime/src/binder.rs`

**Remove the stub, implement properly:**

```rust
Stmt::ForIn { variable, iterable, body, .. } => {
    // Bind iterable expression in current scope
    self.bind_expr(iterable);

    // Create new scope for loop body
    self.push_scope();

    // Add loop variable to scope (type unknown at binding time)
    self.declare_variable(variable.clone(), None)?;

    // Bind body statements
    for stmt in body {
        self.bind_stmt(stmt)?;
    }

    // Pop loop scope
    self.pop_scope();

    Ok(())
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | grep binder
```

**Acceptance:**
- ✅ Binder compiles
- ✅ No binder errors
- ✅ Loop variable scoped correctly

---

### GATE 2: Implement TypeChecker Desugaring

**File:** `crates/atlas-runtime/src/typechecker/mod.rs`

**Remove the stub, implement desugaring:**

```rust
Stmt::ForIn { variable, iterable, body, span } => {
    // Check iterable is Array type
    let iter_type = self.check_expr(iterable)?;

    if !matches!(iter_type, Type::Array(_)) {
        return Err(TypeError::new(
            TypeErrorKind::TypeMismatch,
            format!("for-in requires array, got {:?}", iter_type),
            *span,
        ));
    }

    // Generate unique variable names
    let iter_var = format!("__iter_{}", self.next_unique_id());
    let index_var = format!("__i_{}", self.next_unique_id());

    // Desugar to:
    // {
    //     let __iter = iterable;
    //     for (let __i = 0; __i < len(__iter); __i = __i + 1) {
    //         let variable = __iter[__i];
    //         body...
    //     }
    // }

    let desugared = Stmt::Block {
        stmts: vec![
            // let __iter = iterable;
            Stmt::Let {
                name: iter_var.clone(),
                type_annotation: Some(iter_type.clone()),
                value: iterable.clone(),
                span: *span,
            },
            // for (let __i = 0; __i < len(__iter); __i = __i + 1)
            Stmt::For {
                init: Some(Box::new(Stmt::Let {
                    name: index_var.clone(),
                    type_annotation: Some(Type::Number),
                    value: Expr::Number(0.0, *span),
                    span: *span,
                })),
                condition: Some(Expr::Binary {
                    op: BinaryOp::Less,
                    left: Box::new(Expr::Variable(index_var.clone(), *span)),
                    right: Box::new(Expr::Call {
                        callee: Box::new(Expr::Variable("len".to_string(), *span)),
                        args: vec![Expr::Variable(iter_var.clone(), *span)],
                        span: *span,
                    }),
                    span: *span,
                }),
                update: Some(Box::new(Stmt::Expr(Expr::Assignment {
                    target: Box::new(Expr::Variable(index_var.clone(), *span)),
                    value: Box::new(Expr::Binary {
                        op: BinaryOp::Add,
                        left: Box::new(Expr::Variable(index_var.clone(), *span)),
                        right: Box::new(Expr::Number(1.0, *span)),
                        span: *span,
                    }),
                    span: *span,
                }))),
                body: {
                    let mut desugared_body = vec![
                        // let variable = __iter[__i];
                        Stmt::Let {
                            name: variable.clone(),
                            type_annotation: None, // Inferred from array element
                            value: Expr::Index {
                                object: Box::new(Expr::Variable(iter_var.clone(), *span)),
                                index: Box::new(Expr::Variable(index_var.clone(), *span)),
                                span: *span,
                            },
                            span: *span,
                        },
                    ];
                    desugared_body.extend(body.clone());
                    desugared_body
                },
                span: *span,
            },
        ],
        span: *span,
    };

    // Type check the desugared form
    self.check_stmt(&desugared)
}
```

**Add helper method if needed:**
```rust
impl TypeChecker {
    fn next_unique_id(&mut self) -> usize {
        let id = self.unique_id_counter;
        self.unique_id_counter += 1;
        id
    }
}
```

**Add field to TypeChecker struct:**
```rust
pub struct TypeChecker {
    // ... existing fields ...
    unique_id_counter: usize,
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ TypeChecker compiles
- ✅ Desugaring logic implemented
- ✅ Array type validation works

---

### GATE 3: Create Semantic Tests

**Create:** `crates/atlas-runtime/tests/test_for_in_semantic.rs`

```rust
use atlas_runtime::{TypeChecker, Binder, Parser, Lexer};

#[test]
fn test_for_in_binds_variable() {
    let source = r#"
        fn test() -> void {
            let arr: array = [1, 2, 3];
            for item in arr {
                print(item);
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut binder = Binder::new();
    let result = binder.bind(&ast);
    assert!(result.is_ok(), "Binder should handle for-in");
}

#[test]
fn test_for_in_type_checks_array() {
    let source = r#"
        fn test() -> void {
            let arr: array = [1, 2, 3];
            for item in arr {
                print(item);
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut binder = Binder::new();
    binder.bind(&ast).unwrap();

    let mut typechecker = TypeChecker::new();
    let result = typechecker.check(&ast);
    assert!(result.is_ok(), "TypeChecker should desugar for-in");
}

#[test]
fn test_for_in_rejects_non_array() {
    let source = r#"
        fn test() -> void {
            let x: number = 5;
            for item in x {
                print(item);
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut binder = Binder::new();
    binder.bind(&ast).unwrap();

    let mut typechecker = TypeChecker::new();
    let result = typechecker.check(&ast);

    assert!(result.is_err(), "Should reject non-array in for-in");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("array"), "Error should mention array");
}

#[test]
fn test_for_in_variable_scoped() {
    let source = r#"
        fn test() -> void {
            let arr: array = [1, 2, 3];
            for item in arr {
                print(item);
            }
            print(item); // Should error - item not in scope
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut binder = Binder::new();
    let result = binder.bind(&ast);

    assert!(result.is_err(), "Variable should not be accessible outside loop");
}

#[test]
fn test_for_in_nested() {
    let source = r#"
        fn test() -> void {
            let matrix: array = [[1, 2], [3, 4]];
            for row in matrix {
                for item in row {
                    print(item);
                }
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut binder = Binder::new();
    binder.bind(&ast).unwrap();

    let mut typechecker = TypeChecker::new();
    let result = typechecker.check(&ast);
    assert!(result.is_ok(), "Should handle nested for-in");
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_for_in_semantic -- --exact
```

**Acceptance:**
- ✅ All semantic tests pass
- ✅ Binder creates proper scopes
- ✅ TypeChecker validates array type
- ✅ TypeChecker desugars correctly
- ✅ Error cases handled

---

### GATE 4: Verify Desugaring Output (Debug)

**Add debug test to see desugared AST:**

```rust
#[test]
fn test_for_in_desugaring_output() {
    let source = r#"
        for item in [1, 2, 3] {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut binder = Binder::new();
    binder.bind(&ast).unwrap();

    let mut typechecker = TypeChecker::new();
    let desugared = typechecker.check(&ast).unwrap();

    // Print desugared AST for verification
    println!("Desugared: {:#?}", desugared);

    // Should contain a Block with Let + For
    // This is a sanity check that desugaring happened
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_for_in_desugaring_output -- --exact --nocapture
```

**Acceptance:**
- ✅ Can see desugared AST structure
- ✅ Contains Let(__iter) + For(__i) pattern
- ✅ Desugaring is correct

---

### GATE 5: Test Integration with Existing Features

**Test for-in works with:**
- Functions
- Type annotations
- Break/continue (if supported - add in 20c if not)
- Nested loops

**Example test:**
```rust
#[test]
fn test_for_in_with_function() {
    let source = r#"
        fn sum(arr: array) -> number {
            let total: number = 0;
            for item in arr {
                total = total + item;
            }
            return total;
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse().unwrap();

    let mut binder = Binder::new();
    binder.bind(&ast).unwrap();

    let mut typechecker = TypeChecker::new();
    let result = typechecker.check(&ast);
    assert!(result.is_ok(), "Should work in function");
}
```

**Acceptance:**
- ✅ Integrates with existing features
- ✅ No regressions

---

### GATE 6: Clippy & Format

```bash
cargo clippy -p atlas-runtime -- -D warnings
cargo fmt -p atlas-runtime
```

**Acceptance:**
- ✅ Zero clippy warnings
- ✅ Code formatted

---

## Acceptance Criteria

**ALL must be met:**

1. ✅ Binder creates scope for loop variable
2. ✅ Binder binds iterable and body
3. ✅ TypeChecker validates array type
4. ✅ TypeChecker desugars to traditional for loop
5. ✅ Desugared AST is correct (Let + For pattern)
6. ✅ All semantic tests pass
7. ✅ Non-array iterables rejected with clear error
8. ✅ Variable scoping correct (not accessible outside loop)
9. ✅ Nested for-in works
10. ✅ Zero clippy warnings

**DO NOT:**
- ❌ Implement interpreter execution (that's phase-20c)
- ❌ Implement VM execution (that's phase-20c)
- ❌ Test actual execution (that's phase-20d)

---

## Handoff

**Commit message:**
```
feat(typechecker): Implement for-in desugaring (phase 20b)

Part 2/4 of for-in loop implementation.

**Changes:**
- Binder: Proper scope handling for loop variable
- TypeChecker: Array validation + desugaring to traditional for loop
- Tests: Semantic analysis tests pass

**Desugaring:**
```atlas
// Source
for item in arr { body }

// Desugared to
{
    let __iter = arr;
    for (let __i = 0; __i < len(__iter); __i + 1) {
        let item = __iter[__i];
        body
    }
}
```

**Status:** Semantic analysis complete, execution not yet implemented

**Next:** Phase-20c (interpreter + VM execution)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-20b: For-In Loops - Semantic (Complete)"
- Note: "2/4 sub-phases complete"

---

## Notes

**Why desugaring?**
- Reuses existing for loop implementation (no new bytecode)
- Type checking on desugared form (no new type rules)
- Simpler implementation (less code, fewer bugs)
- Industry standard (Swift, Rust do similar)

**What's desugared:**
- For-in loops → traditional for loops
- Loop variable → let binding inside loop
- Unique variable names prevent conflicts

**Phase 20c will:**
- Remove stubs in interpreter/compiler
- But they'll just execute the desugared form (already implemented)
- Should be trivial since desugaring is done

**Time estimate:** 3-4 hours (desugaring logic is complex)
