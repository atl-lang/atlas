# Pattern Matching Design

**Status:** Approved Design (v0.2)
**Last Updated:** 2026-02-13

---

## Objective

Add pattern matching to Atlas for destructuring and conditional logic. Essential for ergonomic `Result<T,E>` and `Option<T>` handling. Enables exhaustiveness checking and safer code.

---

## Syntax

### Match Expression

```atlas
match expression {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression
}
```

**Key points:**
- `match` is an expression (has a value/type)
- Each arm: `pattern => expression`
- Arms separated by commas
- Trailing comma optional
- All arms must return compatible types

### Pattern Types

**Literal patterns:**
```atlas
match x {
    42 => "the answer",
    0 => "zero",
    _ => "something else"
}
```

**Wildcard pattern:**
```atlas
match x {
    _ => "matches anything"
}
```

**Variable binding:**
```atlas
match x {
    value => value + 1  // Binds x to 'value'
}
```

**Constructor patterns (for Result/Option):**
```atlas
match result {
    Ok(value) => value,
    Err(error) => handle_error(error)
}

match option {
    Some(x) => x,
    None => default_value
}
```

**Array patterns (v0.2 - fixed length only):**
```atlas
match arr {
    [] => "empty",
    [x] => "one element",
    [x, y] => "two elements",
    _ => "more elements"
}
```

---

## Semantics

### Exhaustiveness Checking

**Required:** Match must handle all possible values of scrutinee type.

**For enums (Result, Option):**
```atlas
// ✅ Exhaustive - all constructors covered
match result {
    Ok(value) => value,
    Err(error) => error
}

// ❌ Non-exhaustive - missing Err
match result {
    Ok(value) => value
}
```

**For numbers/strings:**
```atlas
// ❌ Non-exhaustive - numbers are infinite
match x {
    0 => "zero",
    1 => "one"
}

// ✅ Exhaustive - wildcard catches all
match x {
    0 => "zero",
    1 => "one",
    _ => "other"
}
```

**For booleans:**
```atlas
// ✅ Exhaustive - both values covered
match flag {
    true => "yes",
    false => "no"
}

// ✅ Also exhaustive - wildcard
match flag {
    true => "yes",
    _ => "no"
}
```

**Algorithm:** Matrix-based exhaustiveness checking (adapted from Rust/OCaml)

### Pattern Matching Order

**Top-to-bottom, first-match-wins:**
```atlas
match x {
    0 => "zero",        // Checked first
    _ => "non-zero",    // Checked second
    42 => "unreachable" // Warning: unreachable pattern
}
```

**Unreachable patterns:** Warning (not error) in v0.2

### Variable Binding

**Patterns bind variables:**
```atlas
match result {
    Ok(value) => value + 1,   // 'value' bound here
    Err(error) => len(error)  // 'error' bound here
}
// 'value' and 'error' not accessible here
```

**Scope:** Bindings scoped to arm expression only

**Shadowing:** Pattern variables shadow outer scope

### Type Checking

**All arms must return compatible type:**
```atlas
// ✅ Both arms return number
match result {
    Ok(value) => value,     // number
    Err(error) => 0         // number
}

// ❌ Type mismatch
match result {
    Ok(value) => value,     // number
    Err(error) => error     // string
}
```

**Match expression type:** Least upper bound of all arm types

**Scrutinee type checking:** Pattern must be compatible with scrutinee

---

## Grammar (EBNF Extension)

```ebnf
(* Match expression *)
match_expr = "match" expr "{" match_arms "}" ;
match_arms = match_arm { "," match_arm } [ "," ] ;
match_arm  = pattern "=>" expr ;

(* Patterns *)
pattern         = literal_pattern
                | wildcard_pattern
                | variable_pattern
                | constructor_pattern
                | array_pattern ;

literal_pattern     = number | string | "true" | "false" | "null" ;
wildcard_pattern    = "_" ;
variable_pattern    = ident ;
constructor_pattern = ident "(" [ pattern_list ] ")" ;
array_pattern       = "[" [ pattern_list ] "]" ;
pattern_list        = pattern { "," pattern } ;

(* Update primary to include match *)
primary = number | string | "true" | "false" | "null" | ident
        | array_literal | "(" expr ")" | match_expr ;
```

