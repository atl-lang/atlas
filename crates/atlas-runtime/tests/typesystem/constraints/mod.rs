// Type constraint test suite split into 2 files
// Originally constraints.rs (544 lines, 24KB)
// Split to stay under 12KB file size limit per atlas-testing.md

mod constraint_tests; // Type constraint solving and unification tests
mod type_rules;       // Type system rules (operators, arrays, methods)
