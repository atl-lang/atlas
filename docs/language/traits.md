# Traits

Traits define shared behavior — a named set of method signatures that types can implement. Atlas traits follow TypeScript's comma-separated inheritance style and Rust's `impl` block mechanics.

---

## Overview

A trait is a named contract: any type that `impl`s the trait must provide all required methods. Traits support:
- Abstract method signatures (required)
- Default method implementations (optional override)
- Supertrait bounds (comma-separated, TypeScript style)
- Generic type parameters
- Trait bounds on function type parameters

---

## Trait Declaration

### Basic Trait

```atlas
trait Printable {
    fn print(self): void;
}
```

Method signatures end with `;`. The `self` parameter implicitly has the implementing type.

### Trait with Multiple Methods

```atlas
trait Serializable {
    fn serialize(self): string;
    fn deserialize(data: string): void;
}
```

### Default Method Implementations

Methods in a trait body may include a default implementation. Implementing types can override it or inherit the default:

```atlas
trait Describable {
    fn describe(self): string;

    fn print(self): void {
        console.log(self.describe());
    }
}
```

A method with a `{ body }` is a default implementation. A method ending in `;` is required (abstract).

### Generic Traits

```atlas
trait Converter<T> {
    fn convert(self): T;
}

trait Container<T> {
    fn get(self): T;
    fn set(self, value: T): void;
}
```

### Supertrait Bounds (D-026)

Traits can require that implementors also implement other traits. The syntax uses `extends` with comma-separated supertrait names — TypeScript style, not Rust's `+` style:

```atlas
trait Stringable {
    fn toString(self): string;
}

trait Printable extends Stringable {
    fn print(self): void;
}
```

Multiple supertraits:

```atlas
trait Serializable extends Readable, Writable, Stringable {
    fn serialize(self): string;
}
```

A type implementing `Serializable` must also implement `Readable`, `Writable`, and `Stringable`.

### Visibility

```atlas
pub trait Drawable {
    fn draw(self): void;
}

internal trait InternalProtocol {
    fn handle(self): void;
}
```

Default visibility is `private` (file-local). Use `pub` to make a trait importable.

---

## Implementing a Trait

### Trait Implementation

```atlas
struct Point {
    x: number,
    y: number,
}

trait Printable {
    fn print(self): void;
}

impl Printable for Point {
    fn print(self): void {
        console.log(`(${self.x}, ${self.y})`);
    }
}

let p = Point { x: 3, y: 4 };
p.print();
```

Syntax: `impl TraitName for TypeName { ... }`

All abstract methods from the trait must be implemented. Default methods are inherited automatically.

### Inherent Impl (No Trait)

Methods can be added to a type without a trait via inherent impl:

```atlas
impl Point {
    fn distance(self): number {
        let sq = self.x * self.x + self.y * self.y;
        return math.sqrt(sq).unwrap();
    }

    fn scale(self, factor: number): Point {
        return Point { x: self.x * factor, y: self.y * factor };
    }
}
```

Syntax: `impl TypeName { ... }` (no `for` clause)

### Static Methods

Methods declared with `static fn` have no `self` parameter and are called on the type name directly:

```atlas
impl Point {
    static fn origin(): Point {
        return Point { x: 0, y: 0 };
    }

    static fn fromPair(x: number, y: number): Point {
        return Point { x, y };
    }
}

let p = Point.origin();
let p2 = Point.fromPair(3, 4);
```

Static methods are called as `TypeName.methodName()`, not `instance.methodName()`.

### Generic Trait Implementation

```atlas
trait Converter<T> {
    fn convert(self): T;
}

struct Celsius {
    value: number,
}

impl Converter<number> for Celsius {
    fn convert(self): number {
        return self.value;
    }
}
```

The trait type args are specified at the impl site: `impl Converter<number> for Celsius`.

---

## Trait Bounds on Functions

Trait bounds constrain generic type parameters to types that implement a specific trait.

### Single Bound

```atlas
fn printAll<T extends Printable>(items: T[]): void {
    for item in items {
        item.print();
    }
}
```

Syntax: `<T extends TraitName>`

### Multiple Bounds (Intersection)

```atlas
fn process<T extends Serializable & Comparable>(item: T): string {
    return item.serialize();
}
```

Multiple trait bounds use `&` (TypeScript intersection style, D-039).

### Combining Bounds on Multiple Params

```atlas
fn merge<T extends Readable, U extends Writable>(source: T, dest: U): void {
    let data = source.read();
    dest.write(data);
}
```

---

## Self Parameter

In trait methods and impl methods, `self` refers to the instance. `self` is a bare parameter with no explicit type annotation — the type is inferred from the impl context:

```atlas
impl Point {
    fn distance(self): number {
        // self is Point here
        return math.sqrt(self.x * self.x + self.y * self.y).unwrap();
    }
}
```

The default ownership for `self` is `borrow` (D-040). Write `own self` to take ownership (consuming the receiver):

```atlas
impl Buffer {
    fn flush(own self): void {
        // self is consumed; caller's binding is invalidated
    }
}
```

---

## Method Dispatch

Atlas uses static dispatch for trait methods. The typechecker resolves which impl to call at compile time based on the concrete type of the receiver. There is no vtable or dynamic dispatch — calling a trait method compiles to a direct jump to the specific impl.

---

## Full Example

```atlas
pub trait Animal {
    fn name(self): string;
    fn sound(self): string;

    fn describe(self): string {
        return self.name() + " says " + self.sound();
    }
}

pub struct Dog {
    breed: string,
}

pub struct Cat {
    indoor: bool,
}

impl Animal for Dog {
    fn name(self): string {
        return "Dog";
    }

    fn sound(self): string {
        return "woof";
    }
}

impl Animal for Cat {
    fn name(self): string {
        return "Cat";
    }

    fn sound(self): string {
        return "meow";
    }
}

let dog = Dog { breed: "Labrador" };
let cat = Cat { indoor: true };

console.log(dog.describe());  // "Dog says woof"
console.log(cat.describe());  // "Cat says meow"
```

---

## Gotchas

**Supertrait syntax is TypeScript-style commas, not Rust `+`.** Use `trait B extends A, C {}` not `trait B: A + C {}`.

**Trait bounds on type parameters use `extends` with `&`.** Use `<T extends A & B>` for multiple bounds on a single type parameter, but `extends A, B` in the trait header for supertraits. These are different contexts.

**All abstract methods must be implemented.** Forgetting any method in an `impl` block is a compile error.

**No `impl` visibility modifier.** `impl` blocks themselves have no visibility keyword — the visibility lives on the trait and the individual methods in the trait declaration.

**Static methods are not part of trait contracts.** Traits define instance methods only. `static fn` lives in inherent impl blocks and cannot be required by a trait.

**Default implementations are inherited automatically.** You do not need to repeat a default method's body in the `impl` block. Providing a body in the impl block overrides the default.
