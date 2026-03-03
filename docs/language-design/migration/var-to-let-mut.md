# Migration: var → let mut

**Decision:** D-006 (Remove `var` keyword)
**Effort:** Mechanical replacement

---

## Rule

```
var NAME = VALUE;  →  let mut NAME = VALUE;
```

All occurrences. No exceptions.

---

## Examples

### Simple variable
```atlas
// Before (v0.2)
var count = 0;

// After (v0.3)
let mut count = 0;
```

### With type annotation
```atlas
// Before
var name: string = "Atlas";

// After
let mut name: string = "Atlas";
```

### Multiple declarations
```atlas
// Before
var x = 1;
var y = 2;
var z = 3;

// After
let mut x = 1;
let mut y = 2;
let mut z = 3;
```

---

## Regex Pattern

For automated replacement:

```regex
Search:  \bvar\s+
Replace: let mut
```

**Note:** Preserve whitespace after `mut` to match original spacing.

---

## Edge Cases

### Immutable variables using var (incorrect usage)

If code uses `var` but never reassigns, consider using `let` (immutable):

```atlas
// Before
var config = loadConfig();  // Never reassigned

// After (if never mutated)
let config = loadConfig();

// After (if mutated later)
let mut config = loadConfig();
```

### In for loop init (deprecated pattern)

C-style for loops with `var` are double-deprecated:

```atlas
// Before
for (var i = 0; i < 10; i++) { }

// After (see loops.md for full migration)
let mut i = 0;
while i < 10 {
    // body
    i += 1;
}
```

---

## Verification

After migration, run:
```bash
grep -r '\bvar\b' --include="*.atl" .
```

Should return no matches in `.atl` files.
