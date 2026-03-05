# String Functions

Complete string manipulation API with Unicode support.

## split

```atlas
fn split(str: string, separator: string) -> string[]
```

Splits a string by separator into array of parts. If separator is empty, returns array of individual characters.

**Parameters:**
- `str` - String to split
- `separator` - Separator string (empty = split into chars)

**Returns:** `string[]` - Array of string parts

## join

```atlas
fn join(arr: string[], separator: string) -> string
```

Joins an array of strings with separator. Alias: `strJoin`.

**Parameters:**
- `arr` - Array of strings
- `separator` - Separator between elements

**Returns:** `string` - Joined string

## trim

```atlas
fn trim(str: string) -> string
```

Removes leading and trailing whitespace (Unicode-aware).

**Parameters:**
- `str` - String to trim

**Returns:** `string` - Trimmed string

## trimStart

```atlas
fn trimStart(str: string) -> string
```

Removes leading whitespace (Unicode-aware).

**Parameters:**
- `str` - String to trim

**Returns:** `string` - Trimmed string

## trimEnd

```atlas
fn trimEnd(str: string) -> string
```

Removes trailing whitespace (Unicode-aware).

**Parameters:**
- `str` - String to trim

**Returns:** `string` - Trimmed string

## indexOf

```atlas
fn indexOf(str: string, search: string) -> number?
```

Finds first occurrence of search string. Returns None if not found. Alias: `strIndexOf`.

**Parameters:**
- `str` - String to search in
- `search` - String to find

**Returns:** `number?` - Index of first match or None

## lastIndexOf

```atlas
fn lastIndexOf(str: string, search: string) -> number?
```

Finds last occurrence of search string. Returns None if not found. Alias: `strLastIndexOf`.

**Parameters:**
- `str` - String to search in
- `search` - String to find

**Returns:** `number?` - Index of last match or None

## includes

```atlas
fn includes(str: string, search: string) -> bool
```

Checks if string contains substring.

**Parameters:**
- `str` - String to search in
- `search` - Substring to find

**Returns:** `bool` - True if found, false otherwise

## toUpperCase

```atlas
fn toUpperCase(str: string) -> string
```

Converts string to uppercase (Unicode-aware).

**Parameters:**
- `str` - String to convert

**Returns:** `string` - Uppercase string

## toLowerCase

```atlas
fn toLowerCase(str: string) -> string
```

Converts string to lowercase (Unicode-aware).

**Parameters:**
- `str` - String to convert

**Returns:** `string` - Lowercase string

## substring

```atlas
fn substring(str: string, start: number, end: number) -> string
```

Extracts substring from start (inclusive) to end (exclusive). Validates UTF-8 boundaries.

**Parameters:**
- `str` - String to slice
- `start` - Start index (integer)
- `end` - End index (integer)

**Returns:** `string` - Substring

**Errors:**
- Indices must be integers
- Start index > end index
- Indices out of bounds
- Indices not on UTF-8 boundaries

## charAt

```atlas
fn charAt(str: string, index: number) -> string?
```

Gets character at index (returns grapheme cluster, not byte).

**Parameters:**
- `str` - String to access
- `index` - Character index (integer)

**Returns:** `string?` - Character or None if out of bounds

## repeat

```atlas
fn repeat(str: string, count: number) -> string
```

Repeats string count times. Limited to prevent memory abuse.

**Parameters:**
- `str` - String to repeat
- `count` - Number of repetitions (integer)

**Returns:** `string` - Repeated string

**Errors:**
- Count must be integer
- Count cannot be negative
- Count exceeds maximum (1,000,000)

## replace

```atlas
fn replace(str: string, search: string, replacement: string) -> string
```

Replaces first occurrence of search with replacement.

**Parameters:**
- `str` - String to search in
- `search` - Text to find
- `replacement` - Text to replace with

**Returns:** `string` - String with replacement made

## padStart

```atlas
fn padStart(str: string, length: number, fill: string) -> string
```

Pads string at start to reach target length. Fill string is repeated as needed. If already >= length, returns original.

**Parameters:**
- `str` - String to pad
- `length` - Target length (integer)
- `fill` - Padding string

**Returns:** `string` - Padded string

## padEnd

```atlas
fn padEnd(str: string, length: number, fill: string) -> string
```

Pads string at end to reach target length. Fill string is repeated as needed. If already >= length, returns original.

**Parameters:**
- `str` - String to pad
- `length` - Target length (integer)
- `fill` - Padding string

**Returns:** `string` - Padded string

## startsWith

```atlas
fn startsWith(str: string, prefix: string) -> bool
```

Checks if string starts with prefix.

**Parameters:**
- `str` - String to check
- `prefix` - Prefix to match

**Returns:** `bool` - True if matches, false otherwise

## endsWith

```atlas
fn endsWith(str: string, suffix: string) -> bool
```

Checks if string ends with suffix.

**Parameters:**
- `str` - String to check
- `suffix` - Suffix to match

**Returns:** `bool` - True if matches, false otherwise
