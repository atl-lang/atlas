# Atlas Prelude (v0.1)

## Purpose
Define globally available built-ins without explicit import.

## Built-ins
- `print(value: string|number|bool|null) -> void`
- `len(value: string|T[]) -> number`
- `str(value: number|bool|null) -> string`

## Rules
- Prelude functions are always in scope.
- User code may shadow prelude names only in nested scopes.
- Shadowing prelude names in global scope is a compile-time error (`AT1012`).
