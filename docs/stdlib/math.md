# Math

The `Math` namespace provides mathematical functions and constants. All functions follow IEEE 754 semantics: NaN propagates, infinities are handled correctly, signed zero is preserved, and domain errors return `Result::Err` rather than panicking.

Access via `Math.functionName(...)` syntax (D-049).

---

## Constants

Constants are accessed as zero-argument calls or directly via the namespace.

| Name | Value | Description |
|------|-------|-------------|
| `Math.PI` | ~3.14159265358979 | Ratio of circle's circumference to diameter |
| `Math.E` | ~2.71828182845904 | Euler's number, base of natural logarithm |
| `Math.SQRT2` | ~1.41421356237310 | Square root of 2 |
| `Math.LN2` | ~0.69314718055994 | Natural logarithm of 2 |
| `Math.LN10` | ~2.30258509299404 | Natural logarithm of 10 |

```atlas
let circumference = 2 * Math.PI() * radius;
let base = Math.E();
```

---

## Basic Operations

### `Math.abs(x: number): number`

Returns the absolute value of `x`. Preserves signed zero: `abs(-0) = +0`. `abs(NaN) = NaN`, `abs(±∞) = +∞`.

```atlas
let n = Math.abs(-42);
// n == 42

let z = Math.abs(-0);
// z == 0
```

---

### `Math.floor(x: number): number`

Returns the largest integer less than or equal to `x`. `floor(-1.1) = -2`.

```atlas
let n = Math.floor(3.9);
// n == 3

let n2 = Math.floor(-1.1);
// n2 == -2
```

---

### `Math.ceil(x: number): number`

Returns the smallest integer greater than or equal to `x`. `ceil(-1.9) = -1`.

```atlas
let n = Math.ceil(1.1);
// n == 2

let n2 = Math.ceil(-1.9);
// n2 == -1
```

---

### `Math.round(x: number): number`

Rounds to the nearest integer using **banker's rounding** (ties-to-even). `round(2.5) = 2`, `round(3.5) = 4`.

```atlas
let n = Math.round(2.5);
// n == 2  (ties to even)

let n2 = Math.round(3.5);
// n2 == 4  (ties to even)

let n3 = Math.round(2.7);
// n3 == 3
```

---

### `Math.trunc(x: number): number`

Returns the integer part of `x` by removing the fractional digits (truncates toward zero). `trunc(-1.9) = -1`.

```atlas
let n = Math.trunc(3.9);
// n == 3

let n2 = Math.trunc(-1.9);
// n2 == -1
```

---

### `Math.min(a: number, b: number): number`

Returns the smaller of two numbers. If either is NaN, returns NaN.

```atlas
let n = Math.min(3, 7);
// n == 3

let n2 = Math.min(-5, 0);
// n2 == -5
```

---

### `Math.max(a: number, b: number): number`

Returns the larger of two numbers. If either is NaN, returns NaN.

```atlas
let n = Math.max(3, 7);
// n == 7
```

---

### `Math.sign(x: number): number`

Returns the sign of `x`: `-1` for negative, `1` for positive, `0`/`-0` for zero. Preserves signed zero. `sign(NaN) = NaN`.

```atlas
let s = Math.sign(-42);
// s == -1

let s2 = Math.sign(100);
// s2 == 1

let s3 = Math.sign(0);
// s3 == 0
```

---

## Exponential and Logarithmic

### `Math.sqrt(x: number): Result<number, string>`

Returns `Ok(√x)` for non-negative `x`, or `Err("sqrt() domain error: ...")` for negative or NaN input. `sqrt(+∞) = Ok(+∞)`.

```atlas
let r = Math.sqrt(9);
match r {
    Ok(n) => console.log(n.toString()),  // "3"
    Err(e) => console.error(e),
}

let bad = Math.sqrt(-1);
// bad == Err("sqrt() domain error: argument must be non-negative")
```

---

### `Math.cbrt(x: number): number`

Returns the cube root of `x`. Unlike `sqrt`, `cbrt` accepts negative values. `cbrt(-8) = -2`.

```atlas
let n = Math.cbrt(27);
// n == 3

let n2 = Math.cbrt(-8);
// n2 == -2
```

---

### `Math.pow(base: number, exponent: number): number`

Returns `base` raised to the power of `exponent`. `pow(x, 0) = 1` for any `x` (including NaN). `pow(1, y) = 1` for any `y`.

