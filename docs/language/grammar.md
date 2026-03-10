# Atlas Grammar (Parser-Accurate)

This document describes the **actual grammar implemented by the parser** in `crates/atlas-runtime/src/parser/` and the AST in `crates/atlas-runtime/src/ast.rs`.

**Notes:**
- Only syntax that is parsed today is listed here.
- Examples are tested (see `docs/tooling/cli.md` for how the checks were run).

**Top-Level Items**
```
program        := item*
item           := import_decl
               | export_decl
               | extern_decl
               | function_decl
               | type_alias_decl
               | trait_decl
               | impl_block
               | struct_decl
               | enum_decl
               | statement
```

**Function Declarations**
```
function_decl  := "fn" IDENT type_params? "(" params? ")" "->" type_ref block
params         := param ("," param)* (",")?
param          := ownership? IDENT ":" type_ref
ownership      := "own" | "borrow" | "share"
```
Return type is **required** on named functions. Use `-> void` for functions that return nothing.
Closures (`anon_fn`) may omit param types and return type (inferred from context).
`ownership` is optional — bare parameters default to `borrow`. Only `own` and `share` need to be written explicitly. (D-040)

**Type Parameters and Bounds**
```
type_params    := "<" type_param ("," type_param)* ">"
type_param     := IDENT ("extends" IDENT ("&" IDENT)*)?
```
Generic bounds use TypeScript-style `extends` keyword. Multiple bounds separated by `&`. (H-227, D-039)
Example: `fn foo<T extends Copy & Display>(borrow x: T) -> T`

**Trait Declarations**
```
trait_decl     := "trait" IDENT type_params? ("extends" IDENT ("," IDENT)*)? "{" trait_method_sig* "}"
trait_method_sig := "fn" IDENT type_params? "(" params? ")" "->" type_ref (";" | block)
```

Traits declare method signatures. Methods may have default bodies (using `block` instead of `;`).
Supertrait inheritance: `trait C extends A, B { ... }` — comma-separated, TypeScript style. (H-226, D-026)
`self` parameter type is inferred from the impl block — write `self` not `self: Type`.
Implementations live in `impl` blocks.

**Impl Blocks**
```
impl_block     := "impl" IDENT type_args? ("for" IDENT)? "{" impl_method* "}"
impl_method    := "fn" IDENT type_params? "(" params? ")" "->" type_ref block
```

Two forms:
- **Inherent impl** — `impl TypeName { ... }`: methods owned by the type, no trait required (D-036).
- **Trait impl** — `impl TraitName for TypeName { ... }`: polymorphism contract (existing behaviour).

`TraitName` may include type arguments (e.g., `impl Functor<number> for MyType`).
Inherent methods resolve before trait methods at call sites (D-037).
Self receiver requires an ownership annotation — `borrow self`, `own self`, or `share self` (D-038).

**Import / Export / Extern**
```
import_decl    := "import" "{" IDENT ("," IDENT)* "}" "from" STRING ";"
               | "import" "*" "as" IDENT "from" STRING ";"

export_decl    := "export" (function_decl | var_decl | type_alias_decl | struct_decl | enum_decl)

extern_decl    := "extern" STRING "fn" IDENT ("as" STRING)? "(" extern_params? ")" ("->" type_ref)? ";"

type_alias_decl := "type" type_params? IDENT "=" type_ref ";"
```

Examples:
```atlas
import { split, join } from "./utils";
import * as utils from "./helpers";
export fn greet(name: string) -> string { return `Hello {name}`; }
export type ID = number | string;
extern "libname" fn compress(data: string) -> string;
```

**Statements**
```
statement      := var_decl
               | function_decl
               | assign_stmt
               | compound_assign_stmt
               | if_stmt
               | while_stmt
               | for_in_stmt
               | match_stmt
               | return_stmt
               | break_stmt
               | continue_stmt
               | expr_stmt
               | block_stmt

var_decl       := "let" ("mut")? IDENT (":" type_ref)? "=" expr ";"
               | "let" ("mut")? "(" IDENT ("," IDENT)* ")" "=" expr ";"   // tuple destructure
assign_stmt    := assign_target "=" expr ";"
compound_assign_stmt := assign_target ("+="|"-="|"*="|"/="|"%=") expr ";"
assign_target  := IDENT
               | expr "[" expr "]"
               | expr "." IDENT

if_stmt        := "if" expr block ("else" (if_stmt | block))?
while_stmt     := "while" expr block
for_in_stmt    := "for" IDENT "in" expr block
match_stmt     := "match" expr "{" match_arm ( (","|";") match_arm )* (","|";")? "}"
return_stmt    := "return" expr? ";"
break_stmt     := "break" ";"
continue_stmt  := "continue" ";"
expr_stmt      := expr ";"
block_stmt     := block

condition      := ("(")? expr (")")?
block          := "{" statement* tail_expr? "}"
tail_expr      := expr  // only when no trailing semicolon
```

