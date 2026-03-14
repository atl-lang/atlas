# Atlas Formal Grammar

This document gives the formal grammar of Atlas derived directly from the parser source (`crates/atlas-runtime/src/parser/`). The notation is EBNF:

- `X*` — zero or more X
- `X+` — one or more X
- `X?` — zero or one X
- `X | Y` — X or Y (alternatives)
- `( X )` — grouping
- `"text"` — literal terminal
- `UPPER` — named terminal (lexical token)

---

## Top Level

```ebnf
Program       ::= Item*

Item          ::= Attribute* Visibility?
                  ( FunctionDecl
                  | AsyncFunctionDecl
                  | ImportDecl
                  | ExportDecl
                  | ExternDecl
                  | TypeAliasDecl
                  | ConstDecl
                  | TraitDecl
                  | ImplBlock
                  | StructDecl
                  | EnumDecl
                  | Statement
                  )

Visibility    ::= "pub" | "private" | "internal"

Attribute     ::= "@" IDENTIFIER ( "(" AttributeArgs ")" )?
AttributeArgs ::= IDENTIFIER ( "," IDENTIFIER )*
```

---

## Declarations

### Functions

```ebnf
FunctionDecl      ::= "fn" IDENTIFIER TypeParams? "(" Params ")" ":" ReturnOwnership? TypeRef TypePredicate? Block

AsyncFunctionDecl ::= "async" "fn" IDENTIFIER TypeParams? "(" Params ")" ":" ReturnOwnership? TypeRef TypePredicate? Block

ReturnOwnership   ::= "own" | "borrow"

TypePredicate     ::= "is" IDENTIFIER ":" TypeRef
```

Named functions **require** the `: ReturnType` annotation. Using `->` is rejected with a migration error.

### Parameters

```ebnf
Params    ::= ( Param ( "," Param )* ","? )?

Param     ::= "mut"? "..."? Ownership? IDENTIFIER ":" TypeRef DefaultValue?
            | "mut"? Ownership? "self"           (* bare self in impl methods *)

Ownership ::= "own" | "borrow" | "share"

DefaultValue ::= "=" Expr
```

`borrow` is implicit when no ownership annotation is written. Rest parameters (`...name: T[]`) must be last and may not have a default value.

### Type parameters

```ebnf
TypeParams ::= "<" TypeParam ( "," TypeParam )* ">"

TypeParam  ::= IDENTIFIER ( "extends" TraitBound ( "&" TraitBound )* )?

TraitBound ::= IDENTIFIER
```

### Import / Export

```ebnf
ImportDecl ::= "import" "{" ImportSpecifiers "}" "from" STRING ";"

ImportSpecifiers ::= ImportSpecifier ( "," ImportSpecifier )* ","?

ImportSpecifier  ::= IDENTIFIER ( "as" IDENTIFIER )?
                   | "*" "as" IDENTIFIER

ExportDecl ::= "export" ExportItem
             | "export" "{" ReExportSpecifier ( "," ReExportSpecifier )* "}" "from" STRING ";"

ExportItem ::= "async"? "fn" ...      (* function declaration *)
             | "type" ...             (* type alias *)
             | "const" ...            (* constant *)
             | "struct" ...           (* struct declaration *)
             | "enum" ...             (* enum declaration *)

ReExportSpecifier ::= IDENTIFIER ( "as" IDENTIFIER )?
```

### Type aliases and constants

```ebnf
TypeAliasDecl ::= "type" IDENTIFIER TypeParams? "=" TypeRef ";"

ConstDecl     ::= "const" IDENTIFIER ( ":" TypeRef )? "=" Expr ";"
```

### Structs

```ebnf
StructDecl  ::= Visibility? "struct" IDENTIFIER TypeParams? "{" StructFields? "}"

StructFields ::= StructField ( "," StructField )* ","?

StructField  ::= IDENTIFIER ":" TypeRef
```

Struct fields use commas as separators (not semicolons). Trailing comma is allowed.

### Enums

```ebnf
EnumDecl    ::= Visibility? "enum" IDENTIFIER "{" EnumVariant ( "," EnumVariant )* ","? "}"

EnumVariant ::= IDENTIFIER                        (* unit variant *)
              | IDENTIFIER "(" TypeRef ( "," TypeRef )* ")"   (* tuple variant *)
              | IDENTIFIER "{" StructFields "}"               (* struct variant *)
```

### Traits

```ebnf
TraitDecl     ::= Visibility? "trait" IDENTIFIER TypeParams?
                  ( "extends" TraitBound ( "," TraitBound )* )?
                  "{" TraitMethod* "}"

TraitMethod   ::= "fn" IDENTIFIER TypeParams? "(" TraitParams ")" ":" TypeRef ";"

TraitParams   ::= ( Param ( "," Param )* ","? )?
```

### Impl blocks

```ebnf
ImplBlock  ::= "impl" TypeRef? "for"? TypeRef "{" ImplMethod* "}"
             (* bare impl: "impl TypeName { ... }" *)
             (* trait impl: "impl TraitName for TypeName { ... }" *)

ImplMethod ::= Visibility? "static"? "async"? "fn" IDENTIFIER TypeParams?
               "(" ImplParams ")" ":" TypeRef Block

ImplParams ::= ( Param ( "," Param )* ","? )?
```

