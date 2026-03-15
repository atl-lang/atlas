//! HTTP server standard library functions
//!
//! Provides `http.serve(port, handlerFn)` — an axum-based HTTP server primitive.
//! Server-side request/response types are separate from the client HttpRequest/HttpResponse.

use crate::security::SecurityContext;
use crate::span::Span;
use crate::stdlib::collections::hash::HashKey;
use crate::stdlib::OutputWriter;
use crate::value::{RuntimeError, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Incoming server-side HTTP request.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpServerRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub query: HashMap<String, String>,
}

/// Build a request map from an `HttpServerRequest`.
///
/// Returns a `Value::Map` with string keys so handlers can access fields via
/// `req.method`, `req.path`, `req.headers`, `req.body`, `req.query` directly.
/// This is Atlas-idiomatic: req is a plain data map, not an opaque object.
pub fn build_request_value(req: HttpServerRequest) -> Value {
    use crate::stdlib::collections::hashmap::AtlasHashMap;
    use crate::value::ValueHashMap;

    let mut headers_map = AtlasHashMap::new();
    for (k, v) in &req.headers {
        headers_map.insert(
            HashKey::String(Arc::new(k.clone())),
            Value::string(v.clone()),
        );
    }
    let headers_val = Value::Map(ValueHashMap::from_atlas(headers_map));

    let mut query_map = AtlasHashMap::new();
    for (k, v) in &req.query {
        query_map.insert(
            HashKey::String(Arc::new(k.clone())),
            Value::string(v.clone()),
        );
    }
    let query_val = Value::Map(ValueHashMap::from_atlas(query_map));

    let mut req_map = AtlasHashMap::new();
    req_map.insert(
        HashKey::String(Arc::new("method".to_string())),
        Value::string(req.method.clone()),
    );
    req_map.insert(
        HashKey::String(Arc::new("path".to_string())),
        Value::string(req.path.clone()),
    );
    req_map.insert(
        HashKey::String(Arc::new("body".to_string())),
        Value::string(req.body.clone()),
    );
    req_map.insert(
        HashKey::String(Arc::new("headers".to_string())),
        headers_val,
    );
    req_map.insert(HashKey::String(Arc::new("query".to_string())), query_val);
    Value::Map(ValueHashMap::from_atlas(req_map))
}

// ── Instance method helpers ─────────────────────────────────────────────────

/// `req.method` → String
pub fn http_server_request_method(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestMethod(): expected 1 argument, got {}",
                args.len()
            ),
            span,
        });
    }
    match &args[0] {
        Value::HttpServerRequest(req) => Ok(Value::string(req.method.clone())),
        other => Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestMethod(): expected HttpServerRequest, got {}",
                other.type_name()
            ),
            span,
        }),
    }
}

/// `req.path` → String
pub fn http_server_request_path(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestPath(): expected 1 argument, got {}",
                args.len()
            ),
            span,
        });
    }
    match &args[0] {
        Value::HttpServerRequest(req) => Ok(Value::string(req.path.clone())),
        other => Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestPath(): expected HttpServerRequest, got {}",
                other.type_name()
            ),
            span,
        }),
    }
}

/// `req.headers` → Map<String, String>
pub fn http_server_request_headers(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestHeaders(): expected 1 argument, got {}",
                args.len()
            ),
            span,
        });
    }
    match &args[0] {
        Value::HttpServerRequest(req) => {
            let mut atlas_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            for (k, v) in &req.headers {
                atlas_map.insert(
                    HashKey::String(Arc::new(k.clone())),
                    Value::string(v.clone()),
                );
            }
            Ok(Value::Map(crate::value::ValueHashMap::from_atlas(
                atlas_map,
            )))
        }
        other => Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestHeaders(): expected HttpServerRequest, got {}",
                other.type_name()
            ),
            span,
        }),
    }
}

