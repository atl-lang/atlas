# Component Audit: StateStore

**Grammar Friction Points**
- Assignment targets cannot be `record.field`, so state updates require rebuilding records.
- Structural type annotations are verbose and undocumented in the main syntax spec.
- `hashMapPut` returns a new map (no in-place mutation); you must reassign everywhere.
- Empty array literals require hacks (`slice([""],0,0)`) even with type annotations.
- `if` requires parentheses; missing parens cause confusing parse errors.

**Missing Features (Atlas should have)**
- Record field assignment or mutable record syntax.
- First-class `Map`/`HashMap` literals or constructors without stdlib calls.
- Map update helpers that make immutable update patterns explicit (`mapPut!` macro or syntax).

**Syntax Quality Rating:** 7/10

**AI Generation Experience**
- Pure-functional style is forced by grammar, which increases boilerplate for simple state updates.
- Type alias syntax works but is not documented in the primary syntax reference.
