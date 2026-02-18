# Phase 20a: For-In Loops - Frontend (Lexer + Parser + AST)

## 🎯 Scope: Lexer, AST, Parser Only

**What THIS phase does:** Add `for...in` syntax to lexer, AST, and parser
**Depends on:** Phase-19 complete (parser features enabled)
**Estimated time:** 2-3 hours

---

## 🚨 DEPENDENCIES

**REQUIRED:** Phase-19 complete (match/import enabled)
**BLOCKS:** Phases 20b, 20c, 20d

**Verify dependency:**
```bash
# Verify match/import work
echo 'fn test() -> number { match 5 { 5 => 100, _ => 0 } } test();' > /tmp/test.atl
atlas run /tmp/test.atl
# Should output: 100
```

---

## Objective

Add `for...in` syntax to the frontend of the compiler. This phase adds the lexer token, AST node, and parser logic. No semantic analysis or execution yet - just parsing.

**Syntax:**
```atlas
for item in array {
    print(item);
}
```

**Desugaring happens in phase-20b** (typechecker), execution in 20c.

---

## Design

**AST Node:**
```rust
pub enum Stmt {
    // ... existing ...
    ForIn {
        variable: String,      // "item"
        iterable: Expr,        // array expression
        body: Vec<Stmt>,       // loop body
        span: Span,
    },
}
```

**Lexer Token:**
```rust
pub enum TokenKind {
    // ... existing ...
    In,  // "in" keyword
}
```

**Parser Logic:**
```rust
// for item in array { body }
//  ^    ^   ^   ^     ^
//  |    |   |   |     body
//  |    |   |   iterable expr
//  |    |   In keyword
//  |    variable name
//  For keyword
```

---

## Files

**Update:** `crates/atlas-runtime/src/lexer.rs` (~10 lines)
**Update:** `crates/atlas-runtime/src/ast.rs` (~15 lines)
**Update:** `crates/atlas-runtime/src/parser/stmt.rs` (~50 lines)
**Create:** `crates/atlas-runtime/tests/test_for_in_parsing.rs` (~100 lines)

---

## Implementation

### GATE -1: Sanity Check ✅

```bash
cargo clean
cargo check -p atlas-runtime
# Must pass before starting
```

---

### GATE 0: Study Existing For Loop

**Read current for loop implementation:**
```bash
grep -A 30 "TokenKind::For =>" crates/atlas-runtime/src/parser/stmt.rs
```

**Understand:**
- How `parse_for_stmt()` works
- How to parse `(init; cond; update)` pattern
- We'll use similar structure for `for...in`

**Acceptance:**
- ✅ Understand existing for loop parsing
- ✅ Ready to add for-in variant

---

### GATE 1: Add `In` Token to Lexer

**File:** `crates/atlas-runtime/src/lexer.rs`

**Find the keyword matching:**
```rust
// Look for where "for" keyword is defined
// Add "in" near it
```

**Add:**
```rust
"in" => TokenKind::In,
```

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ TokenKind::In added
- ✅ Lexer compiles

---

### GATE 2: Add ForIn to AST

**File:** `crates/atlas-runtime/src/ast.rs`

**Find Stmt enum, add variant:**
```rust
pub enum Stmt {
    // ... existing variants ...

    ForIn {
        variable: String,      // Loop variable name
        iterable: Box<Expr>,   // Expression to iterate over
        body: Vec<Stmt>,       // Loop body
        span: Span,
    },
}
```

**Update Display impl if it exists** (for debugging):
```rust
impl Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // ... existing ...
            Stmt::ForIn { variable, .. } => write!(f, "for {} in ...", variable),
        }
    }
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ ForIn variant added to Stmt
- ✅ AST compiles
- ✅ Will break binder/typechecker/interpreter/VM (expected - phase 20b/c fix)

---

### GATE 3: Implement Parser for For-In

**File:** `crates/atlas-runtime/src/parser/stmt.rs`

**Find the `TokenKind::For` case in `parse_stmt()` or wherever it's handled.**

**Current:**
```rust
TokenKind::For => {
    // Parses: for (init; cond; update) { body }
}
```

**Update to handle both patterns:**

```rust
TokenKind::For => {
    self.advance(); // consume 'for'

    // Check if it's for-in or traditional for
    if self.peek() == TokenKind::LeftParen {
        // Traditional: for (init; cond; update) { body }
        self.parse_for_stmt()
    } else {
        // For-in: for item in array { body }
        self.parse_for_in_stmt()
    }
}
```

**Add new method:**
```rust
fn parse_for_in_stmt(&mut self) -> Result<Stmt, ()> {
    let start = self.current_span();

    // Parse variable name
    let variable = if let TokenKind::Identifier(name) = &self.current().kind {
        let var_name = name.clone();
        self.advance();
        var_name
    } else {
        self.error("Expected variable name after 'for'");
        return Err(());
    };

    // Expect 'in' keyword
    if !self.match_token(TokenKind::In) {
        self.error("Expected 'in' after variable name");
        return Err(());
    }

    // Parse iterable expression
    let iterable = Box::new(self.parse_expression()?);

    // Expect '{'
    if !self.match_token(TokenKind::LeftBrace) {
        self.error("Expected '{' after for-in header");
        return Err(());
    }

    // Parse body
    let mut body = Vec::new();
    while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
        body.push(self.parse_stmt()?);
    }

    // Expect '}'
    if !self.match_token(TokenKind::RightBrace) {
        self.error("Expected '}' after for-in body");
        return Err(());
    }

    let span = start.extend_to(self.previous_span());
    Ok(Stmt::ForIn { variable, iterable, body, span })
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib 2>&1 | head -20
```

**Acceptance:**
- ✅ Parser compiles
- ✅ Will have errors in binder/typechecker/interpreter/VM (expected)

---

### GATE 4: Create Parsing Tests

**Create:** `crates/atlas-runtime/tests/test_for_in_parsing.rs`

```rust
use atlas_runtime::{Parser, Lexer};