**Expressions**
```
expr           := literal
               | template_string
               | IDENT
               | unary_expr
               | binary_expr
               | call_expr
               | index_expr
               | member_expr
               | array_literal
               | record_literal
               | anon_struct_literal
               | struct_expr
               | enum_variant_expr
               | range_expr
               | group_expr
               | tuple_literal
               | match_expr
               | try_expr
               | anon_fn
               | block

literal        := NUMBER | STRING | "true" | "false" | "null"

array_literal  := "[" (expr ("," expr)*)? "]"
record_literal := "record" "{" (IDENT ":" expr ("," IDENT ":" expr)*)? (",")? "}"

anon_struct_literal := "{" (IDENT (":" expr)? ("," IDENT (":" expr)? )*) "}"
struct_expr    := TypeName "{" (IDENT ":" expr ("," IDENT ":" expr)*)? (",")? "}"

enum_variant_expr := EnumName "::" VariantName ("(" (expr ("," expr)*)? ")")?

range_expr     := expr? (".."|"..=") expr?

unary_expr     := ("-" | "!") expr
binary_expr    := expr bin_op expr
bin_op         := "+" | "-" | "*" | "/" | "%"
               | "==" | "!=" | "<" | "<=" | ">" | ">="
               | "&&" | "||"

call_expr      := expr "(" (expr ("," expr)*)? ")"
index_expr     := expr "[" expr "]"
member_expr    := expr "." IDENT ("(" (expr ("," expr)*)? ")")?
try_expr       := expr "?"

group_expr     := "(" expr ")"
tuple_literal  := "()"                              // unit
               | "(" expr "," ")"                  // 1-tuple (trailing comma required)
               | "(" expr "," expr ("," expr)* ")" // 2+ element tuple
member_expr    := expr "." IDENT
               | expr "." NUMBER                    // tuple index: t.0, t.1

match_expr     := "match" expr "{" match_arm ( (","|";") match_arm )* (","|";")? "}"
match_arm      := pattern ("if" expr)? "=>" expr

anon_fn        := "fn" "(" anon_params? ")" ("->" type_ref)? block
anon_params    := anon_param ("," anon_param)* (",")?
anon_param     := ownership? IDENT (":" type_ref)?
```

**Patterns (match)**
```
pattern        := literal
               | "_"
               | IDENT
               | IDENT "(" pattern_list? ")"     // constructor pattern
               | EnumName "::" VariantName ("(" pattern_list? ")")?
               | "[" pattern_list? "]"
               | "(" pattern_list? ")"     // tuple pattern: (p1, p2, ...)
               | pattern "|" pattern

pattern_list   := pattern ("," pattern)*
```

**Types**
```
type_ref       := union_type
union_type     := intersection_type ("|" intersection_type)*
intersection_type := type_primary ("&" type_primary)*

type_primary   := named_type
               | generic_type
               | array_type
               | function_type
               | structural_type
               | "(" type_ref ("," type_ref)* ")"   // tuple type: (T1, T2) or function type if followed by ->
               | "()"                               // unit tuple type

named_type     := IDENT | "null"
array_type     := "[]" type_primary   // prefix, nestable: []T, [][]T
function_type  := "(" type_ref ("," type_ref)* ")" "->" type_ref
structural_type := "{" IDENT ":" type_ref ("," IDENT ":" type_ref)* "}"

generic_type   := IDENT "<" type_ref ("," type_ref)* ">"
```

**Compound Assignment**
- `+=`, `-=`, `*=`, `/=`, `%=` operate on assignment targets.
- Example: `total += value;`

**Range Expressions**
- `1..10` (exclusive end), `1..=10` (inclusive end)
- `..10` (open start), `1..` (open end)

**Try Operator**
- `expr?` unwraps `Result` or `Option`, returning early on `Err`/`None`.

**Structural Types**
- `{ field: type }` declares a structural shape type.
- Structural types can include multiple members: `{ x: number, y: number }`.

**Intersection Types**
- `A & B` combines multiple type constraints.

**Operators and Precedence** (highest to lowest)
1. Call, member, index, try: `()` `.` `[]` `?`
2. Unary: `!` `-`
3. Multiplicative: `*` `/` `%`
4. Additive: `+` `-`
5. Comparison: `<` `<=` `>` `>=`
6. Equality: `==` `!=`
7. Logical AND: `&&`
8. Logical OR: `||`
9. Range: `..` `..=`

**Template Strings**
- Backtick-delimited: `` `hello ${name}` ``
- Interpolation uses `${ ... }` inside backticks.
- Double-quoted strings support `${ ... }` interpolation and are desugared into concatenations.

