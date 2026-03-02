//! Encoding/decoding functions for Atlas stdlib.
//!
//! Provides base64, hex, and URL encoding/decoding.

use crate::span::Span;
use crate::value::{RuntimeError, Value};

// ── Base64 ───────────────────────────────────────────────────────────

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::engine::general_purpose::URL_SAFE as BASE64_URL_SAFE;

/// base64Encode(data: string) -> string
pub fn base64_encode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("base64Encode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "base64Encode", span)?;
    Ok(Value::string(BASE64_STANDARD.encode(s.as_bytes())))
}

/// base64Decode(encoded: string) -> string
pub fn base64_decode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("base64Decode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "base64Decode", span)?;
    let bytes = BASE64_STANDARD.decode(s).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("base64Decode(): invalid base64: {}", e),
        span,
    })?;
    String::from_utf8(bytes).map(Value::string).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("base64Decode(): decoded data is not valid UTF-8: {}", e),
        span,
    })
}

/// base64UrlEncode(data: string) -> string (URL-safe base64)
pub fn base64_url_encode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("base64UrlEncode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "base64UrlEncode", span)?;
    Ok(Value::string(BASE64_URL_SAFE.encode(s.as_bytes())))
}

/// base64UrlDecode(encoded: string) -> string (URL-safe base64)
pub fn base64_url_decode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("base64UrlDecode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "base64UrlDecode", span)?;
    let bytes = BASE64_URL_SAFE.decode(s).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("base64UrlDecode(): invalid URL-safe base64: {}", e),
        span,
    })?;
    String::from_utf8(bytes).map(Value::string).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("base64UrlDecode(): decoded data is not valid UTF-8: {}", e),
        span,
    })
}

// ── Hex ──────────────────────────────────────────────────────────────

/// hexEncode(data: string) -> string
pub fn hex_encode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("hexEncode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "hexEncode", span)?;
    Ok(Value::string(hex::encode(s.as_bytes())))
}

/// hexDecode(hex_string: string) -> string
pub fn hex_decode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("hexDecode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "hexDecode", span)?;
    let bytes = hex::decode(s).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("hexDecode(): invalid hex: {}", e),
        span,
    })?;
    String::from_utf8(bytes).map(Value::string).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("hexDecode(): decoded data is not valid UTF-8: {}", e),
        span,
    })
}

// ── URL Encoding ─────────────────────────────────────────────────────

/// urlEncode(data: string) -> string (percent-encoded)
pub fn url_encode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("urlEncode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "urlEncode", span)?;
    Ok(Value::string(urlencoding::encode(s).into_owned()))
}

/// urlDecode(encoded: string) -> string
pub fn url_decode(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("urlDecode", 1, args.len(), span));
    }
    let s = extract_str(&args[0], "urlDecode", span)?;
    urlencoding::decode(s)
        .map(|decoded| Value::string(decoded.into_owned()))
        .map_err(|e| RuntimeError::InvalidStdlibArgument {
            msg: format!("urlDecode(): invalid percent-encoded string: {}", e),
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
