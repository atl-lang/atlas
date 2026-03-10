# Encoding and Crypto Functions

Encoding/decoding and cryptographic operations.

## Base64 Encoding

### base64Encode

```atlas
fn base64Encode(data: string) : string
```

Encodes string to Base64.

**Parameters:**
- `data` - String to encode

**Returns:** `string` - Base64 encoded

### base64Decode

```atlas
fn base64Decode(encoded: string) : Result<string, string>
```

Decodes Base64 string.

**Parameters:**
- `encoded` - Base64 encoded string

**Returns:**
- `Ok(string)` - Decoded string
- `Err(string)` if invalid Base64

### base64UrlEncode

```atlas
fn base64UrlEncode(data: string) : string
```

Encodes string to URL-safe Base64 (RFC 4648).

**Parameters:**
- `data` - String to encode

**Returns:** `string` - URL-safe Base64 encoded

### base64UrlDecode

```atlas
fn base64UrlDecode(encoded: string) : Result<string, string>
```

Decodes URL-safe Base64 string.

**Parameters:**
- `encoded` - URL-safe Base64 encoded string

**Returns:**
- `Ok(string)` - Decoded string
- `Err(string)` if invalid

## Hexadecimal Encoding

### hexEncode

```atlas
fn hexEncode(data: string) : string
```

Encodes string to hexadecimal.

**Parameters:**
- `data` - String to encode

**Returns:** `string` - Hex encoded (lowercase)

### hexDecode

```atlas
fn hexDecode(hex: string) : Result<string, string>
```

Decodes hexadecimal string.

**Parameters:**
- `hex` - Hex encoded string

**Returns:**
- `Ok(string)` - Decoded string
- `Err(string)` if invalid hex

## URL Encoding

### urlEncode

```atlas
fn urlEncode(str: string) : string
```

URL-encodes string (percent encoding).

**Parameters:**
- `str` - String to encode

**Returns:** `string` - URL encoded

### urlDecode

```atlas
fn urlDecode(encoded: string) : Result<string, string>
```

URL-decodes string.

**Parameters:**
- `encoded` - URL encoded string

**Returns:**
- `Ok(string)` - Decoded string
- `Err(string)` if invalid encoding

## Hashing Functions

### sha256

```atlas
fn sha256(data: string) : string
```

Computes SHA-256 hash.

**Parameters:**
- `data` - String to hash

**Returns:** `string` - Hex encoded hash

### sha512

```atlas
fn sha512(data: string) : string
```

Computes SHA-512 hash.

**Parameters:**
- `data` - String to hash

**Returns:** `string` - Hex encoded hash

### blake3Hash

```atlas
fn blake3Hash(data: string) : string
```

Computes BLAKE3 hash.

**Parameters:**
- `data` - String to hash

**Returns:** `string` - Hex encoded hash

## HMAC Functions

### hmacSha256

```atlas
fn hmacSha256(data: string, key: string) : string
```

Computes HMAC-SHA256.

**Parameters:**
- `data` - Data to authenticate
- `key` - Secret key

**Returns:** `string` - Hex encoded HMAC

### hmacSha256Verify

```atlas
fn hmacSha256Verify(data: string, key: string, signature: string) : Result<bool, string>
```

Verifies HMAC-SHA256 signature.

**Parameters:**
- `data` - Original data
- `key` - Secret key
- `signature` - Hex encoded HMAC to verify

**Returns:**
- `Ok(bool)` - True if valid
- `Err(string)` on error

## Symmetric Encryption

### aesGcmGenerateKey

```atlas
fn aesGcmGenerateKey() : Result<string, string>
```

Generates random AES-GCM encryption key.

**Returns:**
- `Ok(string)` - Hex encoded 256-bit key
- `Err(string)` on error

### aesGcmEncrypt

```atlas
fn aesGcmEncrypt(plaintext: string, key: string) : Result<string, string>
```

Encrypts with AES-256-GCM.

**Parameters:**
- `plaintext` - Data to encrypt
- `key` - Hex encoded 256-bit key

**Returns:**
- `Ok(string)` - Hex encoded nonce + ciphertext with auth tag
- `Err(string)` on error

**Note:** Nonce is generated internally and prefixed to the ciphertext.

### aesGcmDecrypt

```atlas
fn aesGcmDecrypt(ciphertext: string, key: string) : Result<string, string>
```

Decrypts AES-256-GCM encrypted data.

**Parameters:**
- `ciphertext` - Hex encoded nonce + ciphertext with auth tag
- `key` - Hex encoded 256-bit key (must match encryption key)

**Returns:**
- `Ok(string)` - Decrypted plaintext
- `Err(string)` if decryption fails or auth tag invalid

## Example Usage

```atlas
// Base64
let encoded = base64Encode("Hello");
print(encoded); // "SGVsbG8="
let decoded = base64Decode(encoded)?;
print(decoded); // "Hello"

// Hex
let hex = hexEncode("Hello");
print(hex); // "48656c6c6f"

// Hashing
let hash = sha256("password");
print(hash);

// HMAC
let hmac = hmacSha256("message", "secret_key");
let valid = hmacSha256Verify("message", "secret_key", hmac)?;
print(valid); // true

// URL encoding
let encoded = urlEncode("hello world");
print(encoded); // "hello%20world"
```
