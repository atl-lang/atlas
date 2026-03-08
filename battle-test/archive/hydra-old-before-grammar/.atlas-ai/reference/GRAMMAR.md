# Atlas Grammar Reference

**Source**: Atlas Specification (`~/dev/projects/atlas/docs/specification/syntax.md`)
**Version**: v0.2 | **Updated**: 2026-03-03

---

## File Format

- Extension: `.atl`
- Encoding: UTF-8
- Newlines: LF or CRLF

---

## Keywords

```
let  mut  var  fn  if  else  while  for  return  break  continue
true  false  null  match  import  export  from  as  trait  impl  in
struct  enum
```

---

## Types

### Primitive Types
| Type | Description |
|------|-------------|
| `number` | 64-bit float (IEEE 754) |
| `string` | UTF-8 string |
| `bool` | `true` or `false` |
| `null` | Null value |
| `void` | No value (function return) |
| `json` | JSON value |

### Composite Types
| Syntax | Description |
|--------|-------------|
| `T[]` | Array of T |
| `Option<T>` | Optional value |
| `Result<T, E>` | Success or error |
| `HashMap` | Key-value map |
| `HashSet` | Unique value set |
| `(T1, T2) -> R` | Function type |

### User-Defined Types (Coming Soon)
```atlas
struct User { name: string, age: number }
enum State { Stopped, Running, Failed }
```

---

## Literals

### Numbers
```atlas
42          // Integer
3.14        // Decimal
1e10        // Scientific
-5          // Negative
```

### Strings
```atlas
"hello"
"line1\nline2"    // \n newline
"quote: \""       // \" escaped quote
"path: C:\\dir"   // \\ backslash
```

### Booleans & Null
```atlas
true
false
null
```

### Arrays
```atlas
[1, 2, 3]
["a", "b"]
[]              // Empty (needs type context)
```

---

## Variables

### Immutable (recommended)
```atlas
let x: number = 42;
let name = "Atlas";     // Type inferred
```

### Mutable
```atlas
let mut count: number = 0;
var legacy = "old";     // Deprecated, use let mut
```

---

## Functions

### Declaration
```atlas
fn add(a: number, b: number) -> number {
    return a + b;
}

// Return type inferred
fn greet(name: string) {
    print("Hello " + name);
}

// Generic function
fn identity<T>(x: T) -> T {
    return x;
}
```

### Anonymous Functions
```atlas
// fn expression
let double = fn(x: number) -> number { return x * 2; };

// Arrow function (single expression)
let double = (x) => x * 2;
let add = (a, b) => a + b;
```

### Nested Functions
```atlas
fn outer() -> number {
    fn helper(x: number) -> number {
        return x * 2;
    }
    return helper(21);  // 42
}
```

---

## Control Flow

### If/Else
```atlas
if (condition) {
    // true branch
} else {
    // false branch
}
```

### While Loop
```atlas
while (condition) {
    // body
}
```

### For Loop (C-style)
```atlas
for (let i = 0; i < 10; i++) {
    // body
}
```

### For-In Loop
```atlas
for item in array {
    print(item);
}

for i in 0..10 {
    print(i);
}
```

### Break/Continue
```atlas
break;      // Exit loop
continue;   // Next iteration
```

---

## Match Expressions

```atlas
match value {
    0 => "zero",
    1 => "one",
    _ => "other",     // Wildcard (catch-all)
}

// Option matching
match opt {
    Some(v) => v,
    None() => default,
}

// Result matching
match result {
    Ok(v) => v,
    Err(e) => handle_error(e),
}

// With guards
match n {
    x if x < 0 => "negative",
    x if x == 0 => "zero",
    _ => "positive",
}
```

---

## Operators

### Arithmetic
| Op | Description |
|----|-------------|
| `+` | Add (number or string concat) |
| `-` | Subtract |
| `*` | Multiply |
| `/` | Divide |
| `%` | Modulo |

### Comparison
| Op | Description |
|----|-------------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less or equal |
| `>` | Greater than |
| `>=` | Greater or equal |

### Logical
| Op | Description |
|----|-------------|
| `&&` | AND (short-circuit) |
| `\|\|` | OR (short-circuit) |
| `!` | NOT |

### Assignment
| Op | Description |
|----|-------------|
| `=` | Assign |
| `+=` | Add-assign |
| `-=` | Subtract-assign |
| `*=` | Multiply-assign |
| `/=` | Divide-assign |
| `%=` | Modulo-assign |

### Increment/Decrement (statements only)
```atlas
++i;    // Pre-increment
i++;    // Post-increment
--i;    // Pre-decrement
i--;    // Post-decrement
```

### Try Operator
```atlas
let value = some_result?;  // Propagates Err/None
```

---

## Block Expressions

Blocks can be expressions. The last expression without `;` is the value.

