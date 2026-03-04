# Atlas Syntax Specification

**Purpose:** Define Atlas grammar, keywords, and syntax rules.
**Status:** Living document — reflects current v0.3 implementation.

> **v0.3 Current Grammar:** This document describes v0.3 syntax. The following changes from
> v0.2 have been implemented:
> - `var` keyword REMOVED — use `let` (immutable) or `let mut` (mutable)
> - `++`/`--` operators REMOVED — use `+= 1` or `-= 1`
> - C-style `for(init; cond; step)` REMOVED — use `for-in` loops or `while`
> - Arrow functions `() => expr` REMOVED — use `fn(...) { ... }` syntax
> - Object literals now require `record` keyword: `record { key: val }`
> - `if` requires parentheses: `if (condition) { ... }` (no bare `if condition`)
> - `match` arms require commas between cases
> - Anonymous functions and closure capture fully implemented

---

## File Format

- Source files use `.atl` extension
- UTF-8 encoding required
- Newline-agnostic (LF or CRLF)

---

## Lexical Structure

### Whitespace
- Whitespace is insignificant except to separate tokens
- Newlines are statement separators in REPL only
- In files, semicolons terminate simple statements and braces delimit blocks

### Comments

```atlas
// Single-line comment

/*
 * Multi-line comment
 * Can span multiple lines
 */
```

---

## Keywords

### Keywords
`let`, `mut`, `fn`, `if`, `else`, `while`, `for`, `in`, `return`, `break`, `continue`, `true`, `false`, `null`, `match`, `import`, `export`, `from`, `as`, `trait`, `impl`, `struct`, `enum`, `type`, `record`

**Note:** Keywords cannot be used as identifiers

**Removed Keywords (v0.2):**
- `var` — Use `let mut` instead for mutable variables

---

## Literals

### Number Literals

All numbers are 64-bit floating-point (IEEE 754)

```atlas
// Integer form
42
0
-5

// Decimal form
3.14
0.5
-2.7

// Scientific notation
1e10        // 10 billion
1.5e-3      // 0.0015
6.022e23    // Avogadro's number
1e874       // Supports arbitrary exponents
```

**Syntax:** `digit { digit } [ "." digit { digit } ] [ ("e" | "E") ["+" | "-"] digit { digit } ]`

### String Literals

```atlas
"hello"
"world"
""  // Empty string
```

#### String Escapes
- `\"` - Double quote
- `\\` - Backslash
- `\n` - Newline
- `\r` - Carriage return
- `\t` - Tab

**Example:**
```atlas
"Line 1\nLine 2"
"She said \"Hello\""
"C:\\Users\\name"
```

### Boolean Literals

```atlas
true
false
```

### Null Literal

```atlas
null
```

### Array Literals

```atlas
[1, 2, 3]           // number[]
["a", "b", "c"]     // string[]
[true, false]       // bool[]
[]                  // Empty array (requires type context)
```

**Rules:**
- All elements must have the same type
- `[]` not allowed without type context (no implicit empty array)
- Trailing commas not allowed
- **Note:** Empty array workaround if type context is unavailable: `slice([""], 0, 0)` creates an empty array

### Record Literals

Record literals represent anonymous objects with named fields.

```atlas
// Basic record
let user = record { name: "Alice", age: 30 };

// Empty record
let empty = record { };

// Nested records
let data = record {
    user: record { name: "Bob", id: 1 },
    status: "active"
};
```

**Rules:**
- Use the `record` keyword — bare `{ }` is a block, not a record
- Keys are identifiers, not quoted strings
- Values can be any expression
- Records support field assignment: `record.field = value`
- Field assignment updates the existing record binding

**Common Mistake:**
```atlas
// ❌ WRONG: bare braces parse as a block
let obj = { name: "Alice" };  // Syntax error or unexpected behavior

// ✅ CORRECT: use record keyword
let obj = record { name: "Alice" };
```

---

## Expressions

### Operator Precedence (highest to lowest)

1. **Primary:** literals, identifiers, grouping `(expr)`
2. **Call/Index:** `fn(args)`, `arr[index]`
3. **Unary:** `-expr`, `!expr`
4. **Multiplicative:** `*`, `/`, `%`
5. **Additive:** `+`, `-`
6. **Comparison:** `<`, `<=`, `>`, `>=`
7. **Equality:** `==`, `!=`
8. **Logical AND:** `&&`
9. **Logical OR:** `||`

### Arithmetic Operators