#[test]
fn test_parse_for_in_basic() {
    let source = r#"
        for item in array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    assert!(ast.is_ok(), "Should parse for-in loop");
}

#[test]
fn test_parse_for_in_with_array_literal() {
    let source = r#"
        for x in [1, 2, 3] {
            print(x);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    assert!(ast.is_ok(), "Should parse for-in with array literal");
}

#[test]
fn test_parse_for_in_empty_body() {
    let source = r#"
        for x in arr {
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    assert!(ast.is_ok(), "Should parse for-in with empty body");
}

#[test]
fn test_parse_for_in_error_missing_in() {
    let source = r#"
        for item array {
            print(item);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    assert!(ast.is_err(), "Should error without 'in' keyword");
}

#[test]
fn test_traditional_for_still_works() {
    let source = r#"
        for (let i = 0; i < 10; i = i + 1) {
            print(i);
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    assert!(ast.is_ok(), "Traditional for loops should still work");
}
```

**Test:**
```bash
cargo test -p atlas-runtime test_parse_for_in -- --exact
```

**Acceptance:**
- ✅ All parsing tests pass
- ✅ For-in syntax parses correctly
- ✅ Traditional for loops still work
- ✅ Error cases handled

---

### GATE 5: Stub Binder (Temporary)

**File:** `crates/atlas-runtime/src/binder.rs`

**Add temporary stub so binder compiles:**
```rust
Stmt::ForIn { variable, iterable, body, .. } => {
    // TODO: Phase-20b will implement this properly
    // For now, just bind the iterable and body
    self.bind_expr(iterable);
    for stmt in body {
        self.bind_stmt(stmt);
    }
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ Binder compiles (stub added)
- ✅ Still errors in typechecker/interpreter/VM (expected)

---

### GATE 6: Stub TypeChecker (Temporary)

**File:** `crates/atlas-runtime/src/typechecker/mod.rs`

**Add temporary stub:**
```rust
Stmt::ForIn { .. } => {
    // TODO: Phase-20b will implement desugaring
    Err(TypeError::new(TypeErrorKind::NotImplemented,
        "for-in loops not yet implemented", span))
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ TypeChecker compiles (stub returns error)
- ✅ Still errors in interpreter/VM (expected)

---

### GATE 7: Stub Interpreter (Temporary)

**File:** `crates/atlas-runtime/src/interpreter/mod.rs`

**Add temporary stub:**
```rust
Stmt::ForIn { .. } => {
    return Err(RuntimeError::new(
        ErrorCode::AT9999,
        "for-in loops not yet implemented in interpreter",
    ));
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib
```

**Acceptance:**
- ✅ Interpreter compiles (stub returns error)

---

### GATE 8: Stub Compiler/VM (Temporary)

**File:** `crates/atlas-runtime/src/compiler/mod.rs`

**Add temporary stub:**
```rust
Stmt::ForIn { .. } => {
    return Err(CompileError::new(
        "for-in loops not yet implemented in VM",
    ));
}
```

**Test:**
```bash
cargo check -p atlas-runtime --lib
cargo check -p atlas-runtime
```

**Acceptance:**
- ✅ Compiler compiles (stub returns error)
- ✅ VM compiles (no changes needed)
- ✅ ENTIRE CRATE compiles

---

### GATE 9: Clippy & Format

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

1. ✅ TokenKind::In added to lexer
2. ✅ Stmt::ForIn added to AST
3. ✅ parse_for_in_stmt() implemented
4. ✅ For-in syntax parses correctly
5. ✅ Traditional for loops still work
6. ✅ Parsing tests pass
7. ✅ Binder/typechecker/interpreter/VM have stubs (compile but don't execute)
8. ✅ Entire crate compiles
9. ✅ Zero clippy warnings

**DO NOT:**
- ❌ Implement desugaring (that's phase-20b)
- ❌ Implement execution (that's phase-20c)
- ❌ Test execution (that's phase-20d)

---

## Handoff

**Commit message:**
```
feat(parser): Add for-in loop syntax to frontend (phase 20a)

Part 1/4 of for-in loop implementation.

**Changes:**
- Lexer: Added TokenKind::In
- AST: Added Stmt::ForIn variant
- Parser: Implemented parse_for_in_stmt()
- Tests: Parsing tests pass

**Syntax:**
```atlas
for item in array {
    print(item);
}
```

**Status:** Parses correctly, execution not yet implemented

**Next:** Phase-20b (binder + typechecker desugaring)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
```

**Update STATUS.md:**
- Foundation: Mark "Phase-20a: For-In Loops - Frontend (Complete)"
- Note: "1/4 sub-phases complete"

---

## Notes

**Why frontend only?**
- Clear separation: parsing vs semantic analysis vs execution
- Can test parsing independently
- Compiler still compiles (stubs added)
- ~160 lines of new code (manageable)

**What's stubbed:**
- Binder: Just binds children (no scope for loop variable yet)
- TypeChecker: Returns "not implemented" error
- Interpreter: Returns runtime error
- Compiler: Returns compile error

**Phase 20b will:**
- Implement proper binder logic (add loop variable to scope)
- Implement desugaring in typechecker (ForIn → For)
- Remove stubs

**Time estimate:** 2-3 hours
