// Integration test suite split into subdomain files
// All tests were originally in integration.rs (1702 lines, 56KB)
// Split to stay under 12KB file size limit per atlas-testing.md

mod error_codes;
mod ownership_part1;
mod ownership_part2;
mod ownership_part3;
mod typecheck_dump;
mod typing_union;