```atlas
a + b   // Addition (number + number OR string + string)
a - b   // Subtraction (number only)
a * b   // Multiplication (number only)
a / b   // Division (number only)
a % b   // Modulo (number only)
```

**Type rules:**
- `+` allowed for `number + number` and `string + string` only
- `-`, `*`, `/`, `%` allowed for `number` only

### Comparison Operators

```atlas
a == b  // Equality (requires same type)
a != b  // Inequality (requires same type)
a < b   // Less than (number only)
a <= b  // Less than or equal (number only)
a > b   // Greater than (number only)
a >= b  // Greater than or equal (number only)
```

**Type rules:**
- `==`, `!=` require both operands have the same type
- `<`, `<=`, `>`, `>=` valid for `number` only

### Logical Operators

```atlas
a && b  // Logical AND (short-circuits)
a || b  // Logical OR (short-circuits)
!a      // Logical NOT
```

**Type rules:**
- All operands must be `bool`
- `&&` and `||` are short-circuiting

### Unary Operators

```atlas
-expr   // Negation (number only)
!expr   // Logical NOT (bool only)
```

### Increment/Decrement

**Note:** Increment/decrement operators (`++`, `--`) are **not supported** in v0.3.

Use compound assignment instead:

```atlas
// Instead of: i++
i = i + 1;

// Instead of: ++i
i = i + 1;

// With compound operators
i += 1;    // Equivalent to i = i + 1
j -= 1;    // Equivalent to j = j - 1
```

**Rules:**
- Use `+=` and `-=` for increment/decrement
- Variable must be mutable (`let mut`)

### Grouping

```atlas
(expr)  // Explicit precedence control
```

### Function Calls

```atlas
fnName(arg1, arg2, arg3)
fnName()  // No arguments

// With type arguments
identity<number>(42)
```

### Member Access / Method Calls

```atlas
// Property access
expr.member

// Method calls
arr.length()
str.indexOf("x")
result.unwrap()

// Chained method calls
str.trim().toLowerCase()
```

**Dispatch:** Method calls on values are desugared to trait method dispatch:
- `arr.length()` → `Array::length(arr)`
- `str.indexOf("x")` → `String::indexOf(str, "x")`

**Available methods:** See stdlib documentation for methods on each type.

### Try Operator (`?`)

The try operator propagates `None` or `Err` values, enabling early returns from functions that return `Option<T>` or `Result<T, E>`.

```atlas
fn parse_and_double(s: string) -> Option<number> {
    let n = parseInt(s)?;    // Returns None if parseInt fails
    Some(n * 2)
}

fn read_config(path: string) -> Result<string, string> {
    let content = readFile(path)?;  // Propagates Err if file read fails
    Ok(content)
}
```

**Rules:**
- `?` can only be used inside functions returning `Option<T>` or `Result<T, E>`
- On `None` or `Err(e)`, immediately returns from the enclosing function
- On `Some(v)` or `Ok(v)`, unwraps to `v` and continues

### Block Expressions

Blocks can be used as expressions. The value of a block is determined by its **tail expression** — the last expression without a trailing semicolon.

```atlas
// Block as expression
let x = {
    let a = 5;
    let b = 10;
    a + b           // No semicolon → this is the block's value
};
// x == 15

// With semicolon → returns null
let y = {
    let a = 5;
    a + 10;         // Semicolon → expression statement, not tail
};
// y == null
```

**Rules (Rust semantics):**
- Last item with NO semicolon = tail expression (block returns its value)
- Last item WITH semicolon = statement (block returns `null`)
- Explicit `return` inside block propagates to enclosing function

```atlas
fn example() -> number {
    let result = {
        if (true) {
            return 42;    // Returns from example(), not from block
        }
        0
    };
    result
}
```

### Array Indexing

```atlas
arr[0]      // Access first element
arr[i + 1]  // Index can be any number expression
```

**Rules:**
- Index must be a `number`
- Non-integer indices are runtime errors (`AT0103`)
- Negative indices are out-of-bounds (`AT0006`)
- `1.0` is valid; fractional values (e.g., `1.5`) are not

### JSON Indexing 
```atlas
data["user"]        // String key (object)
data[0]             // Number index (array)
data["user"]["name"] // Chained indexing
```

**Rules:**
- Accepts `string` or `number` index
- Returns `json` type
- Missing keys/invalid indices return `null` (safe)

### Array Semantics

- Array element types are invariant and homogeneous
- Arrays are mutable; element assignment supported
- Array equality is reference identity (not deep equality)

