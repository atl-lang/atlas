# crypto

Cryptographic primitives for Atlas. Provides hashing (SHA-256, SHA-512, BLAKE3), HMAC authentication, and authenticated encryption (AES-256-GCM).

All functions are synchronous. Keys and digests are represented as lowercase hex strings. Inputs are `string` values; bytes are interpreted as their UTF-8 encoding.

---

## Hashing

### `sha256`

```atlas
fn sha256(data: string): string
```

Computes the SHA-256 hash of `data`. Returns a 64-character lowercase hex string.

```atlas
let digest = sha256("hello world");
// => "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576c05e4c7fa3b7b73c" (example)
console.log(digest);
```

---

### `sha512`

```atlas
fn sha512(data: string): string
```

Computes the SHA-512 hash of `data`. Returns a 128-character lowercase hex string.

```atlas
let digest = sha512("hello world");
console.log(digest);
```

---

### `blake3Hash`

```atlas
fn blake3Hash(data: string): string
```

Computes the BLAKE3 hash of `data`. Returns a 64-character lowercase hex string. BLAKE3 is significantly faster than SHA-256 and SHA-512 while providing equivalent security.

```atlas
let digest = blake3Hash("hello world");
console.log(digest);
```

---

## HMAC

HMAC functions provide message authentication — they verify that a message was produced by someone holding the secret key, and that it has not been tampered with.

### `hmacSha256`

```atlas
fn hmacSha256(data: string, key: string): string
```

Computes HMAC-SHA256 over `data` using `key`. Returns a 64-character lowercase hex string.

**Parameters:**
- `data` — the message to authenticate
- `key` — the secret key (any string; UTF-8 bytes are used)

```atlas
let sig = hmacSha256("my message", "secret-key");
console.log(sig);
```

---

### `hmacSha256Verify`

```atlas
fn hmacSha256Verify(data: string, key: string, signature: string): bool
```

Verifies an HMAC-SHA256 signature using a constant-time comparison (timing-safe).

**Parameters:**
- `data` — the original message
- `key` — the secret key
- `signature` — hex string of the expected HMAC (as returned by `hmacSha256`)

**Returns:** `true` if the signature is valid, `false` otherwise.

```atlas
let sig  = hmacSha256("payload", "my-secret");
let ok   = hmacSha256Verify("payload", "my-secret", sig);
let bad  = hmacSha256Verify("payload", "wrong-key", sig);

console.log(ok.toString());   // true
console.log(bad.toString());  // false
```

---

## Namespace Methods (`crypto.*`)

The `crypto` namespace exposes additional variants that accept an algorithm string, allowing runtime selection of the hash algorithm.

### `crypto.blake3`

```atlas
fn crypto.blake3(data: string): string
```

Alias for `blake3Hash`. Computes BLAKE3 hash of `data`, returns hex string.

```atlas
let h = crypto.blake3("data");
```

---

### `crypto.hmac`

```atlas
fn crypto.hmac(key: string, data: string, algo: string): string
```

Computes HMAC over `data` using `key` with the specified algorithm.

**Parameters:**
- `key` — secret key string
- `data` — message string
- `algo` — `"sha256"` or `"sha512"`

**Returns:** Lowercase hex HMAC digest. Throws `InvalidStdlibArgument` if `algo` is not `"sha256"` or `"sha512"`.

```atlas
let sig256 = crypto.hmac("my-key", "my-data", "sha256");
let sig512 = crypto.hmac("my-key", "my-data", "sha512");
```

---

### `crypto.hmacVerify`

```atlas
fn crypto.hmacVerify(key: string, data: string, sig: string, algo: string): bool
```

Verifies an HMAC signature with timing-safe comparison.

**Parameters:**
- `key` — secret key string
- `data` — message string
- `sig` — hex string of the expected HMAC
- `algo` — `"sha256"` or `"sha512"`

**Returns:** `true` if signature matches, `false` otherwise.

```atlas
let sig  = crypto.hmac("key", "message", "sha512");
let ok   = crypto.hmacVerify("key", "message", sig, "sha512");
console.log(ok.toString()); // true
```

---

## Authenticated Encryption (AES-256-GCM)

AES-256-GCM provides authenticated encryption — it encrypts data and produces an authentication tag that detects tampering. The nonce is generated randomly per encryption and prepended to the ciphertext output.

### `aesGcmGenerateKey`

```atlas
fn aesGcmGenerateKey(): string
```

Generates a cryptographically random 256-bit (32-byte) AES key. Returns a 64-character lowercase hex string.

```atlas
let key = aesGcmGenerateKey();
console.log(key); // 64 hex chars
```

---

### `aesGcmEncrypt`

```atlas
fn aesGcmEncrypt(plaintext: string, key: string): string
```

Encrypts `plaintext` using AES-256-GCM with a randomly generated nonce.

**Parameters:**
- `plaintext` — the string to encrypt
- `key` — 64-character hex string representing 32 bytes (use `aesGcmGenerateKey`)

**Returns:** Lowercase hex string encoding `nonce (12 bytes) || ciphertext || GCM auth tag`. The nonce is prepended automatically and required for decryption.

**Errors:** Throws `InvalidStdlibArgument` if the key is not valid hex or not exactly 32 bytes.

```atlas
let key        = aesGcmGenerateKey();
let ciphertext = aesGcmEncrypt("secret message", key);
console.log(ciphertext);
```

---

### `aesGcmDecrypt`

```atlas
fn aesGcmDecrypt(ciphertext: string, key: string): string
```

Decrypts a ciphertext produced by `aesGcmEncrypt`.

**Parameters:**
- `ciphertext` — hex string from `aesGcmEncrypt` (includes prepended nonce)
- `key` — 64-character hex string, same key used during encryption

**Returns:** The original plaintext string.

**Errors:** Throws `InvalidStdlibArgument` if:
- The key is not valid hex or not 32 bytes
- The ciphertext is too short (missing nonce; minimum 12 bytes / 24 hex chars)
- Decryption fails (wrong key, corrupted data, or authentication tag mismatch)
- The decrypted bytes are not valid UTF-8

```atlas
let key    = aesGcmGenerateKey();
let enc    = aesGcmEncrypt("hello atlas", key);
let plain  = aesGcmDecrypt(enc, key);
console.log(plain); // "hello atlas"
```

---

## Full Example

```atlas
fn secureRoundtrip(): void {
    // Key generation
    let key = aesGcmGenerateKey();

    // Encrypt
    let message    = "sensitive user data";
    let ciphertext = aesGcmEncrypt(message, key);
    console.log("Encrypted: " + ciphertext);

    // Decrypt
    let recovered = aesGcmDecrypt(ciphertext, key);
    console.log("Recovered: " + recovered);

    // HMAC sign + verify
    let tag = hmacSha256(message, "signing-key");
    let ok  = hmacSha256Verify(message, "signing-key", tag);
    console.log("Signature valid: " + ok.toString());

    // Hashing
    console.log("SHA-256:  " + sha256(message));
    console.log("SHA-512:  " + sha512(message));
    console.log("BLAKE3:   " + blake3Hash(message));
}
```

---

## Notes

- All string inputs are processed as their UTF-8 byte representation.
- Keys for HMAC may be any length; very short keys weaken security.
- AES-256-GCM nonces are 96 bits (12 bytes), generated from the OS CSPRNG (`OsRng`) at encryption time. Never reuse nonces with the same key.
- `hmacSha256Verify` and `crypto.hmacVerify` use constant-time comparison to prevent timing attacks.
