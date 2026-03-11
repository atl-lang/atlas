//! Shared method dispatch table for interpreter/VM parity.
//!
//! Both the interpreter and compiler consult this module to map
//! (TypeTag, method_name) → stdlib function name.

use serde::{Deserialize, Serialize};

/// Runtime-stable type tag for method dispatch.
/// Mirrors the types that support method call syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TypeTag {
    JsonValue,
    Array,
    HttpResponse,
    String,
    HashMap,
    HashSet,
    Queue,
    Stack,
    Option,
    Result,
    /// Static namespace: Json.parse(), Json.stringify(), etc.
    JsonNs,
    /// Static namespace: Math.sqrt(), Math.abs(), etc.
    MathNs,
    /// Static namespace: Env.get(), Env.set(), Env.unset()
    EnvNs,
    /// Static namespace: File.read(), File.write(), File.exists(), etc.
    FileNs,
    /// Static namespace: Process.cwd(), Process.pid(), Process.spawn(), etc.
    ProcessNs,
    /// Static namespace: DateTime.now(), DateTime.fromTimestamp(), etc.
    DateTimeNs,
    /// Static namespace: Path.join(), Path.dirname(), Path.basename(), etc.
    PathNs,
    /// Static namespace: Http.get(), Http.post(), Http.put(), etc.
    HttpNs,
    /// Static namespace: Net.tcpConnect(), Net.tcpListen(), etc.
    NetNs,
    /// Static namespace: Crypto.sha256(), Crypto.sha512(), etc.
    CryptoNs,
    /// Static namespace: Regex.test(), Regex.match(), Regex.replace(), etc.
    RegexNs,
    /// Static namespace: Io.readLine(), Io.readLinePrompt(), etc.
    IoNs,
    /// Static namespace: console.log(), console.error(), console.warn(), etc.
    ConsoleNs,
    /// Instance methods on DateTime values (year, month, day, format, etc.)
    DateTime,
    /// Instance methods on Regex values (test, find, findAll, replace, etc.)
    RegexValue,
    /// Instance methods on ProcessOutput values (stdout, stderr, exitCode, success)
    ProcessOutput,
}

/// Resolve a method call to its stdlib function name.
/// Returns None if the type/method combination is not registered.
pub fn resolve_method(type_tag: TypeTag, method_name: &str) -> Option<String> {
    match type_tag {
        TypeTag::JsonValue => Some(format!("json{}", capitalize_first(method_name))),
        TypeTag::Array => resolve_array_method(method_name),
        TypeTag::HttpResponse => resolve_http_response_method(method_name),
        TypeTag::String => resolve_string_method(method_name),
        TypeTag::HashMap => resolve_hashmap_method(method_name),
        TypeTag::HashSet => resolve_hashset_method(method_name),
        TypeTag::Queue => resolve_queue_method(method_name),
        TypeTag::Stack => resolve_stack_method(method_name),
        TypeTag::Option => resolve_option_method(method_name),
        TypeTag::Result => resolve_result_method(method_name),
        TypeTag::JsonNs => resolve_json_ns_method(method_name),
        TypeTag::MathNs => resolve_math_ns_method(method_name),
        TypeTag::EnvNs => resolve_env_ns_method(method_name),
        TypeTag::FileNs => resolve_file_ns_method(method_name),
        TypeTag::ProcessNs => resolve_process_ns_method(method_name),
        TypeTag::DateTimeNs => resolve_datetime_ns_method(method_name),
        TypeTag::PathNs => resolve_path_ns_method(method_name),
        TypeTag::HttpNs => resolve_http_ns_method(method_name),
        TypeTag::NetNs => resolve_net_ns_method(method_name),
        TypeTag::CryptoNs => resolve_crypto_ns_method(method_name),
        TypeTag::RegexNs => resolve_regex_ns_method(method_name),
        TypeTag::IoNs => resolve_io_ns_method(method_name),
        TypeTag::ConsoleNs => resolve_console_ns_method(method_name),
        TypeTag::DateTime => resolve_datetime_instance_method(method_name),
        TypeTag::RegexValue => resolve_regex_instance_method(method_name),
        TypeTag::ProcessOutput => resolve_process_output_method(method_name),
    }
}

