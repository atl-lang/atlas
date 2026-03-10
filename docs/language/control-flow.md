# Control Flow

This document reflects the actual parser in `crates/atlas-runtime/src/parser/stmt.rs` and `crates/atlas-runtime/src/parser/expr.rs`.

**if / else**
```
if condition { ... } else { ... }
if (condition) { ... } // parentheses are optional
```
- `else if` is supported.
- `if` parses in expression position but does not produce a value (use explicit assignments or returns).

Example (tested):
```atlas
let mut result: number = 0;
if true {
    result = 1;
} else {
    result = 2;
}
```

**while Loops**
```
while condition { ... }
while (condition) { ... }
```

Example (tested):
```atlas
let mut i: number = 0;
while i < 3 {
    i += 1;
}
```

**for-in Loops**
```
for item in iterable { ... }
for (item in iterable) { ... }
```

Example (tested):
```atlas
for n in [1, 2, 3] {
    let x: number = n;
}
```

**match Expressions**
```
match expr {
    pattern => expr,
    pattern if guard => expr;
}
```
- Arms may be separated by commas or semicolons.
- A trailing separator is allowed.
- `match` at statement position does not require a trailing semicolon (consistent with `if`, `while`, and `for`).
- An empty block arm `{}` has type `void`.

Example (tested):
```atlas
let status: Option<number> = Some(1);
let value: number = match status {
    Some(x) => x,
    None => 0,
};
```

**break / continue / return**
```
break;
continue;
return;
return expr;
```

Example (tested):
```atlas
fn first_positive(values: []number) : Option<number> {
    for v in values {
        if v > 0 {
            return Some(v);
        }
    }
    return None;
}
```

