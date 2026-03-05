# JSON Functions

JSON parsing, serialization, validation, and manipulation.

## parseJSON

```atlas
fn parseJSON(str: string) -> Result<JsonValue, string>
```

Parses JSON string into JsonValue.

**Parameters:**
- `str` - JSON text

**Returns:**
- `Ok(JsonValue)` on success
- `Err(string)` if JSON is malformed

**Type Mapping:**
- JSON null → `JsonValue::Null`
- JSON boolean → `JsonValue::Bool`
- JSON number → `JsonValue::Number`
- JSON string → `JsonValue::String`
- JSON array → `JsonValue::Array`
- JSON object → `JsonValue::Object`

## toJSON

```atlas
fn toJSON(value: any) -> string
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
fn isValidJSON(str: string) -> bool
```

Validates if string is valid JSON without parsing.

**Parameters:**
- `str` - String to validate

**Returns:** `bool` - True if valid JSON, false otherwise

**Note:** More efficient than `parseJSON` when only validation is needed

## prettifyJSON

```atlas
fn prettifyJSON(str: string, indent: number) -> string
```

Formats JSON string with indentation.

**Parameters:**
- `str` - JSON string
- `indent` - Number of spaces per indentation level

**Returns:** `string` - Prettified JSON

## minifyJSON

```atlas
fn minifyJSON(str: string) -> string
```

Removes all whitespace from JSON string.

**Parameters:**
- `str` - JSON string

**Returns:** `string` - Minified JSON (compact)

## jsonAsString

```atlas
fn jsonAsString(json: JsonValue) -> string?
```

Extracts string value from JsonValue.

**Parameters:**
- `json` - JsonValue

**Returns:** `string?` - String value or None if not a string

## jsonAsNumber

```atlas
fn jsonAsNumber(json: JsonValue) -> number?
```

Extracts number value from JsonValue.

**Parameters:**
- `json` - JsonValue

**Returns:** `number?` - Number value or None if not a number

## jsonAsBool

```atlas
fn jsonAsBool(json: JsonValue) -> bool?
```

Extracts boolean value from JsonValue.

**Parameters:**
- `json` - JsonValue

**Returns:** `bool?` - Boolean value or None if not a boolean

## jsonGetString

```atlas
fn jsonGetString(json: JsonValue, key: string) -> string?
```

Gets string value from JSON object by key.

**Parameters:**
- `json` - JsonValue (must be object)
- `key` - Object key

**Returns:** `string?` - String value or None if key missing or wrong type

## jsonGetNumber

```atlas
fn jsonGetNumber(json: JsonValue, key: string) -> number?
```

Gets number value from JSON object by key.

**Parameters:**
- `json` - JsonValue (must be object)
- `key` - Object key

**Returns:** `number?` - Number value or None if key missing or wrong type

## jsonGetBool

```atlas
fn jsonGetBool(json: JsonValue, key: string) -> bool?
```

Gets boolean value from JSON object by key.

**Parameters:**
- `json` - JsonValue (must be object)
- `key` - Object key

**Returns:** `bool?` - Boolean value or None if key missing or wrong type

## jsonGetArray

```atlas
fn jsonGetArray(json: JsonValue, key: string) -> JsonValue?
```

Gets array value from JSON object by key.

**Parameters:**
- `json` - JsonValue (must be object)
- `key` - Object key

**Returns:** `JsonValue?` - Array value or None if key missing or wrong type

## jsonGetObject

```atlas
fn jsonGetObject(json: JsonValue, key: string) -> JsonValue?
```

Gets object value from JSON object by key.

**Parameters:**
- `json` - JsonValue (must be object)
- `key` - Object key

**Returns:** `JsonValue?` - Object value or None if key missing or wrong type

## jsonIsNull

```atlas
fn jsonIsNull(json: JsonValue) -> bool
```

Checks if JsonValue is null.

**Parameters:**
- `json` - JsonValue

**Returns:** `bool` - True if null, false otherwise