```atlas
let x = {
    let a = 5;
    let b = 10;
    a + b           // No semicolon = block value
};
// x == 15

let y = {
    let a = 5;
    a + 10;         // With semicolon = null
};
// y == null
```

---

## Array Operations

### Access
```atlas
arr[0]          // First element
arr[len(arr)-1] // Last element
```

### Mutation (mutable arrays)
```atlas
arr[0] = value;
```

### Methods
```atlas
push(arr, val)
pop(arr)
map(arr, fn)
filter(arr, fn)
reduce(arr, fn, init)
slice(arr, start, end)
sort(arr)
```

---

## Option & Result

### Option
```atlas
Some(value)     // Has value
None()          // No value

// Match
match opt {
    Some(v) => use(v),
    None() => default,
}

// With ?
let v = opt?;   // Returns None from function if None
```

### Result
```atlas
Ok(value)       // Success
Err(error)      // Error

// Match
match result {
    Ok(v) => use(v),
    Err(e) => handle(e),
}

// With ?
let v = result?;  // Returns Err from function if Err
```

---

## Traits (Advanced)

### Declaration
```atlas
trait Display {
    fn display(self: Display) -> string;
}
```

### Implementation
```atlas
impl Display for number {
    fn display(self: number) -> string {
        return str(self);
    }
}
```

### Bounds
```atlas
fn print_all<T: Display>(items: T[]) -> void {
    for item in items {
        print(item.display());
    }
}
```

---

## Modules

### Export
```atlas
export fn helper() -> void { }
export let CONFIG = 42;
```

### Import
```atlas
import { helper, CONFIG } from "./module.atl";
import * as utils from "./utils.atl";
```

---

## Comments

```atlas
// Single line comment

/*
 * Multi-line
 * comment
 */
```

---

## EBNF Grammar Summary

```ebnf
program     = { stmt } ;
stmt        = var_decl | fn_decl | if_stmt | while_stmt | for_stmt
            | return_stmt | break_stmt | continue_stmt | expr_stmt ;

var_decl    = ("let" ["mut"] | "var") ident [":" type] "=" expr ";" ;
fn_decl     = "fn" ident ["<" type_params ">"] "(" params ")" ["->" type] block ;

if_stmt     = "if" "(" expr ")" block ["else" block] ;
while_stmt  = "while" "(" expr ")" block ;
for_stmt    = "for" "(" init ";" cond ";" step ")" block
            | "for" ident "in" expr block ;

expr        = logic_or ;
logic_or    = logic_and { "||" logic_and } ;
logic_and   = equality { "&&" equality } ;
equality    = comparison { ("==" | "!=") comparison } ;
comparison  = term { ("<" | "<=" | ">" | ">=") term } ;
term        = factor { ("+" | "-") factor } ;
factor      = unary { ("*" | "/" | "%") unary } ;
unary       = ("!" | "-") unary | postfix ;
postfix     = primary { "(" args ")" | "[" expr "]" | "." ident | "?" } ;
primary     = literal | ident | "(" expr ")" | array | match_expr | block ;

match_expr  = "match" expr "{" { pattern "=>" expr "," } "}" ;
pattern     = literal | ident | "_" | ctor_pattern ;

type        = "number" | "string" | "bool" | "void" | "null" | "json"
            | ident ["<" types ">"] | type "[]" | "(" types ")" "->" type ;
```

---

## Quick Reference Table

| Concept | Syntax |
|---------|--------|
| Variable (immut) | `let x = 42;` |
| Variable (mut) | `let mut x = 42;` |
| Function | `fn name(p: T) -> R { }` |
| Arrow fn | `(x) => x * 2` |
| If/else | `if (cond) { } else { }` |
| For-in | `for x in arr { }` |
| Match | `match v { p => e, _ => d }` |
| Option | `Some(v)`, `None()` |
| Result | `Ok(v)`, `Err(e)` |
| Try | `expr?` |
| Array | `[1, 2, 3]` |
| Index | `arr[0]` |
| Call | `fn(args)` |
| Method | `val.method()` |
| Block value | `{ stmts; expr }` |

---

## Common Patterns

### Error Handling
```atlas
fn safe_op() -> Result<number, string> {
    let file = readFile("config.json")?;
    let data = parseJSON(file)?;
    Ok(42)
}
```

### Collection Processing
```atlas
let doubled = map(nums, (x) => x * 2);
let evens = filter(nums, (x) => x % 2 == 0);
let sum = reduce(nums, (acc, x) => acc + x, 0);
```

### HashMap Usage
```atlas
let map = hashMapNew();
hashMapPut(map, "key", "value");
let val = hashMapGet(map, "key");
```

### File Operations
```atlas
let content = readFile("file.txt")?;
writeFile("out.txt", content)?;
createDir("new_dir")?;
```

### JSON Handling
```atlas
let data = parseJSON(json_str)?;
let json_out = toJSON(data);
```
