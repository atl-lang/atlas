# Control Flow

## If Statements

```atlas
if (condition) {
    // do something
}

if (x > 10) {
    print("Greater");
} else {
    print("Smaller");
}
```

**Note**: Parentheses required around condition.

## For Loops

### Iterate Over Arrays

```atlas
let items: array = [1, 2, 3];

for item in items {
    print(str(item));
}
```

### Iterate Over Split Strings

```atlas
let text: string = "a\nb\nc";
let lines: array = split(text, "\n");

for line in lines {
    print(line);
}
```

**Note**:
- No C-style `for (i = 0; i < n; i++)` loops
- No `while` loops observed yet
- Variable in loop is immutable

## Early Returns

```atlas
fn check_value(x: number) -> bool {
    if (x < 0) {
        return false;  // Early return OK
    }

    if (x > 100) {
        return false;
    }

    return true;
}
```

## Ternary / Inline Conditions

Not observed - use match or if/else:

```atlas
// Instead of: x ? a : b
// Use match or if/else
let result: string = if (x > 0) {
    "positive"
} else {
    "negative"
};
```

## Boolean Logic

```atlas
if (x > 0 && y < 10) {
    // AND
}

if (flag1 || flag2) {
    // OR
}

if (!is_valid) {
    // NOT
}
```

## Comparison

```atlas
x == y     // Equal
x != y     // Not equal
x > y      // Greater
x < y      // Less
x >= y     // Greater or equal
x <= y     // Less or equal
```
