# Traits

This document reflects the actual parser in `crates/atlas-runtime/src/parser/mod.rs` and typechecker in `crates/atlas-runtime/src/typechecker/`.

**Trait Declarations**
```
trait Name {
    fn method(self) -> Type;
    fn with_default(self) -> Type { default_body }
}
```
- Methods use `self` — the type is inferred from the impl block.
- Methods ending with `;` are required (implementors must provide a body).
- Methods with a block body are defaults (implementors may override).
- Return type annotation is optional.

Example (tested):
```atlas
trait Greetable {
    fn greet(self) -> string;
    fn farewell(self) -> string {
        return "Goodbye";
    }
}
```

**Impl Blocks**
```
impl TraitName for TypeName {
    fn method(self) -> Type { body }
}
```
- The `self` parameter type is inferred — write `self`, not `self: TypeName`.
- Must implement all required methods (those without default bodies).
- May override default methods.

Example (tested):
```atlas
struct Person { name: string }

impl Greetable for Person {
    fn greet(self) -> string {
        return `Hello, I'm {self.name}`;
    }
    // farewell uses the default implementation
}

let p: Person = Person { name: "Ada" };
print(p.greet());    // "Hello, I'm Ada"
print(p.farewell()); // "Goodbye"
```

**Trait Objects (Bounded Polymorphism)**
```
fn process(item: TraitName) -> Type { ... }
```
- Trait names can be used as parameter types.
- Any type that implements the trait can be passed.

Example (tested):
```atlas
fn introduce(g: Greetable) -> string {
    return g.greet();
}

let p: Person = Person { name: "Ada" };
print(introduce(p)); // "Hello, I'm Ada"
```

**Generic Trait Bounds**
```
fn identity<T: Copy>(value: T) -> T { ... }
fn process<T: Display + Copy>(value: T) -> string { ... }
```
- Type parameters can require trait bounds using `:`.
- Multiple bounds use `+`.
- `extends` is also supported: `T extends number`.

**Current limitations:**
- Trait inheritance (`trait A: B`) — not yet implemented (H-076)
- Generic traits (`trait Container<T>`) — not yet implemented (H-077)
