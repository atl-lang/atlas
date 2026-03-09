// Integration test suite split into subdomain files
// All tests were originally in integration.rs (1702 lines, 56KB)
// Split to stay under 12KB file size limit per atlas-testing.md

mod array_alias;
mod enum_types;
mod error_codes;
mod hashmap_generics;
mod if_expr;
mod ownership_part1;
mod ownership_part2;
mod ownership_part3;
mod range;
mod struct_member_access;
mod typecheck_dump;
mod typing_union;

mod method_types;
mod suggestions;