---

## Pattern Types Detail

### Literal Patterns

**Match exact values:**
```atlas
match x {
    42 => "number forty-two",
    "hello" => "string hello",
    true => "boolean true",
    null => "null value",
    _ => "something else"
}
```

**Comparison:** Uses `==` semantics (same type, equal value)

### Wildcard Pattern

**Matches anything, binds nothing:**
```atlas
match x {
    _ => "catch all"
}
```

**Usage:** Default case, exhaustiveness filler

### Variable Binding Pattern

**Matches anything, binds to name:**
```atlas
match x {
    value => value + 1  // Binds x to 'value'
}
```

**Always matches** (like wildcard but binds)

### Constructor Patterns

**For Result<T,E>:**
```atlas
match result {
    Ok(value) => "success: " + str(value),
    Err(error) => "error: " + error
}
```

**For Option<T>:**
```atlas
match option {
    Some(value) => value,
    None => default_value
}
```

**Nested constructors:**
```atlas
match result {
    Ok(Some(value)) => value,
    Ok(None) => 0,
    Err(error) => -1
}
```

**Pattern checking:**
- Constructor name must exist
- Arity must match (Ok takes 1 arg, None takes 0)
- Inner pattern must match constructor arg type

### Array Patterns (v0.2 - Basic)

**Fixed-length matching:**
```atlas
match arr {
    [] => "empty",
    [x] => "single: " + str(x),
    [x, y] => "pair: " + str(x) + "," + str(y),
    _ => "longer array"
}
```

**Limitations (v0.2):**
- No rest patterns (`[first, ...rest]`)
- No slice patterns
- Fixed length only

**v0.3+:** Rest patterns for flexible matching

---

## Exhaustiveness Algorithm

**Goal:** Ensure all possible values are handled

**Approach:** Matrix-based algorithm (adapted from Maranget 2007, Rust uses this)

**Simplified algorithm:**
1. Build pattern matrix from match arms
2. Compute "missing patterns" for scrutinee type
3. If missing patterns is empty → exhaustive
4. If missing patterns is non-empty → error

**For Result<T,E>:**
- Constructors: {Ok, Err}
- Missing if either constructor not covered

**For Option<T>:**
- Constructors: {Some, None}
- Missing if either constructor not covered

**For primitives:**
- Infinite values → require wildcard

**For bool:**
- Constructors: {true, false}
- Can enumerate all

---

## Implementation Strategy

### Phase 1: Syntax & Type Checking (1 week)
- Add MatchExpr to AST
- Parser for match syntax
- Parser for all pattern types
- Type checker for match expressions
- Exhaustiveness checking
- Tests: syntax, type checking

### Phase 2: Runtime Execution (1-2 weeks)
- Interpreter: pattern matching execution
- Interpreter: variable binding
- VM: compile match to bytecode (jump table or if-else chain)
- Tests: execution, parity

**Total:** 2-3 weeks (matches BLOCKER 03 estimate)

---

## Examples

### Basic Result Handling

```atlas
fn divide(a: number, b: number) -> Result<number, string> {
    if (b == 0) {
        return Err("division by zero");
    }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(value) => print("Result: " + str(value)),
    Err(error) => print("Error: " + error)
}
```

### Option Handling

```atlas
fn find(arr: number[], target: number) -> Option<number> {
    for (let i = 0; i < len(arr); i++) {
        if (arr[i] == target) {
            return Some(i);
        }
    }
    return None;
}

match find([1, 2, 3], 2) {
    Some(index) => print("Found at index " + str(index)),
    None => print("Not found")
}
```

### Nested Patterns

```atlas
fn parseConfig(json: string) -> Result<Option<Config>, Error> {
    // ... parsing logic
}

match parseConfig(input) {
    Ok(Some(config)) => use_config(config),
    Ok(None) => use_default_config(),
    Err(error) => handle_error(error)
}
```

