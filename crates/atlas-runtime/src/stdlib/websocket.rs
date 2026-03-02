//! WebSocket client for Atlas stdlib.
//!
//! Provides WebSocket connect, send, receive, and close operations.
//! Built on the tungstenite library.

use crate::security::SecurityContext;
use crate::span::Span;
use crate::value::{RuntimeError, Value, ValueArray};

use std::collections::HashMap as StdHashMap;
use std::net::TcpStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tungstenite::client::IntoClientRequest;
use tungstenite::protocol::Message;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::WebSocket;

// ── Handle management ────────────────────────────────────────────────

type WsHandle = Arc<Mutex<WebSocket<MaybeTlsStream<TcpStream>>>>;

static NEXT_WS_ID: AtomicU64 = AtomicU64::new(1);
static WS_HANDLES: std::sync::OnceLock<Mutex<StdHashMap<u64, WsHandle>>> =
    std::sync::OnceLock::new();

fn ws_handles() -> &'static Mutex<StdHashMap<u64, WsHandle>> {
    WS_HANDLES.get_or_init(|| Mutex::new(StdHashMap::new()))
}

const WS_TAG: &str = "__websocket__";

fn wrap_ws(ws: WebSocket<MaybeTlsStream<TcpStream>>) -> Value {
    let id = NEXT_WS_ID.fetch_add(1, Ordering::Relaxed);
    let handle = Arc::new(Mutex::new(ws));
    ws_handles().lock().unwrap().insert(id, handle);

    Value::Array(ValueArray::from_vec(vec![
        Value::string(WS_TAG.to_string()),
        Value::Number(id as f64),
    ]))
}

fn extract_ws(value: &Value, func_name: &str, span: Span) -> Result<WsHandle, RuntimeError> {
    let id = extract_handle_id(value, func_name, span)?;
    ws_handles()
        .lock()
        .unwrap()
        .get(&id)
        .cloned()
        .ok_or_else(|| RuntimeError::InvalidStdlibArgument {
            msg: format!("{}(): WebSocket handle has been closed", func_name),
            span,
        })
}

fn extract_handle_id(value: &Value, func_name: &str, span: Span) -> Result<u64, RuntimeError> {
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
            if tag == WS_TAG {
                Ok(id)
            } else {
                Err(RuntimeError::InvalidStdlibArgument {
                    msg: format!(
                        "{}(): expected WebSocket handle, got {} handle",
                        func_name, tag
                    ),
                    span,
                })
            }
        }
        _ => Err(super::stdlib_arg_error(func_name, "handle", value, span)),
    }
}

// ── WebSocket operations ─────────────────────────────────────────────

/// wsConnect(url: string) -> websocket handle
///
/// Connects to a WebSocket server. Supports ws:// and wss:// URLs.
pub fn ws_connect(
    args: &[Value],
    span: Span,
    security: &SecurityContext,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("wsConnect", 1, args.len(), span));
    }
    let url = extract_str(&args[0], "wsConnect", span)?;

    security
        .check_network(url)
        .map_err(|e| RuntimeError::TypeError {
            msg: format!("wsConnect: {}", e),
            span,
        })?;

    let request = url
        .into_client_request()
        .map_err(|e| RuntimeError::InvalidStdlibArgument {
            msg: format!("wsConnect(): invalid URL: {}", e),
            span,
        })?;

    let (ws, _response) = tungstenite::connect(request).map_err(|e| RuntimeError::IoError {
        message: format!("wsConnect(): connection failed: {}", e),
        span,
    })?;

    Ok(wrap_ws(ws))
}

/// wsSend(handle: websocket, message: string) -> null
pub fn ws_send(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("wsSend", 2, args.len(), span));
    }
    let ws = extract_ws(&args[0], "wsSend", span)?;
    let msg = extract_str(&args[1], "wsSend", span)?;

    let mut socket = ws.lock().unwrap();
    socket
        .send(Message::Text(msg.into()))
        .map_err(|e| RuntimeError::IoError {
            message: format!("wsSend(): {}", e),
            span,
        })?;
    Ok(Value::Null)
}