/// Check if an identifier name is a static namespace sentinel.
pub fn is_static_namespace(name: &str) -> bool {
    matches!(
        name,
        "Json"
            | "Math"
            | "env"
            | "file"
            | "process"
            | "datetime"
            | "path"
            | "http"
            | "net"
            | "crypto"
            | "regex"
            | "io"
            | "console"
    )
}

/// Map a static namespace identifier to its TypeTag.
pub fn namespace_type_tag(name: &str) -> Option<TypeTag> {
    match name {
        "Json" => Some(TypeTag::JsonNs),
        "Math" => Some(TypeTag::MathNs),
        "env" => Some(TypeTag::EnvNs),
        "file" => Some(TypeTag::FileNs),
        "process" => Some(TypeTag::ProcessNs),
        "datetime" => Some(TypeTag::DateTimeNs),
        "path" => Some(TypeTag::PathNs),
        "http" => Some(TypeTag::HttpNs),
        "net" => Some(TypeTag::NetNs),
        "crypto" => Some(TypeTag::CryptoNs),
        "regex" => Some(TypeTag::RegexNs),
        "io" => Some(TypeTag::IoNs),
        "console" => Some(TypeTag::ConsoleNs),
        _ => None,
    }
}

/// Resolve an HttpResponse method call to its stdlib function name.
fn resolve_http_response_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "status" => "httpStatus",
        "body" => "httpBody",
        "headers" => "httpHeaders",
        "header" => "httpHeader",
        "url" => "httpUrl",
        "isSuccess" => "httpIsSuccess",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a string method call to its stdlib function name.
fn resolve_string_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // Core methods
        "len" | "length" => "len",
        "charAt" => "charAt",
        "substring" | "slice" => "substring",
        // Search methods
        "indexOf" => "indexOf",
        "lastIndexOf" => "lastIndexOf",
        "includes" => "includes",
        "startsWith" => "startsWith",
        "endsWith" => "endsWith",
        // Transform methods
        "toUpperCase" => "toUpperCase",
        "toLowerCase" => "toLowerCase",
        "trim" => "trim",
        "trimStart" => "trimStart",
        "trimEnd" => "trimEnd",
        "repeat" => "repeat",
        "replace" => "replace",
        "replaceAll" => "replaceAll",
        "split" => "split",
        // Padding methods
        "padStart" => "padStart",
        "padEnd" => "padEnd",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve an array method call to its stdlib function name.
fn resolve_array_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // Mutating methods — CoW write-back to receiver
        "push" => "arrayPush",
        "pop" => "arrayPop",
        "shift" => "arrayShift",
        "unshift" => "arrayUnshift",
        "reverse" => "arrayReverse",
        // Non-mutating — return new value
        "sort" => "arraySort",
        "sortBy" => "arraySortBy",
        "enumerate" => "arrayEnumerate",
        "len" | "length" => "len",
        "isEmpty" => "arrayIsEmpty",
        "includes" => "arrayIncludes",
        "indexOf" => "arrayIndexOf",
        "lastIndexOf" => "arrayLastIndexOf",
        "find" => "find",
        "findIndex" => "findIndex",
        "some" => "some",
        "every" => "every",
        "forEach" => "forEach",
        "map" => "map",
        "filter" => "filter",
        "reduce" => "reduce",
        "slice" => "slice",
        "concat" => "concat",
        "flat" | "flatten" => "flatten",
        "flatMap" => "flatMap",
        "join" => "join",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a HashMap method call to its stdlib function name.
/// HashMap uses CoW semantics — mutating methods return a new map (write-back required).
fn resolve_hashmap_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // Read methods
        "get" => "hashMapGet",
        "has" | "containsKey" => "hashMapHas",
        "size" | "len" => "hashMapSize",
        "isEmpty" => "hashMapIsEmpty",
        "keys" => "hashMapKeys",
        "values" => "hashMapValues",
        "entries" => "hashMapEntries",
        "forEach" => "hashMapForEach",
        "map" => "hashMapMap",
        "filter" => "hashMapFilter",
        // Mutating methods — CoW, return new map (write-back required)
        "set" | "put" => "hashMapPut",
        "remove" | "delete" => "hashMapRemove",
        "clear" => "hashMapClear",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a HashSet method call to its stdlib function name.