### Literal Patterns

```atlas
fn describe(x: number) -> string {
    return match x {
        0 => "zero",
        1 => "one",
        2 => "two",
        _ => "many"
    };
}
```

### Array Patterns

```atlas
fn describe_array(arr: number[]) -> string {
    return match arr {
        [] => "empty",
        [x] => "singleton: " + str(x),
        [x, y] => "pair: " + str(x) + ", " + str(y),
        _ => "array with " + str(len(arr)) + " elements"
    };
}
```

---

## Limitations (v0.2)

### No Guard Clauses

**Cannot add conditions to patterns:**
```atlas
// NOT in v0.2:
match x {
    value if value > 0 => "positive",
    value if value < 0 => "negative",
    _ => "zero"
}
```

**v0.3+:** Guards for conditional matching

### No OR Patterns

**Cannot match multiple patterns in one arm:**
```atlas
// NOT in v0.2:
match x {
    0 | 1 | 2 => "small",
    _ => "large"
}
```

**v0.3+:** OR patterns for conciseness

### No Rest Patterns in Arrays

**Cannot match "rest of array":**
```atlas
// NOT in v0.2:
match arr {
    [first, ...rest] => process(first, rest)
}
```

**v0.3+:** Rest patterns for flexible array matching

### No Struct Patterns

**No user-defined structs in v0.2:**
```atlas
// NOT in v0.2 (no structs):
match point {
    Point { x: 0, y: 0 } => "origin",
    Point { x, y } => "at " + str(x) + "," + str(y)
}
```

**v0.3+:** Struct patterns when structs added

---

## Rationale

### Why Rust-Style Match?

**Alternatives considered:**
1. **Switch statement** (C, JavaScript)
   - Pros: Familiar
   - Cons: No exhaustiveness, fall-through bugs, not an expression

2. **Pattern matching with exhaustiveness** (Rust, Swift, OCaml) ✅ **CHOSEN**
   - Pros: Exhaustiveness prevents bugs, expression-based, type-safe
   - Cons: More complex implementation

**Decision:** Exhaustiveness checking is critical for Atlas's type safety goals. AI agents benefit from compiler-enforced safety.

### Why Expression-Based?

**Match as expression (not statement):**
```atlas
// Can use in assignments
let message = match result {
    Ok(value) => "Success",
    Err(error) => "Failure"
};

// Can return directly
return match option {
    Some(value) => value,
    None => default
};
```

**Benefit:** More concise, functional style

### Why Exhaustiveness?

**Forces explicit handling:**
```atlas
// Compiler error if you forget Err case:
match result {
    Ok(value) => value
    // Error: non-exhaustive match, missing Err
}
```

**Prevents bugs:** Can't forget error cases

**AI-friendly:** Explicit is better than implicit

---

## Verification

**After implementation, verify:**
1. ✅ Parse match expressions
2. ✅ All pattern types work
3. ✅ Exhaustiveness checking catches missing cases
4. ✅ Type checking validates arm types
5. ✅ Variable binding works correctly
6. ✅ Interpreter executes matches
7. ✅ VM executes matches
8. ✅ 100% parity between engines
9. ✅ Result and Option patterns work
10. ✅ Error messages clear and helpful

---

## References

**Inspiration:**
- Rust: Match syntax, exhaustiveness, expression-based
- OCaml: Pattern matching algorithm
- Swift: Pattern matching with optionals
- Haskell: Functional pattern matching

**Algorithm:**
- Maranget (2007): "Warnings for pattern matching"
- Rust compiler: Exhaustiveness checking implementation

**Atlas Philosophy:**
- Explicit error handling (no exceptions, use Result)
- Type safety (exhaustiveness prevents bugs)
- AI-optimized (compiler enforces correctness)

---

**Design Status:** ✅ Complete and approved
**Ready for Implementation:** Yes (BLOCKER 03, requires BLOCKER 02 complete)
