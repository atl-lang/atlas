# Ownership Annotations

Atlas requires every function and closure parameter to declare ownership intent.
This is enforced at parse time — missing annotation is a parse error (AT1007).
Decision: D-034.

## The Three Keywords

| Keyword | Meaning | Caller after call |
|---------|---------|-------------------|
| `own`   | Callee takes ownership — caller's binding is invalid after the call | invalid (moved) |
| `borrow` | Read-only reference — callee cannot store, return, or alias it | unchanged |
| `share` | Both caller and callee hold a valid reference — neither may mutate | unchanged |

## Decision Tree

Choose the annotation by answering:

1. Does this function **consume** the value (transform/store it permanently)?
   → `own`
2. Does this function only **read** the value and return or discard it?
   → `borrow`
3. Do both caller and callee need to **hold a live reference** at the same time?
   → `share`

When in doubt: use `borrow`. It is the most permissive read-only annotation.

## Examples

### `own` — Caller gives up the binding

```atlas
fn consume(own data: string) : string {
    return data;
}

let s = "hello";
let result = consume(s);
// s is now invalid — using s here would produce AT3053
```

### `borrow` — Read-only, no escape

```atlas
fn greet(borrow name: string) : void {
    print(name);
    return;
}

let s = "world";
greet(s);
// s is still valid here
let again = s;  // ok
```

A `borrow` parameter cannot escape: returning it, storing it in `let`, or using it as a struct field value produces AT3054.

### `share` — Co-held reference, no mutation

```atlas
fn display(share cache: HashMap<string, number>) : void {
    print(str(hashMapSize(cache)));
    return;
}

let m = hashMapNew();
display(m);
// m is still valid and unchanged
```

A `share` parameter cannot be assigned to or passed to an `own` parameter — that would transfer ownership of something the caller still holds, which produces AT3055.

## What Each Prevents

| Error | Trigger | Code |
|-------|---------|------|
| AT1007 | Missing annotation on any fn param | parse error |
| AT3053 | Using a variable after it was passed to `own` | typechecker error |
| AT3054 | Returning, storing, or aliasing a `borrow` param | typechecker error |
| AT3055 | Assigning to or moving a `share` param | typechecker error |

## Notes

- Return ownership (`-> own Type` / `-> borrow Type`) is optional — omit it if not needed.
- Closures follow the same rules as named functions: every parameter needs an annotation.
- Stdlib functions have implicit `borrow` annotations on all parameters.
