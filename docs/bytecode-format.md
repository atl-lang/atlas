# Atlas Bytecode Format (v0.1)

## File Extension
- `.atb`

## Header (v0.1)
- Magic: `ATB1` (4 bytes)
- Version: `u16` (bytecode format version)
- Flags: `u16` (bitflags; 0 for v0.1)

## Sections (in order)
1. Constants Pool
   - Count: `u32`
   - Each constant:
     - Type tag: `u8`
     - Payload:
       - Number: `f64` (8 bytes, IEEE 754)
       - String: `u32` length + UTF-8 bytes
2. Code Section
   - Instruction count: `u32`
   - Each instruction:
     - Opcode: `u8`
     - Operands: fixed or variable by opcode (defined in compiler)
3. Debug Info Section
   - Span table count: `u32`
   - Each span:
     - File index: `u32`
     - Line: `u32`
     - Column: `u32`
     - Length: `u32`
   - Instruction span mapping:
     - `u32` per instruction (index into span table)
4. File Table
   - File count: `u32`
   - Each file:
     - `u32` length + UTF-8 bytes

## Notes
- Endianness: little-endian.
- Debug info is present by default in v0.1.
- Version mismatch should produce a diagnostic.