Static methods use the `static` keyword. They receive no `self` parameter.

### Extern declarations

```ebnf
ExternDecl ::= "extern" "{" ExternItem* "}"

ExternItem ::= "fn" IDENTIFIER "(" ExternParams ")" ":" ExternType ";"
             | "type" IDENTIFIER ";"

ExternType ::= "CInt" | "CLong" | "CDouble" | "CCharPtr" | "CVoid" | "CBool"
```

---

## Statements

```ebnf
Statement ::= VarDecl
            | LetDestructure
            | IfStmt
            | WhileStmt
            | ForInStmt
            | ReturnStmt
            | BreakStmt
            | ContinueStmt
            | DeferStmt
            | FunctionDecl      (* nested fn *)
            | AssignStmt
            | CompoundAssignStmt
            | ExprStmt
            | BlockStmt

VarDecl         ::= "let" "mut"? IDENTIFIER ( ":" TypeRef )? "=" Expr ";"

LetDestructure  ::= "let" "mut"? "(" IDENTIFIER ( "," IDENTIFIER )* ")" "=" Expr ";"

IfStmt          ::= "if" Condition Block ( "else" ( IfStmt | Block ) )?

WhileStmt       ::= "while" Condition Block

ForInStmt       ::= "for" IDENTIFIER "in" Expr Block

Condition       ::= Expr   (* no surrounding parens required; parens emit a warning *)

ReturnStmt      ::= "return" Expr? ";"

BreakStmt       ::= "break" ";"
                  | "break"               (* as match arm body, no semicolon *)

ContinueStmt    ::= "continue" ";"
                  | "continue"            (* as match arm body, no semicolon *)

DeferStmt       ::= "defer" Block
                  | "defer" Expr ";"

AssignStmt      ::= AssignTarget "=" Expr ";"

CompoundAssignStmt ::= AssignTarget CompoundOp Expr ";"

CompoundOp      ::= "+=" | "-=" | "*=" | "/=" | "%="

AssignTarget    ::= IDENTIFIER
                  | AssignTarget "[" Expr "]"
                  | AssignTarget "." IDENTIFIER

ExprStmt        ::= Expr ";"

BlockStmt       ::= Block
```

**Every expression used as a statement requires a trailing semicolon.** `f(x)` without `;` is a parse error.

**`match` at statement position** does not require a trailing semicolon (it ends with `}`).

---

## Expressions

Expressions follow Pratt precedence parsing. The table below lists levels from lowest to highest:

| Level | Operators / forms |
|---|---|
| Range | `start..end` `start..=end` `..end` `start..` |
| Or | `\|\|` |
| And | `&&` |
| Equality | `==` `!=` |
| Comparison | `<` `<=` `>` `>=` |
| Term | `+` `-` |
| Factor | `*` `/` `%` |
| Unary | `!` `-` (prefix) |
| Call | `expr(args)` `expr[index]` `expr.member` `expr.method(args)` `await expr` |

```ebnf
Expr ::= RangeExpr
       | BinaryExpr
       | UnaryExpr
       | PostfixExpr
       | PrimaryExpr

PrimaryExpr ::= Literal
              | TemplateString
              | IDENTIFIER
              | "(" Expr ")"                  (* grouping or tuple *)
              | "[" ( Expr ( "," Expr )* ","? )? "]"  (* array literal *)
              | "record" "{" RecordFields "}"  (* record literal *)
              | StructExpr                     (* Ident "{" fields "}" *)
              | AnonFn
              | MatchExpr
              | BlockExpr
              | NewExpr
              | "await" Expr

Literal ::= NUMBER | STRING | "true" | "false" | "null"

TemplateString ::= "`" ( TEXT | "${" Expr "}" )* "`"
```

### Range expressions

```ebnf
RangeExpr ::= Expr ".." Expr?
            | Expr "..=" Expr
            | ".." Expr
```

### Anonymous functions

```ebnf
AnonFn ::= "fn" "(" AnonParams ")" ( ":" TypeRef )? BlockExpr

AnonParams ::= ( AnonParam ( "," AnonParam )* ","? )?

AnonParam  ::= Ownership? IDENTIFIER ( ":" TypeRef )? DefaultValue?
```

Return type is optional for anonymous functions. Parameter type annotations are optional (default to `any` when omitted).

### Match expressions

```ebnf
MatchExpr ::= "match" Expr "{" MatchArm ( ( "," | ";" ) MatchArm )* ( "," | ";" )? "}"

MatchArm  ::= Pattern Guard? "=>" Expr

Guard     ::= "if" Expr

Pattern   ::= OrPattern

OrPattern ::= PrimaryPattern ( "|" PrimaryPattern )*

PrimaryPattern ::= LiteralPattern
                 | WildcardPattern
                 | BindingPattern
                 | TuplePattern
                 | EnumVariantPattern
                 | StructPattern

LiteralPattern  ::= NUMBER | STRING | "true" | "false" | "null"
WildcardPattern ::= "_"
BindingPattern  ::= IDENTIFIER
TuplePattern    ::= "(" Pattern ( "," Pattern )* ","? ")"
EnumVariantPattern ::= IDENTIFIER "::" IDENTIFIER ( "(" Pattern ( "," Pattern )* ")" )?
                     | IDENTIFIER "." IDENTIFIER ( "(" Pattern ( "," Pattern )* ")" )?
StructPattern   ::= IDENTIFIER "{" StructPatternField ( "," StructPatternField )* ","? "}"
                  | "{" StructPatternField ( "," StructPatternField )* ","? "}"

StructPatternField ::= IDENTIFIER ( ":" Pattern )?
```

