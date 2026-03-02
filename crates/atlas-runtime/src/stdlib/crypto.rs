//! Cryptographic functions for Atlas stdlib.
//!
//! Provides hashing (SHA-256, SHA-512, BLAKE3), HMAC, and
//! authenticated encryption (AES-256-GCM).

use crate::span::Span;
use crate::value::{RuntimeError, Value};

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, AeadCore};
use blake3;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

// ── Hashing ──────────────────────────────────────────────────────────

/// sha256(data: string) -> string (hex)
pub fn sha256(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("sha256", 1, args.len(), span));
    }
    let data = extract_string_or_bytes(&args[0], "sha256", span)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(Value::string(hex::encode(result)))
}

/// sha512(data: string) -> string (hex)
pub fn sha512(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("sha512", 1, args.len(), span));
    }
    let data = extract_string_or_bytes(&args[0], "sha512", span)?;
    let mut hasher = Sha512::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(Value::string(hex::encode(result)))
}

/// blake3Hash(data: string) -> string (hex)
pub fn blake3_hash(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(super::stdlib_arity_error("blake3Hash", 1, args.len(), span));
    }
    let data = extract_string_or_bytes(&args[0], "blake3Hash", span)?;
    let hash = blake3::hash(&data);
    Ok(Value::string(hash.to_hex().to_string()))
}

// ── HMAC ─────────────────────────────────────────────────────────────

type HmacSha256 = Hmac<Sha256>;

/// hmacSha256(data: string, key: string) -> string (hex)
pub fn hmac_sha256(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("hmacSha256", 2, args.len(), span));
    }
    let data = extract_string_or_bytes(&args[0], "hmacSha256", span)?;
    let key = extract_string_or_bytes(&args[1], "hmacSha256", span)?;

    let mut mac = <HmacSha256 as Mac>::new_from_slice(&key).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("hmacSha256(): invalid key: {}", e),
        span,
    })?;
    mac.update(&data);
    let result = mac.finalize();
    Ok(Value::string(hex::encode(result.into_bytes())))
}

/// hmacSha256Verify(data: string, key: string, signature: string) -> bool
pub fn hmac_sha256_verify(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 3 {
        return Err(super::stdlib_arity_error("hmacSha256Verify", 3, args.len(), span));
    }
    let data = extract_string_or_bytes(&args[0], "hmacSha256Verify", span)?;
    let key = extract_string_or_bytes(&args[1], "hmacSha256Verify", span)?;
    let sig_hex = extract_str(&args[2], "hmacSha256Verify", span)?;

    let sig_bytes = hex::decode(sig_hex).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("hmacSha256Verify(): invalid hex signature: {}", e),
        span,
    })?;

    let mut mac = <HmacSha256 as Mac>::new_from_slice(&key).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("hmacSha256Verify(): invalid key: {}", e),
        span,
    })?;
    mac.update(&data);
    Ok(Value::Bool(mac.verify_slice(&sig_bytes).is_ok()))
}

// ── Authenticated Encryption (AES-256-GCM) ───────────────────────────

/// aesGcmEncrypt(plaintext: string, key: string(hex, 32 bytes)) -> string (hex: nonce + ciphertext)
pub fn aes_gcm_encrypt(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("aesGcmEncrypt", 2, args.len(), span));
    }
    let plaintext = extract_string_or_bytes(&args[0], "aesGcmEncrypt", span)?;
    let key_hex = extract_str(&args[1], "aesGcmEncrypt", span)?;

    let key_bytes = hex::decode(key_hex).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmEncrypt(): invalid hex key: {}", e),
        span,
    })?;
    if key_bytes.len() != 32 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!("aesGcmEncrypt(): key must be 32 bytes (64 hex chars), got {} bytes", key_bytes.len()),
            span,
        });
    }

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmEncrypt(): {}", e),
        span,
    })?;

    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmEncrypt(): encryption failed: {}", e),
        span,
    })?;

    // Output: hex(nonce || ciphertext)
    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(Value::string(hex::encode(combined)))
}

/// aesGcmDecrypt(ciphertext_hex: string, key: string(hex, 32 bytes)) -> string
pub fn aes_gcm_decrypt(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(super::stdlib_arity_error("aesGcmDecrypt", 2, args.len(), span));
    }
    let combined_hex = extract_str(&args[0], "aesGcmDecrypt", span)?;
    let key_hex = extract_str(&args[1], "aesGcmDecrypt", span)?;

    let combined = hex::decode(combined_hex).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmDecrypt(): invalid hex ciphertext: {}", e),
        span,
    })?;
    let key_bytes = hex::decode(key_hex).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmDecrypt(): invalid hex key: {}", e),
        span,
    })?;
    if key_bytes.len() != 32 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: format!("aesGcmDecrypt(): key must be 32 bytes (64 hex chars), got {} bytes", key_bytes.len()),
            span,
        });
    }
    if combined.len() < 12 {
        return Err(RuntimeError::InvalidStdlibArgument {
            msg: "aesGcmDecrypt(): ciphertext too short (missing nonce)".into(),
            span,
        });
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);

    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmDecrypt(): {}", e),
        span,
    })?;

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmDecrypt(): decryption failed (wrong key or corrupted data): {}", e),
        span,
    })?;

    String::from_utf8(plaintext).map(Value::string).map_err(|e| RuntimeError::InvalidStdlibArgument {
        msg: format!("aesGcmDecrypt(): decrypted data is not valid UTF-8: {}", e),
        span,
    })
}

/// aesGcmGenerateKey() -> string (hex, 32 random bytes)
pub fn aes_gcm_generate_key(args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(super::stdlib_arity_error("aesGcmGenerateKey", 0, args.len(), span));
    }
    let key = Aes256Gcm::generate_key(&mut OsRng);
    Ok(Value::string(hex::encode(key)))
}

// ── Helpers ──────────────────────────────────────────────────────────

fn extract_str<'a>(value: &'a Value, func_name: &str, span: Span) -> Result<&'a str, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_str()),
        _ => Err(super::stdlib_arg_error(func_name, "string", value, span)),
    }
}

fn extract_string_or_bytes(value: &Value, func_name: &str, span: Span) -> Result<Vec<u8>, RuntimeError> {
    match value {
        Value::String(s) => Ok(s.as_bytes().to_vec()),
        _ => Err(super::stdlib_arg_error(func_name, "string", value, span)),
    }
}
