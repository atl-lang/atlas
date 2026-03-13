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
    Map,
    Set,
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
    /// Static namespace: reflect.typeOf(), reflect.fields(), reflect.hasMethod(), etc.
    ReflectNs,
    /// Static namespace: sqlite.open(), etc.
    SqliteNs,
    /// Instance methods on SqliteConnection values (execute, query, close)
    SqliteConnection,
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
    /// Static namespace: sync.atomic(), sync.rwLock(), sync.semaphore(), etc.
    SyncNs,
    /// Instance methods on atomic handle (returned by sync.atomic())
    AtomicValue,
    /// Instance methods on rwlock handle (returned by sync.rwLock())
    RwLockValue,
    /// Instance methods on semaphore handle (returned by sync.semaphore())
    SemaphoreValue,
    /// Static namespace: future.resolve(), future.all(), future.race(), etc.
    FutureNs,
    /// Instance methods on Future values (.then(), .catch(), .finally(), etc.)
    FutureValue,
    /// Static namespace: test.assert(), test.equal(), test.throws(), etc.
    TestNs,
    /// Instance methods on number values: toString(), toFixed(), toInt()
    Number,
    /// Instance methods on bool values: toString()
    Bool,
}

/// Resolve a method call to its stdlib function name.
/// Returns None if the type/method combination is not registered.
pub fn resolve_method(type_tag: TypeTag, method_name: &str) -> Option<String> {
    match type_tag {
        TypeTag::JsonValue => Some(format!("json{}", capitalize_first(method_name))),
        TypeTag::Array => resolve_array_method(method_name),
        TypeTag::HttpResponse => resolve_http_response_method(method_name),
        TypeTag::String => resolve_string_method(method_name),
        TypeTag::Map => resolve_hashmap_method(method_name),
        TypeTag::Set => resolve_hashset_method(method_name),
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
        TypeTag::ReflectNs => resolve_reflect_ns_method(method_name),
        TypeTag::SqliteNs => resolve_sqlite_ns_method(method_name),
        TypeTag::SqliteConnection => resolve_sqlite_connection_method(method_name),
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
        TypeTag::SyncNs => resolve_sync_ns_method(method_name),
        TypeTag::AtomicValue => resolve_atomic_method(method_name),
        TypeTag::RwLockValue => resolve_rwlock_method(method_name),
        TypeTag::SemaphoreValue => resolve_semaphore_method(method_name),
        TypeTag::FutureNs => resolve_future_ns_method(method_name),
        TypeTag::FutureValue => resolve_future_instance_method(method_name),
        TypeTag::TestNs => resolve_test_ns_method(method_name),
        TypeTag::Number => resolve_number_method(method_name),
        TypeTag::Bool => resolve_bool_method(method_name),
    }
}

fn resolve_number_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "toString" => "numberToString",
        "toFixed" => "numberToFixed",
        "toInt" => "numberToInt",
        _ => return None,
    };
    Some(func_name.to_string())
}

fn resolve_bool_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "toString" => "boolToString",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Check if a name is a fundamental stdlib function that remains as a bare global.
/// These are NOT migrated to namespace syntax because they are core language constructs.
pub fn is_allowed_bare_global(name: &str) -> bool {
    matches!(
        name,
        // Result/Option constructors
        "Ok" | "Err" | "Some" | "None"
        // Result/Option helpers (both snake_case and camelCase)
        | "unwrap" | "unwrap_or" | "expect"
        | "is_ok" | "is_err" | "is_some" | "is_none"
        | "isOk" | "isErr" | "isSome" | "isNone"
        // Core utilities (print is console.log, not a bare global)
        | "len" | "typeof" | "type_of"
        // Type constructors
        | "Map" | "Set" | "Queue" | "Stack"
    )
}

/// Check if an identifier name is a static namespace sentinel.
/// Case-insensitive matching for AI-friendliness (both `Math.sqrt` and `math.sqrt` work).
pub fn is_static_namespace(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "json"
            | "math"
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
            | "reflect"
            | "sqlite"
            | "gzip"
            | "tar"
            | "zip"
            | "task"
            | "sync"
            | "future"
            | "test"
    )
}

