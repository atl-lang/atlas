# Audit: Diagnostics Quality

**Friction Level:** MEDIUM

## Error Message Quality

### Excellent (AI can self-correct)

**AT3003: Immutable Variable**
```
error[AT3003]: Cannot assign to immutable variable 'counter'
  --> file.atlas:10:5
   |
10 |     counter = counter + 1;
   |     ^^^^^^^ immutable variable
   = help: declare 'counter' as mutable: `let mut counter = ...`
```
**Rating:** 10/10 - Suggests exact fix

**AT3001: Type Mismatch**
```
error[AT3001]: Type mismatch: expected string[], found ?[]
  --> file.atlas:5:9
   |
 5 |         args: [],
   |         ^^^^^^^^ expected string[], found ?[]
   = help: field 'args' must be of type string[]
```
**Rating:** 8/10 - Clear problem, fix requires knowledge

**AT3005: Argument Count**
```
error[AT3005]: Function expects 2 arguments, found 1
  --> file.atlas:424:10
   |
424|     let sorted = sort(nums);
   |          ^^^^^^^^^^ argument count mismatch
   = help: add 1 argument; function signature: (any[], (any, any) -> number) -> any[]
```
**Rating:** 9/10 - Shows expected signature

### Good (AI needs context)

**AT2002: Unknown Symbol**
```
error[AT2002]: Unknown symbol 'default_registry'
  --> file.atlas:80:20
   |
80 |     let registry = default_registry();
   |                    ^^^^^^^^^^^^^^^^ undefined variable
   = help: declare 'default_registry' before using it, or check for typos
```
**Rating:** 7/10 - Doesn't suggest import

**AT0001: Type Error**
```
error[AT0001]: Type error: is_err() requires Result value in function sanitizer_classify
  --> file.atlas:286:55
```
**Rating:** 7/10 - Says what's wrong, not what to use instead

### Poor (AI can't self-correct)

**AT3010: Method Not Found on Any**
```
error[AT3010]: Type 'any' has no method named 'state'
  --> file.atlas:814:55
   |
814|         print(`  After start: ${proxy_state_to_string(running.state)}`);
   |                                                       ^^^^^ method not found
   = help: type 'any' does not support method 'state'
```
**Rating:** 4/10 - Doesn't say WHY it's `any` (unwrap loses type)

## Warning Quality

### Helpful Warnings

**AT9000: Deprecated Function**
```
warning[AT9000]: Deprecated: use str.method() instead of trim()
  --> file.atlas:254:19
   |
254|     let trimmed = trim(chunk);
   |                   ^^^^^^^^^^^ deprecated global
   = help: Use method syntax or static namespace instead.
```
**Rating:** 9/10 - Clear migration path

### Unhelpful Warnings (Noise)

**AT2013: Ownership Warning**
```
warning[AT2013]: Type '{ ... }' is not Copy — consider annotating with 'own' or 'borrow'
```
**Rating:** 3/10 - Fires on every struct, no real guidance

**AT2001: Unused Variable**
```
warning[AT2001]: Unused variable 'index'
```
**Rating:** 5/10 - Helpful but fires even when variable IS used in expression

## Diagnostic Coverage

| Category | Errors | Quality |
|----------|--------|---------|
| Type mismatches | Complete | GOOD |
| Missing symbols | Complete | GOOD |
| Ownership | Partial | NOISY |
| Syntax errors | Complete | EXCELLENT |
| Runtime errors | Limited | OK |

## Stack Traces

Not tested in this port - no runtime panics encountered.

## Comparison to Go Compiler

| Aspect | Go | Atlas | Better |
|--------|----|----|--------|
| Error clarity | Good | Good | TIE |
| Fix suggestions | Rare | Common | Atlas |
| Warning noise | Low | HIGH | Go |
| Location info | Good | Good | TIE |
| Multi-error mode | Yes | Yes | TIE |

## Recommendations

1. **P1:** AT3010 should explain WHY type is `any` (unwrap loses type info)
2. **P2:** AT2013 should only fire on genuinely ambiguous ownership
3. **P2:** AT2002 should suggest imports if symbol exists in another module
4. **P3:** Add "did you mean?" suggestions for typos
