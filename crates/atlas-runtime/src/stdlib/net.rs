//! Networking primitives for Atlas stdlib.
//!
//! Provides TCP (client + server), UDP, and TLS support.
//! All operations respect the SecurityContext network permission model.

use crate::security::SecurityContext;
use crate::span::Span;
use crate::value::{RuntimeError, Value, ValueArray};

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::time::Duration;

// ── Security helper ──────────────────────────────────────────────────

fn check_net_permission(security: &SecurityContext, host: &str, span: Span) -> Result<(), RuntimeError> {
    security.check_network(host).map_err(|e| RuntimeError::TypeError {
        msg: format!("network permission denied: {}", e),
        span,
    })
}

// ── TCP Client ───────────────────────────────────────────────────────

/// tcpConnect(address: string, timeout_ms?: number) -> TcpStream handle (as opaque Value)
///
/// Returns a HashMap with a `_handle` key containing the stream.
pub fn tcp_connect(args: &[Value], span: Span, security: &SecurityContext) -> Result<Value, RuntimeError> {
    if args.is_empty() || args.len() > 2 {
        return Err(super::stdlib_arity_error("tcpConnect", 1, args.len(), span));
    }
    let addr_str = extract_str(&args[0], "tcpConnect", span)?;
    check_net_permission(security, addr_str, span)?;

    let timeout_ms = if args.len() == 2 {
        Some(extract_number(&args[1], "tcpConnect", span)? as u64)
    } else {
        None
    };

    let addr: SocketAddr = resolve_addr(addr_str, "tcpConnect", span)?;

    let stream = if let Some(ms) = timeout_ms {
        TcpStream::connect_timeout(&addr, Duration::from_millis(ms))
    } else {
        TcpStream::connect(addr)
    }
    .map_err(|e| RuntimeError::IoError {
        message: format!("tcpConnect(): {}", e),
        span,
    })?;

    Ok(wrap_tcp_stream(stream))
}

/// tcpWrite(stream: handle, data: string) -> number (bytes written)
pub fn tcp_write(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("tcpWrite", 2, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpWrite", span)?;
    let data = extract_str(&args[1], "tcpWrite", span)?;

    let mut stream = stream_mutex.lock().unwrap();
    let n = stream.write(data.as_bytes()).map_err(|e| RuntimeError::IoError {
        message: format!("tcpWrite(): {}", e),
        span,
    })?;
    stream.flush().map_err(|e| RuntimeError::IoError {
        message: format!("tcpWrite(): flush failed: {}", e),
        span,
    })?;
    Ok(Value::Number(n as f64))
}

/// tcpRead(stream: handle, max_bytes: number) -> string
pub fn tcp_read(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("tcpRead", 2, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpRead", span)?;
    let max_bytes = extract_number(&args[1], "tcpRead", span)? as usize;

    let mut buf = vec![0u8; max_bytes.min(1024 * 1024)]; // Cap at 1MB per read
    let mut stream = stream_mutex.lock().unwrap();
    let n = stream.read(&mut buf).map_err(|e| RuntimeError::IoError {
        message: format!("tcpRead(): {}", e),
        span,
    })?;
    buf.truncate(n);

    String::from_utf8(buf).map(Value::string).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("tcpRead(): received non-UTF-8 data: {}", e),
        span,
    })
}

/// tcpReadBytes(stream: handle, max_bytes: number) -> array of numbers (raw bytes)
pub fn tcp_read_bytes(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("tcpReadBytes", 2, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpReadBytes", span)?;
    let max_bytes = extract_number(&args[1], "tcpReadBytes", span)? as usize;

    let mut buf = vec![0u8; max_bytes.min(1024 * 1024)];
    let mut stream = stream_mutex.lock().unwrap();
    let n = stream.read(&mut buf).map_err(|e| RuntimeError::IoError {
        message: format!("tcpReadBytes(): {}", e),
        span,
    })?;

    let values: Vec<Value> = buf[..n].iter().map(|&b| Value::Number(b as f64)).collect();
    Ok(Value::Array(ValueArray::from_vec(values)))
}

/// tcpClose(stream: handle) -> null
pub fn tcp_close(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tcpClose", 1, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpClose", span)?;
    let stream = stream_mutex.lock().unwrap();
    stream.shutdown(std::net::Shutdown::Both).map_err(|e| RuntimeError::IoError {
        message: format!("tcpClose(): {}", e),
        span,
    })?;
    Ok(Value::Null)
}

