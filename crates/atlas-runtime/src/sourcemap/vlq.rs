//! VLQ (Variable-Length Quantity) encoding for Source Map v3.
//!
//! Encodes signed integers as base64-VLQ sequences per the source map spec.
//! Each VLQ digit encodes 5 bits of data plus a continuation bit.

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

const VLQ_BASE_SHIFT: u32 = 5;
const VLQ_BASE: i64 = 1 << VLQ_BASE_SHIFT; // 32
const VLQ_BASE_MASK: i64 = VLQ_BASE - 1; // 0x1F
const VLQ_CONTINUATION_BIT: i64 = VLQ_BASE; // 0x20

/// Encode a signed integer as a base64-VLQ string.
///
/// The sign is encoded in the least significant bit (0 = positive, 1 = negative).
pub fn encode(value: i64) -> String {
    let mut vlq = if value < 0 {
        ((-value) << 1) | 1
    } else {
        value << 1
    };

    let mut result = String::new();
    loop {
        let mut digit = vlq & VLQ_BASE_MASK;
        vlq >>= VLQ_BASE_SHIFT;
        if vlq > 0 {
            digit |= VLQ_CONTINUATION_BIT;
        }
        result.push(BASE64_CHARS[digit as usize] as char);
        if vlq == 0 {
            break;
        }
    }
    result
}

/// Encode multiple values into a single VLQ segment (concatenated).
pub fn encode_segment(values: &[i64]) -> String {
    let mut result = String::new();
    for &v in values {
        result.push_str(&encode(v));
    }
    result
}

/// Decode a single base64-VLQ value from the start of `input`.
///
/// Returns `(decoded_value, bytes_consumed)` or `None` if invalid.
pub fn decode(input: &str) -> Option<(i64, usize)> {
    let bytes = input.as_bytes();
    let mut result: i64 = 0;
    let mut shift: u32 = 0;
    let mut i = 0;

    loop {
        if i >= bytes.len() {
            return None;
        }
        let char_value = base64_decode_char(bytes[i])?;
        let digit = char_value as i64;
        i += 1;

        let value_part = digit & VLQ_BASE_MASK;
        result |= value_part << shift;

        if (digit & VLQ_CONTINUATION_BIT) == 0 {
            break;
        }
        shift += VLQ_BASE_SHIFT;
        if shift > 60 {
            return None; // overflow protection
        }
    }

    // LSB is the sign bit
    let is_negative = (result & 1) == 1;
    let magnitude = result >> 1;
    let value = if is_negative { -magnitude } else { magnitude };

    Some((value, i))
}

/// Decode all VLQ values from a segment string.
pub fn decode_segment(input: &str) -> Option<Vec<i64>> {
    let mut values = Vec::new();
    let mut pos = 0;
    while pos < input.len() {
        let (value, consumed) = decode(&input[pos..])?;
        values.push(value);
        pos += consumed;
    }
    Some(values)
}

/// Decode a single base64 character to its 6-bit value.
fn base64_decode_char(c: u8) -> Option<u8> {
    match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}