### Structural Types

Structural types describe the required fields (and method signatures) a value must support.

```atlas
// Field-only structural type
type User = { id: number, name: string };

// Structural type with function member
type Logger = { log: (string) -> void };
```

**Rules:**
- Structural types use `{ field: type }` syntax
- At least one member is required (empty `{}` is not allowed)
- Members are named and separated by commas

---

## Statements

### Variable Declaration

```atlas
// Immutable (default)
let x: number = 42;

// Mutable (required for mutations)
let mut y: number = 10;

// Type inference works with both syntaxes
let a = 3.14;      // Immutable, inferred as number
let mut b = "hi";  // Mutable, inferred as string
```

**Rules:**
- `let` declares an immutable variable
- `let mut` declares a mutable variable
- Type can be inferred from initializer
- Initializer required
- `var` keyword is **not supported** — use `let mut` for mutable variables

### Type Alias Declaration

```atlas
// Basic alias
type UserId = number;

// Generic alias
type Box<T> = T[];

// Exported alias
export type Point = { x: number, y: number };
```

**Rules:**
- `type` declarations are module-level (top-level) only
- Alias names must be unique in the module
- Generic parameters are optional: `type Result<T, E> = ...`
- Use `export type` to expose aliases from a module

### Assignment

```atlas
// Simple assignment
name = value;

// Array element assignment
arr[i] = value;

// Compound assignment (mutable variables only)
let mut x = 10;
x += 5;    // Addition
x -= 2;    // Subtraction
x *= 2;    // Multiplication
x /= 4;    // Division
x %= 3;    // Modulo
```

### Function Declaration

```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}

// Generic function fn identity<T>(x: T) -> T {
    return x;
}

// No return value
fn greet(name: string) -> void {
    print("Hello " + name);
}

// Nested function fn outer() -> number {
    fn helper(x: number) -> number {
        return x * 2;
    }
    return helper(21);  // Returns 42
}
```