/// tcpSetTimeout(stream: handle, read_ms: number, write_ms: number) -> null
pub fn tcp_set_timeout(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(super::stdlib_arity_error("tcpSetTimeout", 3, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpSetTimeout", span)?;
    let read_ms = extract_number(&args[1], "tcpSetTimeout", span)? as u64;
    let write_ms = extract_number(&args[2], "tcpSetTimeout", span)? as u64;

    let stream = stream_mutex.lock().unwrap();
    stream.set_read_timeout(Some(Duration::from_millis(read_ms))).map_err(|e| RuntimeError::IoError {
        message: format!("tcpSetTimeout(): {}", e),
        span,
    })?;
    stream.set_write_timeout(Some(Duration::from_millis(write_ms))).map_err(|e| RuntimeError::IoError {
        message: format!("tcpSetTimeout(): {}", e),
        span,
    })?;
    Ok(Value::Null)
}

/// tcpSetNodelay(stream: handle, nodelay: bool) -> null
pub fn tcp_set_nodelay(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("tcpSetNodelay", 2, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpSetNodelay", span)?;
    let nodelay = extract_bool(&args[1], "tcpSetNodelay", span)?;

    let stream = stream_mutex.lock().unwrap();
    stream.set_nodelay(nodelay).map_err(|e| RuntimeError::IoError {
        message: format!("tcpSetNodelay(): {}", e),
        span,
    })?;
    Ok(Value::Null)
}

/// tcpLocalAddr(stream: handle) -> string
pub fn tcp_local_addr(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tcpLocalAddr", 1, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpLocalAddr", span)?;
    let stream = stream_mutex.lock().unwrap();
    let addr = stream.local_addr().map_err(|e| RuntimeError::IoError {
        message: format!("tcpLocalAddr(): {}", e),
        span,
    })?;
    Ok(Value::string(addr.to_string()))
}

/// tcpRemoteAddr(stream: handle) -> string
pub fn tcp_remote_addr(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tcpRemoteAddr", 1, args.len(), span));
    }
    let stream_mutex = extract_tcp_stream(&args[0], "tcpRemoteAddr", span)?;
    let stream = stream_mutex.lock().unwrap();
    let addr = stream.peer_addr().map_err(|e| RuntimeError::IoError {
        message: format!("tcpRemoteAddr(): {}", e),
        span,
    })?;
    Ok(Value::string(addr.to_string()))
}

// ── TCP Server ───────────────────────────────────────────────────────

/// tcpListen(address: string) -> listener handle
pub fn tcp_listen(args: &[Value], span: Span, security: &SecurityContext) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tcpListen", 1, args.len(), span));
    }
    let addr_str = extract_str(&args[0], "tcpListen", span)?;
    check_net_permission(security, addr_str, span)?;

    let listener = TcpListener::bind(addr_str).map_err(|e| RuntimeError::IoError {
        message: format!("tcpListen(): {}", e),
        span,
    })?;
    Ok(wrap_tcp_listener(listener))
}

/// tcpAccept(listener: handle) -> stream handle
pub fn tcp_accept(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tcpAccept", 1, args.len(), span));
    }
    let listener_mutex = extract_tcp_listener(&args[0], "tcpAccept", span)?;
    let listener = listener_mutex.lock().unwrap();
    let (stream, _addr) = listener.accept().map_err(|e| RuntimeError::IoError {
        message: format!("tcpAccept(): {}", e),
        span,
    })?;
    Ok(wrap_tcp_stream(stream))
}

