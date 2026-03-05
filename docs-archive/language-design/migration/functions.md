# Migration: Arrow functions → fn expressions

**Decision:** D-008 (Remove arrow function syntax)
**Effort:** Requires adding type annotations

---

## Rule

```atlas
// Before
(params) => expression

// After
fn(params: types) -> ReturnType { return expression; }
```

Arrow functions have implicit types. `fn` expressions require explicit types.

---

## Examples

### Simple arrow
```atlas
// Before
let double = (x) => x * 2;

// After
let double = fn(x: number) -> number { return x * 2; };
```

### Multiple parameters
```atlas
// Before
let add = (a, b) => a + b;

// After
let add = fn(a: number, b: number) -> number { return a + b; };
```

### In higher-order functions
```atlas
// Before
let doubled = map([1, 2, 3], (x) => x * 2);
let evens = filter([1, 2, 3, 4], (x) => x % 2 == 0);
let sum = reduce([1, 2, 3], (acc, x) => acc + x, 0);

// After
let doubled = map([1, 2, 3], fn(x: number) -> number { return x * 2; });
let evens = filter([1, 2, 3, 4], fn(x: number) -> bool { return x % 2 == 0; });
let sum = reduce([1, 2, 3], fn(acc: number, x: number) -> number { return acc + x; }, 0);
```

### String operations
```atlas
// Before
let lengths = map(words, (s) => len(s));
let upper = map(words, (s) => toUpperCase(s));

// After
let lengths = map(words, fn(s: string) -> number { return len(s); });
let upper = map(words, fn(s: string) -> string { return toUpperCase(s); });
```

### Boolean predicates
```atlas
// Before
let hasAdmin = some(users, (u) => u.role == "admin");

// After
let hasAdmin = some(users, fn(u: User) -> bool { return u.role == "admin"; });
```

---

## Type Inference Note

v0.3 supports return type inference, so you can often omit `-> ReturnType`:

```atlas
// Explicit (always valid)
let double = fn(x: number) -> number { return x * 2; };

// With inference (v0.3+)
let double = fn(x: number) { return x * 2; };
```

Parameter types are ALWAYS required. Return type can be inferred.

---

## Common Type Mappings

| Arrow param usage | Likely type |
|-------------------|-------------|
| `x * 2`, `x + 1` | `number` |
| `x + " "`, `len(x)` | `string` |
| `x && y`, `!x` | `bool` |
| `x[0]`, `len(x)` | `array` or `string` |
| `x.field` | struct type |

---

## Verification

After migration:
```bash
grep -rE '=>' --include="*.atl" . | grep -v 'match'
```

Should return no matches (excluding `=>` in match arms, which is valid).

**Note:** `=>` is still valid in match expressions:
```atlas
match value {
    Some(x) => process(x),  // This is fine
    None() => default(),
}
```
