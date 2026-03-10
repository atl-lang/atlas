# JSON Functions

JSON parsing, serialization, validation, and manipulation.

## parseJSON

```atlas
fn parseJSON(str: string) : Result<json, string>
```

Parses JSON string into `json`.

> **⚠️ AI Generation Note:** `parseJSON` returns `Result<json, string>`, **not** `json`.
> Writing `let data: json = parseJSON(raw)` fails — typechecker sees `Result`, not `json`.
> **Correct pattern:** `let data = unwrap(parseJSON(raw));`
> Use `match` when the input may be invalid JSON.

**Parameters:**
- `str` - JSON text

**Returns:**
- `Ok(json)` on success
- `Err(string)` if JSON is malformed

**Type Mapping:**
- JSON null → `null`
- JSON boolean → `bool`
- JSON number → `number`
- JSON string → `string`
- JSON array → `[]json`
- JSON object → `record` (JSON object)

## toJSON

```atlas
fn toJSON(value: any) : string
```

Serializes Atlas value to JSON string.

**Parameters:**
- `value` - Any Atlas value

**Returns:** `string` - Compact JSON (no whitespace)

**Errors:**
- Circular references detected
- Functions cannot be serialized

## isValidJSON

```atlas
fn isValidJSON(str: string) : bool
```

Validates if string is valid JSON without parsing.

**Parameters:**
- `str` - String to validate

**Returns:** `bool` - True if valid JSON, false otherwise

**Note:** More efficient than `parseJSON` when only validation is needed

## prettifyJSON

```atlas
fn prettifyJSON(str: string, indent: number) : string
```

Formats JSON string with indentation.

**Parameters:**
- `str` - JSON string
- `indent` - Number of spaces per indentation level

**Returns:** `string` - Prettified JSON

## minifyJSON

```atlas
fn minifyJSON(str: string) : string
```

Removes all whitespace from JSON string.

**Parameters:**
- `str` - JSON string

**Returns:** `string` - Minified JSON (compact)

## jsonAsString

```atlas
fn jsonAsString(json: json) : Option<string>
```

Extracts string value from `json`.

**Parameters:**
- `json` - JSON value

**Returns:** `Option<string>` - String value or None if not a string

## jsonAsNumber

```atlas
fn jsonAsNumber(json: json) : Option<number>
```

Extracts number value from `json`.

**Parameters:**
- `json` - JSON value

**Returns:** `Option<number>` - Number value or None if not a number

## jsonAsBool

```atlas
fn jsonAsBool(json: json) : Option<bool>
```

Extracts boolean value from `json`.

**Parameters:**
- `json` - JSON value

**Returns:** `Option<bool>` - Boolean value or None if not a boolean

## jsonGetString

```atlas
fn jsonGetString(json: json, key: string) : Option<string>
```

Gets string value from JSON object by key.

**Parameters:**
- `json` - JSON value (must be object)
- `key` - Object key

**Returns:** `Option<string>` - String value or None if key missing or wrong type

## jsonGetNumber

```atlas
fn jsonGetNumber(json: json, key: string) : Option<number>
```

Gets number value from JSON object by key.

**Parameters:**
- `json` - JSON value (must be object)
- `key` - Object key

**Returns:** `Option<number>` - Number value or None if key missing or wrong type

## jsonGetBool

```atlas
fn jsonGetBool(json: json, key: string) : Option<bool>
```

Gets boolean value from JSON object by key.

**Parameters:**
- `json` - JSON value (must be object)
- `key` - Object key

**Returns:** `Option<bool>` - Boolean value or None if key missing or wrong type

## jsonGetArray

```atlas
fn jsonGetArray(json: json, key: string) : Option<json>
```

Gets array value from JSON object by key.

**Parameters:**
- `json` - JSON value (must be object)
- `key` - Object key

**Returns:** `Option<json>` - Array value or None if key missing or wrong type

## jsonGetObject

```atlas
fn jsonGetObject(json: json, key: string) : Option<json>
```

Gets object value from JSON object by key.

**Parameters:**
- `json` - JSON value (must be object)
- `key` - Object key

**Returns:** `Option<json>` - Object value or None if key missing or wrong type

## jsonIsNull

```atlas
fn jsonIsNull(json: json) : bool
```

Checks if JSON value is null.

**Parameters:**
- `json` - JSON value

**Returns:** `bool` - True if null, false otherwise