/// tcpListenerAddr(listener: handle) -> string
pub fn tcp_listener_addr(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tcpListenerAddr", 1, args.len(), span));
    }
    let listener_mutex = extract_tcp_listener(&args[0], "tcpListenerAddr", span)?;
    let listener = listener_mutex.lock().unwrap();
    let addr = listener.local_addr().map_err(|e| RuntimeError::IoError {
        message: format!("tcpListenerAddr(): {}", e),
        span,
    })?;
    Ok(Value::string(addr.to_string()))
}

/// tcpListenerClose(listener: handle) -> null
pub fn tcp_listener_close(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tcpListenerClose", 1, args.len(), span));
    }
    // Drop the listener by extracting and dropping the inner value
    let _ = extract_tcp_listener(&args[0], "tcpListenerClose", span)?;
    Ok(Value::Null)
}

// ── UDP ──────────────────────────────────────────────────────────────

/// udpBind(address: string) -> socket handle
pub fn udp_bind(args: &[Value], span: Span, security: &SecurityContext) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("udpBind", 1, args.len(), span));
    }
    let addr_str = extract_str(&args[0], "udpBind", span)?;
    check_net_permission(security, addr_str, span)?;

    let socket = UdpSocket::bind(addr_str).map_err(|e| RuntimeError::IoError {
        message: format!("udpBind(): {}", e),
        span,
    })?;
    Ok(wrap_udp_socket(socket))
}

/// udpSend(socket: handle, data: string, target: string) -> number (bytes sent)
pub fn udp_send(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(super::stdlib_arity_error("udpSend", 3, args.len(), span));
    }
    let socket_mutex = extract_udp_socket(&args[0], "udpSend", span)?;
    let data = extract_str(&args[1], "udpSend", span)?;
    let target = extract_str(&args[2], "udpSend", span)?;

    let socket = socket_mutex.lock().unwrap();
    let n = socket.send_to(data.as_bytes(), target).map_err(|e| RuntimeError::IoError {
        message: format!("udpSend(): {}", e),
        span,
    })?;
    Ok(Value::Number(n as f64))
}

/// udpReceive(socket: handle, max_bytes: number) -> { data: string, sender: string }
pub fn udp_receive(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("udpReceive", 2, args.len(), span));
    }
    let socket_mutex = extract_udp_socket(&args[0], "udpReceive", span)?;
    let max_bytes = extract_number(&args[1], "udpReceive", span)? as usize;

    let mut buf = vec![0u8; max_bytes.min(65536)]; // UDP max is 65535
    let socket = socket_mutex.lock().unwrap();
    let (n, sender) = socket.recv_from(&mut buf).map_err(|e| RuntimeError::IoError {
        message: format!("udpReceive(): {}", e),
        span,
    })?;

    let data = String::from_utf8_lossy(&buf[..n]).into_owned();

    // Return as [data, sender_address]
    Ok(Value::Array(ValueArray::from_vec(vec![
        Value::string(data),
        Value::string(sender.to_string()),
    ])))
}

/// udpSetTimeout(socket: handle, read_ms: number) -> null
pub fn udp_set_timeout(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("udpSetTimeout", 2, args.len(), span));
    }
    let socket_mutex = extract_udp_socket(&args[0], "udpSetTimeout", span)?;
    let ms = extract_number(&args[1], "udpSetTimeout", span)? as u64;

    let socket = socket_mutex.lock().unwrap();
    socket.set_read_timeout(Some(Duration::from_millis(ms))).map_err(|e| RuntimeError::IoError {
        message: format!("udpSetTimeout(): {}", e),
        span,
    })?;
    Ok(Value::Null)
}

/// udpClose(socket: handle) -> null
pub fn udp_close(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("udpClose", 1, args.len(), span));
    }
    let _ = extract_udp_socket(&args[0], "udpClose", span)?;
    Ok(Value::Null)
}