### Block expressions

```ebnf
Block     ::= "{" Statement* TailExpr? "}"
BlockExpr ::= Block

TailExpr  ::= Expr   (* expression without trailing ";" — becomes block's value *)
```

The tail expression is the block's value. If no tail expression, the block produces `null`/`void`.

### Struct expressions

```ebnf
StructExpr ::= IDENTIFIER "{" StructField ( "," StructField )* ","? "}"

StructField ::= IDENTIFIER ":" Expr
              | IDENTIFIER           (* shorthand: field name = variable of same name *)
```

Inside conditions (`if`/`while`), `Identifier {` is NOT parsed as a struct expression to avoid ambiguity with the condition block. Use explicit `let s = MyStruct { ... }` before the condition.

### New expressions

```ebnf
NewExpr ::= "new" IDENTIFIER ( "<" TypeRef ( "," TypeRef )* ">" )? "(" Args ")"

Args    ::= ( Expr ( "," Expr )* ","? )?
```

### Try expression

```ebnf
TryExpr ::= Expr "?"
```

`?` unwraps `Ok(v)` → `v` or propagates `Err(e)` as an early return. Only valid inside functions returning `Result<T, E>`.

### Call expressions

```ebnf
CallExpr   ::= Expr TypeArgs? "(" Args ")"

MemberCall ::= Expr "." IDENTIFIER TypeArgs? "(" Args ")"

TypeArgs   ::= "<" TypeRef ( "," TypeRef )* ">"
```

### Await

```ebnf
AwaitExpr ::= "await" Expr
```

---

## Type References

```ebnf
TypeRef      ::= UnionType

UnionType    ::= IntersectionType ( "|" IntersectionType )*

IntersectionType ::= TypePrimary ( "&" TypePrimary )*

TypePrimary  ::= NamedType
               | GenericType
               | ArrayType
               | TupleType
               | FunctionType
               | StructuralType
               | "null"

NamedType    ::= IDENTIFIER

GenericType  ::= IDENTIFIER "<" TypeRef ( "," TypeRef )* ">"

ArrayType    ::= TypePrimary "[]"    (* postfix; "[]Type" prefix form is rejected *)

TupleType    ::= "(" TypeRef ( "," TypeRef )* ","? ")"
               | "()"                          (* unit *)

FunctionType ::= "(" ( TypeRef ( "," TypeRef )* )? ")" "=>" TypeRef

StructuralType ::= "{" StructuralMember ( "," StructuralMember )* ","? "}"

StructuralMember ::= IDENTIFIER ":" TypeRef
```

**Note:** In function type position, `(name: TypeRef) => TypeRef` is valid (TypeScript style). The parameter name is discarded — only the type matters.

`Future<T>` is a first-class type alias handled by the parser: `Future<T>` produces `TypeRef::Future { inner: T }`.

---

## Lexical Terminals

### Keywords

```
let  mut  fn  type  if  else  while  for  in  return  break  continue
import  export  from  extern  match  as  is
async  await  own  borrow  share
trait  impl  extends  struct  enum  record
pub  private  internal  static  const  defer  new  null  true  false
```

### Operators and punctuation

```
+  -  *  /  %  !  ==  !=  <  <=  >  >=  &&  ||  &  |  @
+=  -=  *=  /=  %=
=  (  )  {  }  [  ]  ;  ,  .  ..  ..=  ...  :  ::  ->  =>  _  ?
```

### Literals

- `NUMBER` — decimal integer or float (`42`, `3.14`, `1_000`)
- `STRING` — double-quoted string (`"hello"`)
- `TemplateString` — backtick string with `${}` interpolation
- `IDENTIFIER` — letter or `_` followed by letters, digits, `_`

### Comments

- `// single line`
- `/* block */`
- `/// doc comment`

Comments are skipped during parsing (trivia). The parser can be run in comment-preserving mode for tooling.

---

## Foreign Syntax Rejection

The following patterns from other languages are detected and rejected with specific migration errors:

| Pattern | Error |
|---|---|
| `var x = 1` | Use `let mut x = 1` |
| `function foo()` | Use `fn foo()` |
| `class Foo` | Use `struct` + `impl` |
| `echo` | Use `console.log()` |
| `x++` / `x--` | Use `x += 1` / `x -= 1` |
| `[]Type` (prefix array) | Use `Type[]` |
| `fn foo() -> T` (arrow return) | Use `fn foo(): T` |
| `import x from "m"` (default import) | Use `import { x } from "m"` |