**Rules:**
- Parameter types must be explicit
- Return type may be omitted — it is inferred from the function body (see [Type Inference](#type-inference))
- Can be declared at top-level or nested within functions/blocks - Nested functions are hoisted within their scope (forward references allowed)
- Nested functions can shadow outer functions and globals
- Nested functions can call sibling functions at the same scope level

See also: [Anonymous Functions](#anonymous-functions) for closure and lambda syntax.

### If Statement

```atlas
if (condition) {
    // true branch
}

if (condition) {
    // true branch
} else {
    // false branch
}
```

**Rules:**
- Condition must be `bool` and **enclosed in parentheses**
- Parentheses are **required** — `if condition {}` is a syntax error
- Braces required (no single-statement if)

### While Loop

```atlas
while (condition) {
    // loop body
}
```

**Rules:**
- Condition must be `bool`
- Braces required

### For Loop (C-style)

**Note:** C-style `for (init; condition; step)` loops are **not supported** in v0.3.

Use `for-in` loops or `while` instead:

```atlas
// ✅ Use for-in for iteration
for item in items {
    // loop body
}

// ✅ Use while for custom control flow
let mut i = 0;
while (i < 10) {
    // loop body
    i += 1;
}
```

**Previous v0.2 syntax (no longer supported):**
```atlas
// ❌ This does NOT work in v0.3
for (let i = 0; i < 10; i++) {
    // Syntax error
}
```

### For-In Loop

```atlas
// Iterate over array elements
for item in array {
    print(item);
}

// With explicit type annotation
for x in [1, 2, 3] {
    print(x);
}

// Nested iteration
for row in matrix {
    for item in row {
        process(item);
    }
}

// With break and continue
for item in items {
    if (item == target) {
        break;
    }
    if (item < 0) {
        continue;
    }
    process(item);
}
```

**Syntax:** `for IDENTIFIER in expression block`

**Type Requirements:**
- Iterable expression must be of type `array`
- Loop variable has type of array elements
- Type is inferred from array element type

**Scope:**
- Loop variable is scoped to the loop body
- Not accessible outside the loop
- Can shadow outer variables

**Control Flow:**
- `break` exits the for-in loop
- `continue` skips to next iteration
- Early `return` from enclosing function works as expected

**Implementation:**
For-in loops iterate directly over array elements without explicit indexing.

### Return Statement

```atlas
return;           // void return
return expr;      // return value
```

**Rules:**
- Must be inside function body
- Type must match function return type

### Break/Continue

```atlas
break;      // Exit loop
continue;   // Skip to next iteration
```

**Rules:**
- Must be inside loop body

### Expression Statement

```atlas
fn();       // Function call
expr;       // Any expression (value discarded)
```

### Match Expression

Pattern matching on values with exhaustive case handling.

```atlas
match value {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression,    // Wildcard catches all remaining cases
}
```

**Example — Literal patterns:**
```atlas
fn describe(n: number) -> string {
    match n {
        0 => "zero",
        1 => "one",
        _ => "many",
    }
}
```

**Example — Option/Result patterns:**
```atlas
fn safe_divide(a: number, b: number) -> string {
    let result = if (b == 0) { None() } else { Some(a / b) };
    match result {
        Some(v) => toString(v),
        None() => "division by zero",
    }
}
```

**Example — With guards:**
```atlas
fn classify(n: number) -> string {
    match n {
        x if x < 0 => "negative",
        x if x == 0 => "zero",
        _ => "positive",
    }
}
```

**Rules:**
- Match arms must be **separated by commas**
- All arms must return the same type
- Wildcard `_` matches any value (use as last arm for exhaustiveness)
- Guards (`if condition`) add extra conditions to patterns
- When used as a statement, add a trailing semicolon: `match x { ... };`

---

## Trait Declarations

### Trait Declaration

```
trait_decl      := "trait" IDENT type_params? "{" trait_method_sig* "}"
trait_method_sig := "fn" IDENT type_params? "(" params ")" "->" type_ref ";"
```

Note the `;` terminator — trait method signatures have no body.

```atlas
trait Display {
    fn display(self: Display) -> string;
}
```

### Impl Blocks

```
impl_block  := "impl" IDENT type_args? "for" IDENT "{" impl_method* "}"
impl_method := "fn" IDENT type_params? "(" params ")" "->" type_ref block
```

```atlas
impl Display for number {
    fn display(self: number) -> string {
        return str(self);
    }
}
```

### Type Parameter Bounds

```
type_params  := "<" type_param ("," type_param)* ">"
type_param   := IDENT (":" trait_bound ("+" trait_bound)*)?
trait_bound  := IDENT
```

```atlas
fn safe_copy<T: Copy>(x: T) -> T {
    return x;
}
```

---

## Grammar (EBNF)

```ebnf
program        = { module_item } ;
module_item    = export_decl | import_decl | type_alias | decl_or_stmt ;           decl_or_stmt   = fn_decl | stmt ;

(* Module system *)
export_decl    = "export" ( fn_decl | var_decl | type_alias ) ;
import_decl    = "import" import_clause "from" string ";" ;
import_clause  = named_imports | namespace_import ;
named_imports  = "{" import_specifiers "}" ;
import_specifiers = import_specifier { "," import_specifier } ;
import_specifier  = ident ;
namespace_import  = "*" "as" ident ;

fn_decl        = "fn" ident [ type_params ] "(" [ params ] ")" "->" type block ;
type_params    = "<" type_param_list ">" ;                           type_param_list = ident { "," ident } ;                              params         = param { "," param } ;
param          = ident ":" type ;
type_alias     = "type" ident [ type_params ] "=" type ";" ;

stmt           = fn_decl | var_decl | assign_stmt | compound_assign_stmt | increment_stmt
               | decrement_stmt | if_stmt | while_stmt | for_stmt
               | return_stmt | break_stmt | continue_stmt | expr_stmt ;

var_decl       = ("let" | "var") ident [ ":" type ] "=" expr ";" ;
assign_stmt    = assign_target "=" expr ";" ;
assign_expr    = assign_target "=" expr ;
assign_target  = ident { "[" expr "]" } ;
compound_assign_stmt = ident compound_op expr ";" ;
compound_op    = "+=" | "-=" | "*=" | "/=" | "%=" ;
increment_stmt = ( "++" ident | ident "++" ) ";" ;
decrement_stmt = ( "--" ident | ident "--" ) ";" ;
if_stmt        = "if" "(" expr ")" block [ "else" block ] ;
while_stmt     = "while" "(" expr ")" block ;
for_stmt       = "for" "(" [ for_init ] ";" [ expr ] ";" [ for_step ] ")" block ;
for_init       = var_decl_no_semi | assign_expr ;
for_step       = assign_expr | compound_assign_expr | increment_expr | decrement_expr ;
compound_assign_expr = ident compound_op expr ;
increment_expr = "++" ident | ident "++" ;
decrement_expr = "--" ident | ident "--" ;
var_decl_no_semi = ("let" | "var") ident [ ":" type ] "=" expr ;
return_stmt    = "return" [ expr ] ";" ;
break_stmt     = "break" ";" ;
continue_stmt  = "continue" ";" ;
expr_stmt      = expr ";" ;

block          = "{" { stmt } "}" ;

expr           = logic_or ;
logic_or       = logic_and { "||" logic_and } ;
logic_and      = equality { "&&" equality } ;
equality       = comparison { ("==" | "!=") comparison } ;
comparison     = term { ("<" | "<=" | ">" | ">=") term } ;
term           = factor { ("+" | "-") factor } ;
factor         = unary { ("*" | "/" | "%") unary } ;
unary          = ("!" | "-") unary | call ;
call           = primary { [ type_args ] "(" [ args ] ")" | "[" expr "]" } ;  (*  type_args *)
type_args      = "<" type_arg_list ">" ;                             type_arg_list  = type { "," type } ;                                 args           = expr { "," expr } ;
array_literal  = "[" [ args ] "]" ;
primary        = number | string | "true" | "false" | "null" | ident
               | array_literal | "(" expr ")" | match_expr ;           (*  match_expr *)

(* Pattern matching *)
match_expr     = "match" expr "{" match_arms "}" ;
match_arms     = match_arm { "," match_arm } [ "," ] ;
match_arm      = pattern "=>" expr ;
pattern        = literal_pattern | wildcard_pattern | variable_pattern
               | constructor_pattern | array_pattern ;
literal_pattern = number | string | "true" | "false" | "null" ;
wildcard_pattern = "_" ;
variable_pattern = ident ;
constructor_pattern = ident "(" [ pattern_list ] ")" ;
array_pattern  = "[" [ pattern_list ] "]" ;
pattern_list   = pattern { "," pattern } ;

type           = primary_type [ "[]" ] | generic_type | function_type | structural_type ;  primary_type   = "number" | "string" | "bool" | "void" | "null" | "json" ; (*  json *)
generic_type   = ident "<" type_arg_list ">" ;                       function_type  = "(" [ type_list ] ")" "->" type ;
structural_type = "{" structural_members "}" ;
structural_members = structural_member { "," structural_member } ;
structural_member = ident ":" type ;
type_list      = type { "," type } ;
ident          = letter { letter | digit | "_" } ;
number         = digit { digit } [ "." digit { digit } ] [ ("e" | "E") ["+" | "-"] digit { digit } ] ;
string         = "\"" { char } "\"" ;
```

---

## Scoping Rules

### Lexical Scoping
- Block scope for `let` and `var`
- Function parameters scoped to function body
- Shadowing allowed in nested scopes

### Redeclaration
- Redeclaring a name in the same scope is a compile-time error
- Shadowing in nested scope is allowed

### Examples

```atlas
let x = 1;
{
    let x = 2;  // OK: shadows outer x
    print(str(x));  // 2
}
print(str(x));  // 1

// Error: redeclaration in same scope
let y = 1;
let y = 2;  // Compile error
```

---

## Identifier Rules

**Syntax:** `letter { letter | digit | "_" }`

**Valid:**
```atlas
x
myVar
_private
user_id
count2
```

**Invalid:**
```atlas
2fast     // Cannot start with digit
my-var    // Hyphens not allowed
fn        // Keywords reserved
```

---

## Anonymous Functions

Anonymous functions (closures) are first-class expressions. Only the `fn` syntax is supported.

### fn Expression

```atlas
fn ( param-list ) -> return-type { body }
fn ( param-list ) { body }          // return type inferred
```

Example:

```atlas
let double = fn(x: number) -> number { return x * 2; };
double(5);  // → 10

let add = fn(x: number, y: number) { return x + y; };
add(3, 4);  // → 7
```

**Parameter types are required** — type inference on closure parameters is not supported.

### Arrow Expression

**Note:** Arrow function syntax `(x) => x * 2` is **not supported** in v0.3.

Use `fn` syntax instead:

```atlas
// ✅ Correct (v0.3)
let double = fn(x: number) { return x * 2; };

// ❌ Not supported (v0.2)
let double = (x) => x * 2;
```

### Function Type Syntax

```
( type-list ) -> type
```

Example: `(number, number) -> number` is a function type taking two numbers and returning a number.

### Closure Capture Semantics

Closures capture outer variables at **creation time** (snapshot semantics):

- `let` bindings (immutable, Copy types): captured by value
- `let mut` bindings: snapshotted at closure creation — outer mutations after creation are not visible inside the closure

```atlas
fn run() -> number {
    let mut x = 5;
    let f = fn() -> number { return x; };
    x = 99;         // Mutation after creation
    return f();     // Returns 5, not 99 (snapshot at creation time)
}
```

### Higher-Order Functions

Anonymous functions work with all stdlib higher-order functions:

```atlas
let doubled = map([1, 2, 3], fn(x: number) { return x * 2; });        // [2, 4, 6]
let evens   = filter([1, 2, 3, 4], fn(x: number) { return x % 2 == 0; });  // [2, 4]
let sum     = reduce([1, 2, 3], fn(acc: number, x: number) { return acc + x; }, 0); // 6
```

---

## Type Inference

Atlas uses local type inference. Full Hindley-Milner is not in scope.

### What is inferred

- **Local variable types:** `let x = 42` → `x: number` (from initializer)
- **Function return types:** `fn f(x: number) { return x * 2; }` → return type `number`
- **Generic type arguments at call sites:** `identity(42)` → `T = number`

### What requires explicit annotation

- Function parameter types (always required)
- Generic type parameters that do not appear in parameter position
- When inference is ambiguous or two branches return different types

### Examples

```atlas
// Variable type inferred from literal
let count = 0;          // count: number
let name = "Atlas";     // name: string

// Return type inferred from body
fn double(x: number) { return x * 2; }    // inferred: -> number
fn greet(s: string) { return s + "!"; }   // inferred: -> string

// Generic type argument inferred at call site
fn identity<T>(x: T) -> T { return x; }
let n = identity(42);  // T inferred as number
```

### Inference failure

When inference cannot determine a type, one of the following errors is emitted:

| Code | Cause |
|------|-------|
| `AT3050` | Function body returns different types on different paths |
| `AT3051` | Generic type parameter only appears in return type — cannot infer |
| `AT3052` | Inferred type is incompatible with actual usage |

Add an explicit annotation to resolve: `fn f(x: number) -> number { ... }`.

---

## Notes

- All syntax is case-sensitive
- Semicolons required for statements in file mode
- REPL mode allows semicolon omission for single expressions
- Unicode identifiers not supported (ASCII only)

---

## v0.3 Breaking Changes Summary

This section summarizes the most common migration issues from v0.2 to v0.3:

### Variable Declaration

| v0.2 | v0.3 | Reason |
|------|------|--------|
| `var x = 5;` | `let mut x = 5;` | `var` keyword removed; use `let mut` |
| `let x = 5;` | `let x = 5;` | No change for immutable |

### Loops

| v0.2 | v0.3 | Reason |
|------|------|--------|
| `for (let i = 0; i < 10; i++)` | `let mut i = 0; while (i < 10) { ... i += 1; }` | C-style `for` removed |
| `for (item in array)` | `for item in array { ... }` | Parentheses removed from for-in |

### Anonymous Functions

| v0.2 | v0.3 | Reason |
|------|------|--------|
| `let f = (x) => x * 2;` | `let f = fn(x: number) { return x * 2; };` | Arrow syntax removed; parameter types required |

### Object Literals

| v0.2 | v0.3 | Reason |
|------|------|--------|
| `let obj = { name: "Alice" };` | `let obj = record { name: "Alice" };` | `record` keyword required |

### Increment/Decrement

| v0.2 | v0.3 | Reason |
|------|------|--------|
| `i++;` | `i += 1;` | `++`/`--` operators removed |
| `++i;` | `i += 1;` | `++`/`--` operators removed |

### Key Rules

1. **`if` requires parentheses:** `if (x > 5) { ... }` — bare conditions not allowed
2. **`record` keyword required:** `record { ... }` for object literals, not `{ ... }`
3. **Match arms need commas:** `match x { 1 => a, 2 => b, _ => c }`
4. **`let mut` only:** All mutable variables declared with `let mut`, not `var`
5. **For-in only:** Use `for item in array { ... }` or `while` loops
6. **Function types required:** Closure parameters need explicit types: `fn(x: number)`

### Important Library Notes

- `hashMapPut(map, key, val)` returns a new map (copy-on-write) — reassign the result: `map = hashMapPut(map, key, val)`
- Array concatenation with `+` is not supported — use `concat()` or `arrayPush()`
- Empty array `[]` requires type context — use `slice([""], 0, 0)` as workaround