fn resolve_hashset_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "add" => "hashSetAdd",
        "remove" | "delete" => "hashSetRemove",
        "has" | "contains" => "hashSetHas",
        "size" | "len" => "hashSetSize",
        "isEmpty" => "hashSetIsEmpty",
        "toArray" => "hashSetToArray",
        "forEach" => "hashSetForEach",
        "clear" => "hashSetClear",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a Queue method call to its stdlib function name.
fn resolve_queue_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "enqueue" | "push" => "queueEnqueue",
        "dequeue" | "pop" => "queueDequeue",
        "peek" => "queuePeek",
        "size" | "len" => "queueSize",
        "isEmpty" => "queueIsEmpty",
        "toArray" => "queueToArray",
        "clear" => "queueClear",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a Stack method call to its stdlib function name.
fn resolve_stack_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "push" => "stackPush",
        "pop" => "stackPop",
        "peek" => "stackPeek",
        "size" | "len" => "stackSize",
        "isEmpty" => "stackIsEmpty",
        "toArray" => "stackToArray",
        "clear" => "stackClear",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve an Option<T> method call to its stdlib function name.
fn resolve_option_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "unwrap" => "unwrap",
        "unwrapOr" => "unwrapOr",
        "isSome" => "isSome",
        "isNone" => "isNone",
        "map" => "optionMap",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve a Result<T,E> method call to its stdlib function name.
fn resolve_result_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "unwrap" => "unwrap",
        "unwrapOr" => "unwrapOr",
        "isOk" => "isOk",
        "isErr" => "isErr",
        "map" => "resultMap",
        "mapErr" => "resultMapErr",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Json.method() → stdlib function name.
fn resolve_json_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "parse" => "parseJSON",
        "stringify" => "toJSON",
        "isValid" => "isValidJSON",
        "prettify" => "prettifyJSON",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Math.method() → stdlib function name.
fn resolve_math_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "sqrt" => "sqrt",
        "abs" => "abs",
        "floor" => "floor",
        "ceil" => "ceil",
        "round" => "round",
        "min" => "min",
        "max" => "max",
        "pow" => "pow",
        "log" => "log",
        "sin" => "sin",
        "cos" => "cos",
        "tan" => "tan",
        "clamp" => "clamp",
        "sign" => "sign",
        "random" => "random",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Env.method() → stdlib function name.
fn resolve_env_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "get" => "getEnv",
        "set" => "setEnv",
        "unset" => "unsetEnv",
        "list" => "listEnv",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve File.method() → stdlib function name.
fn resolve_file_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "read" => "readFile",
        "write" => "writeFile",
        "append" => "appendFile",
        "exists" => "fileExists",
        "remove" => "removeFile",
        "createDir" => "createDir",
        "removeDir" => "removeDir",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Process.method() → stdlib function name.
fn resolve_process_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "cwd" => "getCwd",
        "pid" => "getPid",
        "spawn" => "spawnProcess",
        "exec" => "exec",
        "shell" => "shell",
        "shellOut" => "shellOut",
        "args" => "getProcessArgs",
        "run" => "processRun",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve ProcessOutput.method() → stdlib function name.
fn resolve_process_output_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "stdout" => "processOutputStdout",
        "stderr" => "processOutputStderr",
        "exitCode" => "processOutputExitCode",
        "success" => "processOutputSuccess",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve DateTime.method() → stdlib function name.
fn resolve_datetime_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "now" => "dateTimeNow",
        "fromTimestamp" => "dateTimeFromTimestamp",
        "fromComponents" => "dateTimeFromComponents",
        "parseIso" => "dateTimeParseIso",
        "parse" => "dateTimeParse",
        "parseRfc3339" => "dateTimeParseRfc3339",
        "parseRfc2822" => "dateTimeParseRfc2822",
        "utc" => "dateTimeUtc",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Path.method() → stdlib function name.
fn resolve_path_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "join" => "pathJoin",
        "dirname" => "pathDirname",
        "basename" => "pathBasename",
        "extension" => "pathExtension",
        "exists" => "pathExists",
        "isAbsolute" => "pathIsAbsolute",
        "isRelative" => "pathIsRelative",
        "normalize" => "pathNormalize",
        "absolute" => "pathAbsolute",
        "parent" => "pathParent",
        "canonical" => "pathCanonical",
        "homedir" => "pathHomedir",
        "cwd" => "pathCwd",
        "tempdir" => "pathTempdir",
        "separator" => "pathSeparator",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Http.method() → stdlib function name.
