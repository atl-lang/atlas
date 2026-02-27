// Control flow analysis test suite split into 4 files
// Originally flow.rs (1414 lines, 36KB)
// Split to stay under 12KB file size limit per atlas-testing.md

mod return_analysis_part1; // Function return analysis — always return, missing return, if/else
mod return_analysis_part2; // Function return analysis — complex control flow, loops, edge cases
mod type_guards_part1;     // Type guard predicates and narrowing
mod type_guards_part2;     // Guard composition and control flow
