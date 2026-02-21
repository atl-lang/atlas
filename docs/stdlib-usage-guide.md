# Atlas Standard Library Usage Guide

**Version:** v0.2 | **Audience:** Atlas developers

This guide demonstrates practical patterns and real-world usage of the Atlas standard library.

---

## Table of Contents

- [String Manipulation](#string-manipulation)
- [Array Operations](#array-operations)
- [Working with JSON](#working-with-json)
- [File I/O Patterns](#file-io-patterns)
- [Math and Numbers](#math-and-numbers)
- [Type Checking and Conversion](#type-checking-and-conversion)
- [Working with Results and Options](#working-with-results-and-options)
- [Collections (HashMap, HashSet)](#collections)
- [Date and Time](#date-and-time)
- [Regular Expressions](#regular-expressions)
- [HTTP Requests](#http-requests)
- [Process and Environment](#process-and-environment)
- [Async and Concurrency](#async-and-concurrency)
- [Complete Example Programs](#complete-example-programs)

---

## String Manipulation

### Parsing CSV Data

```atlas
fn parse_csv(content: string) -> array {
    let lines = split(content, "\n");
    let result = [];
    for line in lines {
        let trimmed = trim(line);
        if len(trimmed) > 0 {
            result = result + [split(trimmed, ",")];
        }
    }
    return result;
}

let csv = "Alice,30,Engineer\nBob,25,Designer\n";
let rows = parse_csv(csv);
// rows = [["Alice", "30", "Engineer"], ["Bob", "25", "Designer"]]
```

### Building Template Strings

```atlas
fn render_template(template: string, vars: object) -> string {
    let result = template;
    result = replace(result, "{{name}}", vars.name);
    result = replace(result, "{{version}}", vars.version);
    return result;
}

let tmpl = "Welcome to {{name}} v{{version}}!";
let output = render_template(tmpl, { name: "Atlas", version: "0.2" });
// "Welcome to Atlas v0.2!"
```

### Slug Generation

```atlas
fn slugify(title: string) -> string {
    let lower = toLowerCase(title);
    let re = regexNew("[^a-z0-9]+");
    let slug = regexReplaceAll(re, lower, "-");
    return trim(slug);
}

slugify("Hello, World!");    // "hello-world"
slugify("  My Blog Post  "); // "my-blog-post"
```

### String Padding and Formatting

```atlas
fn format_table_row(name: string, value: string, width: number) -> string {
    let padded_name = padEnd(name, width, " ");
    return concat(padded_name, value);
}

print(format_table_row("Name:", "Alice", 12));    // "Name:        Alice"
print(format_table_row("Version:", "0.2", 12));   // "Version:     0.2"
```

---

## Array Operations

### Functional-Style Array Processing

```atlas
fn map_array(arr: array, transform: function) -> array {
    let result = [];
    for item in arr {
        result = result + [transform(item)];
    }
    return result;
}

fn filter_array(arr: array, predicate: function) -> array {
    let result = [];
    for item in arr {
        if predicate(item) {
            result = result + [item];
        }
    }
    return result;
}

fn reduce_array(arr: array, initial: any, reducer: function) -> any {
    let acc = initial;
    for item in arr {
        acc = reducer(acc, item);
    }
    return acc;
}

let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
let evens = filter_array(numbers, |n| n % 2 == 0);      // [2, 4, 6, 8, 10]
let doubled = map_array(evens, |n| n * 2);               // [4, 8, 12, 16, 20]
let sum = reduce_array(doubled, 0, |acc, n| acc + n);    // 60
```

### Deduplication

```atlas
fn unique(arr: array) -> array {
    let seen = hashSetNew();
    let result = [];
    for item in arr {
        if !hashSetHas(seen, item) {
            hashSetAdd(seen, item);
            result = result + [item];
        }
    }
    return result;
}

unique([1, 2, 2, 3, 3, 3]);    // [1, 2, 3]
```

### Chunking Arrays

```atlas
fn chunk(arr: array, size: number) -> array {
    let result = [];
    let i = 0;
    while i < len(arr) {
        let end = min(i + size, len(arr));
        result = result + [slice(arr, i, end)];
        i = i + size;
    }
    return result;
}

chunk([1, 2, 3, 4, 5], 2);    // [[1, 2], [3, 4], [5]]
```

### Sorting with Custom Comparator

```atlas
fn sort_by_key(arr: array, key: string) -> array {
    // Bubble sort by object key
    let sorted = arr;
    let n = len(sorted);
    for i in 0..n {
        for j in 0..(n - i - 1) {
            if sorted[j][key] > sorted[j + 1][key] {
                let tmp = sorted[j];
                sorted[j] = sorted[j + 1];
                sorted[j + 1] = tmp;
            }
        }
    }
    return sorted;
}

let people = [
    { name: "Charlie", age: 35 },
    { name: "Alice", age: 25 },
    { name: "Bob", age: 30 }
];
let sorted = sort_by_key(people, "age");
// [Alice/25, Bob/30, Charlie/35]
```

---

## Working with JSON

### Config File Pattern

```atlas
fn load_config(path: string) -> object {
    if !fileExists(path) {
        return { debug: false, port: 8080, host: "localhost" };
    }
    let content = readFile(path);
    if !isValidJSON(content) {
        return { debug: false, port: 8080, host: "localhost" };
    }
    return parseJSON(content);
}

fn save_config(path: string, config: object) -> void {
    let pretty = prettifyJSON(toJSON(config));
    writeFile(path, pretty);
}

let config = load_config("config.json");
config.debug = true;
save_config("config.json", config);
```

### JSON Transformation Pipeline

```atlas
fn transform_users(json_str: string) -> array {
    let data = parseJSON(json_str);
    let result = [];
    for user in data.users {
        result = result + [{
            id: user.id,
            display_name: concat(user.first_name, concat(" ", user.last_name)),
            email: toLowerCase(user.email)
        }];
    }
    return result;
}
```

### Safe JSON Access

```atlas
fn safe_get(obj: object, key: string, default_val: any) -> any {
    if hasField(obj, key) {
        return obj[key];
    }
    return default_val;
}

let data = parseJSON('{"name": "Atlas", "version": "0.2"}');
let name = safe_get(data, "name", "unknown");       // "Atlas"
let author = safe_get(data, "author", "N/A");       // "N/A"
```

---

## File I/O Patterns

### Reading and Processing Lines

```atlas
fn process_lines(path: string) -> array {
    let content = readFile(path);
    let lines = split(content, "\n");
    let result = [];
    for line in lines {
        let trimmed = trim(line);
        if len(trimmed) > 0 && !startsWith(trimmed, "#") {
            result = result + [trimmed];
        }
    }
    return result;
}
```

### Atomic File Write (Write-Then-Rename Pattern)

```atlas
fn atomic_write(path: string, content: string) -> void {
    let tmp = fsTmpfileNamed(concat(pathBasename(path), ".tmp"));
    writeFile(tmp, content);
    // In production: rename tmp to path
    removeFile(tmp);
    writeFile(path, content);
}
```

### Directory Walking

```atlas
fn find_files_by_extension(dir: string, ext: string) -> array {
    let all_files = fsWalk(dir);
    let result = [];
    for file in all_files {
        if endsWith(file, concat(".", ext)) {
            result = result + [file];
        }
    }
    return result;
}

let atlas_files = find_files_by_extension("./src", "atl");
```

### Log Writer

```atlas
fn make_logger(path: string) -> function {
    return |level: string, message: string| -> void {
        let now = dateTimeToIso(dateTimeNow());
        let entry = concat("[", concat(now, concat("] [", concat(level, concat("] ", concat(message, "\n"))))));
        appendFile(path, entry);
    };
}

let log = make_logger("app.log");
log("INFO", "Application started");
log("ERROR", "Connection failed");
```

---

## Math and Numbers

### Statistics Functions

```atlas
fn mean(numbers: array) -> number {
    if len(numbers) == 0 {
        return 0;
    }
    let sum = 0;
    for n in numbers {
        sum = sum + n;
    }
    return sum / len(numbers);
}

fn variance(numbers: array) -> number {
    let m = mean(numbers);
    let sum_sq = 0;
    for n in numbers {
        let diff = n - m;
        sum_sq = sum_sq + (diff * diff);
    }
    return sum_sq / len(numbers);
}

fn std_deviation(numbers: array) -> number {
    return sqrt(variance(numbers));
}

let data = [2, 4, 4, 4, 5, 5, 7, 9];
print(mean(data));          // 5
print(std_deviation(data)); // 2
```

### Number Formatting

```atlas
fn format_currency(amount: number, symbol: string) -> string {
    let rounded = round(amount * 100) / 100;
    let s = str(rounded);
    if !includes(s, ".") {
        s = concat(s, ".00");
    }
    return concat(symbol, s);
}

format_currency(1234.5, "$");    // "$1234.5"
format_currency(99.0, "€");     // "€99.00"
```

### Random Utilities

```atlas
fn random_int(low: number, high: number) -> number {
    return floor(random() * (high - low + 1)) + low;
}

fn random_choice(arr: array) -> any {
    let idx = random_int(0, len(arr) - 1);
    return arr[idx];
}

random_int(1, 6);             // dice roll
random_choice(["a", "b", "c"]);  // random element
```

---

## Type Checking and Conversion

### Safe Type Conversion

```atlas
fn safe_to_number(value: any) -> Result<number, string> {
    if isNumber(value) {
        return Ok(value);
    }
    if isString(value) {
        let trimmed = trim(value);
        if len(trimmed) > 0 {
            return Ok(parseFloat(trimmed));
        }
    }
    if isBool(value) {
        return Ok(toBool(value));
    }
    return Err(concat("Cannot convert to number: ", str(value)));
}
```

### Type-Safe Object Access

```atlas
fn get_string_field(obj: object, field: string) -> Result<string, string> {
    if !hasField(obj, field) {
        return Err(concat("Missing field: ", field));
    }
    let val = obj[field];
    if !isString(val) {
        return Err(concat("Field '", concat(field, "' is not a string")));
    }
    return Ok(val);
}
```

---

## Working with Results and Options

### Chaining Results

```atlas
fn parse_and_validate(input: string) -> Result<number, string> {
    let trimmed = trim(input);
    if len(trimmed) == 0 {
        return Err("Input is empty");
    }
    let n = parseFloat(trimmed);
    if n < 0 {
        return Err("Number must be non-negative");
    }
    if n > 1000 {
        return Err("Number must be <= 1000");
    }
    return Ok(n);
}

let inputs = ["42", "-5", "999", "abc", "1001"];
for input in inputs {
    let result = parse_and_validate(input);
    if is_ok(result) {
        print(concat("Valid: ", str(unwrap(result))));
    } else {
        print(concat("Error: ", unwrap(result_err(result))));
    }
}
```

### Option Chaining Pattern

```atlas
fn find_user(id: number) -> Option<object> {
    let users = [
        { id: 1, name: "Alice", email: "alice@example.com" },
        { id: 2, name: "Bob", email: "bob@example.com" }
    ];
    for user in users {
        if user.id == id {
            return Some(user);
        }
    }
    return None;
}

fn get_user_email(id: number) -> string {
    let user = find_user(id);
    if is_some(user) {
        return unwrap(user).email;
    }
    return "unknown@example.com";
}
```

---

## Collections

### HashMap as Cache

```atlas
fn make_memoized(fn_impl: function) -> function {
    let cache = hashMapNew();
    return |key: string| -> any {
        if hashMapHas(cache, key) {
            return unwrap(hashMapGet(cache, key));
        }
        let result = fn_impl(key);
        hashMapPut(cache, key, result);
        return result;
    };
}
```

### Frequency Counter

```atlas
fn count_frequencies(words: array) -> object {
    let freq = hashMapNew();
    for word in words {
        if hashMapHas(freq, word) {
            let count = unwrap(hashMapGet(freq, word));
            hashMapPut(freq, word, count + 1);
        } else {
            hashMapPut(freq, word, 1);
        }
    }
    return freq;
}

let words = split("the quick brown fox the lazy fox", " ");
let freq = count_frequencies(words);
// { "the": 2, "fox": 2, "quick": 1, ... }
```

### Set Operations for Data Analysis

```atlas
let group_a = hashSetFromArray(["alice", "bob", "charlie"]);
let group_b = hashSetFromArray(["bob", "charlie", "dave"]);

let both = hashSetIntersection(group_a, group_b);    // {bob, charlie}
let either = hashSetUnion(group_a, group_b);          // {alice, bob, charlie, dave}
let only_a = hashSetDifference(group_a, group_b);     // {alice}
```

---

## Date and Time

### Date Arithmetic

```atlas
fn business_days_from_now(n: number) -> object {
    let current = dateTimeNow();
    let added = 0;
    while added < n {
        current = dateTimeAddDays(current, 1);
        let wd = dateTimeWeekday(current);
        if wd != 0 && wd != 6 {   // skip Sunday=0 and Saturday=6
            added = added + 1;
        }
    }
    return current;
}

let deadline = business_days_from_now(5);
print(dateTimeToIso(deadline));
```

### Duration Formatting

```atlas
fn format_duration(seconds: number) -> string {
    let d = durationFromSeconds(seconds);
    return durationFormat(d);
}

format_duration(3661);     // "1h 1m 1s"
format_duration(90);       // "1m 30s"
```

### Timestamp Comparison

```atlas
fn is_expired(timestamp: number, ttl_seconds: number) -> bool {
    let now = dateTimeToTimestamp(dateTimeNow());
    return (now - timestamp) > ttl_seconds;
}

let created_at = dateTimeToTimestamp(dateTimeNow());
sleep(100);
is_expired(created_at, 10);   // false (only 100ms passed)
```

---

## Regular Expressions

### Input Validation

```atlas
fn is_valid_email(email: string) -> bool {
    let re = regexNew("^[a-zA-Z0-9._%+\\-]+@[a-zA-Z0-9.\\-]+\\.[a-zA-Z]{2,}$");
    return regexIsMatch(re, email);
}

fn is_valid_url(url: string) -> bool {
    let re = regexNew("^https?://[\\w\\-]+(\\.[\\w\\-]+)+(/[\\w\\-./]*)?$");
    return regexIsMatch(re, url);
}

is_valid_email("user@example.com");    // true
is_valid_email("not-an-email");        // false
```

### Text Extraction

```atlas
fn extract_numbers(text: string) -> array {
    let re = regexNew("-?[0-9]+(\\.[0-9]+)?");
    return regexFindAll(re, text);
}

extract_numbers("The price is $19.99 for 3 items");   // ["19.99", "3"]
```

### Template Variable Replacement

```atlas
fn fill_template(template: string, vars: object) -> string {
    let re = regexNew("\\{\\{([a-zA-Z_]+)\\}\\}");
    let result = template;
    let keys = hashMapKeys(vars);
    for key in keys {
        let pattern = regexNew(concat("\\{\\{", concat(key, "\\}\\}")));
        let val = str(unwrap(hashMapGet(vars, key)));
        result = regexReplaceAll(pattern, result, val);
    }
    return result;
}
```

---

## HTTP Requests

### REST API Client Pattern

```atlas
fn make_api_client(base_url: string, token: string) -> object {
    return {
        get: |path: string| -> Result<any, string> {
            let url = concat(base_url, path);
            let resp = httpGet(url);
            if !httpIsSuccess(resp) {
                return Err(concat("HTTP ", str(httpStatus(resp))));
            }
            return Ok(parseJSON(httpBody(resp)));
        },
        post: |path: string, data: any| -> Result<any, string> {
            let url = concat(base_url, path);
            let resp = httpPostJson(url, data);
            if !httpIsSuccess(resp) {
                return Err(concat("HTTP ", str(httpStatus(resp))));
            }
            return Ok(parseJSON(httpBody(resp)));
        }
    };
}

// Usage:
// let api = make_api_client("https://api.example.com", "my-token");
// let users = assertOk(api.get("/users"));
```

### Retry with Backoff

```atlas
fn fetch_with_retry(url: string, max_retries: number) -> Result<string, string> {
    let attempt = 0;
    while attempt < max_retries {
        let resp = httpGet(url);
        if httpIsSuccess(resp) {
            return Ok(httpBody(resp));
        }
        attempt = attempt + 1;
        if attempt < max_retries {
            sleep(pow(2, attempt) * 100);  // exponential backoff
        }
    }
    return Err(concat("Failed after ", concat(str(max_retries), " retries")));
}
```

---

## Process and Environment

### Environment-Aware Configuration

```atlas
fn get_config() -> object {
    let env = getEnv("ATLAS_ENV");
    let is_prod = env == "production";
    let is_dev = env == "development";

    return {
        debug: is_dev,
        log_level: is_prod ? "error" : "debug",
        db_url: getEnv("DATABASE_URL"),
        port: toNumber(getEnv("PORT"))
    };
}
```

### Shell Command Pipeline

```atlas
fn count_lines(path: string) -> number {
    let result = exec(concat("wc -l ", path));
    if result.exit_code != 0 {
        return 0;
    }
    let parts = split(trim(result.stdout), " ");
    return toNumber(parts[0]);
}
```

---

## Async and Concurrency

### Parallel Data Fetching

```atlas
fn fetch_all_users(ids: array) -> array {
    let tasks = [];
    for id in ids {
        let task = spawn(|| {
            let resp = httpGet(concat("https://api.example.com/users/", str(id)));
            return parseJSON(httpBody(resp));
        });
        tasks = tasks + [task];
    }
    return joinAll(tasks);
}
```

### Producer-Consumer Pattern

```atlas
fn run_pipeline(items: array, process: function) -> array {
    let ch = channelBounded(10);
    let results = [];

    // Producer
    spawn(|| {
        for item in items {
            channelSend(ch, item);
        }
    });

    // Consumer
    for _ in items {
        let item = channelReceive(ch);
        results = results + [process(item)];
    }

    return results;
}
```

---

## Complete Example Programs

### Word Frequency Analyzer

```atlas
fn analyze_text(path: string) -> void {
    let content = readFile(path);
    let re = regexNew("[a-zA-Z]+");
    let words = regexFindAll(re, toLowerCase(content));

    // Count frequencies
    let freq = hashMapNew();
    for word in words {
        if hashMapHas(freq, word) {
            hashMapPut(freq, word, unwrap(hashMapGet(freq, word)) + 1);
        } else {
            hashMapPut(freq, word, 1);
        }
    }

    // Sort by frequency (top 10)
    let entries = hashMapEntries(freq);
    let sorted = entries;
    let n = len(sorted);
    for i in 0..n {
        for j in 0..(n - i - 1) {
            if sorted[j][1] < sorted[j + 1][1] {
                let tmp = sorted[j];
                sorted[j] = sorted[j + 1];
                sorted[j + 1] = tmp;
            }
        }
    }

    print("Top 10 words:");
    let top = slice(sorted, 0, min(10, len(sorted)));
    for entry in top {
        let line = concat(padEnd(entry[0], 20, " "), str(entry[1]));
        print(line);
    }
    print(concat("Total unique words: ", str(hashMapSize(freq))));
}
```

### JSON Config Validator

```atlas
fn validate_config(path: string) -> Result<object, string> {
    if !fileExists(path) {
        return Err(concat("Config file not found: ", path));
    }

    let content = readFile(path);
    if !isValidJSON(content) {
        return Err("Invalid JSON in config file");
    }

    let config = parseJSON(content);

    // Validate required fields
    let required = ["host", "port", "database"];
    for field in required {
        if !hasField(config, field) {
            return Err(concat("Missing required field: ", field));
        }
    }

    // Validate types
    if !isString(config.host) {
        return Err("'host' must be a string");
    }
    if !isNumber(config.port) {
        return Err("'port' must be a number");
    }
    if config.port < 1 || config.port > 65535 {
        return Err("'port' must be between 1 and 65535");
    }

    return Ok(config);
}

let result = validate_config("config.json");
if is_ok(result) {
    let config = unwrap(result);
    print(concat("Config valid! Connecting to ", config.host));
} else {
    print(concat("Config error: ", unwrap(result_err(result))));
}
```

### File Backup Utility

```atlas
fn backup_files(source_dir: string, dest_dir: string) -> void {
    let timestamp = dateTimeFormat(dateTimeNow(), "%Y%m%d_%H%M%S");
    let backup_dir = pathJoin(dest_dir, concat("backup_", timestamp));
    fsMkdirp(backup_dir);

    let files = fsWalk(source_dir);
    let count = 0;
    let errors = [];

    for file in files {
        let rel_path = substring(file, len(source_dir) + 1, len(file));
        let dest_path = pathJoin(backup_dir, rel_path);

        // Create parent directories
        let parent = pathDirname(dest_path);
        fsMkdirp(parent);

        // Copy file
        let content = readFile(file);
        writeFile(dest_path, content);
        count = count + 1;
    }

    print(concat("Backup complete: ", str(count), " files to ", backup_dir));
}
```

---

## Best Practices

### 1. Always Validate External Input

```atlas
fn process_user_input(input: any) -> Result<string, string> {
    if !isString(input) {
        return Err(concat("Expected string, got ", typeof(input)));
    }
    let trimmed = trim(input);
    if len(trimmed) == 0 {
        return Err("Input cannot be empty");
    }
    if len(trimmed) > 1000 {
        return Err("Input too long (max 1000 characters)");
    }
    return Ok(trimmed);
}
```

### 2. Use Results for Error Propagation

```atlas
fn load_and_parse(path: string) -> Result<object, string> {
    if !fileExists(path) {
        return Err(concat("File not found: ", path));
    }
    let content = readFile(path);
    if !isValidJSON(content) {
        return Err("Invalid JSON");
    }
    return Ok(parseJSON(content));
}
```

### 3. Prefer Immutability

```atlas
// Prefer returning new values over mutation
fn add_item(arr: array, item: any) -> array {
    return arr + [item];    // returns new array
}
```

### 4. Handle Edge Cases Explicitly

```atlas
fn safe_divide(a: number, b: number) -> Result<number, string> {
    if b == 0 {
        return Err("Division by zero");
    }
    return Ok(a / b);
}
```

---

*See also: [API Reference](api/stdlib.md) | [CLI Reference](cli-reference.md)*