/// udpLocalAddr(socket: handle) -> string
pub fn udp_local_addr(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("udpLocalAddr", 1, args.len(), span));
    }
    let socket_mutex = extract_udp_socket(&args[0], "udpLocalAddr", span)?;
    let socket = socket_mutex.lock().unwrap();
    let addr = socket.local_addr().map_err(|e| RuntimeError::IoError {
        message: format!("udpLocalAddr(): {}", e),
        span,
    })?;
    Ok(Value::string(addr.to_string()))
}

// ── TLS Client ───────────────────────────────────────────────────────

/// tlsConnect(host: string, port: number) -> TLS stream handle
pub fn tls_connect(args: &[Value], span: Span, security: &SecurityContext) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("tlsConnect", 2, args.len(), span));
    }
    let host = extract_str(&args[0], "tlsConnect", span)?;
    let port = extract_number(&args[1], "tlsConnect", span)? as u16;
    check_net_permission(security, host, span)?;

    let addr_str = format!("{}:{}", host, port);
    let tcp_stream = TcpStream::connect(&addr_str).map_err(|e| RuntimeError::IoError {
        message: format!("tlsConnect(): TCP connection failed: {}", e),
        span,
    })?;

    let root_store = rustls::RootCertStore::from_iter(
        webpki_roots::TLS_SERVER_ROOTS.iter().cloned(),
    );

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let server_name = rustls::pki_types::ServerName::try_from(host.to_string())
        .map_err(|e| RuntimeError::InvalidStdlibArgument {
            msg: format!("tlsConnect(): invalid hostname: {}", e),
            span,
        })?;

    let conn = rustls::ClientConnection::new(Arc::new(config), server_name)
        .map_err(|e| RuntimeError::IoError {
            message: format!("tlsConnect(): TLS handshake failed: {}", e),
            span,
        })?;

    let mut tls_stream = rustls::StreamOwned::new(conn, tcp_stream);
    // Force handshake completion
    tls_stream.flush().map_err(|e| RuntimeError::IoError {
        message: format!("tlsConnect(): TLS handshake failed: {}", e),
        span,
    })?;

    Ok(wrap_tls_stream(tls_stream))
}

/// tlsWrite(stream: handle, data: string) -> number (bytes written)
pub fn tls_write(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("tlsWrite", 2, args.len(), span));
    }
    let stream_mutex = extract_tls_stream(&args[0], "tlsWrite", span)?;
    let data = extract_str(&args[1], "tlsWrite", span)?;

    let mut stream = stream_mutex.lock().unwrap();
    let n = stream.write(data.as_bytes()).map_err(|e| RuntimeError::IoError {
        message: format!("tlsWrite(): {}", e),
        span,
    })?;
    stream.flush().map_err(|e| RuntimeError::IoError {
        message: format!("tlsWrite(): flush failed: {}", e),
        span,
    })?;
    Ok(Value::Number(n as f64))
}

/// tlsRead(stream: handle, max_bytes: number) -> string
pub fn tls_read(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("tlsRead", 2, args.len(), span));
    }
    let stream_mutex = extract_tls_stream(&args[0], "tlsRead", span)?;
    let max_bytes = extract_number(&args[1], "tlsRead", span)? as usize;

    let mut buf = vec![0u8; max_bytes.min(1024 * 1024)];
    let mut stream = stream_mutex.lock().unwrap();
    let n = stream.read(&mut buf).map_err(|e| RuntimeError::IoError {
        message: format!("tlsRead(): {}", e),
        span,
    })?;
    buf.truncate(n);

    String::from_utf8(buf).map(Value::string).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("tlsRead(): received non-UTF-8 data: {}", e),
        span,
    })
}

/// tlsClose(stream: handle) -> null
pub fn tls_close(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("tlsClose", 1, args.len(), span));
    }
    let stream_mutex = extract_tls_stream(&args[0], "tlsClose", span)?;
    let mut stream = stream_mutex.lock().unwrap();
    stream.conn.send_close_notify();
    let _ = stream.flush();
    Ok(Value::Null)
}

