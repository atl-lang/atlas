# Atlas Standard Library API Reference

**Version:** v0.2 | **Status:** Production Ready

Complete reference for all built-in functions available in Atlas programs.

---

## Table of Contents

- [Core Functions](#core-functions)
- [String Functions](#string-functions)
- [Array Functions](#array-functions)
- [Math Functions](#math-functions)
- [Type Functions](#type-functions)
- [JSON Functions](#json-functions)
- [File System Functions](#file-system-functions)
- [Path Functions](#path-functions)
- [Process Functions](#process-functions)
- [Environment Functions](#environment-functions)
- [DateTime Functions](#datetime-functions)
- [Duration Functions](#duration-functions)
- [HTTP Functions](#http-functions)
- [Regex Functions](#regex-functions)
- [Collections: HashMap](#collections-hashmap)
- [Collections: HashSet](#collections-hashset)
- [Collections: Queue](#collections-queue)
- [Collections: Stack](#collections-stack)
- [Async/Concurrency Functions](#asyncconcurrency-functions)
- [Future Functions](#future-functions)
- [Compression Functions](#compression-functions)
- [Reflection Functions](#reflection-functions)
- [Testing/Assertion Functions](#testingassertion-functions)
- [Result Functions](#result-functions)
- [Option Functions](#option-functions)

---

## Core Functions

### `print(value: any) -> void`

Prints a value to standard output, followed by a newline.

```atlas
print("Hello, World!");   // Hello, World!
print(42);                // 42
print([1, 2, 3]);         // [1, 2, 3]
```

---

### `len(value: string | array) -> number`

Returns the length of a string (number of characters) or an array (number of elements).

```atlas
len("hello");       // 5
len([1, 2, 3]);     // 3
len("");            // 0
```

---

### `str(value: any) -> string`

Converts any value to its string representation.

```atlas
str(42);            // "42"
str(true);          // "true"
str(null);          // "null"
str([1, 2]);        // "[1, 2]"
```

---

### `typeof(value: any) -> string`

Returns the type name of a value as a string.

```atlas
typeof(42);         // "number"
typeof("hello");    // "string"
typeof(true);       // "bool"
typeof(null);       // "null"
typeof([]);         // "array"
typeof({});         // "object"
```

---

### `toString(value: any) -> string`

Alias for `str`. Converts any value to its string representation.

```atlas
toString(3.14);     // "3.14"
toString(false);    // "false"
```

---

### `toNumber(value: string | bool) -> number`

Converts a string or boolean to a number. Throws on invalid input.

```atlas
toNumber("42");     // 42
toNumber("3.14");   // 3.14
toNumber(true);     // 1
toNumber(false);    // 0
```

---

### `toBool(value: any) -> bool`

Converts a value to boolean using Atlas truthiness rules.

```atlas
toBool(0);          // false
toBool(1);          // true
toBool("");         // false
toBool("hello");    // true
toBool(null);       // false
```

---

### `hasField(obj: object, field: string) -> bool`

Returns `true` if an object has the specified field.

```atlas
let p = { name: "Alice", age: 30 };
hasField(p, "name");    // true
hasField(p, "email");   // false
```

---

### `hasMethod(obj: object, method: string) -> bool`

Returns `true` if an object has the specified method.

```atlas
hasMethod(myObj, "greet");   // true or false
```

---

### `hasTag(value: any, tag: string) -> bool`

Returns `true` if a tagged union value has the specified tag.

```atlas
let result = Ok(42);
hasTag(result, "Ok");    // true
hasTag(result, "Err");   // false
```

---

### `expect(value: Option<T>, message: string) -> T`

Unwraps an `Option`, throwing with `message` if `None`.

```atlas
let x = Some(42);
expect(x, "must have value");   // 42
```

---

### `unwrap(value: Option<T> | Result<T, E>) -> T`

Unwraps an `Option` or `Result`. Throws if `None` or `Err`.

```atlas
unwrap(Some(42));    // 42
unwrap(Ok(99));      // 99
```

---

### `unwrap_or(value: Option<T> | Result<T, E>, default: T) -> T`

Unwraps an `Option` or `Result`, returning `default` if absent/error.

```atlas
unwrap_or(None, 0);        // 0
unwrap_or(Some(5), 0);     // 5
unwrap_or(Err("x"), -1);   // -1
```

---

### `sleep(ms: number) -> void`

Pauses execution for `ms` milliseconds.

```atlas
sleep(1000);   // wait 1 second
```

---

## String Functions

### `split(s: string, separator: string) -> array`

Splits a string by a separator, returning an array of substrings.

```atlas
split("a,b,c", ",");       // ["a", "b", "c"]
split("hello", "");        // ["h", "e", "l", "l", "o"]
split("no-sep", "|");      // ["no-sep"]
```

---

### `join(arr: array, separator: string) -> string`

Joins an array of values into a string with the given separator.

```atlas
join(["a", "b", "c"], ",");    // "a,b,c"
join([1, 2, 3], " - ");        // "1 - 2 - 3"
join([], ",");                 // ""
```

---

### `trim(s: string) -> string`

Removes leading and trailing whitespace from a string.

```atlas
trim("  hello  ");    // "hello"
trim("\t\n test\n");  // "test"
```

---

### `trimStart(s: string) -> string`

Removes leading whitespace from a string.

```atlas
trimStart("  hello  ");    // "hello  "
```

---

### `trimEnd(s: string) -> string`

Removes trailing whitespace from a string.

```atlas
trimEnd("  hello  ");    // "  hello"
```

---

### `toUpperCase(s: string) -> string`

Converts all characters to uppercase.

```atlas
toUpperCase("hello");    // "HELLO"
toUpperCase("Hello!");   // "HELLO!"
```

---

### `toLowerCase(s: string) -> string`

Converts all characters to lowercase.

```atlas
toLowerCase("HELLO");    // "hello"
toLowerCase("Hello!");   // "hello!"
```

---

### `startsWith(s: string, prefix: string) -> bool`

Returns `true` if the string starts with the given prefix.

```atlas
startsWith("hello world", "hello");    // true
startsWith("hello world", "world");    // false
```

---

### `endsWith(s: string, suffix: string) -> bool`

Returns `true` if the string ends with the given suffix.

```atlas
endsWith("hello world", "world");    // true
endsWith("hello world", "hello");    // false
```

---

### `includes(s: string, substr: string) -> bool`

Returns `true` if the string contains the given substring.

```atlas
includes("hello world", "lo wo");    // true
includes("hello world", "xyz");      // false
```

---

### `indexOf(s: string, substr: string) -> number`

Returns the index of the first occurrence of `substr`, or `-1` if not found.

```atlas
indexOf("hello", "ll");     // 2
indexOf("hello", "xyz");    // -1
```

---

### `lastIndexOf(s: string, substr: string) -> number`

Returns the index of the last occurrence of `substr`, or `-1` if not found.

```atlas
lastIndexOf("hello hello", "hello");    // 6
lastIndexOf("hello", "xyz");            // -1
```

---

### `replace(s: string, from: string, to: string) -> string`

Replaces the first occurrence of `from` with `to`.

```atlas
replace("hello world", "world", "Atlas");    // "hello Atlas"
replace("aaa", "a", "b");                    // "baa"
```

---

### `repeat(s: string, n: number) -> string`

Repeats a string `n` times.

```atlas
repeat("ab", 3);    // "ababab"
repeat("x", 0);     // ""
```

---

### `substring(s: string, start: number, end: number) -> string`

Extracts a substring from index `start` (inclusive) to `end` (exclusive).

```atlas
substring("hello world", 6, 11);    // "world"
substring("hello", 0, 3);           // "hel"
```

---

### `charAt(s: string, index: number) -> string`

Returns the character at the given index.

```atlas
charAt("hello", 0);     // "h"
charAt("hello", 4);     // "o"
```

---

### `padStart(s: string, length: number, pad: string) -> string`

Pads the start of a string until it reaches the target length.

```atlas
padStart("42", 5, "0");     // "00042"
padStart("hi", 6, ".-");    // ".-.-hi"
```

---

### `padEnd(s: string, length: number, pad: string) -> string`

Pads the end of a string until it reaches the target length.

```atlas
padEnd("hi", 5, ".");    // "hi..."
padEnd("x", 4, "ab");   // "xaba"
```

---

### `concat(a: string, b: string) -> string`

Concatenates two strings.

```atlas
concat("hello", " world");    // "hello world"
```

---

### `parseInt(s: string) -> number`

Parses a string as an integer. Throws on invalid input.

```atlas
parseInt("42");      // 42
parseInt("-7");      // -7
parseInt("0xff");    // 255 (hex)
```

---

### `parseFloat(s: string) -> number`

Parses a string as a floating-point number. Throws on invalid input.

```atlas
parseFloat("3.14");      // 3.14
parseFloat("-2.718");    // -2.718
```

---

## Array Functions

### `reverse(arr: array) -> array`

Returns a new array with elements in reverse order.

```atlas
reverse([1, 2, 3]);        // [3, 2, 1]
reverse(["a", "b", "c"]); // ["c", "b", "a"]
```

---

### `flatten(arr: array) -> array`

Flattens a nested array by one level.

```atlas
flatten([[1, 2], [3, 4]]);      // [1, 2, 3, 4]
flatten([1, [2, [3]]]);         // [1, 2, [3]]
```

---

### `slice(arr: array, start: number, end: number) -> array`

Returns a sub-array from index `start` (inclusive) to `end` (exclusive).

```atlas
slice([1, 2, 3, 4, 5], 1, 4);    // [2, 3, 4]
slice([1, 2, 3], 0, 2);           // [1, 2]
```

---

### `pop(arr: array) -> any`

Removes and returns the last element of an array. Throws on empty array.

```atlas
pop([1, 2, 3]);    // 3 (modifies array in place)
```

---

### `shift(arr: array) -> any`

Removes and returns the first element of an array. Throws on empty array.

```atlas
shift([1, 2, 3]);    // 1 (modifies array in place)
```

---

### `unshift(arr: array, value: any) -> void`

Inserts a value at the beginning of an array.

```atlas
let arr = [2, 3];
unshift(arr, 1);    // arr is now [1, 2, 3]
```

---

### `arrayIncludes(arr: array, value: any) -> bool`

Returns `true` if the array contains the given value (deep equality).

```atlas
arrayIncludes([1, 2, 3], 2);        // true
arrayIncludes(["a", "b"], "c");     // false
arrayIncludes([[1, 2]], [1, 2]);    // true (deep)
```

---

### `arrayIndexOf(arr: array, value: any) -> number`

Returns the index of the first occurrence of `value` in the array, or `-1`.

```atlas
arrayIndexOf([10, 20, 30], 20);    // 1
arrayIndexOf([1, 2, 3], 99);       // -1
```

---

### `arrayLastIndexOf(arr: array, value: any) -> number`

Returns the index of the last occurrence of `value` in the array, or `-1`.

```atlas
arrayLastIndexOf([1, 2, 1], 1);    // 2
```

---

## Math Functions

### `abs(x: number) -> number`

Returns the absolute value of `x`.

```atlas
abs(-5);      // 5
abs(3.14);    // 3.14
abs(0);       // 0
```

---

### `floor(x: number) -> number`

Returns the largest integer less than or equal to `x`.

```atlas
floor(3.7);     // 3
floor(-1.2);    // -2
floor(4.0);     // 4
```

---

### `ceil(x: number) -> number`

Returns the smallest integer greater than or equal to `x`.

```atlas
ceil(3.1);     // 4
ceil(-1.9);    // -1
ceil(4.0);     // 4
```

---

### `round(x: number) -> number`

Rounds `x` to the nearest integer (ties round to even).

```atlas
round(3.5);     // 4
round(2.5);     // 2
round(-1.5);    // -2
```

---

### `sqrt(x: number) -> number`

Returns the square root of `x`. Throws if `x` is negative.

```atlas
sqrt(9);      // 3
sqrt(2);      // 1.4142135623730951
sqrt(0);      // 0
```

---

### `pow(base: number, exp: number) -> number`

Returns `base` raised to the power of `exp`.

```atlas
pow(2, 10);     // 1024
pow(3, 3);      // 27
pow(4, 0.5);    // 2
```

---

### `log(x: number) -> number`

Returns the natural logarithm (base e) of `x`.

```atlas
log(1);        // 0
log(2.718);    // ~1.0
```

---

### `sin(x: number) -> number`

Returns the sine of `x` (in radians).

```atlas
sin(0);          // 0
sin(3.14159);    // ~0
```

---

### `cos(x: number) -> number`

Returns the cosine of `x` (in radians).

```atlas
cos(0);    // 1
cos(3.14159);    // ~-1
```

---

### `tan(x: number) -> number`

Returns the tangent of `x` (in radians).

```atlas
tan(0);    // 0
```

---

### `asin(x: number) -> number`

Returns the arcsine of `x` in radians. `x` must be in `[-1, 1]`.

```atlas
asin(0);    // 0
asin(1);    // 1.5707963267948966
```

---

### `acos(x: number) -> number`

Returns the arccosine of `x` in radians. `x` must be in `[-1, 1]`.

```atlas
acos(1);    // 0
acos(0);    // 1.5707963267948966
```

---

### `atan(x: number) -> number`

Returns the arctangent of `x` in radians.

```atlas
atan(0);    // 0
atan(1);    // 0.7853981633974483
```

---

### `max(a: number, b: number) -> number`

Returns the larger of two numbers.

```atlas
max(3, 7);      // 7
max(-1, -5);    // -1
```

---

### `min(a: number, b: number) -> number`

Returns the smaller of two numbers.

```atlas
min(3, 7);      // 3
min(-1, -5);    // -5
```

---

### `clamp(x: number, low: number, high: number) -> number`

Clamps `x` to the range `[low, high]`.

```atlas
clamp(5, 0, 10);     // 5
clamp(-3, 0, 10);    // 0
clamp(15, 0, 10);    // 10
```

---

### `sign(x: number) -> number`

Returns `-1`, `0`, or `1` based on the sign of `x`.

```atlas
sign(-7);    // -1
sign(0);     // 0
sign(3);     // 1
```

---

### `random() -> number`

Returns a random floating-point number in `[0, 1)`.

```atlas
random();    // e.g., 0.7231894...
```

---

## Type Functions

### `isString(value: any) -> bool`

Returns `true` if the value is a string.

```atlas
isString("hello");    // true
isString(42);         // false
```

---

### `isNumber(value: any) -> bool`

Returns `true` if the value is a number.

```atlas
isNumber(3.14);    // true
isNumber("3");     // false
```

---

### `isBool(value: any) -> bool`

Returns `true` if the value is a boolean.

```atlas
isBool(true);    // true
isBool(1);       // false
```

---

### `isNull(value: any) -> bool`

Returns `true` if the value is `null`.

```atlas
isNull(null);     // true
isNull(false);    // false
```

---

### `isArray(value: any) -> bool`

Returns `true` if the value is an array.

```atlas
isArray([1, 2]);    // true
isArray("abc");     // false
```

---

### `isObject(value: any) -> bool`

Returns `true` if the value is an object.

```atlas
isObject({ x: 1 });    // true
isObject([]);           // false
```

---

### `isFunction(value: any) -> bool`

Returns `true` if the value is a function.

```atlas
fn greet() -> void {}
isFunction(greet);    // true
isFunction(42);       // false
```

---

### `isType(value: any, type_name: string) -> bool`

Returns `true` if the value's type matches the given type name.

```atlas
isType(42, "number");     // true
isType("hi", "string");   // true
isType(null, "null");     // true
```

---

## JSON Functions

### `parseJSON(s: string) -> any`

Parses a JSON string and returns the corresponding Atlas value.

```atlas
parseJSON('{"name": "Alice", "age": 30}');
// { name: "Alice", age: 30 }

parseJSON("[1, 2, 3]");    // [1, 2, 3]
parseJSON('"hello"');      // "hello"
```

---

### `toJSON(value: any) -> string`

Serializes an Atlas value to a JSON string (compact format).

```atlas
toJSON({ name: "Alice", age: 30 });    // '{"name":"Alice","age":30}'
toJSON([1, 2, 3]);                      // "[1,2,3]"
toJSON("hello");                        // '"hello"'
```

---

### `prettifyJSON(s: string) -> string`

Formats a JSON string with indentation for readability.

```atlas
prettifyJSON('{"a":1,"b":2}');
// {
//   "a": 1,
//   "b": 2
// }
```

---

### `minifyJSON(s: string) -> string`

Removes all whitespace from a JSON string.

```atlas
minifyJSON('{ "a" : 1 }');    // '{"a":1}'
```

---

### `isValidJSON(s: string) -> bool`

Returns `true` if the string is valid JSON.

```atlas
isValidJSON('{"key": "value"}');    // true
isValidJSON("not json");             // false
isValidJSON("null");                 // true
```

---

### `jsonAsString(value: any) -> string`

Extracts a string from a parsed JSON value, throwing if not a string.

```atlas
let data = parseJSON('"hello"');
jsonAsString(data);    // "hello"
```

---

### `jsonAsNumber(value: any) -> number`

Extracts a number from a parsed JSON value.

```atlas
let data = parseJSON("42");
jsonAsNumber(data);    // 42
```

---

### `jsonAsBool(value: any) -> bool`

Extracts a boolean from a parsed JSON value.

```atlas
let data = parseJSON("true");
jsonAsBool(data);    // true
```

---

### `jsonIsNull(value: any) -> bool`

Returns `true` if the parsed JSON value is null.

```atlas
let data = parseJSON("null");
jsonIsNull(data);    // true
```

---

## File System Functions

### `readFile(path: string) -> string`

Reads a file and returns its contents as a string. Requires file-read permission.

```atlas
let content = readFile("config.json");
```

---

### `writeFile(path: string, content: string) -> void`

Writes a string to a file, creating or overwriting it. Requires file-write permission.

```atlas
writeFile("output.txt", "Hello, World!\n");
```

---

### `appendFile(path: string, content: string) -> void`

Appends content to a file. Requires file-write permission.

```atlas
appendFile("log.txt", "New entry\n");
```

---

### `fileExists(path: string) -> bool`

Returns `true` if the file exists.

```atlas
fileExists("config.json");    // true or false
```

---

### `fileInfo(path: string) -> object`

Returns metadata about a file: `name`, `size`, `is_dir`, `is_file`, `modified`.

```atlas
let info = fileInfo("config.json");
print(info.size);
```

---

### `readDir(path: string) -> array`

Lists entries in a directory. Returns array of entry objects.

```atlas
let entries = readDir("./src");
```

---

### `createDir(path: string) -> void`

Creates a directory. Throws if it already exists.

```atlas
createDir("./output");
```

---

### `removeFile(path: string) -> void`

Deletes a file. Requires file-write permission.

```atlas
removeFile("temp.txt");
```

---

### `removeDir(path: string) -> void`

Deletes an empty directory.

```atlas
removeDir("./empty-dir");
```

---

### `getCwd() -> string`

Returns the current working directory as an absolute path.

```atlas
let cwd = getCwd();
print(cwd);    // e.g., "/home/user/project"
```

---

### `fsIsFile(path: string) -> bool`

Returns `true` if the path is a regular file.

```atlas
fsIsFile("main.atl");    // true
fsIsFile("./src");       // false
```

---

### `fsIsDir(path: string) -> bool`

Returns `true` if the path is a directory.

```atlas
fsIsDir("./src");        // true
fsIsDir("main.atl");    // false
```

---

### `fsIsSymlink(path: string) -> bool`

Returns `true` if the path is a symbolic link.

```atlas
fsIsSymlink("link");    // true or false
```

---

### `fsSize(path: string) -> number`

Returns the size of a file in bytes.

```atlas
fsSize("data.bin");    // e.g., 1024
```

---

### `fsMtime(path: string) -> number`

Returns the last modification time as a Unix timestamp (seconds).

```atlas
fsMtime("file.txt");    // e.g., 1708444800
```

---

### `fsMkdir(path: string) -> void`

Creates a directory (single level). Throws if parent doesn't exist.

```atlas
fsMkdir("./output");
```

---

### `fsMkdirp(path: string) -> void`

Creates a directory and all parent directories (like `mkdir -p`).

```atlas
fsMkdirp("./a/b/c/d");
```

---

### `fsRmdir(path: string) -> void`

Removes an empty directory.

```atlas
fsRmdir("./empty");
```

---

### `fsRmdirRecursive(path: string) -> void`

Recursively removes a directory and all its contents.

```atlas
fsRmdirRecursive("./build");
```

---

### `fsReaddir(path: string) -> array`

Returns an array of filenames in the directory (not full paths).

```atlas
let names = fsReaddir("./src");
```

---

### `fsWalk(path: string) -> array`

Recursively walks a directory tree, returning all file paths.

```atlas
let files = fsWalk("./src");
```

---

### `fsSymlink(target: string, link: string) -> void`

Creates a symbolic link.

```atlas
fsSymlink("./real-file.txt", "./link.txt");
```

---

### `fsReadlink(path: string) -> string`

Returns the target of a symbolic link.

```atlas
fsReadlink("./link.txt");    // "./real-file.txt"
```

---

### `fsResolveSymlink(path: string) -> string`

Resolves a symbolic link to its canonical path.

```atlas
fsResolveSymlink("./link.txt");    // "/absolute/path/real-file.txt"
```

---

### `fsTmpdir() -> string`

Returns the system temporary directory path.

```atlas
fsTmpdir();    // e.g., "/tmp" or "C:\\Users\\User\\AppData\\Local\\Temp"
```

---

### `fsTmpfile() -> string`

Creates a temporary file and returns its path.

```atlas
let tmp = fsTmpfile();
writeFile(tmp, "temp content");
```

---

### `fsTmpfileNamed(name: string) -> string`

Creates a named temporary file and returns its path.

```atlas
let tmp = fsTmpfileNamed("my-temp.txt");
```

---

### `fsFilterEntries(entries: array, predicate: string) -> array`

Filters directory entries by type: `"file"`, `"dir"`, or `"symlink"`.

```atlas
let entries = fsReaddir("./src");
let files = fsFilterEntries(entries, "file");
```

---

### `fsSortEntries(entries: array, by: string) -> array`

Sorts directory entries by `"name"`, `"size"`, or `"mtime"`.

```atlas
let sorted = fsSortEntries(entries, "name");
```

---

### `fsGetTempDir() -> string`

Returns the system temporary directory (alias for `fsTmpdir`).

```atlas
let tmp = fsGetTempDir();
```

---

## Path Functions

### `pathJoin(base: string, ...parts: string) -> string`

Joins path components using the platform separator.

```atlas
pathJoin("/home/user", "projects", "atlas");    // "/home/user/projects/atlas"
```

---

### `pathJoinArray(parts: array) -> string`

Joins an array of path components.

```atlas
pathJoinArray(["home", "user", "file.txt"]);    // "home/user/file.txt"
```

---

### `pathDirname(path: string) -> string`

Returns the directory component of a path.

```atlas
pathDirname("/home/user/file.txt");    // "/home/user"
```

---

### `pathBasename(path: string) -> string`

Returns the filename component of a path.

```atlas
pathBasename("/home/user/file.txt");    // "file.txt"
```

---

### `pathExtension(path: string) -> string`

Returns the file extension (without the dot).

```atlas
pathExtension("file.txt");    // "txt"
pathExtension("no-ext");      // ""
```

---

### `pathIsAbsolute(path: string) -> bool`

Returns `true` if the path is absolute.

```atlas
pathIsAbsolute("/home/user");    // true
pathIsAbsolute("./relative");    // false
```

---

### `pathIsRelative(path: string) -> bool`

Returns `true` if the path is relative.

```atlas
pathIsRelative("./file.txt");    // true
pathIsRelative("/absolute");     // false
```

---

### `pathAbsolute(path: string) -> string`

Resolves a relative path against the current directory.

```atlas
pathAbsolute("./src");    // "/home/user/project/src"
```

---

### `pathNormalize(path: string) -> string`

Normalizes a path, resolving `.` and `..` components.

```atlas
pathNormalize("./src/../lib");    // "lib"
pathNormalize("/a/b/../c");       // "/a/c"
```

---

### `pathRelative(from: string, to: string) -> string`

Computes the relative path from `from` to `to`.

```atlas
pathRelative("/a/b", "/a/c");    // "../c"
```

---

### `pathCanonical(path: string) -> string`

Returns the canonical absolute path, resolving symlinks.

```atlas
pathCanonical("./link");    // "/real/path/target"
```

---

### `pathExists(path: string) -> bool`

Returns `true` if the path exists on the filesystem.

```atlas
pathExists("./src");    // true or false
```

---

### `pathParent(path: string) -> string`

Returns the parent directory of a path (alias for `pathDirname`).

```atlas
pathParent("/home/user/file.txt");    // "/home/user"
```

---

### `pathSeparator() -> string`

Returns the platform path separator (`"/"` on Unix, `"\\"` on Windows).

```atlas
pathSeparator();    // "/" or "\\"
```

---

### `pathDelimiter() -> string`

Returns the platform PATH variable delimiter (`:` on Unix, `;` on Windows).

```atlas
pathDelimiter();    // ":" or ";"
```

---

### `pathToPosix(path: string) -> string`

Converts a path to POSIX format (forward slashes).

```atlas
pathToPosix("C:\\Users\\file");    // "C:/Users/file"
```

---

### `pathToWindows(path: string) -> string`

Converts a path to Windows format (backslashes).

```atlas
pathToWindows("/home/user/file");    // "\\home\\user\\file"
```

---

### `pathToPlatform(path: string) -> string`

Converts a path to the current platform's format.

```atlas
pathToPlatform("/home/user/file");    // platform-native separators
```

---

### `pathHomedir() -> string`

Returns the current user's home directory.

```atlas
pathHomedir();    // e.g., "/home/user"
```

---

### `pathTempdir() -> string`

Returns the system temporary directory.

```atlas
pathTempdir();    // e.g., "/tmp"
```

---

### `pathCwd() -> string`

Returns the current working directory.

```atlas
pathCwd();    // e.g., "/home/user/project"
```

---

### `pathDrive(path: string) -> string`

Returns the drive letter on Windows, empty string on Unix.

```atlas
pathDrive("C:\\Users\\file");    // "C:"
pathDrive("/home/user");         // ""
```

---

### `pathExtSeparator() -> string`

Returns the extension separator character (`.`).

```atlas
pathExtSeparator();    // "."
```

---

### `pathEquals(a: string, b: string) -> bool`

Returns `true` if two paths refer to the same location (normalizes before comparing).

```atlas
pathEquals("./src", "src");         // true
pathEquals("/a/../b", "/b");        // true
```

---

### `pathParse(path: string) -> object`

Parses a path into components: `root`, `dir`, `base`, `ext`, `name`.

```atlas
let p = pathParse("/home/user/file.txt");
// { root: "/", dir: "/home/user", base: "file.txt", ext: "txt", name: "file" }
```

---

## Process Functions

### `exec(command: string) -> object`

Executes a shell command synchronously. Returns `{ stdout, stderr, exit_code }`.

Requires process permission.

```atlas
let result = exec("echo hello");
print(result.stdout);      // "hello\n"
print(result.exit_code);   // 0
```

---

### `shell(command: string) -> string`

Executes a shell command and returns stdout. Throws on non-zero exit.

```atlas
let output = shell("ls -la");
```

---

### `spawn(command: string) -> object`

Spawns a process asynchronously. Returns a handle for waiting.

```atlas
let handle = spawn("long-running-task");
```

---

### `getPid() -> number`

Returns the current process ID.

```atlas
getPid();    // e.g., 12345
```

---

## Environment Functions

### `getEnv(name: string) -> string`

Returns the value of an environment variable. Throws if not set.

```atlas
let home = getEnv("HOME");
```

---

### `setEnv(name: string, value: string) -> void`

Sets an environment variable for the current process.

```atlas
setEnv("MY_VAR", "hello");
```

---

### `unsetEnv(name: string) -> void`

Unsets an environment variable.

```atlas
unsetEnv("TEMP_VAR");
```

---

### `listEnv() -> object`

Returns all environment variables as an object.

```atlas
let env = listEnv();
print(env.PATH);
```

---

## DateTime Functions

### `dateTimeNow() -> object`

Returns the current local date/time.

```atlas
let now = dateTimeNow();
print(now.year);    // e.g., 2026
```

---

### `dateTimeUtc() -> object`

Returns the current UTC date/time.

```atlas
let utc = dateTimeUtc();
```

---

### `dateTimeFromTimestamp(ts: number) -> object`

Creates a DateTime from a Unix timestamp (seconds since epoch).

```atlas
let dt = dateTimeFromTimestamp(0);    // 1970-01-01T00:00:00Z
```

---

### `dateTimeFromComponents(year: number, month: number, day: number, hour: number, minute: number, second: number) -> object`

Creates a DateTime from individual components.

```atlas
let dt = dateTimeFromComponents(2026, 2, 20, 12, 0, 0);
```

---

### `dateTimeParse(s: string, format: string) -> object`

Parses a date/time string with the given format.

```atlas
let dt = dateTimeParse("2026-02-20", "%Y-%m-%d");
```

---

### `dateTimeParseIso(s: string) -> object`

Parses an ISO 8601 date/time string.

```atlas
let dt = dateTimeParseIso("2026-02-20T12:00:00Z");
```

---

### `dateTimeTryParse(s: string, format: string) -> Result<object, string>`

Tries to parse a date/time string, returning a Result.

```atlas
let result = dateTimeTryParse("invalid", "%Y-%m-%d");
```

---

### `dateTimeFormat(dt: object, format: string) -> string`

Formats a DateTime with the given format string.

```atlas
let s = dateTimeFormat(dateTimeNow(), "%Y-%m-%d");    // "2026-02-20"
```

---

### `dateTimeToIso(dt: object) -> string`

Converts a DateTime to ISO 8601 format.

```atlas
dateTimeToIso(dateTimeNow());    // "2026-02-20T12:00:00+00:00"
```

---

### `dateTimeToTimestamp(dt: object) -> number`

Converts a DateTime to a Unix timestamp.

```atlas
dateTimeToTimestamp(dateTimeFromTimestamp(0));    // 0
```

---

### `dateTimeYear(dt: object) -> number`

Returns the year component.

```atlas
dateTimeYear(dateTimeNow());    // e.g., 2026
```

---

### `dateTimeMonth(dt: object) -> number`

Returns the month component (1-12).

```atlas
dateTimeMonth(dateTimeNow());    // e.g., 2
```

---

### `dateTimeDay(dt: object) -> number`

Returns the day of month (1-31).

```atlas
dateTimeDay(dateTimeNow());    // e.g., 20
```

---

### `dateTimeHour(dt: object) -> number`

Returns the hour component (0-23).

```atlas
dateTimeHour(dateTimeNow());    // e.g., 14
```

---

### `dateTimeMinute(dt: object) -> number`

Returns the minute component (0-59).

```atlas
dateTimeMinute(dateTimeNow());    // e.g., 30
```

---

### `dateTimeSecond(dt: object) -> number`

Returns the second component (0-59).

```atlas
dateTimeSecond(dateTimeNow());    // e.g., 45
```

---

### `dateTimeWeekday(dt: object) -> number`

Returns the day of week (0=Sunday, 6=Saturday).

```atlas
dateTimeWeekday(dateTimeNow());    // e.g., 4 (Thursday)
```

---

### `dateTimeDayOfYear(dt: object) -> number`

Returns the day of the year (1-366).

```atlas
dateTimeDayOfYear(dateTimeNow());    // e.g., 51
```

---

### `dateTimeAddDays(dt: object, n: number) -> object`

Returns a new DateTime with `n` days added.

```atlas
let tomorrow = dateTimeAddDays(dateTimeNow(), 1);
```

---

### `dateTimeAddHours(dt: object, n: number) -> object`

Returns a new DateTime with `n` hours added.

```atlas
let later = dateTimeAddHours(dateTimeNow(), 2);
```

---

### `dateTimeAddMinutes(dt: object, n: number) -> object`

Returns a new DateTime with `n` minutes added.

```atlas
let later = dateTimeAddMinutes(dateTimeNow(), 30);
```

---

### `dateTimeAddSeconds(dt: object, n: number) -> object`

Returns a new DateTime with `n` seconds added.

```atlas
let later = dateTimeAddSeconds(dateTimeNow(), 60);
```

---

### `dateTimeDiff(a: object, b: object) -> number`

Returns the difference in seconds between two DateTimes.

```atlas
let diff = dateTimeDiff(end, start);   // seconds
```

---

### `dateTimeCompare(a: object, b: object) -> number`

Compares two DateTimes. Returns `-1`, `0`, or `1`.

```atlas
dateTimeCompare(a, b);    // -1 if a < b, 0 if equal, 1 if a > b
```

---

### `dateTimeInTimezone(dt: object, tz: string) -> object`

Converts a DateTime to the specified timezone.

```atlas
let utcDt = dateTimeInTimezone(dateTimeNow(), "UTC");
```

---

### `dateTimeToUtc(dt: object) -> object`

Converts a DateTime to UTC.

```atlas
let utcDt = dateTimeToUtc(dateTimeNow());
```

---

### `dateTimeToLocal(dt: object) -> object`

Converts a DateTime to local time.

```atlas
let localDt = dateTimeToLocal(utcDt);
```

---

### `dateTimeToTimezone(dt: object, tz: string) -> object`

Converts a DateTime to a specific timezone (alias for `dateTimeInTimezone`).

```atlas
let nyDt = dateTimeToTimezone(dt, "America/New_York");
```

---

### `dateTimeGetTimezone(dt: object) -> string`

Returns the timezone name of a DateTime.

```atlas
dateTimeGetTimezone(dateTimeUtc());    // "UTC"
```

---

### `dateTimeGetOffset(dt: object) -> number`

Returns the UTC offset in seconds.

```atlas
dateTimeGetOffset(dateTimeUtc());    // 0
```

---

### `dateTimeToCustom(dt: object, format: string) -> string`

Formats a DateTime using a custom format string.

```atlas
dateTimeToCustom(dateTimeNow(), "%d/%m/%Y");    // "20/02/2026"
```

---

## Duration Functions

### `durationFromSeconds(n: number) -> object`

Creates a Duration from seconds.

```atlas
let d = durationFromSeconds(3600);   // 1 hour
```

---

### `durationFromMinutes(n: number) -> object`

Creates a Duration from minutes.

```atlas
let d = durationFromMinutes(90);    // 1.5 hours
```

---

### `durationFromHours(n: number) -> object`

Creates a Duration from hours.

```atlas
let d = durationFromHours(24);    // 1 day
```

---

### `durationFromDays(n: number) -> object`

Creates a Duration from days.

```atlas
let d = durationFromDays(7);    // 1 week
```

---

### `durationFormat(d: object) -> string`

Formats a Duration as a human-readable string.

```atlas
durationFormat(durationFromSeconds(3661));    // "1h 1m 1s"
```

---

## HTTP Functions

### `httpGet(url: string) -> object`

Performs a synchronous HTTP GET request. Returns a response object.

Requires network permission.

```atlas
let resp = httpGet("https://api.example.com/data");
print(resp.status);   // 200
print(resp.body);     // response body string
```

---

### `httpPost(url: string, body: string) -> object`

Performs a synchronous HTTP POST request.

```atlas
let resp = httpPost("https://api.example.com/create", '{"name":"Alice"}');
```

---

### `httpPut(url: string, body: string) -> object`

Performs a synchronous HTTP PUT request.

```atlas
let resp = httpPut("https://api.example.com/item/1", '{"name":"Bob"}');
```

---

### `httpPatch(url: string, body: string) -> object`

Performs a synchronous HTTP PATCH request.

```atlas
let resp = httpPatch("https://api.example.com/item/1", '{"age":31}');
```

---

### `httpDelete(url: string) -> object`

Performs a synchronous HTTP DELETE request.

```atlas
let resp = httpDelete("https://api.example.com/item/1");
```

---

### `httpPostJson(url: string, data: any) -> object`

Serializes `data` as JSON and posts it with appropriate content-type header.

```atlas
let resp = httpPostJson("https://api.example.com/create", { name: "Alice" });
```

---

### `httpGetJson(url: string) -> any`

Performs GET and parses the JSON response body.

```atlas
let data = httpGetJson("https://api.example.com/users");
```

---

### `httpStatus(resp: object) -> number`

Returns the HTTP status code from a response.

```atlas
httpStatus(resp);    // e.g., 200
```

---

### `httpBody(resp: object) -> string`

Returns the response body as a string.

```atlas
httpBody(resp);    // response body
```

---

### `httpIsSuccess(resp: object) -> bool`

Returns `true` if the status code is 2xx.

```atlas
httpIsSuccess(resp);    // true for 200-299
```

---

## Regex Functions

### `regexNew(pattern: string) -> object`

Compiles a regex pattern. Throws if invalid.

```atlas
let re = regexNew("[0-9]+");
```

---

### `regexNewWithFlags(pattern: string, flags: string) -> object`

Compiles a regex with flags (`i` = case-insensitive, `m` = multiline).

```atlas
let re = regexNewWithFlags("hello", "i");
```

---

### `regexIsMatch(re: object, s: string) -> bool`

Returns `true` if the regex matches anywhere in `s`.

```atlas
let re = regexNew("[0-9]+");
regexIsMatch(re, "abc123");    // true
regexIsMatch(re, "abc");       // false
```

---

### `regexTest(re: object, s: string) -> bool`

Alias for `regexIsMatch`.

```atlas
regexTest(re, "abc123");    // true
```

---

### `regexFind(re: object, s: string) -> Option<string>`

Returns the first match, or `None`.

```atlas
let re = regexNew("[0-9]+");
regexFind(re, "abc123def");    // Some("123")
regexFind(re, "abc");          // None
```

---

### `regexFindAll(re: object, s: string) -> array`

Returns all matches as an array of strings.

```atlas
let re = regexNew("[0-9]+");
regexFindAll(re, "a1 b22 c333");    // ["1", "22", "333"]
```

---

### `regexReplace(re: object, s: string, replacement: string) -> string`

Replaces the first match with the replacement string.

```atlas
let re = regexNew("[0-9]+");
regexReplace(re, "abc123def456", "NUM");    // "abcNUMdef456"
```

---

### `regexReplaceAll(re: object, s: string, replacement: string) -> string`

Replaces all matches with the replacement string.

```atlas
let re = regexNew("[0-9]+");
regexReplaceAll(re, "a1 b2 c3", "N");    // "aN bN cN"
```

---

### `regexSplit(re: object, s: string) -> array`

Splits a string by the regex pattern.

```atlas
let re = regexNew("[,;]");
regexSplit(re, "a,b;c");    // ["a", "b", "c"]
```

---

### `regexCaptures(re: object, s: string) -> array`

Returns capture groups for the first match.

```atlas
let re = regexNew("([0-9]+)-([a-z]+)");
regexCaptures(re, "42-hello");    // ["42", "hello"]
```

---

### `regexEscape(s: string) -> string`

Escapes special regex characters in a string.

```atlas
regexEscape("a.b+c");    // "a\\.b\\+c"
```

---

## Collections: HashMap

### `hashMapNew() -> object`

Creates an empty HashMap.

```atlas
let map = hashMapNew();
```

---

### `hashMapPut(map: object, key: string, value: any) -> void`

Inserts or updates a key-value pair.

```atlas
hashMapPut(map, "name", "Alice");
```

---

### `hashMapGet(map: object, key: string) -> Option<any>`

Returns the value for a key, or `None` if absent.

```atlas
let v = hashMapGet(map, "name");    // Some("Alice") or None
```

---

### `hashMapHas(map: object, key: string) -> bool`

Returns `true` if the key exists.

```atlas
hashMapHas(map, "name");    // true
```

---

### `hashMapRemove(map: object, key: string) -> Option<any>`

Removes a key and returns its value, or `None`.

```atlas
let removed = hashMapRemove(map, "name");
```

---

### `hashMapKeys(map: object) -> array`

Returns all keys as an array.

```atlas
hashMapKeys(map);    // ["name", "age", ...]
```

---

### `hashMapValues(map: object) -> array`

Returns all values as an array.

```atlas
hashMapValues(map);    // ["Alice", 30, ...]
```

---

### `hashMapEntries(map: object) -> array`

Returns all key-value pairs as `[key, value]` arrays.

```atlas
hashMapEntries(map);    // [["name", "Alice"], ["age", 30]]
```

---

### `hashMapFromEntries(entries: array) -> object`

Creates a HashMap from an array of `[key, value]` pairs.

```atlas
let map = hashMapFromEntries([["a", 1], ["b", 2]]);
```

---

### `hashMapSize(map: object) -> number`

Returns the number of entries.

```atlas
hashMapSize(map);    // e.g., 3
```

---

### `hashMapIsEmpty(map: object) -> bool`

Returns `true` if the map has no entries.

```atlas
hashMapIsEmpty(hashMapNew());    // true
```

---

### `hashMapClear(map: object) -> void`

Removes all entries from the map.

```atlas
hashMapClear(map);
```

---

## Collections: HashSet

### `hashSetNew() -> object`

Creates an empty HashSet.

```atlas
let set = hashSetNew();
```

---

### `hashSetAdd(set: object, value: any) -> void`

Adds a value to the set.

```atlas
hashSetAdd(set, "hello");
```

---

### `hashSetHas(set: object, value: any) -> bool`

Returns `true` if the value is in the set.

```atlas
hashSetHas(set, "hello");    // true
```

---

### `hashSetRemove(set: object, value: any) -> bool`

Removes a value. Returns `true` if it was present.

```atlas
hashSetRemove(set, "hello");
```

---

### `hashSetFromArray(arr: array) -> object`

Creates a HashSet from an array (duplicates removed).

```atlas
let set = hashSetFromArray([1, 2, 2, 3]);    // {1, 2, 3}
```

---

### `hashSetToArray(set: object) -> array`

Converts a HashSet to an array.

```atlas
hashSetToArray(set);    // [1, 2, 3]
```

---

### `hashSetSize(set: object) -> number`

Returns the number of elements.

```atlas
hashSetSize(set);    // e.g., 3
```

---

### `hashSetUnion(a: object, b: object) -> object`

Returns a new set with all elements from both sets.

```atlas
hashSetUnion(setA, setB);
```

---

### `hashSetIntersection(a: object, b: object) -> object`

Returns a new set with elements in both sets.

```atlas
hashSetIntersection(setA, setB);
```

---

### `hashSetDifference(a: object, b: object) -> object`

Returns elements in `a` but not in `b`.

```atlas
hashSetDifference(setA, setB);
```

---

### `hashSetSymmetricDifference(a: object, b: object) -> object`

Returns elements in either set but not both.

```atlas
hashSetSymmetricDifference(setA, setB);
```

---

### `hashSetIsSubset(a: object, b: object) -> bool`

Returns `true` if all elements of `a` are in `b`.

```atlas
hashSetIsSubset(small, large);
```

---

### `hashSetIsSuperset(a: object, b: object) -> bool`

Returns `true` if `a` contains all elements of `b`.

```atlas
hashSetIsSuperset(large, small);
```

---

## Collections: Queue

### `queueNew() -> object`

Creates an empty FIFO queue.

```atlas
let q = queueNew();
```

---

### `queueEnqueue(q: object, value: any) -> void`

Adds a value to the back of the queue.

```atlas
queueEnqueue(q, "first");
```

---

### `queueDequeue(q: object) -> Option<any>`

Removes and returns the front value, or `None` if empty.

```atlas
let v = queueDequeue(q);
```

---

### `queuePeek(q: object) -> Option<any>`

Returns the front value without removing it.

```atlas
let front = queuePeek(q);
```

---

### `queueSize(q: object) -> number`

Returns the number of elements.

```atlas
queueSize(q);
```

---

### `queueIsEmpty(q: object) -> bool`

Returns `true` if the queue has no elements.

```atlas
queueIsEmpty(q);    // true
```

---

### `queueToArray(q: object) -> array`

Returns all elements as an array (front first).

```atlas
queueToArray(q);
```

---

### `queueClear(q: object) -> void`

Removes all elements from the queue.

```atlas
queueClear(q);
```

---

## Collections: Stack

### `stackNew() -> object`

Creates an empty LIFO stack.

```atlas
let s = stackNew();
```

---

### `stackPush(s: object, value: any) -> void`

Pushes a value onto the top of the stack.

```atlas
stackPush(s, 42);
```

---

### `stackPop(s: object) -> Option<any>`

Removes and returns the top value, or `None` if empty.

```atlas
let v = stackPop(s);
```

---

### `stackPeek(s: object) -> Option<any>`

Returns the top value without removing it.

```atlas
let top = stackPeek(s);
```

---

### `stackSize(s: object) -> number`

Returns the number of elements.

```atlas
stackSize(s);
```

---

### `stackIsEmpty(s: object) -> bool`

Returns `true` if the stack has no elements.

```atlas
stackIsEmpty(s);    // true
```

---

### `stackToArray(s: object) -> array`

Returns all elements as an array (top first).

```atlas
stackToArray(s);
```

---

### `stackClear(s: object) -> void`

Removes all elements from the stack.

```atlas
stackClear(s);
```

---

## Async/Concurrency Functions

### `spawn(fn: function) -> object`

Spawns an async task. Returns a task handle.

```atlas
let task = spawn(|| heavyComputation());
```

---

### `await(task: object) -> any`

Awaits the result of an async task.

```atlas
let result = await(task);
```

---

### `joinAll(tasks: array) -> array`

Awaits all tasks in parallel, returning results in order.

```atlas
let results = joinAll([task1, task2, task3]);
```

---

### `sleep(ms: number) -> void`

Pauses the current task for `ms` milliseconds.

```atlas
sleep(500);
```

---

### `timeout(ms: number, fn: function) -> Result<any, string>`

Runs `fn` with a timeout. Returns `Err("timeout")` if exceeded.

```atlas
let result = timeout(1000, || expensiveOp());
```

---

### `timer(ms: number, fn: function) -> object`

Schedules `fn` to run after `ms` milliseconds. Returns a timer handle.

```atlas
let t = timer(5000, || print("done"));
```

---

### `interval(ms: number, fn: function) -> object`

Schedules `fn` to run repeatedly every `ms` milliseconds.

```atlas
let t = interval(1000, || print("tick"));
```

---

### `channelUnbounded() -> object`

Creates an unbounded channel for message passing.

```atlas
let ch = channelUnbounded();
```

---

### `channelBounded(capacity: number) -> object`

Creates a bounded channel with a fixed capacity.

```atlas
let ch = channelBounded(10);
```

---

### `channelSend(ch: object, value: any) -> void`

Sends a value through a channel.

```atlas
channelSend(ch, "hello");
```

---

### `channelReceive(ch: object) -> any`

Receives a value from a channel (blocks until available).

```atlas
let msg = channelReceive(ch);
```

---

### `channelIsClosed(ch: object) -> bool`

Returns `true` if the channel is closed.

```atlas
channelIsClosed(ch);
```

---

### `channelSelect(channels: array) -> object`

Waits for the first of multiple channels to be ready.

```atlas
let ready = channelSelect([ch1, ch2]);
```

---

### `asyncMutex() -> object`

Creates an async mutex for shared mutable state.

```atlas
let m = asyncMutex();
```

---

### `asyncMutexGet(m: object) -> any`

Acquires the mutex and returns its value.

```atlas
let val = asyncMutexGet(m);
```

---

### `asyncMutexSet(m: object, value: any) -> void`

Acquires the mutex and sets its value.

```atlas
asyncMutexSet(m, 42);
```

---

## Future Functions

### `futureNew(fn: function) -> object`

Creates a new future from a function.

```atlas
let f = futureNew(|| computeValue());
```

---

### `futureResolve(value: any) -> object`

Creates an already-resolved future.

```atlas
let f = futureResolve(42);
```

---

### `futureReject(error: any) -> object`

Creates an already-rejected future.

```atlas
let f = futureReject("error message");
```

---

### `futureThen(f: object, fn: function) -> object`

Chains a callback to run when the future resolves.

```atlas
let result = futureThen(f, |val| val * 2);
```

---

### `futureCatch(f: object, fn: function) -> object`

Chains a callback to run when the future rejects.

```atlas
let safe = futureCatch(f, |err| defaultValue);
```

---

### `futureAll(futures: array) -> object`

Returns a future that resolves when all given futures resolve.

```atlas
let combined = futureAll([f1, f2, f3]);
```

---

### `futureRace(futures: array) -> object`

Returns a future that resolves/rejects with the first completed future.

```atlas
let fastest = futureRace([f1, f2]);
```

---

### `futureIsResolved(f: object) -> bool`

Returns `true` if the future has resolved.

```atlas
futureIsResolved(f);
```

---

### `futureIsRejected(f: object) -> bool`

Returns `true` if the future has rejected.

```atlas
futureIsRejected(f);
```

---

### `futureIsPending(f: object) -> bool`

Returns `true` if the future is still pending.

```atlas
futureIsPending(f);
```

---

## Compression Functions

### `gzipCompress(data: string) -> string`

Compresses a string using gzip. Returns compressed bytes as a string.

```atlas
let compressed = gzipCompress(largeString);
```

---

### `gzipDecompress(data: string) -> string`

Decompresses gzip-compressed data.

```atlas
let original = gzipDecompress(compressed);
```

---

### `gzipIsGzip(data: string) -> bool`

Returns `true` if the data is gzip-compressed.

```atlas
gzipIsGzip(compressed);    // true
```

---

### `gzipCompressionRatio(original: string, compressed: string) -> number`

Returns the compression ratio (0.0 to 1.0, lower is better).

```atlas
gzipCompressionRatio(original, compressed);    // e.g., 0.3
```

---

### `zipCreate(path: string, files: array) -> void`

Creates a ZIP archive at `path` containing the given files.

```atlas
zipCreate("archive.zip", ["file1.txt", "file2.txt"]);
```

---

### `zipExtract(zip: string, dest: string) -> void`

Extracts all files from a ZIP archive.

```atlas
zipExtract("archive.zip", "./output");
```

---

### `zipList(zip: string) -> array`

Lists all files in a ZIP archive.

```atlas
let files = zipList("archive.zip");
```

---

### `zipContains(zip: string, file: string) -> bool`

Returns `true` if the file exists in the ZIP archive.

```atlas
zipContains("archive.zip", "file1.txt");
```

---

### `tarCreate(path: string, files: array) -> void`

Creates a tar archive.

```atlas
tarCreate("archive.tar", ["file1.txt", "file2.txt"]);
```

---

### `tarExtract(tar: string, dest: string) -> void`

Extracts all files from a tar archive.

```atlas
tarExtract("archive.tar", "./output");
```

---

### `tarList(tar: string) -> array`

Lists all files in a tar archive.

```atlas
let files = tarList("archive.tar");
```

---

## Reflection Functions

### `reflect_typeof(value: any) -> string`

Returns the detailed type name of a value.

```atlas
reflect_typeof(42);          // "number"
reflect_typeof([1, 2]);      // "array"
reflect_typeof(hashMapNew()); // "HashMap"
```

---

### `reflect_type_describe(value: any) -> string`

Returns a human-readable description of the value's type.

```atlas
reflect_type_describe([1, 2, 3]);    // "Array(3)"
```

---

### `reflect_is_primitive(value: any) -> bool`

Returns `true` if the value is a primitive (number, string, bool, null).

```atlas
reflect_is_primitive(42);    // true
reflect_is_primitive([]);    // false
```

---

### `reflect_is_callable(value: any) -> bool`

Returns `true` if the value is callable (function or builtin).

```atlas
reflect_is_callable(print);    // true
reflect_is_callable(42);       // false
```

---

### `reflect_get_function_arity(fn: function) -> number`

Returns the number of parameters a function accepts.

```atlas
fn add(a: number, b: number) -> number { return a + b; }
reflect_get_function_arity(add);    // 2
```

---

### `reflect_get_function_name(fn: function) -> string`

Returns the name of a function.

```atlas
reflect_get_function_name(add);    // "add"
```

---

### `reflect_clone(value: any) -> any`

Creates a deep clone of a value.

```atlas
let original = [1, 2, 3];
let copy = reflect_clone(original);
```

---

### `reflect_deep_equals(a: any, b: any) -> bool`

Returns `true` if two values are deeply equal.

```atlas
reflect_deep_equals([1, 2], [1, 2]);    // true
reflect_deep_equals([1], [2]);           // false
```

---

### `reflect_same_type(a: any, b: any) -> bool`

Returns `true` if two values have the same type.

```atlas
reflect_same_type(1, 2);       // true (both number)
reflect_same_type(1, "1");     // false
```

---

### `reflect_get_length(value: any) -> Option<number>`

Returns the length of a string or array, or `None`.

```atlas
reflect_get_length("hello");    // Some(5)
reflect_get_length(42);         // None
```

---

### `reflect_is_empty(value: any) -> bool`

Returns `true` if the value is an empty string, array, or collection.

```atlas
reflect_is_empty([]);     // true
reflect_is_empty("");     // true
reflect_is_empty([1]);    // false
```

---

### `reflect_value_to_string(value: any) -> string`

Returns the debug string representation of any value.

```atlas
reflect_value_to_string([1, 2, 3]);    // "[1, 2, 3]"
```

---

## Testing/Assertion Functions

### `assert(condition: bool, message: string) -> void`

Asserts that `condition` is `true`. Throws with `message` if false.

```atlas
assert(1 + 1 == 2, "math is broken");
assert(len("hello") == 5, "unexpected length");
```

---

### `assertFalse(condition: bool, message: string) -> void`

Asserts that `condition` is `false`.

```atlas
assertFalse(1 > 2, "1 should not be greater than 2");
```

---

### `assertEqual(actual: T, expected: T) -> void`

Asserts that `actual` deeply equals `expected`.

```atlas
assertEqual(1 + 1, 2);
assertEqual(["a", "b"], ["a", "b"]);
```

---

### `assertNotEqual(actual: T, expected: T) -> void`

Asserts that `actual` does not equal `expected`.

```atlas
assertNotEqual(1, 2);
assertNotEqual("hello", "world");
```

---

### `assertOk(result: Result<T, E>) -> T`

Asserts `result` is `Ok` and returns the unwrapped value.

```atlas
let value = assertOk(divide(10, 2));
assertEqual(value, 5);
```

---

### `assertErr(result: Result<T, E>) -> E`

Asserts `result` is `Err` and returns the error value.

```atlas
let err = assertErr(divide(10, 0));
assertEqual(err, "division by zero");
```

---

### `assertSome(option: Option<T>) -> T`

Asserts `option` is `Some` and returns the unwrapped value.

```atlas
let v = assertSome(findUser("alice"));
assertEqual(v.name, "alice");
```

---

### `assertNone(option: Option<T>) -> void`

Asserts `option` is `None`.

```atlas
assertNone(findUser("nonexistent"));
```

---

### `assertContains(array: array, value: T) -> void`

Asserts that `array` contains `value` (deep equality).

```atlas
assertContains([1, 2, 3], 2);
assertContains(["a", "b"], "a");
```

---

### `assertEmpty(array: array) -> void`

Asserts that `array` has zero elements.

```atlas
assertEmpty([]);
assertEmpty(filterItems([]));
```

---

### `assertLength(array: array, expected: number) -> void`

Asserts that `array` has exactly `expected` elements.

```atlas
assertLength([1, 2, 3], 3);
```

---

### `assertThrows(fn: NativeFunction) -> void`

Asserts that `fn` throws (returns `Err`).

```atlas
assertThrows(|| divide(1, 0));
```

---

### `assertNoThrow(fn: NativeFunction) -> void`

Asserts that `fn` does not throw.

```atlas
assertNoThrow(|| divide(10, 2));
```

---

## Result Functions

### `Ok(value: T) -> Result<T, E>`

Creates a successful `Result`.

```atlas
return Ok(42);
```

---

### `Err(error: E) -> Result<T, E>`

Creates an error `Result`.

```atlas
return Err("something went wrong");
```

---

### `is_ok(result: Result<T, E>) -> bool`

Returns `true` if the result is `Ok`.

```atlas
is_ok(Ok(42));    // true
is_ok(Err("x")); // false
```

---

### `is_err(result: Result<T, E>) -> bool`

Returns `true` if the result is `Err`.

```atlas
is_err(Err("x"));    // true
is_err(Ok(1));       // false
```

---

### `result_ok(result: Result<T, E>) -> Option<T>`

Returns `Some(value)` if `Ok`, or `None` if `Err`.

```atlas
result_ok(Ok(42));     // Some(42)
result_ok(Err("x"));   // None
```

---

### `result_err(result: Result<T, E>) -> Option<E>`

Returns `Some(error)` if `Err`, or `None` if `Ok`.

```atlas
result_err(Err("x"));   // Some("x")
result_err(Ok(42));     // None
```

---

## Option Functions

### `Some(value: T) -> Option<T>`

Creates a `Some` option containing `value`.

```atlas
return Some(42);
```

---

### `None -> Option<T>`

The empty option value.

```atlas
return None;
```

---

### `is_some(option: Option<T>) -> bool`

Returns `true` if the option is `Some`.

```atlas
is_some(Some(1));    // true
is_some(None);       // false
```

---

### `is_none(option: Option<T>) -> bool`

Returns `true` if the option is `None`.

```atlas
is_none(None);       // true
is_none(Some(1));    // false
```

---

*End of Atlas Standard Library API Reference — v0.2*
