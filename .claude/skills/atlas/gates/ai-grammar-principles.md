# AI-First Grammar Design Principles (Lazy-Loaded)

**Load when:** Making syntax/grammar decisions, reviewing language design, brainstorming features.

---

## Core Principle

Every Atlas syntax decision must pass this filter:
**"Does this make AI code generation easier or harder?"**

Harder = wrong choice. Always.

---

## Research-Backed Guidelines

From [SimPy research](https://arxiv.org/html/2404.16333v1) — AI-oriented grammar reduced tokens by 10-35% while **improving** correctness:

### 1. Minimize Token Count Per Concept

| Principle | Good (fewer tokens) | Bad (more tokens) |
|-----------|---------------------|-------------------|
| Compact keywords | `fn`, `let`, `mut` | `function`, `declare`, `mutable` |
| Single-char operators | `?` for try | `try { } catch { }` |
| Unified types | `number` | `int`, `float`, `double`, `i32`, `u64` |
| Implicit returns | `{ x + y }` | `{ return x + y; }` |
| Optional parens | `if cond { }` | `if (cond) { }` |

**Atlas already does most of this.** Protect these choices.

### 2. Reduce Ambiguity (Fewer Valid Parses = Fewer AI Mistakes)

| Principle | Example |
|-----------|---------|
| One way to declare functions | `fn name() { }` only |
| Consistent block delimiters | Always `{ }`, never indentation-based |
| Explicit type positions | `param: Type` always after colon |
| No operator overloading | `+` always means addition |
| No implicit coercions | `number` + `string` = error, not concatenation |

### 3. Structural Predictability

AI generates better code when patterns are consistent:

| Pattern | Why it helps AI |
|---------|----------------|
| `struct Name { fields }` | Same shape every time |
| `impl Trait for Type { methods }` | Predictable structure |
| `fn name(params) -> ReturnType { body }` | Always this order |
| `match expr { pat => result }` | Consistent arm syntax |

### 4. Error Messages Must Guide AI

When Atlas reports errors, the message should contain enough information for AI to fix the code in one attempt:

```
Error: Type mismatch in function `add`
  Expected: number
  Found: string
  At: line 5, column 12
  Hint: Convert with str_to_number() or change parameter type
```

**Bad:** `Type error` (AI has to guess)
**Good:** Expected/Found/Location/Hint (AI fixes in one shot)

---

## Atlas Grammar Audit Checklist

When adding new syntax, verify:

- [ ] **Token count:** Is this the minimum tokens to express this concept?
- [ ] **Ambiguity:** Can this be parsed only one way?
- [ ] **Consistency:** Does this follow existing Atlas patterns?
- [ ] **Learnability:** Can Haiku generate this correctly from a single example?
- [ ] **Error recovery:** Does a mistake in this syntax produce a helpful error?

---

## Deliberate Atlas Choices (Justified)

| Choice | Tokens saved | AI benefit |
|--------|-------------|-----------|
| `fn` not `function` | 1 token/function | 60%+ of all declarations |
| `number` not `int/float` | Eliminates type selection | AI never picks wrong numeric type |
| `?` operator | 5+ tokens vs try/catch | Error handling in 1 char |
| Optional return type | 2-3 tokens when inferrable | Less boilerplate |
| Template strings `` `{x}` `` | Fewer tokens than `format!()` | Natural interpolation |
| `let`/`let mut` | Clear mutability signal | AI knows exactly what's mutable |

---

## Anti-Patterns (NEVER Add to Atlas)

| Anti-Pattern | Why it hurts AI |
|-------------|----------------|
| Multiple function syntaxes | AI must choose between equivalent options |
| Significant whitespace | Token-invisible semantics = silent bugs |
| Implicit type coercion | AI can't predict runtime behavior |
| Operator overloading | `+` meaning changes per context |
| Complex lifetime annotations | Exponential complexity for AI generation |
| Macro systems | AI generates broken macros constantly |