/// `req.body` → String
pub fn http_server_request_body(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestBody(): expected 1 argument, got {}",
                args.len()
            ),
            span,
        });
    }
    match &args[0] {
        Value::HttpServerRequest(req) => Ok(Value::string(req.body.clone())),
        other => Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestBody(): expected HttpServerRequest, got {}",
                other.type_name()
            ),
            span,
        }),
    }
}

/// `req.query` → Map<String, String>
pub fn http_server_request_query(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestQuery(): expected 1 argument, got {}",
                args.len()
            ),
            span,
        });
    }
    match &args[0] {
        Value::HttpServerRequest(req) => {
            let mut atlas_map = crate::stdlib::collections::hashmap::AtlasHashMap::new();
            for (k, v) in &req.query {
                atlas_map.insert(
                    HashKey::String(Arc::new(k.clone())),
                    Value::string(v.clone()),
                );
            }
            Ok(Value::Map(crate::value::ValueHashMap::from_atlas(
                atlas_map,
            )))
        }
        other => Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "httpServerRequestQuery(): expected HttpServerRequest, got {}",
                other.type_name()
            ),
            span,
        }),
    }
}

// ── http.serve() ────────────────────────────────────────────────────────────

/// `http.serve(port, handlerFn)` — start an axum HTTP server.
///
/// Blocks the calling Atlas thread until the server shuts down.
/// Each incoming request is handled by cloning the template VM and calling the handler.
pub fn http_serve(
    args: &[Value],
    span: Span,
    _security: &SecurityContext,
    _output: &OutputWriter,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!(
                "http.serve(): expected 2 arguments (port, handler), got {}",
                args.len()
            ),
            span,
        });
    }

    // Extract port
    let port = match &args[0] {
        Value::Number(n) => *n as u16,
        other => {
            return Err(RuntimeError::InvalidStdlibArgument {
                msg: format!(
                    "http.serve(): first argument (port) must be a number, got {}",
                    other.type_name()
                ),
                span,
            });
        }
    };

    // Extract handler — must be Function or Closure
    let handler = match &args[1] {
        Value::Function(_) | Value::Closure(_) => args[1].clone(),
        other => {
            return Err(RuntimeError::InvalidStdlibArgument {
                msg: format!(
                    "http.serve(): second argument (handler) must be a function, got {}",
                    other.type_name()
                ),
                span,
            });
        }
    };

    // Get base VM snapshot
    let base_vm =
        crate::async_runtime::blocking_vm().ok_or_else(|| RuntimeError::InternalError {
            msg: "http.serve(): VM not initialized (blocking pool not set up)".to_string(),
            span,
        })?;

    let base_vm_arc = Arc::new(Mutex::new(base_vm));

    // Block on the axum server
    let rt = crate::async_runtime::runtime();
    rt.block_on(serve_inner(port, handler, base_vm_arc))
        .map_err(|e| RuntimeError::InternalError {
            msg: format!("http.serve(): server error: {e}"),
            span,
        })?;

    Ok(Value::Null)
}

/// Properly structured server runner that avoids double-move of handler.
#[cfg(feature = "http")]
pub(crate) async fn serve_inner(
    port: u16,
    handler: Value,
    base_vm: Arc<Mutex<crate::vm::VM>>,
) -> Result<(), String> {
    use axum::{body::Body, http::Request, Router};

    let handler = Arc::new(handler);
    let base_vm_clone = base_vm.clone();

    let app = Router::new().fallback(move |req: Request<Body>| {
        let h = handler.clone();
        let vm = base_vm_clone.clone();
        async move { handle_request(req, h, vm).await }
    });

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| format!("http.serve(): failed to bind to {addr}: {e}"))?;

    axum::serve(listener, app)
        .await
        .map_err(|e| format!("http.serve(): server failed: {e}"))
}

