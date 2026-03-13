# reflect — Type Introspection

Namespace: `reflect` (lowercase, D-049)

The `reflect` namespace provides runtime type inspection, value introspection, structural
field discovery, and function metadata. Use it when you need to inspect values dynamically
at runtime.

---

## Import

No import required. `reflect` is a built-in namespace.

---

## Type Query Functions

### `reflect.typeOf(value: any) -> string`

Return the runtime type name of a value as a string. Equivalent to the top-level `typeof`
builtin.

```atlas
console.log(reflect.typeOf(42));          // "number"
console.log(reflect.typeOf("hello"));     // "string"
console.log(reflect.typeOf(true));        // "bool"
console.log(reflect.typeOf(null));        // "null"
console.log(reflect.typeOf([1, 2]));      // "array"
console.log(reflect.typeOf(hashMapNew())); // "map"
```

Type name reference:

| Value | Type string |
|-------|------------|
| number | `"number"` |
| string | `"string"` |
| bool | `"bool"` |
| null | `"null"` |
| array | `"array"` |
| HashMap | `"map"` |
| Option | `"option"` |
| Result | `"result"` |
| function | `"function"` |
| DateTime | `"datetime"` |
| Regex | `"regex"` |
| JsonValue | `"json"` |

---

### `reflect.typeDescribe(value: any) -> string`

Return a human-readable description of the type. More verbose than `typeOf`.

```atlas
console.log(reflect.typeDescribe(42));       // "primitive number type"
console.log(reflect.typeDescribe([1, 2]));   // "array type"
```

---

### `reflect.isPrimitive(value: any) -> bool`

Returns `true` if the value is a primitive type: `number`, `string`, `bool`, or `null`.

```atlas
reflect.isPrimitive(42);       // true
reflect.isPrimitive("hello");  // true
reflect.isPrimitive(true);     // true
reflect.isPrimitive(null);     // true
reflect.isPrimitive([1, 2]);   // false
```

---

### `reflect.sameType(a: any, b: any) -> bool`

Returns `true` if both values have the same runtime type.

```atlas
reflect.sameType(1, 2);         // true  (both number)
reflect.sameType(1, "hello");   // false (number vs string)
reflect.sameType([], [1, 2]);   // true  (both array)
```

---

### `reflect.isCallable(value: any) -> bool`

Returns `true` if the value is a function (user-defined or native).

```atlas
fn myFn(): void { }
reflect.isCallable(myFn); // true
reflect.isCallable(42);   // false
```

---

## Collection Introspection

### `reflect.getLength(value: string | any[]) -> number`

Return the length of a string (number of Unicode characters) or an array. Raises a
`TypeError` for other types.

```atlas
reflect.getLength("hello");    // 5
reflect.getLength([1, 2, 3]);  // 3
```

---

### `reflect.isEmpty(value: string | any[]) -> bool`

Returns `true` if a string or array has zero elements. Raises a `TypeError` for other
types.

```atlas
reflect.isEmpty("");       // true
reflect.isEmpty([]);       // true
reflect.isEmpty("hi");     // false
reflect.isEmpty([1]);      // false
```

---

### `reflect.fields(value: any) -> string[]`

Return the field/key names of a value as an array of strings:
- For `HashMap` (including struct instances): returns all key names.
- For `JsonValue::Object`: returns the JSON object's key names.
- For all other types: returns an empty array.

```atlas
struct Point { x: number, y: number }
let p = Point { x: 1, y: 2 };
let names = reflect.fields(p); // ["x", "y"]

let m = hashMapNew();
// m = m.set("a", 1).set("b", 2);
let keys = reflect.fields(m); // ["a", "b"]
```

---

### `reflect.hasMethod(value: any, name: string) -> bool`

Returns `true` if the runtime type of `value` has a method with the given name.
Uses the method dispatch table to check — does not check user-defined methods.

```atlas
let arr = [1, 2, 3];
reflect.hasMethod(arr, "push");    // true
reflect.hasMethod(arr, "get");     // false

let m = hashMapNew();
reflect.hasMethod(m, "get");       // true
```

---

## Function Metadata

### `reflect.getFunctionName(fn: function) -> string`

Return the declared name of a function. Returns `"<native function>"` for built-in native
functions.

```atlas
fn add(a: number, b: number): number { return a + b; }
reflect.getFunctionName(add); // "add"
```

---

### `reflect.getFunctionArity(fn: function) -> number`

Return the number of parameters a function accepts. Not supported for native functions
(raises `TypeError`).

```atlas
fn add(a: number, b: number): number { return a + b; }
reflect.getFunctionArity(add); // 2
```

---

## Equality and Cloning

### `reflect.deepEquals(a: any, b: any) -> bool`

Perform a deep structural equality comparison. Unlike `==` (which uses reference equality
for arrays and collections), `deepEquals` recursively compares contents.

```atlas
let a = [1, 2, 3];
let b = [1, 2, 3];
reflect.deepEquals(a, b);  // true
a == b;                    // may be false (reference equality)

reflect.deepEquals(Ok(42), Ok(42));       // true
reflect.deepEquals(Some("x"), Some("x")); // true
```

Supported types for deep comparison: `number`, `string`, `bool`, `null`, `array`,
`JsonValue`, `Option`, `Result`, `Function` (by name only). Other types use reference
comparison and return `false`.

---

### `reflect.clone(value: any) -> any`

Return a copy of a value. For primitives, this is a value copy. For arrays, this performs
a shallow clone of the array (each element is not recursively deep-cloned).

```atlas
let arr = [1, 2, 3];
let arr2 = reflect.clone(arr);
```

---

### `reflect.valueToString(value: any) -> string`

Convert any value to its string representation, including arrays and functions. Unlike
`str()` or `.toString()`, this works uniformly on all value types.

```atlas
reflect.valueToString(42);         // "42"
reflect.valueToString([1, 2, 3]); // "[1, 2, 3]"
reflect.valueToString(null);       // "null"
```

---

## Error Behavior

| Function | Error condition | Result |
|----------|-----------------|--------|
| `reflect.getLength()` | Not string or array | `TypeError` |
| `reflect.isEmpty()` | Not string or array | `TypeError` |
| `reflect.getFunctionArity()` | Native function | `TypeError` |
| `reflect.getFunctionName()` | Non-function value | `TypeError` |
| `reflect.getFunctionArity()` | Non-function value | `TypeError` |
| All functions | Wrong arity | `InvalidStdlibArgument` |
