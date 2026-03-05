# Reflection Functions

Runtime type inspection and value introspection utilities.

### reflect_typeof

```atlas
fn reflect_typeof(value: any) -> string
```

Returns the type name of a value.

**Parameters:**
- `value`: Value to inspect

**Returns:** Type name string.

**Example:**
```atlas
let value = [1, 2, 3];
print(reflect_typeof(value)); // "array"
```

### reflect_is_callable

```atlas
fn reflect_is_callable(value: any) -> bool
```

Checks whether a value is callable (function or native function).

**Parameters:**
- `value`: Value to inspect

**Returns:** `true` if callable, otherwise `false`.

**Example:**
```atlas
fn greet() { print("hi"); }
print(reflect_is_callable(greet)); // true
```

### reflect_is_primitive

```atlas
fn reflect_is_primitive(value: any) -> bool
```

Checks whether a value is a primitive (number, string, bool, or null).

**Parameters:**
- `value`: Value to inspect

**Returns:** `true` for primitives, otherwise `false`.

**Example:**
```atlas
print(reflect_is_primitive(42));
print(reflect_is_primitive([1, 2]));
```

### reflect_same_type

```atlas
fn reflect_same_type(a: any, b: any) -> bool
```

Checks whether two values share the same runtime type.

**Parameters:**
- `a`: First value
- `b`: Second value

**Returns:** `true` if types match.

**Example:**
```atlas
print(reflect_same_type(1, 2));      // true
print(reflect_same_type(1, "one"));  // false
```

### reflect_get_length

```atlas
fn reflect_get_length(value: any) -> number
```

Returns the length of an array or string.

**Parameters:**
- `value`: Array or string

**Returns:** Number of elements or characters.

**Example:**
```atlas
print(reflect_get_length("atlas")); // 5
```

### reflect_is_empty

```atlas
fn reflect_is_empty(value: any) -> bool
```

Checks whether an array or string is empty.

**Parameters:**
- `value`: Array or string

**Returns:** `true` if empty.

**Example:**
```atlas
print(reflect_is_empty(""));
print(reflect_is_empty([]));
```

### reflect_type_describe

```atlas
fn reflect_type_describe(value: any) -> string
```

Returns a human-readable description of a value's type.

**Parameters:**
- `value`: Value to inspect

**Returns:** Description string.

**Example:**
```atlas
print(reflect_type_describe(42));
```

### reflect_clone

```atlas
fn reflect_clone(value: any) -> any
```

Clones a value. Arrays are cloned element-by-element; primitives copy directly.

**Parameters:**
- `value`: Value to clone

**Returns:** A cloned value.

**Example:**
```atlas
let original = [1, 2, 3];
let copy = reflect_clone(original);
```

### reflect_value_to_string

```atlas
fn reflect_value_to_string(value: any) -> string
```

Converts any value to its string representation.

**Parameters:**
- `value`: Value to stringify

**Returns:** String representation.

**Example:**
```atlas
print(reflect_value_to_string([1, 2, 3]));
```

### value_to_string

```atlas
fn value_to_string(value: any) -> string
```

Alias for `reflect_value_to_string`.

**Parameters:**
- `value`: Value to stringify

**Returns:** String representation.

**Example:**
```atlas
print(value_to_string({ answer: 42 }));
```

### reflect_deep_equals

```atlas
fn reflect_deep_equals(a: any, b: any) -> bool
```

Performs deep equality for arrays, Option, Result, and JSON values.

**Parameters:**
- `a`: First value
- `b`: Second value

**Returns:** `true` if values are deeply equal.

**Example:**
```atlas
let a = [1, [2, 3]];
let b = [1, [2, 3]];
print(reflect_deep_equals(a, b));
```

### reflect_get_function_name

```atlas
fn reflect_get_function_name(value: any) -> string
```

Returns the name of a function value.

**Parameters:**
- `value`: Function value

**Returns:** Function name string.

**Example:**
```atlas
fn add(a, b) { return a + b; }
print(reflect_get_function_name(add));
```

### reflect_get_function_arity

```atlas
fn reflect_get_function_arity(value: any) -> number
```

Returns the number of declared parameters for a function value.

**Parameters:**
- `value`: Function value

**Returns:** Parameter count.

**Example:**
```atlas
fn add(a, b) { return a + b; }
print(reflect_get_function_arity(add));
```