fn resolve_http_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "get" => "httpRequestGet",
        "post" => "httpRequestPost",
        "put" => "httpRequestPut",
        "delete" => "httpRequestDelete",
        "patch" => "httpRequestPatch",
        "request" => "httpRequest",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Net.method() → stdlib function name.
fn resolve_net_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "tcpConnect" => "tcpConnect",
        "tcpListen" => "tcpListen",
        "tcpWrite" => "tcpWrite",
        "tcpRead" => "tcpRead",
        "tcpClose" => "tcpClose",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Crypto.method() → stdlib function name.
fn resolve_crypto_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "sha256" => "sha256",
        "sha512" => "sha512",
        "aesEncrypt" => "aesGcmEncrypt",
        "aesDecrypt" => "aesGcmDecrypt",
        "generateKey" => "aesGcmGenerateKey",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Regex.method() → stdlib function name.
/// Note: Regex.new() returns Result<Regex>. Methods like test/isMatch/find take the compiled Regex.
/// Regex.test(r, s) and Regex.isMatch(r, s) both map to regexIsMatch (compiled Regex, string).
/// Regex.escape(s) maps to regexEscape (string pattern only, no Regex arg).
fn resolve_regex_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "new" => "regexNew",
        "test" | "isMatch" => "regexIsMatch",
        "find" => "regexFind",
        "findAll" => "regexFindAll",
        "replace" => "regexReplace",
        "replaceAll" => "regexReplaceAll",
        "split" => "regexSplit",
        "captures" => "regexCaptures",
        "escape" => "regexEscape",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Look up the preferred replacement for a deprecated global stdlib function name.