// ── Handle types ─────────────────────────────────────────────────────
//
// Network handles are stored as SharedValue(Arc<Mutex<Value>>), where
// the inner Value is a NativeFunction with a closure that holds the
// actual resource. This is a pattern: opaque handles stored in Atlas
// values without adding new Value variants.
//
// We use a marker string + Arc<Mutex<Resource>> to tag and extract handles.

const TCP_STREAM_TAG: &str = "__tcp_stream__";
const TCP_LISTENER_TAG: &str = "__tcp_listener__";
const UDP_SOCKET_TAG: &str = "__udp_socket__";
const TLS_STREAM_TAG: &str = "__tls_stream__";

type TcpStreamHandle = Arc<Mutex<TcpStream>>;
type TcpListenerHandle = Arc<Mutex<TcpListener>>;
type UdpSocketHandle = Arc<Mutex<UdpSocket>>;
type TlsStreamHandle = Arc<Mutex<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>>;

// Thread-local storage for network handles, keyed by ID
use std::collections::HashMap as StdHashMap;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

// Global handle stores (must be thread-safe for async contexts)
static TCP_STREAMS: std::sync::OnceLock<Mutex<StdHashMap<u64, TcpStreamHandle>>> = std::sync::OnceLock::new();
static TCP_LISTENERS: std::sync::OnceLock<Mutex<StdHashMap<u64, TcpListenerHandle>>> = std::sync::OnceLock::new();
static UDP_SOCKETS: std::sync::OnceLock<Mutex<StdHashMap<u64, UdpSocketHandle>>> = std::sync::OnceLock::new();
static TLS_STREAMS: std::sync::OnceLock<Mutex<StdHashMap<u64, TlsStreamHandle>>> = std::sync::OnceLock::new();

fn tcp_streams() -> &'static Mutex<StdHashMap<u64, TcpStreamHandle>> {
    TCP_STREAMS.get_or_init(|| Mutex::new(StdHashMap::new()))
}
fn tcp_listeners() -> &'static Mutex<StdHashMap<u64, TcpListenerHandle>> {
    TCP_LISTENERS.get_or_init(|| Mutex::new(StdHashMap::new()))
}
fn udp_sockets() -> &'static Mutex<StdHashMap<u64, UdpSocketHandle>> {
    UDP_SOCKETS.get_or_init(|| Mutex::new(StdHashMap::new()))
}
fn tls_streams() -> &'static Mutex<StdHashMap<u64, TlsStreamHandle>> {
    TLS_STREAMS.get_or_init(|| Mutex::new(StdHashMap::new()))
}

/// Create a handle value: [tag_string, id_number]
fn make_handle(tag: &str, id: u64) -> Value {
    Value::Array(ValueArray::from_vec(vec![
        Value::string(tag.to_string()),
        Value::Number(id as f64),
    ]))
}

fn wrap_tcp_stream(stream: TcpStream) -> Value {
    let id = NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed);
    let handle = Arc::new(Mutex::new(stream));
    tcp_streams().lock().unwrap().insert(id, handle);
    make_handle(TCP_STREAM_TAG, id)
}

fn wrap_tcp_listener(listener: TcpListener) -> Value {
    let id = NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed);
    let handle = Arc::new(Mutex::new(listener));
    tcp_listeners().lock().unwrap().insert(id, handle);
    make_handle(TCP_LISTENER_TAG, id)
}

fn wrap_udp_socket(socket: UdpSocket) -> Value {
    let id = NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed);
    let handle = Arc::new(Mutex::new(socket));
    udp_sockets().lock().unwrap().insert(id, handle);
    make_handle(UDP_SOCKET_TAG, id)
}

fn wrap_tls_stream(stream: rustls::StreamOwned<rustls::ClientConnection, TcpStream>) -> Value {
    let id = NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed);
    let handle = Arc::new(Mutex::new(stream));
    tls_streams().lock().unwrap().insert(id, handle);
    make_handle(TLS_STREAM_TAG, id)
}