/// wsSendBinary(handle: websocket, data: array<number>) -> null
pub fn ws_send_binary(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error(
            "wsSendBinary",
            2,
            args.len(),
            span,
        ));
    }
    let ws = extract_ws(&args[0], "wsSendBinary", span)?;
    let arr = match &args[1] {
        Value::Array(a) => a.as_slice().to_vec(),
        _ => {
            return Err(super::stdlib_arg_error(
                "wsSendBinary",
                "array",
                &args[1],
                span,
            ))
        }
    };

    let bytes: Vec<u8> = arr
        .iter()
        .map(|v| match v {
            Value::Number(n) => Ok(*n as u8),
            _ => Err(RuntimeError::InvalidStdlibArgument {
                msg: "wsSendBinary(): array must contain numbers (0-255)".into(),
                span,
            }),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut socket = ws.lock().unwrap();
    socket
        .send(Message::Binary(bytes.into()))
        .map_err(|e| RuntimeError::IoError {
            message: format!("wsSendBinary(): {}", e),
            span,
        })?;
    Ok(Value::Null)
}

/// wsReceive(handle: websocket) -> { type: string, data: string|array }
pub fn ws_receive(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("wsReceive", 1, args.len(), span));
    }
    let ws = extract_ws(&args[0], "wsReceive", span)?;

    let mut socket = ws.lock().unwrap();
    let msg = socket.read().map_err(|e| RuntimeError::IoError {
        message: format!("wsReceive(): {}", e),
        span,
    })?;

    // Return as [type_string, data]
    let (msg_type, data) = match msg {
        Message::Text(text) => ("text", Value::string(text.to_string())),
        Message::Binary(data) => {
            let values: Vec<Value> = data.iter().map(|&b| Value::Number(b as f64)).collect();
            ("binary", Value::Array(ValueArray::from_vec(values)))
        }
        Message::Ping(_) => ("ping", Value::Null),
        Message::Pong(_) => ("pong", Value::Null),
        Message::Close(frame) => {
            let data = frame
                .map(|cf| Value::string(cf.reason.to_string()))
                .unwrap_or(Value::Null);
            ("close", data)
        }
        Message::Frame(_) => ("frame", Value::Null),
    };

    Ok(Value::Array(ValueArray::from_vec(vec![
        Value::string(msg_type.to_string()),
        data,
    ])))
}

/// wsPing(handle: websocket) -> null
pub fn ws_ping(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("wsPing", 1, args.len(), span));
    }
    let ws = extract_ws(&args[0], "wsPing", span)?;
    let mut socket = ws.lock().unwrap();
    socket
        .send(Message::Ping(vec![].into()))
        .map_err(|e| RuntimeError::IoError {
            message: format!("wsPing(): {}", e),
            span,
        })?;
    Ok(Value::Null)
}

/// wsClose(handle: websocket) -> null
pub fn ws_close(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("wsClose", 1, args.len(), span));
    }
    let ws = extract_ws(&args[0], "wsClose", span)?;
    let mut socket = ws.lock().unwrap();
    socket.close(None).map_err(|e| RuntimeError::IoError {
        message: format!("wsClose(): {}", e),
        span,
    })?;
    // Read remaining messages until close ack
    loop {
        match socket.read() {
            Ok(Message::Close(_)) | Err(_) => break,
            _ => continue,
        }
    }
    drop(socket);
    Ok(Value::Null)
}

// ── Helpers ──────────────────────────────────────────────────────────

fn extract_str<'a>(value: &'a Value, func_name: &str, span: Span) -> Result<&'a str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_str()),
        _ => Err(super::stdlib_arg_error(func_name, "string", value, span)),
    }
}