```atlas
let n = Math.pow(2, 10);
// n == 1024

let n2 = Math.pow(4, 0.5);
// n2 == 2
```

---

### `Math.exp(x: number): number`

Returns `e^x`. `exp(0) = 1`, `exp(+∞) = +∞`, `exp(-∞) = 0`.

```atlas
let n = Math.exp(1);
// n == Math.E()
```

---

### `Math.log(x: number): Result<number, string>`

Returns `Ok(ln(x))` (natural logarithm) for positive `x`, or `Err` for zero, negative, or NaN input.

```atlas
let r = Math.log(Math.E());
// r == Ok(1)

let bad = Math.log(0);
// bad == Err("log() domain error: argument must be positive")
```

---

### `Math.log2(x: number): Result<number, string>`

Returns `Ok(log₂(x))` for positive `x`, or `Err` for non-positive or NaN input.

```atlas
let r = Math.log2(8);
// r == Ok(3)
```

---

### `Math.log10(x: number): Result<number, string>`

Returns `Ok(log₁₀(x))` for positive `x`, or `Err` for non-positive or NaN input.

```atlas
let r = Math.log10(1000);
// r == Ok(3)
```

---

### `Math.hypot(x: number, y: number): number`

Returns `√(x² + y²)` without intermediate overflow or underflow. `hypot(±∞, y) = +∞` even if `y` is NaN.

```atlas
let h = Math.hypot(3, 4);
// h == 5
```

---

## Trigonometry

All trigonometric functions use **radians**.

### `Math.sin(x: number): number`

Returns the sine of `x` in radians. `sin(±∞) = NaN`, `sin(NaN) = NaN`.

```atlas
let s = Math.sin(Math.PI() / 2);
// s == 1
```

---

### `Math.cos(x: number): number`

Returns the cosine of `x` in radians. `cos(±∞) = NaN`.

```atlas
let c = Math.cos(0);
// c == 1
```

---

### `Math.tan(x: number): number`

Returns the tangent of `x` in radians. `tan(±∞) = NaN`.

```atlas
let t = Math.tan(Math.PI() / 4);
// t ≈ 1
```

---

### `Math.asin(x: number): Result<number, string>`

Returns `Ok(asin(x))` in radians for `x` in `[-1, 1]`, or `Err` for out-of-domain input. Output range: `[-π/2, π/2]`.

```atlas
let r = Math.asin(1);
// r == Ok(Math.PI() / 2)

let bad = Math.asin(2);
// bad == Err("asin() domain error: argument must be in [-1, 1]")
```

---

### `Math.acos(x: number): Result<number, string>`

Returns `Ok(acos(x))` in radians for `x` in `[-1, 1]`, or `Err` for out-of-domain input. Output range: `[0, π]`.

```atlas
let r = Math.acos(1);
// r == Ok(0)
```

---

### `Math.atan(x: number): number`

Returns the arctangent of `x` in radians. Output range: `(-π/2, π/2)`. `atan(±∞) = ±π/2`.

```atlas
let a = Math.atan(1);
// a == Math.PI() / 4
```

---

### `Math.atan2(y: number, x: number): number`

Returns the angle in radians between the positive x-axis and the point `(x, y)`. Output range: `(-π, π]`. Handles quadrant determination correctly using the signs of both arguments.

```atlas
let angle = Math.atan2(1, 1);
// angle == Math.PI() / 4

let angle2 = Math.atan2(1, -1);
// angle2 == 3 * Math.PI() / 4
```

---

## Utility

### `Math.clamp(value: number, min: number, max: number): Result<number, string>`

Restricts `value` to the range `[min, max]`. Returns `Err` if `min > max` or any argument is NaN.

```atlas
let n = Math.clamp(15, 0, 10);
// n == Ok(10)

let n2 = Math.clamp(5, 0, 10);
// n2 == Ok(5)

let bad = Math.clamp(5, 10, 0);
// bad == Err("clamp() domain error: min > max")
```

---

### `Math.random(): number`

Returns a pseudo-random floating-point number in the range `[0, 1)` with uniform distribution. Uses a thread-local RNG.

```atlas
let r = Math.random();
// 0 <= r < 1

let die = Math.floor(Math.random() * 6) + 1;
// die in [1, 6]
```
