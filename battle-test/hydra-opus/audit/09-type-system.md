# Audit: Type System

**Friction Level:** HIGH

## Critical Issues

### 1. Empty Array Type Inference FAILS

```atlas
// FAILS with: expected string[], found ?[]
let args: string[] = [];
args = args.push("test");

// WORKAROUND: explicit type annotation
let args: string[] = [];
```

**Impact:** Every empty array literal needs explicit type
**AI Error Rate:** 100% on first attempt

### 2. unwrap() Returns `any` Type

```atlas
let result: Result<Proxy, string> = proxy_start(proxy);
let running = unwrap(result);  // running: any

// running.state fails with "Type 'any' has no method 'state'"

// WORKAROUND: explicit type annotation
let running: Proxy = unwrap(result);
```

**Impact:** All unwrap calls need type annotation
**AI Error Rate:** ~80% (most won't add annotation)

### 3. Ownership Warnings Excessive (AT2013)

Every non-Copy struct pass generates:
```
warning[AT2013]: Type '{ ... }' is not Copy — consider annotating with 'own' or 'borrow'
```

This fires on:
- Function arguments
- Return values
- Variable assignments

**Impact:** Noise obscures real issues. 50+ warnings in 800-line file.
**Fix Needed:** Only warn when ownership is actually ambiguous

### 4. No Return Type Inference

```atlas
// Must always specify return type
fn add(a: number, b: number) -> number {
    return a + b;
}

// Can't do
fn add(a: number, b: number) { return a + b; }  // Missing return type
```

**Impact:** More verbose than TypeScript/Kotlin
**AI Friction:** LOW (easy to remember)

### 5. Option vs Result Confusion

```atlas
// hashMapGet returns Option
let val = hashMapGet(map, "key");  // Option<T>

// Json.parse returns Result
let data = Json.parse(str);  // Result<json, string>

// AI will confuse these:
if is_err(val) { ... }   // WRONG: is_err on Option
if is_none(val) { ... }  // CORRECT
```

**Impact:** Must know return type of every function
**AI Error Rate:** ~30% (will use wrong checker)

## Type Inference That Works

- Numeric literals → `number`
- String literals → `string`
- Boolean literals → `bool`
- Struct literals → Correct struct type
- Array literals with elements → Correct element type
- Match expressions → Union of arm types

## Type Inference That Fails

- Empty arrays → `?[]`
- `unwrap()` results → `any`
- Generic function returns sometimes
- Closures capturing variables

## Workarounds Summary

| Issue | Workaround | Complexity |
|-------|------------|------------|
| Empty array | Add `: T[]` annotation | LOW |
| unwrap → any | Add `: T` annotation | LOW |
| Ownership warnings | Ignore or add `@allow(unused)` | LOW |
| Option/Result confusion | Learn return types | MEDIUM |

## Comparison to Other Languages

| Language | Empty Array | Generic Unwrap | Return Type |
|----------|-------------|----------------|-------------|
| TypeScript | Inferred from context | Preserves type | Optional |
| Rust | Inferred | Preserves type | Required |
| Go | Inferred | No generics | Required |
| Atlas | FAILS | Returns any | Required |

## Recommendations

1. **P1:** Fix empty array inference from context
2. **P1:** Make unwrap generic to preserve type
3. **P2:** Reduce ownership warning noise
4. **P3:** Add optional return type inference for simple functions