fn extract_handle_id(value: &Value, expected_tag: &str, func_name: &str, span: Span) -> Result<u64, RuntimeError> {
    match value {
        Value::Array(arr) if arr.len() == 2 => {
            let tag = match &arr.as_slice()[0] {
                Value::String(s) => s.as_str(),
                _ => return Err(super::stdlib_arg_error(func_name, "handle", value, span)),
            };
            let id = match &arr.as_slice()[1] {
                Value::Number(n) => *n as u64,
                _ => return Err(super::stdlib_arg_error(func_name, "handle", value, span)),
            };
            if tag == expected_tag {
                Ok(id)
            } else {
                Err(RuntimeError::InvalidStdlibArgument {
                    msg: format!("{}(): expected {} handle, got {} handle", func_name, expected_tag, tag),
                    span,
                })
            }
        }
        _ => Err(super::stdlib_arg_error(func_name, "handle", value, span)),
    }
}

fn extract_tcp_stream(value: &Value, func_name: &str, span: Span) -> Result<TcpStreamHandle, RuntimeError> {
    let id = extract_handle_id(value, TCP_STREAM_TAG, func_name, span)?;
    tcp_streams().lock().unwrap().get(&id).cloned().ok_or_else(|| RuntimeError::InvalidStdlibArgument {
        msg: format!("{}(): TCP stream handle has been closed", func_name),
        span,
    })
}

fn extract_tcp_listener(value: &Value, func_name: &str, span: Span) -> Result<TcpListenerHandle, RuntimeError> {
    let id = extract_handle_id(value, TCP_LISTENER_TAG, func_name, span)?;
    tcp_listeners().lock().unwrap().get(&id).cloned().ok_or_else(|| RuntimeError::InvalidStdlibArgument {
        msg: format!("{}(): TCP listener handle has been closed", func_name),
        span,
    })
}

fn extract_udp_socket(value: &Value, func_name: &str, span: Span) -> Result<UdpSocketHandle, RuntimeError> {
    let id = extract_handle_id(value, UDP_SOCKET_TAG, func_name, span)?;
    udp_sockets().lock().unwrap().get(&id).cloned().ok_or_else(|| RuntimeError::InvalidStdlibArgument {
        msg: format!("{}(): UDP socket handle has been closed", func_name),
        span,
    })
}

fn extract_tls_stream(value: &Value, func_name: &str, span: Span) -> Result<TlsStreamHandle, RuntimeError> {
    let id = extract_handle_id(value, TLS_STREAM_TAG, func_name, span)?;
    tls_streams().lock().unwrap().get(&id).cloned().ok_or_else(|| RuntimeError::InvalidStdlibArgument {
        msg: format!("{}(): TLS stream handle has been closed", func_name),
        span,
    })
}

// ── Helpers ──────────────────────────────────────────────────────────

fn extract_str<'a>(value: &'a Value, func_name: &str, span: Span) -> Result<&'a str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_str()),
        _ => Err(super::stdlib_arg_error(func_name, "string", value, span)),
    }
}

fn extract_number(value: &Value, func_name: &str, span: Span) -> Result<f64, RuntimeError> {
    match value {
        Value::Number(n) => Ok(*n),
        _ => Err(super::stdlib_arg_error(func_name, "number", value, span)),
    }
}

fn extract_bool(value: &Value, func_name: &str, span: Span) -> Result<bool, RuntimeError> {
    match value {
        Value::Bool(b) => Ok(*b),
        _ => Err(super::stdlib_arg_error(func_name, "bool", value, span)),
    }
}

fn resolve_addr(addr_str: &str, func_name: &str, span: Span) -> Result<SocketAddr, RuntimeError> {
    addr_str.to_socket_addrs()
        .map_err(|e| RuntimeError::InvalidStdlibArgument {
            msg: format!("{}(): invalid address '{}': {}", func_name, addr_str, e),
            span,
        })?
        .next()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: format!("{}(): could not resolve address '{}'", func_name, addr_str),
            span,
        })
}
