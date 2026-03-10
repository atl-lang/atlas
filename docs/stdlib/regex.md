# Regex Functions

Regular expression pattern matching and manipulation.

## Creating Regex

### regexNew

```atlas
fn regexNew(pattern: string) : Result<Regex, string>
```

Creates regex pattern from string.

**Parameters:**
- `pattern` - Regular expression pattern

**Returns:**
- `Ok(Regex)` on success
- `Err(string)` if pattern is invalid

**Note:** Uses standard Rust regex syntax

### regexNewWithFlags

```atlas
fn regexNewWithFlags(pattern: string, flags: string) : Result<Regex, string>
```

Creates regex with flags.

**Parameters:**
- `pattern` - Regular expression pattern
- `flags` - Flag string (e.g., "i" for case-insensitive, "m" for multiline)

**Returns:**
- `Ok(Regex)` on success
- `Err(string)` if invalid

**Flags:**
- `i` - Case insensitive matching
- `m` - Multiline mode (^ and $ match line boundaries)
- `s` - Dotall mode (. matches newlines)
- `x` - Verbose mode (whitespace ignored)

## Matching

### regexIsMatch

```atlas
fn regexIsMatch(regex: Regex, text: string) : Result<bool, string>
```

Tests if pattern matches any part of text.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search

**Returns:**
- `Ok(bool)` - True if match found
- `Err(string)` on error

### regexTest

```atlas
fn regexTest(regex: Regex, text: string) : bool
```

Tests if pattern matches (simpler version of isMatch).

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search

**Returns:** `bool` - True if match found

## Finding Matches

### regexFind

```atlas
fn regexFind(regex: Regex, text: string) : Result<Option<string>, string>
```

Finds first match in text.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search

**Returns:**
- `Ok(Option<string>)` - First match or None
- `Err(string)` on error

### regexFindAll

```atlas
fn regexFindAll(regex: Regex, text: string) : Result<string[], string>
```

Finds all non-overlapping matches.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search

**Returns:**
- `Ok(string[])` - Array of all matches
- `Err(string)` on error

### regexCaptures

```atlas
fn regexCaptures(regex: Regex, text: string) : Result<string[]?, string>
```

Gets capture groups from first match.

**Parameters:**
- `regex` - Compiled regex with capture groups
- `text` - Text to search

**Returns:**
- `Ok(string[]?)` - First match and capture groups, or None
- `Err(string)` on error

**Note:** Array includes full match at index 0, then groups

### regexCapturesNamed

```atlas
fn regexCapturesNamed(regex: Regex, text: string) : Result<Option<object>, string>
```

Gets named capture groups from first match.

**Parameters:**
- `regex` - Compiled regex with named groups (?P<name>...)
- `text` - Text to search

**Returns:**
- `Ok(Option<object>)` - Object with group names as keys, or None
- `Err(string)` on error

### regexMatchIndices

```atlas
fn regexMatchIndices(regex: Regex, text: string) : Result<[number, number][], string>
```

Gets start/end indices of all matches.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search

**Returns:**
- `Ok([number, number][])` - Array of [start, end] pairs
- `Err(string)` on error

## Replacement

### regexReplace

```atlas
fn regexReplace(regex: Regex, text: string, replacement: string) : Result<string, string>
```

Replaces first match with replacement string.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search
- `replacement` - Replacement string

**Returns:**
- `Ok(string)` - Text with first match replaced
- `Err(string)` on error

**Note:** Supports backreferences: $0 = full match, $1/$2 = groups

### regexReplaceAll

```atlas
fn regexReplaceAll(regex: Regex, text: string, replacement: string) : Result<string, string>
```

Replaces all matches with replacement string.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search
- `replacement` - Replacement string

**Returns:**
- `Ok(string)` - Text with all matches replaced
- `Err(string)` on error

**Note:** Supports backreferences: $0 = full match, $1/$2 = groups

### regexReplaceWith

```atlas
fn regexReplaceWith(regex: Regex, text: string, callback: fn(object) : string) : string
```

Replaces the first match using a callback.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search
- `callback` - Function receiving match data

**Returns:** `string` - Updated text

**Example:**
```atlas
let re = regexNew(\"foo\").unwrap();
let out = regexReplaceWith(re, \"foo bar\", fn(match) { return \"baz\"; });
```

**Match data:**
- `text` - Matched substring
- `start` - Start index
- `end` - End index
- `groups` - Array of capture groups (index 0 is full match)

### regexReplaceAllWith

```atlas
fn regexReplaceAllWith(regex: Regex, text: string, callback: fn(object): string): string
```

Replaces all matches using a callback.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to search
- `callback` - Function receiving match data

**Returns:** `string` - Updated text

**Example:**
```atlas
let re = regexNew(\"foo\").unwrap();
let out = regexReplaceAllWith(re, \"foo foo\", fn(match) { return \"bar\"; });
```

**Match data:**
- `text` - Matched substring
- `start` - Start index
- `end` - End index
- `groups` - Array of capture groups (index 0 is full match)

## Splitting

### regexSplit

```atlas
fn regexSplit(regex: Regex, text: string): Result<string[], string>
```

Splits text by regex pattern.

**Parameters:**
- `regex` - Compiled regex (pattern to split on)
- `text` - Text to split

**Returns:**
- `Ok(string[])` - Array of parts
- `Err(string)` on error

### regexSplitN

```atlas
fn regexSplitN(regex: Regex, text: string, n: number): Result<string[], string>
```

Splits text by pattern, maximum n parts.

**Parameters:**
- `regex` - Compiled regex
- `text` - Text to split
- `n` - Maximum number of parts (integer)

**Returns:**
- `Ok(string[])` - Array of at most n parts
- `Err(string)` on error

## Utility

### regexEscape

```atlas
fn regexEscape(text: string): string
```

Escapes special regex characters in text.

**Parameters:**
- `text` - String to escape

**Returns:** `string` - Text with regex special chars escaped

**Example:**
```atlas
regexEscape("a.b*c?") // Returns "a\\.b\\*c\\?"
```

## Example Usage

```atlas
let pattern = regexNew("[0-9]+");
let text = "abc123def456";

// Check if matches
print(regexIsMatch(pattern, text)); // true

// Find first number
print(regexFind(pattern, text)); // Ok("123")

// Find all numbers
print(regexFindAll(pattern, text)); // Ok(["123", "456"])

// Replace numbers
print(regexReplaceAll(pattern, text, "X")); // Ok("abcXdefX")

// Split by numbers
print(regexSplit(pattern, text)); // Ok(["abc", "def", ""])
```"