/// Map a static namespace identifier to its TypeTag.
/// Case-insensitive matching for AI-friendliness.
pub fn namespace_type_tag(name: &str) -> Option<TypeTag> {
    match name.to_lowercase().as_str() {
        "json" => Some(TypeTag::JsonNs),
        "math" => Some(TypeTag::MathNs),
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
        "reflect" => Some(TypeTag::ReflectNs),
        "sqlite" => Some(TypeTag::SqliteNs),
        "gzip" => Some(TypeTag::GzipNs),
        "tar" => Some(TypeTag::TarNs),
        "zip" => Some(TypeTag::ZipNs),
        "task" => Some(TypeTag::TaskNs),
        "sync" => Some(TypeTag::SyncNs),
        "future" => Some(TypeTag::FutureNs),
        "test" => Some(TypeTag::TestNs),
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
        // Type conversion (bare globals purged)
        "toNumber" => "parseFloat",
        "toInt" => "parseInt",
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
        "expect" => "expect", // H-268
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
        "expect" => "expect", // H-268
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
        "rename" => "fileNsRename",
        "copy" => "fileNsCopy",
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
        "info" => "fileNsInfo",
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
        // Async operations (B40-P07)
        "readAsync" => "fileNsReadAsync",
        "writeAsync" => "fileNsWriteAsync",
        "appendAsync" => "fileNsAppendAsync",
        "renameAsync" => "fileNsRenameAsync",
        "copyAsync" => "fileNsCopyAsync",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Process.method() → stdlib function name.
fn resolve_process_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "cwd" => "getCwd",
        "pid" => "getPid",
        "platform" => "getPlatform",
        "arch" => "getArch",
        "exit" => "processExit",
        "spawn" => "processNsSpawn",
        "exec" => "exec",
        "shell" => "shell",
        "shellOut" => "shellOut",
        "args" | "getProcessArgs" => "getProcessArgs",
        "run" => "processRun",
        "waitFor" => "processNsWaitFor",
        "kill" => "processNsKill",
        "isRunning" => "processNsIsRunning",
        "stdin" => "processNsStdin",
        "stdout" => "processNsStdout",
        "stderr" => "processNsStderr",
        "output" => "processNsOutput",
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
        "extension" | "extname" => "pathExtension", // extname is Node.js alias
        "exists" => "pathExists",
        "isAbsolute" => "pathIsAbsolute",
        "isRelative" => "pathIsRelative",
        "normalize" => "pathNormalize",
        "absolute" => "pathAbsolute",
        "resolve" => "pathResolve",
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

/// Returns the namespace-qualified replacement for a formerly-bare stdlib global name.
/// Used to enrich AT1001 "undefined identifier" errors with a "use X instead" hint.
/// Bare globals are dead — they fail at compile time. This table powers the hint only.
pub fn namespace_hint_for_bare_global(name: &str) -> Option<&'static str> {
    match name {
        // console
        "print" | "println" => Some("console.log()"),
        "eprint" | "eprintln" => Some("console.error()"),
        // Math
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
        // Json
        "parseJSON" => Some("Json.parse(s)"),
        "toJSON" => Some("Json.stringify(v)"),
        "isValidJSON" => Some("Json.isValid(s)"),
        "prettifyJSON" => Some("Json.prettify(s, indent)"),
        "minifyJSON" => Some("Json.minify(s)"),
        // file
        "readFile" => Some("file.read(path)"),
        "writeFile" => Some("file.write(path, content)"),
        "appendFile" => Some("file.append(path, content)"),
        "fileExists" => Some("file.exists(path)"),
        "removeFile" => Some("file.remove(path)"),
        "createDir" => Some("file.createDir(path)"),
        "removeDir" => Some("file.removeDir(path)"),
        "fsMkdir" => Some("file.mkdir(path)"),
        "fsMkdirp" => Some("file.mkdirp(path)"),
        "fsRmdir" => Some("file.rmdir(path)"),
        "fsRmdirRecursive" => Some("file.rmdirRecursive(path)"),
        "fsReaddir" => Some("file.readDir(path)"),
        "fsWalk" => Some("file.walk(path)"),
        "fsSize" => Some("file.size(path)"),
        // process
        "shell" => Some("process.shell(cmd)"),
        "shellOut" => Some("process.shell(cmd)"),
        "exec" => Some("process.exec(cmd, args)"),
        "spawnProcess" => Some("process.spawn(cmd, args)"),
        "getCwd" => Some("process.cwd()"),
        "getPid" => Some("process.pid()"),
        // env
        "getEnv" => Some("env.get(key)"),
        "setEnv" => Some("env.set(key, val)"),
        "unsetEnv" => Some("env.unset(key)"),
        // path
        "pathJoin" => Some("path.join(...)"),
        "pathDirname" => Some("path.dirname(path)"),
        "pathBasename" => Some("path.basename(path)"),
        "pathExtension" => Some("path.extension(path)"),
        "pathExists" => Some("path.exists(path)"),
        "pathIsAbsolute" => Some("path.isAbsolute(path)"),
        "pathNormalize" => Some("path.normalize(path)"),
        "pathAbsolute" => Some("path.absolute(path)"),
        // encoding
        "base64Encode" => Some("encoding.base64Encode(s)"),
        "base64Decode" => Some("encoding.base64Decode(s)"),
        "hexEncode" => Some("encoding.hexEncode(s)"),
        "hexDecode" => Some("encoding.hexDecode(s)"),
        "urlEncode" => Some("encoding.urlEncode(s)"),
        "urlDecode" => Some("encoding.urlDecode(s)"),
        // crypto
        "sha256" => Some("crypto.sha256(s)"),
        "sha512" => Some("crypto.sha512(s)"),
        "blake3Hash" => Some("crypto.blake3(data)"),
        "hmacSha256" => Some("crypto.hmac(key, data, \"sha256\")"),
        // http
        "httpGet" | "httpNsGet" => Some("http.get(url, options?)"),
        "httpPost" | "httpNsPost" => Some("http.post(url, body?, options?)"),
        "httpPut" | "httpNsPut" => Some("http.put(url, body?, options?)"),
        "httpDelete" | "httpNsDelete" => Some("http.delete(url, options?)"),
        // net
        "tcpConnect" => Some("net.tcpConnect(addr)"),
        "udpBind" => Some("net.udpBind(addr)"),
        "wsConnect" => Some("net.wsConnect(url)"),
        "tlsConnect" => Some("net.tlsConnect(host, port)"),
        // io
        "ioReadLine" | "readLine" => Some("io.readLine()"),
        // regex
        "regexNew" => Some("regex.new(pattern)"),
        "regexTest" | "regexIsMatch" => Some("regex.isMatch(r, s)"),
        // datetime
        "dateTimeNow" => Some("datetime.now()"),
        // Type conversion (bare globals purged - use instance methods)
        "parseInt" => Some("\"42\".toInt()"),
        "parseFloat" => Some("\"3.14\".toNumber()"),
        "toNumber" => Some("s.toNumber() or n.toString()"),
        "toString" => Some("value.toString()"),
        "toBool" => Some("value.toBool()"),
        "str" => Some("value.toString()"),
        // String bare globals (use instance methods)
        "split" => Some("s.split(delimiter)"),
        "trim" => Some("s.trim()"),
        "trimStart" => Some("s.trimStart()"),
        "trimEnd" => Some("s.trimEnd()"),
        "indexOf" => Some("s.indexOf(substr)"),
        "lastIndexOf" => Some("s.lastIndexOf(substr)"),
        "includes" => Some("s.includes(substr)"),
        "toUpperCase" => Some("s.toUpperCase()"),
        "toLowerCase" => Some("s.toLowerCase()"),
        "substring" => Some("s.substring(start, end)"),
        "charAt" => Some("s.charAt(index)"),
        "repeat" => Some("s.repeat(n)"),
        "replace" => Some("s.replace(old, new)"),
        "replaceAll" => Some("s.replaceAll(old, new)"),
        "padStart" => Some("s.padStart(len, char)"),
        "padEnd" => Some("s.padEnd(len, char)"),
        "startsWith" => Some("s.startsWith(prefix)"),
        "endsWith" => Some("s.endsWith(suffix)"),
        "join" => Some("arr.join(delimiter)"),
        "len" => Some("value.length or value.len()"),
        // Type checking (use typeof)
        "isString" | "isNumber" | "isBool" | "isArray" | "isFunction" | "isObject" | "isNull" => {
            Some("typeof(value) == \"type\"")
        }
        // Array bare globals
        "arrayPush" => Some("arr.push(x)"),
        "arrayPop" => Some("arr.pop()"),
        "arrayShift" => Some("arr.shift()"),
        "arrayReverse" => Some("arr.reverse()"),
        "arraySort" => Some("arr.sort()"),
        // HashMap bare globals
        "hashMapGet" => Some("m.get(key)"),
        "hashMapPut" => Some("m.set(key, val)"),
        "hashMapRemove" => Some("m.remove(key)"),
        "hashMapHas" => Some("m.has(key)"),
        // test
        "assert" => Some("test.assert(cond, msg?)"),
        "assertEqual" => Some("test.equal(a, b, msg?)"),
        "assertNotEqual" => Some("test.notEqual(a, b, msg?)"),
        "assertThrows" => Some("test.throws(fn, msg?)"),
        "assertOk" => Some("test.ok(result)"),
        "assertErr" => Some("test.err(result)"),
        "assertContains" => Some("test.contains(collection, val)"),
        "assertEmpty" => Some("test.empty(collection)"),
        // Prefixed bare globals that should use namespace syntax
        "mathSqrt" | "mathAbs" | "mathFloor" | "mathCeil" | "mathRound" | "mathMin" | "mathMax"
        | "mathPow" | "mathLog" | "mathSin" | "mathCos" | "mathTan" | "mathAsin" | "mathAcos"
        | "mathAtan" | "mathAtan2" | "mathTrunc" | "mathLog2" | "mathLog10" | "mathExp"
        | "mathCbrt" | "mathHypot" | "mathClamp" | "mathSign" | "mathRandom" | "mathPI"
        | "mathE" | "mathSQRT2" | "mathLN2" | "mathLN10" => {
            Some("Math.method() — use namespace syntax")
        }
        "consoleLog" | "consolePrintln" | "consolePrint" | "consoleError" | "consoleWarn"
        | "consoleDebug" => Some("console.method() — use namespace syntax"),
        "jsonNsParse" | "jsonNsStringify" | "jsonNsIsValid" | "jsonNsPrettify" | "jsonNsMinify"
        | "jsonNsKeys" | "jsonAsString" | "jsonAsNumber" | "jsonAsBool" | "jsonGetString"
        | "jsonGetNumber" | "jsonGetBool" | "jsonGetArray" | "jsonGetObject" | "jsonIsNull"
        | "jsonNsGetString" | "jsonNsGetNumber" | "jsonNsGetBool" | "jsonNsGetArray"
        | "jsonNsGetObject" | "jsonNsIsNull" => Some("Json.method() — use namespace syntax"),
        // Additional prefixed bare globals (only names NOT matched earlier in this function)
        "fsRead" | "fsWrite" | "fsAppend" | "fsExists" | "fsRemove" | "fsCopy" | "fsMove"
        | "fsIsDir" | "fsIsFile" => Some("file.method() — use namespace syntax"),
        "processRun" | "processExec" | "processShell" => {
            Some("process.method() — use namespace syntax")
        }
        "envGet" | "envSet" | "envUnset" | "envAll" => Some("env.method() — use namespace syntax"),
        "cryptoSha256" | "cryptoSha512" | "cryptoBlake3" | "cryptoHmac" | "cryptoRandomBytes"
        | "cryptoAesEncrypt" | "cryptoAesDecrypt" => Some("crypto.method() — use namespace syntax"),
        "encodingBase64Encode"
        | "encodingBase64Decode"
        | "encodingHexEncode"
        | "encodingHexDecode"
        | "encodingUrlEncode"
        | "encodingUrlDecode" => Some("encoding.method() — use namespace syntax"),
        "httpNsPatch" | "httpNsHead" | "httpNsOptions" => {
            Some("http.method() — use namespace syntax")
        }
        "ioReadLinePrompt" | "ioPrintln" | "ioPrint" | "ioEprintln" | "ioEprint" | "ioFlush" => {
            Some("io.method() — use namespace syntax")
        }
        "regexMatch" | "regexMatchAll" | "regexReplace" | "regexSplit" => {
            Some("regex.method() — use namespace syntax")
        }
        "dateTimeFromTimestamp" | "dateTimeParse" | "dateTimeFormat" => {
            Some("datetime.method() — use namespace syntax")
        }
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

/// Resolve reflect.method() → stdlib function name.
fn resolve_reflect_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "typeOf" => "reflectTypeOf",
        "fields" => "reflectFields",
        "hasMethod" => "reflectHasMethod",
        "isCallable" => "reflectIsCallable",
        "isPrimitive" => "reflectIsPrimitive",
        "sameType" => "reflectSameType",
        "getLength" => "reflectGetLength",
        "isEmpty" => "reflectIsEmpty",
        "typeDescribe" => "reflectTypeDescribe",
        "clone" => "reflectClone",
        "valueToString" => "reflectValueToString",
        "deepEquals" => "reflectDeepEquals",
        "getFunctionName" => "reflectGetFunctionName",
        "getFunctionArity" => "reflectGetFunctionArity",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve sqlite.method() → stdlib function name.
fn resolve_sqlite_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "open" => "sqlite_open",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve SqliteConnection instance method → stdlib function name.
fn resolve_sqlite_connection_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "execute" => "sqlite_execute",
        "query" => "sqlite_query",
        "close" => "sqlite_close",
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

/// Resolve sync.method() → stdlib function name (factory namespace).
/// D-049: namespace identifier is lowercase "sync".
fn resolve_sync_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "atomic" => "syncNsAtomic",
        "rwLock" => "syncNsRwLock",
        "semaphore" => "syncNsSemaphore",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve atomic instance method → stdlib function name.
fn resolve_atomic_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "get" => "atomicLoad",
        "set" => "atomicStore",
        "add" => "atomicAdd",
        "sub" => "atomicSub",
        "compareSwap" => "atomicCompareExchange",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve rwlock instance method → stdlib function name.
fn resolve_rwlock_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "read" => "rwLockRead",
        "write" => "rwLockWrite",
        "tryRead" => "rwLockTryRead",
        "tryWrite" => "rwLockTryWrite",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve semaphore instance method → stdlib function name.
fn resolve_semaphore_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "acquire" => "semaphoreAcquire",
        "tryAcquire" => "semaphoreTryAcquire",
        "release" => "semaphoreRelease",
        "available" => "semaphoreAvailable",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve future.method() → stdlib function name (factory namespace).
/// D-049: namespace identifier is lowercase "future".
fn resolve_future_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "resolve" => "futureNsResolve",
        "reject" => "futureNsReject",
        "all" => "futureNsAll",
        "race" => "futureNsRace",
        "allSettled" => "futureNsAllSettled",
        "any" => "futureNsAny",
        "never" => "futureNsNever",
        "delay" => "futureNsDelay",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Resolve Future instance method → stdlib function name (B33).
fn resolve_future_instance_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "then" => "futureNsThen",
        "catch" => "futureNsCatch",
        "finally" => "futureNsFinally",
        "await" => "futureNsAwait",
        "isResolved" => "futureNsIsResolved",
        "isPending" => "futureNsIsPending",
        "isRejected" => "futureNsIsRejected",
        _ => return None,
    };
    Some(func_name.to_string())
}

/// Returns the correct namespace name when the user typed a wrong-case variant.
/// E.g. `File` → `"file"`, `Io` → `"io"`.
/// Derives entirely from the live namespace registry — no hardcoded table.
pub fn wrong_case_namespace_hint(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    // Only hint when the casing differs (avoids hinting on already-valid names)
    if lower != name && is_static_namespace(&lower) {
        Some(lower)
    } else {
        None
    }
}

/// Resolve a test namespace method call to its stdlib function name.
fn resolve_test_ns_method(method_name: &str) -> Option<String> {
    let func_name = match method_name {
        "assert" => "testNsAssert",
        "equal" => "testNsEqual",
        "notEqual" => "testNsNotEqual",
        "throws" => "testNsThrows",
        "noThrow" => "testNsNoThrow",
        "ok" => "testNsOk",
        "err" => "testNsErr",
        "contains" => "testNsContains",
        "empty" => "testNsEmpty",
        "approx" => "testNsApprox",
        _ => return None,
    };
    Some(func_name.to_string())
}
