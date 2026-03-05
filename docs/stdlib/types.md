# Type System Functions

Type checking, conversion, and reflection functions.

## Type Checking

### typeof

```atlas
fn typeof(value: any) -> string
```

Returns the type name of a value as string.

**Returns:** `string` - Type name: "number", "string", "bool", "null", "array", "object", "function", etc.

**Alias:** `type_of`

### isString

```atlas
fn isString(value: any) -> bool
```

Checks if value is a string.

**Returns:** `bool`

### isNumber

```atlas
fn isNumber(value: any) -> bool
```

Checks if value is a number.

**Returns:** `bool`

### isBool

```atlas
fn isBool(value: any) -> bool
```

Checks if value is a boolean.

**Returns:** `bool`

### isNull

```atlas
fn isNull(value: any) -> bool
```

Checks if value is null.

**Returns:** `bool`

### isArray

```atlas
fn isArray(value: any) -> bool
```

Checks if value is an array.

**Returns:** `bool`

### isFunction

```atlas
fn isFunction(value: any) -> bool
```

Checks if value is a function.

**Returns:** `bool`

### isObject

```atlas
fn isObject(value: any) -> bool
```

Checks if value is an object (HashMap).

**Returns:** `bool`

### isType

```atlas
fn isType(value: any) -> bool
```

Checks if value is a type descriptor.

**Returns:** `bool`

## Object Introspection

### hasField

```atlas
fn hasField(obj: any, field: string) -> bool
```

Checks if object has a field with given name.

**Parameters:**
- `obj` - Object to check
- `field` - Field name

**Returns:** `bool` - True if field exists, false otherwise

### hasMethod

```atlas
fn hasMethod(obj: any, method: string) -> bool
```

Checks if object has a method with given name.

**Parameters:**
- `obj` - Object to check
- `method` - Method name

**Returns:** `bool` - True if method exists, false otherwise

### hasTag

```atlas
fn hasTag(value: any, tag: string) -> bool
```

Checks if value has a specific tag (for tagged values).

**Parameters:**
- `value` - Value to check
- `tag` - Tag name

**Returns:** `bool` - True if tag exists, false otherwise

## Type Conversion

### toString

```atlas
fn toString(value: any) -> string
```

Converts value to string representation.

**Parameters:**
- `value` - Any value

**Returns:** `string` - String representation

### toNumber

```atlas
fn toNumber(value: any) -> Result<number, string>
```

Converts value to number.

**Parameters:**
- `value` - Value to convert

**Returns:**
- `Ok(number)` on success
- `Err(string)` if conversion fails

### toBool

```atlas
fn toBool(value: any) -> bool
```

Converts value to boolean.

**Parameters:**
- `value` - Value to convert

**Returns:** `bool`

**Truthiness rules:**
- `0`, `""`, `null`, `false` are falsy
- All other values are truthy

## Parsing

### parseInt

```atlas
fn parseInt(str: string) -> Result<number, string>
```

Parses string as integer.

**Parameters:**
- `str` - String to parse

**Returns:**
- `Ok(number)` on success
- `Err(string)` if invalid

**Behavior:**
- Skips leading whitespace
- Stops at first non-digit character
- Handles optional sign

### parseFloat

```atlas
fn parseFloat(str: string) -> Result<number, string>
```

Parses string as floating-point number.

**Parameters:**
- `str` - String to parse

**Returns:**
- `Ok(number)` on success
- `Err(string)` if invalid

**Behavior:**
- Skips leading whitespace
- Parses decimal point and exponent notation
- Stops at first invalid character
- Handles optional sign

## Option<T> Constructors

### Some

```atlas
fn Some(value: T) -> Option<T>
```

Constructs Some(value) - Option with a value.

**Parameters:**
- `value` - The value to wrap

**Returns:** `Option<T>`

### None

```atlas
fn None() -> Option<any>
```

Constructs None - Option without a value.

**Returns:** `Option<any>`

## Result<T,E> Constructors

### Ok

```atlas
fn Ok(value: T) -> Result<T, any>
```

Constructs Ok(value) - successful Result.

**Parameters:**
- `value` - The success value

**Returns:** `Result<T, any>`

### Err

```atlas
fn Err(error: E) -> Result<any, E>
```

Constructs Err(error) - failed Result.

**Parameters:**
- `error` - The error value

**Returns:** `Result<any, E>`

## Option<T> Operations

### is_some

```atlas
fn is_some(opt: Option<T>) -> bool
```

Checks if Option has a value (is Some).

**Parameters:**
- `opt` - Option to check

**Returns:** `bool`

### is_none

```atlas
fn is_none(opt: Option<T>) -> bool
```

Checks if Option is None.

**Parameters:**
- `opt` - Option to check

**Returns:** `bool`

## Result<T,E> Operations

### is_ok

```atlas
fn is_ok(res: Result<T, E>) -> bool
```

Checks if Result is Ok.

**Parameters:**
- `res` - Result to check

**Returns:** `bool`

### is_err

```atlas
fn is_err(res: Result<T, E>) -> bool
```

Checks if Result is Err.

**Parameters:**
- `res` - Result to check

**Returns:** `bool`

## Unwrapping

### unwrap

```atlas
fn unwrap(opt_or_res: Option<T> | Result<T, E>) -> T
```

Unwraps Option or Result value. Panics if None or Err.

**Parameters:**
- `opt_or_res` - Option or Result

**Returns:** `T` - The unwrapped value

**Errors:** Panics if None or Err

### unwrap_or

```atlas
fn unwrap_or(opt_or_res: Option<T> | Result<T, E>, default: T) -> T
```

Unwraps Option or Result with default value.

**Parameters:**
- `opt_or_res` - Option or Result
- `default` - Default value if None/Err

**Returns:** `T` - The value or default