/// Returns `Some("use X.method() instead")` if the name is deprecated, or `None` if it is current.
pub fn deprecated_global_replacement(name: &str) -> Option<&'static str> {
    match name {
        // Array
        "arrayPush" => Some("arr.push(x)"),
        "arrayPop" => Some("arr.pop()"),
        "arrayShift" => Some("arr.shift()"),
        "arrayUnshift" => Some("arr.unshift(x)"),
        "arrayReverse" => Some("arr.reverse()"),
        "arraySort" => Some("arr.sort()"),
        "arraySortBy" => Some("arr.sortBy(fn)"),
        "arrayIsEmpty" => Some("arr.isEmpty()"),
        "arrayIncludes" => Some("arr.includes(x)"),
        "arrayIndexOf" => Some("arr.indexOf(x)"),
        "arrayLastIndexOf" => Some("arr.lastIndexOf(x)"),
        // HashMap — hashMapNew() is NOT deprecated (no replacement syntax exists yet per spec)
        "hashMapGet" => Some("m.get(key)"),
        "hashMapPut" => Some("m.set(key, val)"),
        "hashMapRemove" => Some("m.remove(key)"),
        "hashMapHas" => Some("m.has(key)"),
        "hashMapSize" => Some("m.size()"),
        "hashMapIsEmpty" => Some("m.isEmpty()"),
        "hashMapKeys" => Some("m.keys()"),
        "hashMapValues" => Some("m.values()"),
        "hashMapEntries" => Some("m.entries()"),
        "hashMapClear" => Some("m.clear()"),
        // HashSet
        "hashSetNew" => Some("let s = hashSet()"),
        "hashSetAdd" => Some("s.add(x)"),
        "hashSetRemove" => Some("s.remove(x)"),
        "hashSetHas" => Some("s.has(x)"),
        "hashSetSize" => Some("s.size()"),
        "hashSetIsEmpty" => Some("s.isEmpty()"),
        "hashSetToArray" => Some("s.toArray()"),
        "hashSetClear" => Some("s.clear()"),
        // Queue / Stack
        "queueEnqueue" => Some("q.enqueue(x)"),
        "queueDequeue" => Some("q.dequeue()"),
        "queuePeek" => Some("q.peek()"),
        "queueSize" => Some("q.size()"),
        "queueIsEmpty" => Some("q.isEmpty()"),
        "stackPush" => Some("s.push(x)"),
        "stackPop" => Some("s.pop()"),
        "stackPeek" => Some("s.peek()"),
        "stackSize" => Some("s.size()"),
        "stackIsEmpty" => Some("s.isEmpty()"),
        // String — bare globals deprecated in favour of method style
        "toUpperCase" => Some("s.toUpperCase()"),
        "toLowerCase" => Some("s.toLowerCase()"),
        "trim" => Some("s.trim()"),
        "trimStart" => Some("s.trimStart()"),
        "trimEnd" => Some("s.trimEnd()"),
        "charAt" => Some("s.charAt(i)"),
        "substring" => Some("s.substring(start, end)"),
        "indexOf" => Some("s.indexOf(sub)"),
        "lastIndexOf" => Some("s.lastIndexOf(sub)"),
        "includes" => Some("s.includes(sub)"),
        "startsWith" => Some("s.startsWith(prefix)"),
        "endsWith" => Some("s.endsWith(suffix)"),
        "repeat" => Some("s.repeat(n)"),
        "replace" => Some("s.replace(from, to)"),
        "replaceAll" => Some("s.replaceAll(from, to)"),
        "padStart" => Some("s.padStart(len, ch)"),
        "padEnd" => Some("s.padEnd(len, ch)"),
        // String/Array — split and join
        "split" => Some("s.split(sep)"),
        "join" => Some("arr.join(sep)"),
        // JSON
        "parseJSON" => Some("Json.parse(s)"),
        "toJSON" => Some("Json.stringify(v)"),
        "isValidJSON" => Some("Json.isValid(s)"),
        // Math globals are NOT deprecated — both Math.sqrt(x) and sqrt(x) are valid per D-032 Rule 3
        // Process / shell execution
        "shell" => Some("Process.shell(cmd)"),
        "shellOut" => Some("Process.shellOut(cmd)"),
        "exec" => Some("Process.exec(cmd, args)"),
        "spawnProcess" => Some("Process.spawn(cmd, args)"),
        // IO / File
        "readFile" => Some("File.read(path)"),
        "writeFile" => Some("File.write(path, content)"),
        "appendFile" => Some("File.append(path, content)"),
        "fileExists" => Some("File.exists(path)"),
        "removeFile" => Some("File.remove(path)"),
        "createDir" => Some("File.createDir(path)"),
        "removeDir" => Some("File.removeDir(path)"),
        // Path
        "pathJoin" => Some("Path.join(...)"),
        "pathDirname" => Some("Path.dirname(path)"),
        "pathBasename" => Some("Path.basename(path)"),
        "pathExtension" => Some("Path.extension(path)"),
        "pathExists" => Some("Path.exists(path)"),
        "pathIsAbsolute" => Some("Path.isAbsolute(path)"),
        "pathIsRelative" => Some("Path.isRelative(path)"),
        "pathNormalize" => Some("Path.normalize(path)"),
        "pathAbsolute" => Some("Path.absolute(path)"),
        "pathParent" => Some("Path.parent(path)"),
        // Process / Env
        "getEnv" => Some("Env.get(key)"),
        "setEnv" => Some("Env.set(key, val)"),
        "unsetEnv" => Some("Env.unset(key)"),
        "getCwd" => Some("Process.cwd()"),
        "getPid" => Some("Process.pid()"),
        // DateTime
        "dateTimeNow" => Some("DateTime.now()"),
        "dateTimeFromTimestamp" => Some("DateTime.fromTimestamp(ts)"),
        "dateTimeParseIso" => Some("DateTime.parseIso(s)"),
        // Regex
        "regexNew" => Some("Regex.new(pattern)"),
        "regexTest" => Some("Regex.test(r, s)"),
        "regexIsMatch" => Some("Regex.isMatch(r, s)"),
        "regexFind" => Some("Regex.find(r, s)"),
        "regexFindAll" => Some("Regex.findAll(r, s)"),
        "regexReplace" => Some("Regex.replace(r, s, rep)"),
        "regexReplaceAll" => Some("Regex.replaceAll(r, s, rep)"),
        "regexSplit" => Some("Regex.split(r, s)"),
        "regexEscape" => Some("Regex.escape(s)"),
        // Crypto
        "sha256" => Some("Crypto.sha256(s)"),
        "sha512" => Some("Crypto.sha512(s)"),
        "aesGcmEncrypt" => Some("Crypto.aesEncrypt(key, data)"),
        "aesGcmDecrypt" => Some("Crypto.aesDecrypt(key, data)"),
        // Http
        "httpRequestGet" => Some("Http.get(url)"),
        "httpRequestPost" => Some("Http.post(url, body)"),
        "httpRequestPut" => Some("Http.put(url, body)"),
        "httpRequestDelete" => Some("Http.delete(url)"),
        _ => None,
    }
}

