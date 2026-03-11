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
    /// Static namespace: Encoding.base64Encode(), Encoding.hexEncode(), etc.
    EncodingNs,
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
    /// Static namespace: Gzip.compress(), Gzip.decompress(), etc.
    GzipNs,
    /// Static namespace: Tar.create(), Tar.extract(), Tar.list(), etc.
    TarNs,
    /// Static namespace: Zip.create(), Zip.extract(), Zip.list(), etc.
    ZipNs,
    /// Static namespace: task.spawn(), task.join(), task.sleep(), etc.
    TaskNs,
    /// Instance methods on ChannelSender values (.send(), .close())
    ChannelSender,
    /// Instance methods on ChannelReceiver values (.receive(), .close())
    ChannelReceiver,
    /// Instance methods on AsyncMutex values (.lock(), .get(), .set())
    AsyncMutexValue,
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
        TypeTag::EncodingNs => resolve_encoding_ns_method(method_name),
        TypeTag::RegexNs => resolve_regex_ns_method(method_name),
        TypeTag::IoNs => resolve_io_ns_method(method_name),
        TypeTag::ConsoleNs => resolve_console_ns_method(method_name),
        TypeTag::DateTime => resolve_datetime_instance_method(method_name),
        TypeTag::RegexValue => resolve_regex_instance_method(method_name),
        TypeTag::ProcessOutput => resolve_process_output_method(method_name),
        TypeTag::GzipNs => resolve_gzip_ns_method(method_name),
        TypeTag::TarNs => resolve_tar_ns_method(method_name),
        TypeTag::ZipNs => resolve_zip_ns_method(method_name),
        TypeTag::TaskNs => resolve_task_ns_method(method_name),
        TypeTag::ChannelSender => resolve_channel_sender_method(method_name),
        TypeTag::ChannelReceiver => resolve_channel_receiver_method(method_name),
        TypeTag::AsyncMutexValue => resolve_async_mutex_method(method_name),
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
            | "encoding"
            | "regex"
            | "io"
            | "console"
            | "Env"
            | "File"
            | "Process"
            | "DateTime"
            | "Path"
            | "Http"
            | "Net"
            | "Crypto"
            | "Regex"
            | "Io"
            | "gzip"
            | "tar"
            | "zip"
            | "task"
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
        "encoding" => Some(TypeTag::EncodingNs),
        "regex" => Some(TypeTag::RegexNs),
        "io" => Some(TypeTag::IoNs),
        "console" => Some(TypeTag::ConsoleNs),
        "Env" => Some(TypeTag::EnvNs),
        "File" => Some(TypeTag::FileNs),
        "Process" => Some(TypeTag::ProcessNs),
        "DateTime" => Some(TypeTag::DateTimeNs),
        "Path" => Some(TypeTag::PathNs),
        "Http" => Some(TypeTag::HttpNs),
        "Net" => Some(TypeTag::NetNs),
        "Crypto" => Some(TypeTag::CryptoNs),
        "Regex" => Some(TypeTag::RegexNs),
        "Io" => Some(TypeTag::IoNs),
        "gzip" => Some(TypeTag::GzipNs),
        "tar" => Some(TypeTag::TarNs),
        "zip" => Some(TypeTag::ZipNs),
        "task" => Some(TypeTag::TaskNs),
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
/// B23: All Json.* methods route through jsonNs* internal keys. No bare globals.
fn resolve_json_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "parse" => "jsonNsParse",
        "stringify" => "jsonNsStringify",
        "isValid" => "jsonNsIsValid",
        "prettify" => "jsonNsPrettify",
        "minify" => "jsonNsMinify",
        "keys" => "jsonNsKeys",
        "getString" => "jsonNsGetString",
        "getNumber" => "jsonNsGetNumber",
        "getBool" => "jsonNsGetBool",
        "getArray" => "jsonNsGetArray",
        "getObject" => "jsonNsGetObject",
        "isNull" => "jsonNsIsNull",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Math.method() → stdlib function name.
