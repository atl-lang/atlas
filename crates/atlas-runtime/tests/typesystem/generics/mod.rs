// Generic type checking test suite split into multiple files
// Originally generics.rs (1309 lines, 36KB)
// Split to stay under 12KB file size limit per atlas-testing.md

mod bounds; // Trait bounds on type parameters (B38): T extends Foo, T extends A & B
mod part1; // Basic declarations, scoping, inference, arity, nested types, errors
mod part2; // Complex scenarios, edge cases, regressions, integration tests
mod part3; // Type improvements (function types, error messages, display)