/// Returns true if a stdlib function name is a mutating array method (returns modified collection).
pub fn is_array_mutating_collection(func_name: &str) -> bool {
    matches!(func_name, "arrayPush" | "arrayUnshift" | "arrayReverse")
}

/// Returns true if a stdlib function name is a mutating array method that returns a pair
/// `[extracted_value, new_array]` (pop/shift pattern).
pub fn is_array_mutating_pair(func_name: &str) -> bool {
    matches!(func_name, "arrayPop" | "arrayShift")
}

/// Returns true if a stdlib function mutates a collection and returns the new collection directly.
/// Covers: HashMap.put/clear, HashSet.add/remove/clear, Queue.enqueue/clear, Stack.push/clear.
pub fn is_collection_mutating_simple(func_name: &str) -> bool {
    matches!(
        func_name,
        "hashMapPut"
            | "hashMapClear"
            | "hashSetAdd"
            | "hashSetRemove"
            | "hashSetClear"
            | "queueEnqueue"
            | "queueClear"
            | "stackPush"
            | "stackClear"
    )
}

/// Returns true if a stdlib function mutates a collection and returns `[extracted_value, new_collection]`.
/// Covers: HashMap.remove, Queue.dequeue, Stack.pop.
pub fn is_collection_mutating_pair(func_name: &str) -> bool {
    matches!(func_name, "hashMapRemove" | "queueDequeue" | "stackPop")
}

/// Capitalize first letter of each snake_case segment and join.
///
/// "as_string" → "AsString"
/// "is_null" → "IsNull"
fn capitalize_first(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect()
}

/// Returns true if the resolved stdlib function name is a callback-based intrinsic.
///
/// These cannot be called via call_builtin (stdlib registry) — they must be dispatched
/// via invoke_callee as Value::Builtin(name) so the interpreter/VM can execute the callback.
pub fn is_callback_intrinsic(func_name: &str) -> bool {
    matches!(
        func_name,
        "map"
            | "filter"
            | "reduce"
            | "forEach"
            | "find"
            | "findIndex"
            | "flatMap"
            | "some"
            | "every"
            | "sort"
            | "sortBy"
            | "resultMap"
            | "resultMapErr"
            | "hashMapForEach"
            | "hashMapMap"
            | "hashMapFilter"
            | "hashSetForEach"
            | "hashSetMap"
            | "hashSetFilter"
    )
}

/// Resolve console.method() → stdlib function name.
fn resolve_console_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "log" => "consoleLog",
        "println" => "consolePrintln",
        "print" => "consolePrint",
        "error" => "consoleError",
        "warn" => "consoleWarn",
        "debug" => "consoleDebug",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Io.method() → stdlib function name.
fn resolve_io_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "readLine" => "ioReadLine",
        "readLinePrompt" => "ioReadLinePrompt",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve DateTime instance method → stdlib function name.
/// These take the DateTime value as the first argument (receiver).
fn resolve_datetime_instance_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "year" => "dateTimeYear",
        "month" => "dateTimeMonth",
        "day" => "dateTimeDay",
        "hour" => "dateTimeHour",
        "minute" => "dateTimeMinute",
        "second" => "dateTimeSecond",
        "weekday" => "dateTimeWeekday",
        "dayOfYear" => "dateTimeDayOfYear",
        "addSeconds" => "dateTimeAddSeconds",
        "addMinutes" => "dateTimeAddMinutes",
        "addHours" => "dateTimeAddHours",
        "addDays" => "dateTimeAddDays",
        "diff" => "dateTimeDiff",
        "compare" => "dateTimeCompare",
        "toIso" => "dateTimeToIso",
        "format" => "dateTimeFormat",
        "toRfc3339" => "dateTimeToRfc3339",
        "toRfc2822" => "dateTimeToRfc2822",
        "timestamp" => "dateTimeTimestamp",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Regex instance method → stdlib function name.
/// These take the Regex value as the first argument (receiver).
fn resolve_regex_instance_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "test" | "isMatch" => "regexTest",
        "find" => "regexFind",
        "findAll" => "regexFindAll",
        "replace" => "regexReplace",
        "replaceAll" => "regexReplaceAll",
        "split" => "regexSplit",
        _ => return None,
    };
    Some(func_name.to_string())
}