fn resolve_math_ns_method(method_name: &str) -> Option<String> {
    // B22: All math functions registered under "math*" keys (no bare globals).
    let func_name = match method_name {
        "sqrt" => "mathSqrt",
        "abs" => "mathAbs",
        "floor" => "mathFloor",
        "ceil" => "mathCeil",
        "round" => "mathRound",
        "min" => "mathMin",
        "max" => "mathMax",
        "pow" => "mathPow",
        "log" => "mathLog",
        "sin" => "mathSin",
        "cos" => "mathCos",
        "tan" => "mathTan",
        "asin" => "mathAsin",
        "acos" => "mathAcos",
        "atan" => "mathAtan",
        "atan2" => "mathAtan2",
        "trunc" => "mathTrunc",
        "log2" => "mathLog2",
        "log10" => "mathLog10",
        "exp" => "mathExp",
        "cbrt" => "mathCbrt",
        "hypot" => "mathHypot",
        "clamp" => "mathClamp",
        "sign" => "mathSign",
        "random" => "mathRandom",
        "PI" => "mathPI",
        "E" => "mathE",
        "SQRT2" => "mathSQRT2",
        "LN2" => "mathLN2",
        "LN10" => "mathLN10",
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
    // B24: all file.* methods map to fileNs* internal keys. Bare globals removed.
    let func_name = match method_name {
        // Core I/O
        "read" => "fileNsRead",
        "write" => "fileNsWrite",
        "append" => "fileNsAppend",
        "exists" => "fileNsExists",
        "remove" => "fileNsRemove",
        // Directory operations
        "createDir" => "fileNsCreateDir",
        "removeDir" => "fileNsRemoveDir",
        "mkdir" => "fileNsMkdir",
        "mkdirp" => "fileNsMkdirp",
        "rmdir" => "fileNsRmdir",
        "rmdirRecursive" => "fileNsRmdirRecursive",
        "readDir" => "fileNsReadDir",
        "walk" => "fileNsWalk",
        "filterEntries" => "fileNsFilterEntries",
        "sortEntries" => "fileNsSortEntries",
        // Metadata
        "size" => "fileNsSize",
        "mtime" => "fileNsMtime",
        "ctime" => "fileNsCtime",
        "atime" => "fileNsAtime",
        "permissions" => "fileNsPermissions",
        "inode" => "fileNsInode",
        // Type checks
        "isDir" => "fileNsIsDir",
        "isFile" => "fileNsIsFile",
        "isSymlink" => "fileNsIsSymlink",
        // Symlinks
        "symlink" => "fileNsSymlink",
        "readLink" => "fileNsReadLink",
        // Temporary files
        "tempFile" => "fileNsTempFile",
        "tempDir" => "fileNsTempDir",
        // Watch
        "watch" => "fileNsWatch",
        "watchNext" => "fileNsWatchNext",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Process.method() → stdlib function name.
fn resolve_process_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "cwd" => "getCwd",
        "pid" => "getPid",
        "spawn" => "processNsSpawn",
        "exec" => "exec",
        "shell" => "shell",
        "shellOut" => "shellOut",
        "args" => "getProcessArgs",
        "run" => "processRun",
        "waitFor" => "processNsWaitFor",
        "kill" => "processNsKill",
        "isRunning" => "processNsIsRunning",
        "stdin" => "processNsStdin",
        "stdout" => "processNsStdout",
        "stderr" => "processNsStderr",
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
        "get" => "httpNsGet",
        "post" => "httpNsPost",
        "put" => "httpNsPut",
        "delete" => "httpNsDelete",
        "patch" => "httpNsPatch",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Net.method() → stdlib function name.
fn resolve_net_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        // TCP
        "tcpConnect" => "tcpConnect",
        "tcpListen" => "tcpListen",
        "tcpWrite" => "tcpWrite",
        "tcpRead" => "tcpRead",
        "tcpReadBytes" => "tcpReadBytes",
        "tcpClose" => "tcpClose",
        "tcpSetTimeout" => "tcpSetTimeout",
        "tcpSetNodelay" => "tcpSetNodelay",
        "tcpLocalAddr" => "tcpLocalAddr",
        "tcpRemoteAddr" => "tcpRemoteAddr",
        "tcpAccept" => "tcpAccept",
        "tcpListenerAddr" => "tcpListenerAddr",
        "tcpListenerClose" => "tcpListenerClose",
        // UDP
        "udpBind" => "udpBind",
        "udpSend" => "udpSend",
        "udpReceive" => "udpReceive",
        "udpClose" => "udpClose",
        "udpLocalAddr" => "udpLocalAddr",
        "udpSetTimeout" => "udpSetTimeout",
        // TLS
        "tlsConnect" => "tlsConnect",
        "tlsRead" => "tlsRead",
        "tlsWrite" => "tlsWrite",
        "tlsClose" => "tlsClose",
        // WebSocket
        "wsConnect" => "wsConnect",
        "wsSend" => "wsSend",
        "wsSendBinary" => "wsSendBinary",
        "wsReceive" => "wsReceive",
        "wsClose" => "wsClose",
        "wsPing" => "wsPing",
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
        "hmac" => "cryptoNsHmac",
        "hmacVerify" => "cryptoNsHmacVerify",
        "blake3" => "cryptoNsBlake3",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Encoding.method() → stdlib function name.
fn resolve_encoding_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "base64Encode" => "encodingNsBase64Encode",
        "base64Decode" => "encodingNsBase64Decode",
        "base64UrlEncode" => "encodingNsBase64UrlEncode",
        "base64UrlDecode" => "encodingNsBase64UrlDecode",
        "hexEncode" => "encodingNsHexEncode",
        "hexDecode" => "encodingNsHexDecode",
        "urlEncode" => "encodingNsUrlEncode",
        "urlDecode" => "encodingNsUrlDecode",
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
        // JSON — B23: all bare json globals deprecated
        "parseJSON" => Some("Json.parse(s)"),
        "toJSON" => Some("Json.stringify(v)"),
        "isValidJSON" => Some("Json.isValid(s)"),
        "prettifyJSON" => Some("Json.prettify(s, indent)"),
        "minifyJSON" => Some("Json.minify(s)"),
        "jsonGetString" => Some("Json.getString(s, key)"),
        "jsonGetNumber" => Some("Json.getNumber(s, key)"),
        "jsonGetBool" => Some("Json.getBool(s, key)"),
        "jsonGetArray" => Some("Json.getArray(s, key)"),
        "jsonGetObject" => Some("Json.getObject(s, key)"),
        "jsonIsNull" => Some("Json.isNull(s, key)"),
        "jsonKeys" => Some("Json.keys(s)"),
        // Math globals — B22: bare math globals removed. Use Math.* namespace.
        "sqrt" => Some("Math.sqrt(x)"),
        "abs" => Some("Math.abs(x)"),
        "floor" => Some("Math.floor(x)"),
        "ceil" => Some("Math.ceil(x)"),
        "round" => Some("Math.round(x)"),
        "min" => Some("Math.min(a, b)"),
        "max" => Some("Math.max(a, b)"),
        "pow" => Some("Math.pow(base, exp)"),
        "log" => Some("Math.log(x)"),
        "sin" => Some("Math.sin(x)"),
        "cos" => Some("Math.cos(x)"),
        "tan" => Some("Math.tan(x)"),
        "asin" => Some("Math.asin(x)"),
        "acos" => Some("Math.acos(x)"),
        "atan" => Some("Math.atan(x)"),
        "atan2" => Some("Math.atan2(y, x)"),
        "trunc" => Some("Math.trunc(x)"),
        "log2" => Some("Math.log2(x)"),
        "log10" => Some("Math.log10(x)"),
        "exp" => Some("Math.exp(x)"),
        "cbrt" => Some("Math.cbrt(x)"),
        "hypot" => Some("Math.hypot(x, y)"),
        "clamp" => Some("Math.clamp(v, min, max)"),
        "sign" => Some("Math.sign(x)"),
        "random" => Some("Math.random()"),
        // Process / shell execution
        "shell" => Some("Process.shell(cmd)"),
        "shellOut" => Some("Process.shellOut(cmd)"),
        "exec" => Some("Process.exec(cmd, args)"),
        "spawnProcess" => Some("process.spawn(cmd, args)"),
        // IO / File
        "readFile" => Some("file.read(path)"),
        "writeFile" => Some("file.write(path, content)"),
        "appendFile" => Some("file.append(path, content)"),
        "fileExists" => Some("file.exists(path)"),
        "removeFile" => Some("file.remove(path)"),
        "createDir" => Some("file.createDir(path)"),
        "removeDir" => Some("file.removeDir(path)"),
        // B24: bare fs* globals deprecated — use file.* namespace
        "fsMkdir" => Some("file.mkdir(path)"),
        "fsMkdirp" => Some("file.mkdirp(path)"),
        "fsRmdir" => Some("file.rmdir(path)"),
        "fsRmdirRecursive" => Some("file.rmdirRecursive(path)"),
        "fsReaddir" => Some("file.readDir(path)"),
        "fsWalk" => Some("file.walk(path)"),
        "fsFilterEntries" => Some("file.filterEntries(entries, pattern)"),
        "fsSortEntries" => Some("file.sortEntries(entries)"),
        "fsSize" => Some("file.size(path)"),
        "fsMtime" => Some("file.mtime(path)"),
        "fsCtime" => Some("file.ctime(path)"),
        "fsAtime" => Some("file.atime(path)"),
        "fsPermissions" => Some("file.permissions(path)"),
        "fsIsDir" => Some("file.isDir(path)"),
        "fsIsFile" => Some("file.isFile(path)"),
        "fsIsSymlink" => Some("file.isSymlink(path)"),
        "fsInode" => Some("file.inode(path)"),
        "fsTmpfile" => Some("file.tempFile()"),
        "fsTmpdir" => Some("file.tempDir()"),
        "fsTmpfileNamed" => Some("file.tempFile()"),
        "fsGetTempDir" => Some("file.tempDir()"),
        "fsSymlink" => Some("file.symlink(target, link)"),
        "fsReadlink" => Some("file.readLink(path)"),
        "fsResolveSymlink" => Some("file.readLink(path)"),
        "fsWatch" => Some("file.watch(path)"),
        "fsWatchNext" => Some("file.watchNext(handle)"),
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
        "sha256" => Some("crypto.sha256(s)"),
        "sha512" => Some("crypto.sha512(s)"),
        "aesGcmEncrypt" => Some("crypto.aesEncrypt(key, data)"),
        "aesGcmDecrypt" => Some("crypto.aesDecrypt(key, data)"),
        // B27: bare encoding/crypto globals removed
        "blake3Hash" => Some("crypto.blake3(data)"),
        "hmacSha256" => Some("crypto.hmac(key, data, \"sha256\")"),
        "hmacSha256Verify" => Some("crypto.hmacVerify(key, data, sig, \"sha256\")"),
        "base64Encode" => Some("encoding.base64Encode(s)"),
        "base64Decode" => Some("encoding.base64Decode(s)"),
        "base64UrlEncode" => Some("encoding.base64UrlEncode(s)"),
        "base64UrlDecode" => Some("encoding.base64UrlDecode(s)"),
        "hexEncode" => Some("encoding.hexEncode(s)"),
        "hexDecode" => Some("encoding.hexDecode(s)"),
        "urlEncode" => Some("encoding.urlEncode(s)"),
        "urlDecode" => Some("encoding.urlDecode(s)"),
        // Http — options-object API (B28)
        "httpNsGet" => Some("http.get(url, options?)"),
        "httpNsPost" => Some("http.post(url, body?, options?)"),
        "httpNsPut" => Some("http.put(url, body?, options?)"),
        "httpNsDelete" => Some("http.delete(url, options?)"),
        "httpNsPatch" => Some("http.patch(url, body?, options?)"),
        // Net namespace — TCP (B29)
        "tcpConnect" => Some("net.tcpConnect(addr)"),
        "tcpListen" => Some("net.tcpListen(addr)"),
        "tcpWrite" => Some("net.tcpWrite(conn, data)"),
        "tcpRead" => Some("net.tcpRead(conn)"),
        "tcpReadBytes" => Some("net.tcpReadBytes(conn, n)"),
        "tcpClose" => Some("net.tcpClose(conn)"),
        "tcpSetTimeout" => Some("net.tcpSetTimeout(conn, ms)"),
        "tcpSetNodelay" => Some("net.tcpSetNodelay(conn, flag)"),
        "tcpLocalAddr" => Some("net.tcpLocalAddr(conn)"),
        "tcpRemoteAddr" => Some("net.tcpRemoteAddr(conn)"),
        "tcpAccept" => Some("net.tcpAccept(listener)"),
        "tcpListenerAddr" => Some("net.tcpListenerAddr(listener)"),
        "tcpListenerClose" => Some("net.tcpListenerClose(listener)"),
        // Net namespace — UDP (B29)
        "udpBind" => Some("net.udpBind(addr)"),
        "udpSend" => Some("net.udpSend(socket, addr, data)"),
        "udpReceive" => Some("net.udpReceive(socket)"),
        "udpClose" => Some("net.udpClose(socket)"),
        "udpLocalAddr" => Some("net.udpLocalAddr(socket)"),
        "udpSetTimeout" => Some("net.udpSetTimeout(socket, ms)"),
        // Net namespace — TLS (B29)
        "tlsConnect" => Some("net.tlsConnect(host, port)"),
        "tlsWrite" => Some("net.tlsWrite(conn, data)"),
        "tlsRead" => Some("net.tlsRead(conn)"),
        "tlsClose" => Some("net.tlsClose(conn)"),
        // Net namespace — WebSocket (B29)
        "wsConnect" => Some("net.wsConnect(url)"),
        "wsSend" => Some("net.wsSend(conn, data)"),
        "wsSendBinary" => Some("net.wsSendBinary(conn, data)"),
        "wsReceive" => Some("net.wsReceive(conn)"),
        "wsClose" => Some("net.wsClose(conn)"),
        "wsPing" => Some("net.wsPing(conn)"),
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
        "readLine" => "ioNsReadLine",
        "readLinePrompt" => "ioNsReadLinePrompt",
        "write" => "ioNsWrite",
        "writeLine" => "ioNsWriteLine",
        "readAll" => "ioNsReadAll",
        "flush" => "ioNsFlush",
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

/// Resolve Gzip.method() → stdlib function name.
fn resolve_gzip_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "compress" => "gzipCompress",
        "decompress" => "gzipDecompress",
        "decompressString" => "gzipDecompressString",
        "isGzip" => "gzipIsGzip",
        "compressionRatio" => "gzipCompressionRatio",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Tar.method() → stdlib function name.
fn resolve_tar_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "create" => "tarCreate",
        "createGz" => "tarCreateGz",
        "extract" => "tarExtract",
        "extractGz" => "tarExtractGz",
        "list" => "tarList",
        "contains" => "tarContains",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve task.method() → stdlib function name (B31).
fn resolve_task_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "spawn" => "taskNsSpawn",
        "join" => "taskNsJoin",
        "joinAll" => "taskNsJoinAll",
        "status" => "taskNsStatus",
        "cancel" => "taskNsCancel",
        "id" => "taskNsId",
        "sleep" => "taskNsSleep",
        "timeout" => "taskNsTimeout",
        "interval" => "taskNsInterval",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve ChannelSender instance method → stdlib function name (B31).
fn resolve_channel_sender_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "send" => "channelNsSend",
        "close" | "isClosed" => "channelNsClose",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve ChannelReceiver instance method → stdlib function name (B31).
fn resolve_channel_receiver_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "receive" => "channelNsReceive",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve AsyncMutex instance method → stdlib function name (B31).
fn resolve_async_mutex_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "lock" | "tryLock" => "asyncMutexNsLock",
        "get" => "asyncMutexNsGet",
        "set" => "asyncMutexNsSet",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Zip.method() → stdlib function name.
fn resolve_zip_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "create" => "zipCreate",
        "createWithComment" => "zipCreateWithComment",
        "extract" => "zipExtract",
        "extractFiles" => "zipExtractFiles",
        "list" => "zipList",
        "contains" => "zipContains",
        "addFile" => "zipAddFile",
        "validate" => "zipValidate",
        "compressionRatio" => "zipCompressionRatio",
        "comment" => "zipComment",
        _ => return None,
    };
    Some(func_name.to_string())
}