/// Handle a single HTTP request: clone VM, call Atlas handler, build response.
#[cfg(feature = "http")]
async fn handle_request(
    req: axum::http::Request<axum::body::Body>,
    handler: Arc<Value>,
    base_vm: Arc<Mutex<crate::vm::VM>>,
) -> axum::http::Response<axum::body::Body> {
    use axum::{
        body::Body,
        http::{Response, StatusCode},
    };

    // Extract request data before consuming the request
    let method = req.method().to_string();
    let path = req.uri().path().to_string();

    // Parse query params from URI query string
    let mut query = HashMap::new();
    if let Some(query_str) = req.uri().query() {
        for pair in query_str.split('&') {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next().unwrap_or("").to_string();
            let val = parts.next().unwrap_or("").to_string();
            if !key.is_empty() {
                query.insert(key, val);
            }
        }
    }

    // Extract headers
    let mut headers = HashMap::new();
    for (name, value) in req.headers() {
        if let Ok(v) = value.to_str() {
            headers.insert(name.to_string(), v.to_string());
        }
    }

    // Read body bytes
    let body_bytes = match axum::body::to_bytes(req.into_body(), usize::MAX).await {
        Ok(b) => b,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from(format!("failed to read request body: {e}")))
                .unwrap_or_else(|_| Response::new(Body::from("bad request")));
        }
    };
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    let server_req = HttpServerRequest {
        method,
        path,
        headers,
        body,
        query,
    };
    let request_value = build_request_value(server_req);

    // Clone VM for this request
    let vm_clone = {
        let guard = base_vm.lock().unwrap_or_else(|e| e.into_inner());
        guard.new_for_worker()
    };

    // Call Atlas handler in a blocking thread
    let handler_val = (*handler).clone();
    let result = tokio::task::spawn_blocking(move || {
        let mut vm = vm_clone;
        vm.call_value(&handler_val, vec![request_value], Span::dummy())
    })
    .await;

    match result {
        Ok(Ok(response_value)) => build_axum_response(response_value),
        Ok(Err(e)) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("handler error: {e}")))
            .unwrap_or_else(|_| Response::new(Body::from("internal server error"))),
        Err(e) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from(format!("handler panicked: {e}")))
            .unwrap_or_else(|_| Response::new(Body::from("internal server error"))),
    }
}

/// Extract status, body, headers from the Atlas handler return value.
#[cfg(feature = "http")]
fn build_axum_response(value: Value) -> axum::http::Response<axum::body::Body> {
    use axum::{
        body::Body,
        http::{HeaderName, HeaderValue, Response, StatusCode},
    };
    use std::str::FromStr;

    // Handler should return a map-like object with `status`, `body`, `headers`
    let map = match value {
        Value::Map(m) => m,
        Value::String(s) => {
            // Plain string response → 200 OK
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(s.as_ref().to_string()))
                .unwrap_or_else(|_| Response::new(Body::empty()));
        }
        Value::Null => {
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::empty())
                .unwrap_or_else(|_| Response::new(Body::empty()));
        }
        other => {
            return Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(other.to_string()))
                .unwrap_or_else(|_| Response::new(Body::empty()));
        }
    };

    // Extract `status`
    let status_code = map
        .get(&HashKey::String(Arc::new("status".to_string())))
        .and_then(|v| {
            if let Value::Number(n) = v {
                Some(*n as u16)
            } else {
                None
            }
        })
        .unwrap_or(200);

    let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    // Extract `body`
    let body_str = map
        .get(&HashKey::String(Arc::new("body".to_string())))
        .map(|v| v.to_string())
        .unwrap_or_default();

    // Build response
    let mut builder = Response::builder().status(status);

    // Extract `headers`
    if let Some(Value::Map(hmap)) = map.get(&HashKey::String(Arc::new("headers".to_string()))) {
        for (k, v) in hmap.entries() {
            if let (HashKey::String(name), Value::String(val)) = (k, v) {
                if let (Ok(hn), Ok(hv)) = (
                    HeaderName::from_str(name.as_ref()),
                    HeaderValue::from_str(val.as_ref()),
                ) {
                    builder = builder.header(hn, hv);
                }
            }
        }
    }

    builder
        .body(Body::from(body_str))
        .unwrap_or_else(|_| Response::new(Body::empty()))
}
