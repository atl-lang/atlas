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

**Inherent Impl Blocks** (B13, D-036)
```
impl TypeName {
    fn method(borrow self: TypeName) -> Type { body }
}
```
- Methods belong directly to the type — no trait required.
- `self` receiver requires an ownership annotation: `borrow self`, `own self`, or `share self` (D-038).
- Inherent methods resolve before trait methods at call sites (D-037).
- Mangling: `__impl__TypeName__MethodName`.

Example (tested):
```atlas
struct Point { x: number, y: number }

impl Point {
    fn magnitude(borrow self: Point) -> number {
        return self.x * self.x + self.y * self.y;
    }
}

let p = Point { x: 3, y: 4 };
print(p.magnitude()); // 25
```

**Trait Impl Blocks**
```
impl TraitName for TypeName {
    fn method(borrow self: TypeName) -> Type { body }
}
```
- Must implement all required methods (those without default bodies).
- May override default methods.
- Mangling: `__impl__TypeName__TraitName__MethodName`.

Example (tested):
```atlas
struct Person { name: string }

impl Greetable for Person {
    fn greet(borrow self: Person) -> string {
        return `Hello, I'm ${self.name}`;
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

**Current limitations:**
- Trait inheritance (`trait A: B`) — not yet implemented (H-076)
- Generic traits (`trait Container<T>`) — not yet implemented (H-077)
