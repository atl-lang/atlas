//! Parsing (tokens to AST)
//!
//! The parser converts a stream of tokens into an Abstract Syntax Tree (AST).
//! Uses Pratt parsing for expressions and recursive descent for statements.

mod expr;
mod stmt;

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

// Parser error codes
const E_GENERIC: &str = "AT1000"; // Generic/uncategorized parse error
const E_BAD_NUMBER: &str = "AT1001"; // Invalid number literal
const E_MISSING_SEMI: &str = "AT1002"; // Missing semicolon
const E_MISSING_BRACE: &str = "AT1003"; // Missing closing brace/bracket/paren
const E_UNEXPECTED: &str = "AT1004"; // Unexpected token
const E_RESERVED: &str = "AT1005"; // Reserved keyword used as identifier

/// Parser state for building AST from tokens
pub struct Parser {
    pub(super) tokens: Vec<Token>,
    pub(super) current: usize,
    pub(super) diagnostics: Vec<Diagnostic>,
}

/// Operator precedence levels for Pratt parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Precedence {
    Lowest,
    Range,      // .. ..=
    Or,         // ||
    And,        // &&
    Equality,   // == !=
    Comparison, // < <= > >=
    Term,       // + -
    Factor,     // * / %
    Unary,      // ! -
    Call,       // () []
}

impl Parser {
    /// Create a new parser for the given tokens
    pub fn new(tokens: Vec<Token>) -> Self {
        let tokens = tokens
            .into_iter()
            .filter(|token| !matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment))
            .collect();
        Self {
            tokens,
            current: 0,
            diagnostics: Vec::new(),
        }
    }

    /// Parse tokens into an AST
    pub fn parse(&mut self) -> (Program, Vec<Diagnostic>) {
        let mut items = Vec::new();

        while !self.is_at_end_raw() {
            let doc_comment = self.collect_doc_comments();
            if self.is_at_end() {
                break;
            }
            match self.parse_item(doc_comment) {
                Ok(item) => items.push(item),
                Err(_) => self.synchronize(),
            }
        }

        (Program { items }, std::mem::take(&mut self.diagnostics))
    }

    // === Top-level parsing ===

    /// Parse a top-level item (function, statement, import, export, or extern)
    fn parse_item(&mut self, doc_comment: Option<String>) -> Result<Item, ()> {
        if self.check(TokenKind::Import) {
            Ok(Item::Import(self.parse_import()?))
        } else if self.check(TokenKind::Export) {
            Ok(Item::Export(self.parse_export()?))
        } else if self.check(TokenKind::Extern) {
            Ok(Item::Extern(self.parse_extern()?))
        } else if self.check(TokenKind::Fn) {
            Ok(Item::Function(self.parse_function()?))
        } else if self.check(TokenKind::Type) {
            Ok(Item::TypeAlias(self.parse_type_alias(doc_comment)?))
        } else if self.check(TokenKind::Trait) {
            Ok(Item::Trait(self.parse_trait()?))
        } else if self.check(TokenKind::Impl) {
            Ok(Item::Impl(self.parse_impl_block()?))
        } else if self.check(TokenKind::Struct) {
            Ok(Item::Struct(self.parse_struct()?))
        } else if self.check(TokenKind::Enum) {
            Ok(Item::Enum(self.parse_enum()?))
        } else {
            Ok(Item::Statement(self.parse_statement()?))
        }
    }

    /// Parse a struct declaration: `struct Name<T> { field: Type, ... }`
    fn parse_struct(&mut self) -> Result<crate::ast::StructDecl, ()> {
        let struct_start = self.consume(TokenKind::Struct, "Expected 'struct'")?.span;

        let name_token = self.consume_identifier("a struct name")?;
        let name = crate::ast::Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        // Parse optional type parameters: <T, U, ...>
        let type_params = self.parse_type_params()?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after struct name")?;

        let mut fields = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let field_name_token = self.consume_identifier("a field name")?;
            let field_name = crate::ast::Identifier {
                name: field_name_token.lexeme.clone(),
                span: field_name_token.span,
            };

            self.consume(TokenKind::Colon, "Expected ':' after field name")?;

            let type_ref = self.parse_type_ref()?;

            let field_span = field_name.span.merge(type_ref.span());
            fields.push(crate::ast::StructField {
                name: field_name,
                type_ref,
                span: field_span,
            });

            // Trailing comma is optional before closing brace
            if !self.check(TokenKind::RightBrace) {
                self.consume(TokenKind::Comma, "Expected ',' between struct fields")?;
            }
        }

        let end_span = self.consume(TokenKind::RightBrace, "Expected '}' after struct fields")?;

        Ok(crate::ast::StructDecl {
            name,
            type_params,
            fields,
            span: struct_start.merge(end_span.span),
        })
    }

    /// Parse an enum declaration: `enum Name<T> { Variant, Variant(Type), ... }`
    fn parse_enum(&mut self) -> Result<crate::ast::EnumDecl, ()> {
        let enum_start = self.consume(TokenKind::Enum, "Expected 'enum'")?.span;

        let name_token = self.consume_identifier("an enum name")?;
        let name = crate::ast::Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        // Parse optional type parameters: <T, U, ...>
        let type_params = self.parse_type_params()?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after enum name")?;

        let mut variants = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            let variant_name_token = self.consume_identifier("a variant name")?;
            // Extract all needed data before making more mutable borrows
            let variant_name_str = variant_name_token.lexeme.clone();
            let variant_name_span = variant_name_token.span;
            let variant_name = crate::ast::Identifier {
                name: variant_name_str,
                span: variant_name_span,
            };

            let variant = if self.check(TokenKind::LeftParen) {
                // Tuple variant: Variant(Type, Type, ...)
                self.advance();
                let mut tuple_fields = Vec::new();
                while !self.check(TokenKind::RightParen) && !self.is_at_end() {
                    tuple_fields.push(self.parse_type_ref()?);
                    if !self.check(TokenKind::RightParen) {
                        self.consume(TokenKind::Comma, "Expected ',' between tuple fields")?;
                    }
                }
                let end = self.consume(TokenKind::RightParen, "Expected ')' after tuple fields")?;
                crate::ast::EnumVariant::Tuple {
                    name: variant_name,
                    fields: tuple_fields,
                    span: variant_name_span.merge(end.span),
                }
            } else if self.check(TokenKind::LeftBrace) {
                // Struct variant: Variant { field: Type, ... }
                self.advance();
                let mut struct_fields = Vec::new();
                while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
                    let field_name_token = self.consume_identifier("a field name")?;
                    let field_name_str = field_name_token.lexeme.clone();
                    let field_name_span = field_name_token.span;
                    let field_name = crate::ast::Identifier {
                        name: field_name_str,
                        span: field_name_span,
                    };
                    self.consume(TokenKind::Colon, "Expected ':' after field name")?;
                    let type_ref = self.parse_type_ref()?;
                    let field_span = field_name_span.merge(type_ref.span());
                    struct_fields.push(crate::ast::StructField {
                        name: field_name,
                        type_ref,
                        span: field_span,
                    });
                    if !self.check(TokenKind::RightBrace) {
                        self.consume(TokenKind::Comma, "Expected ',' between fields")?;
                    }
                }
                let end =
                    self.consume(TokenKind::RightBrace, "Expected '}' after struct fields")?;
                crate::ast::EnumVariant::Struct {
                    name: variant_name,
                    fields: struct_fields,
                    span: variant_name_span.merge(end.span),
                }
            } else {
                // Unit variant: just the name
                crate::ast::EnumVariant::Unit {
                    name: variant_name,
                    span: variant_name_span,
                }
            };

            variants.push(variant);

            // Trailing comma is optional before closing brace
            if !self.check(TokenKind::RightBrace) {
                self.consume(TokenKind::Comma, "Expected ',' between enum variants")?;
            }
        }

        let end_span = self.consume(TokenKind::RightBrace, "Expected '}' after enum variants")?;

        Ok(crate::ast::EnumDecl {
            name,
            type_params,
            variants,
            span: enum_start.merge(end_span.span),
        })
    }

    /// Parse a function declaration
    fn parse_function(&mut self) -> Result<FunctionDecl, ()> {
        let fn_span = self.consume(TokenKind::Fn, "Expected 'fn'")?.span;

        let name_token = self.consume_identifier("a function name")?;
        let name = Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        // Parse optional type parameters: <T, E, ...>
        let type_params = self.parse_type_params()?;

        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;
        let params = self.parse_params()?;
        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse optional return type annotation: `-> [own|borrow] Type`
        let (return_type, return_ownership) = if self.match_token(TokenKind::Arrow) {
            // Peek for ownership annotation: `own` or `borrow` are valid; `shared` is an error
            let ownership = if self.match_token(TokenKind::Own) {
                Some(OwnershipAnnotation::Own)
            } else if self.match_token(TokenKind::Borrow) {
                Some(OwnershipAnnotation::Borrow)
            } else if self.check(TokenKind::Shared) {
                let span = self.peek().span;
                self.advance();
                self.diagnostics.push(Diagnostic::error(
                    "`shared` is not valid as a return ownership annotation; \
                     callers receive a `shared<T>` typed value instead",
                    span,
                ));
                return Err(());
            } else {
                None
            };
            (Some(self.parse_type_ref()?), ownership)
        } else {
            // No `->` annotation — return type will be inferred
            (None, None)
        };

        // Optional type predicate: `-> bool is param: Type`
        let predicate = if self.match_token(TokenKind::Is) {
            let param_token = self.consume_identifier("type predicate parameter")?;
            let param = Identifier {
                name: param_token.lexeme.clone(),
                span: param_token.span,
            };
            self.consume(
                TokenKind::Colon,
                "Expected ':' after type predicate parameter",
            )?;
            let target = self.parse_type_ref()?;
            let span = param.span.merge(target.span());
            Some(TypePredicate {
                param,
                target,
                span,
            })
        } else {
            None
        };

        // Parse body
        let body = self.parse_block()?;
        let end_span = body.span;

        Ok(FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            return_ownership,
            predicate,
            body,
            span: fn_span.merge(end_span),
        })
    }

    /// Parse an import declaration
    ///
    /// Syntax: `import { x, y } from "./path"` or `import * as ns from "./path"`
    fn parse_import(&mut self) -> Result<ImportDecl, ()> {
        let import_span = self.consume(TokenKind::Import, "Expected 'import'")?.span;

        let mut specifiers = Vec::new();

        if self.match_token(TokenKind::Star) {
            // Namespace import: import * as ns from "./path"
            self.consume(TokenKind::As, "Expected 'as' after '*'")?;
            let alias_token = self.consume_identifier("namespace alias")?;
            let alias = Identifier {
                name: alias_token.lexeme.clone(),
                span: alias_token.span,
            };
            specifiers.push(ImportSpecifier::Namespace {
                alias,
                span: import_span,
            });
        } else {
            // Named imports: import { x, y } from "./path"
            self.consume(TokenKind::LeftBrace, "Expected '{' for named imports")?;

            loop {
                let name_token = self.consume_identifier("import name")?;
                let name = Identifier {
                    name: name_token.lexeme.clone(),
                    span: name_token.span,
                };
                specifiers.push(ImportSpecifier::Named {
                    name,
                    span: name_token.span,
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }

                // Handle trailing comma
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }

            self.consume(TokenKind::RightBrace, "Expected '}' after imports")?;
        }

        self.consume(TokenKind::From, "Expected 'from' after imports")?;

        let source_token = self.consume(TokenKind::String, "Expected module path string")?;
        // Lexer already strips quotes from string literals
        let source = source_token.lexeme.clone();

        // Consume the semicolon
        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after import")?
            .span;

        Ok(ImportDecl {
            specifiers,
            source,
            span: import_span.merge(end_span),
        })
    }

    /// Parse an export declaration
    ///
    /// Syntax: `export fn foo() {}` or `export let x = 5`
    fn parse_export(&mut self) -> Result<ExportDecl, ()> {
        let export_span = self.consume(TokenKind::Export, "Expected 'export'")?.span;

        let item = if self.check(TokenKind::Fn) {
            ExportItem::Function(self.parse_function()?)
        } else if self.check(TokenKind::Let) {
            // Parse variable declaration
            let stmt = self.parse_statement()?;
            match stmt {
                Stmt::VarDecl(var) => ExportItem::Variable(var),
                _ => {
                    self.error("Expected variable declaration after 'export'");
                    return Err(());
                }
            }
        } else if self.check(TokenKind::Type) {
            ExportItem::TypeAlias(self.parse_type_alias(None)?)
        } else {
            self.error("Expected 'fn', 'let', or 'type' after 'export'");
            return Err(());
        };

        let end_span = self.peek().span;

        Ok(ExportDecl {
            item,
            span: export_span.merge(end_span),
        })
    }

    /// Parse a type alias declaration
    fn parse_type_alias(&mut self, doc_comment: Option<String>) -> Result<TypeAliasDecl, ()> {
        let type_span = self.consume(TokenKind::Type, "Expected 'type'")?.span;

        let name_token = self.consume_identifier("a type alias name")?;
        let name = Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        // Parse optional type parameters: <T, E, ...>
        let type_params = self.parse_type_params()?;

        self.consume(TokenKind::Equal, "Expected '=' after type alias name")?;
        let type_ref = self.parse_type_ref()?;

        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after type alias")?
            .span;

        Ok(TypeAliasDecl {
            name,
            type_params,
            type_ref,
            doc_comment,
            span: type_span.merge(end_span),
        })
    }

    // =========================================================================
    // Shared parsing helpers
    // =========================================================================

    /// Parse optional type parameters: `<T, U extends Bound, ...>` → `Vec<TypeParam>`.
    /// Returns an empty vec if no `<` is present.
    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>, ()> {
        let mut type_params = Vec::new();
        if self.match_token(TokenKind::Less) {
            loop {
                let type_param_start = self.peek().span;
                let type_param_tok = self.consume_identifier("a type parameter name")?;
                let type_param_name = type_param_tok.lexeme.clone();
                let type_param_span = type_param_tok.span;

                // Existing: `extends` type-level bound
                let mut bound = None;
                if self.match_token(TokenKind::Extends) {
                    bound = Some(self.parse_type_ref()?);
                }

                // NEW: `:` trait bounds (one or more, separated by `+`)
                let mut trait_bounds = Vec::new();
                if self.match_token(TokenKind::Colon) {
                    loop {
                        let bound_start = self.peek().span;
                        let trait_name_tok = self.consume_identifier("a trait name")?;
                        let bound_end = trait_name_tok.span;
                        trait_bounds.push(TraitBound {
                            trait_name: trait_name_tok.lexeme.clone(),
                            span: bound_start.merge(bound_end),
                        });
                        if !self.match_token(TokenKind::Plus) {
                            break;
                        }
                    }
                }

                type_params.push(TypeParam {
                    name: type_param_name,
                    bound,
                    trait_bounds,
                    span: type_param_start.merge(type_param_span),
                });
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
            self.consume(TokenKind::Greater, "Expected '>' after type parameters")?;
        }
        Ok(type_params)
    }

    /// Parse a comma-separated parameter list (without surrounding parens).
    /// Caller is responsible for consuming `(` before and `)` after.
    fn parse_params(&mut self) -> Result<Vec<Param>, ()> {
        let mut params = Vec::new();
        if self.check(TokenKind::RightParen) {
            return Ok(params);
        }
        loop {
            let param_span_start = self.peek().span;

            // Optional ownership annotation: own | borrow | shared
            let ownership = if self.match_token(TokenKind::Own) {
                Some(OwnershipAnnotation::Own)
            } else if self.match_token(TokenKind::Borrow) {
                Some(OwnershipAnnotation::Borrow)
            } else if self.match_token(TokenKind::Shared) {
                Some(OwnershipAnnotation::Shared)
            } else {
                None
            };

            let param_name_tok = if ownership.is_some() {
                match self.consume_identifier("a parameter name after ownership annotation") {
                    Ok(tok) => tok,
                    Err(()) => {
                        let kw = match ownership {
                            Some(OwnershipAnnotation::Own) => "own",
                            Some(OwnershipAnnotation::Borrow) => "borrow",
                            Some(OwnershipAnnotation::Shared) => "shared",
                            None => unreachable!(),
                        };
                        self.error(&format!(
                            "Expected parameter name after ownership annotation '{kw}'"
                        ));
                        return Err(());
                    }
                }
            } else {
                self.consume_identifier("a parameter name")?
            };

            let param_name = param_name_tok.lexeme.clone();
            let param_name_span = param_name_tok.span;

            self.consume(TokenKind::Colon, "Expected ':' after parameter name")?;
            let type_ref = self.parse_type_ref()?;
            let param_span_end = type_ref.span();

            params.push(Param {
                name: Identifier {
                    name: param_name,
                    span: param_name_span,
                },
                type_ref: Some(type_ref),
                ownership,
                span: param_span_start.merge(param_span_end),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
            // Allow trailing comma before `)`
            if self.check(TokenKind::RightParen) {
                break;
            }
        }
        Ok(params)
    }

    // =========================================================================
    // Trait system parsing (v0.3+)
    // =========================================================================

    /// Parse a trait declaration.
    ///
    /// Syntax: `trait Name<T> { fn method(params) -> ReturnType; }`
    fn parse_trait(&mut self) -> Result<TraitDecl, ()> {
        let start_span = self.consume(TokenKind::Trait, "Expected 'trait'")?.span;

        let name_tok = self.consume_identifier("a trait name")?;
        let name = Identifier {
            name: name_tok.lexeme.clone(),
            span: name_tok.span,
        };

        let type_params = self.parse_type_params()?;

        self.consume(TokenKind::LeftBrace, "Expected '{' after trait name")?;

        let mut methods = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_trait_method_sig()?);
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after trait body")?
            .span;

        Ok(TraitDecl {
            name,
            type_params,
            methods,
            span: start_span.merge(end_span),
        })
    }

    /// Parse a method signature inside a trait body.
    ///
    /// Syntax: `fn method_name<T>(param: Type, ...) -> ReturnType;`
    /// Note: NO block body — terminated by `;`
    fn parse_trait_method_sig(&mut self) -> Result<TraitMethodSig, ()> {
        let start_span = self
            .consume(TokenKind::Fn, "Expected 'fn' in trait body")?
            .span;

        let name_tok = self.consume_identifier("a method name")?;
        let name = Identifier {
            name: name_tok.lexeme.clone(),
            span: name_tok.span,
        };

        let type_params = self.parse_type_params()?;

        self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;
        let params = self.parse_params()?;
        self.consume(
            TokenKind::RightParen,
            "Expected ')' after method parameters",
        )?;

        self.consume(TokenKind::Arrow, "Expected '->' after method parameters")?;
        let return_type = self.parse_type_ref()?;

        let end_span = self
            .consume(
                TokenKind::Semicolon,
                "Expected ';' after trait method signature",
            )?
            .span;

        Ok(TraitMethodSig {
            name,
            type_params,
            params,
            return_type,
            span: start_span.merge(end_span),
        })
    }

    // =========================================================================
    // Impl block parsing (v0.3+)
    // =========================================================================

    /// Parse an impl block.
    ///
    /// Syntax: `impl TraitName for TypeName { fn method(...) -> T { body } }`
    fn parse_impl_block(&mut self) -> Result<ImplBlock, ()> {
        let start_span = self.consume(TokenKind::Impl, "Expected 'impl'")?.span;

        let trait_name_tok = self.consume_identifier("a trait name")?;
        let trait_name = Identifier {
            name: trait_name_tok.lexeme.clone(),
            span: trait_name_tok.span,
        };

        // Optional type args: `impl Functor<number> for MyType`
        let trait_type_args = if self.check(TokenKind::Less) {
            self.parse_type_arg_list()?
        } else {
            vec![]
        };

        self.consume(
            TokenKind::For,
            "Expected 'for' after trait name in impl block",
        )?;

        let type_name_tok = self.consume_identifier("a type name")?;
        let type_name = Identifier {
            name: type_name_tok.lexeme.clone(),
            span: type_name_tok.span,
        };

        self.consume(
            TokenKind::LeftBrace,
            "Expected '{' after type name in impl block",
        )?;

        let mut methods = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            methods.push(self.parse_impl_method()?);
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after impl body")?
            .span;

        Ok(ImplBlock {
            trait_name,
            trait_type_args,
            type_name,
            methods,
            span: start_span.merge(end_span),
        })
    }

    /// Parse a method implementation inside an impl block.
    ///
    /// Syntax: `fn method_name<T>(param: Type) -> ReturnType { body }`
    /// Impl methods REQUIRE a body (unlike trait method signatures).
    fn parse_impl_method(&mut self) -> Result<ImplMethod, ()> {
        let start_span = self
            .consume(TokenKind::Fn, "Expected 'fn' in impl body")?
            .span;

        let name_tok = self.consume_identifier("a method name")?;
        let name = Identifier {
            name: name_tok.lexeme.clone(),
            span: name_tok.span,
        };

        let type_params = self.parse_type_params()?;

        self.consume(TokenKind::LeftParen, "Expected '(' after method name")?;
        let params = self.parse_params()?;
        self.consume(
            TokenKind::RightParen,
            "Expected ')' after method parameters",
        )?;

        self.consume(TokenKind::Arrow, "Expected '->' after method parameters")?;
        let return_type = self.parse_type_ref()?;

        let body = self.parse_block()?;
        let end_span = body.span;

        Ok(ImplMethod {
            name,
            type_params,
            params,
            return_type,
            body,
            span: start_span.merge(end_span),
        })
    }

    /// Parse a `<TypeRef, TypeRef, ...>` type argument list (including angle brackets).
    /// Used for generic trait instantiation in impl blocks: `impl Foo<number> for ...`
    fn parse_type_arg_list(&mut self) -> Result<Vec<TypeRef>, ()> {
        self.consume(TokenKind::Less, "Expected '<'")?;
        let mut args = Vec::new();
        loop {
            args.push(self.parse_type_ref()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        self.consume(TokenKind::Greater, "Expected '>' after type arguments")?;
        Ok(args)
    }

    /// Parse an extern declaration (FFI function)
    ///
    /// Syntax: `extern "library" fn foo(x: CInt) -> CDouble;`
    ///         `extern "library" fn foo as "symbol_name"(x: CInt) -> CDouble;`
    fn parse_extern(&mut self) -> Result<ExternDecl, ()> {
        let extern_span = self.consume(TokenKind::Extern, "Expected 'extern'")?.span;

        // Parse library name (required string literal)
        let library_token = self.consume(TokenKind::String, "Expected library name string")?;
        let library = library_token.lexeme.clone();

        // Expect 'fn' keyword
        self.consume(TokenKind::Fn, "Expected 'fn' after library name")?;

        // Parse function name
        let name_token = self.consume_identifier("a function name")?;
        let name = name_token.lexeme.clone();

        // Optional symbol renaming: as "actual_symbol"
        let symbol = if self.match_token(TokenKind::As) {
            let symbol_token =
                self.consume(TokenKind::String, "Expected symbol name string after 'as'")?;
            Some(symbol_token.lexeme.clone())
        } else {
            None
        };

        // Parse parameters
        self.consume(TokenKind::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                let param_name_tok = self.consume_identifier("a parameter name")?;
                let param_name = param_name_tok.lexeme.clone();

                self.consume(TokenKind::Colon, "Expected ':' after parameter name")?;

                // Parse extern type annotation (CInt, CDouble, etc.)
                let type_annotation = self.parse_extern_type()?;

                params.push((param_name, type_annotation));

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        // Parse return type (required)
        self.consume(TokenKind::Arrow, "Expected '->' for return type")?;
        let return_type = self.parse_extern_type()?;

        // Consume the semicolon
        let end_span = self
            .consume(
                TokenKind::Semicolon,
                "Expected ';' after extern declaration",
            )?
            .span;

        Ok(ExternDecl {
            name,
            library,
            symbol,
            params,
            return_type,
            span: extern_span.merge(end_span),
        })
    }

    /// Parse an extern type annotation (CInt, CDouble, CVoid, etc.)
    fn parse_extern_type(&mut self) -> Result<ExternTypeAnnotation, ()> {
        let type_token = self.consume_identifier("an extern type")?;

        let extern_type = match type_token.lexeme.as_str() {
            "CInt" => ExternTypeAnnotation::CInt,
            "CLong" => ExternTypeAnnotation::CLong,
            "CDouble" => ExternTypeAnnotation::CDouble,
            "CCharPtr" => ExternTypeAnnotation::CCharPtr,
            "CVoid" => ExternTypeAnnotation::CVoid,
            "CBool" => ExternTypeAnnotation::CBool,
            other => {
                let error_msg = format!("Unknown extern type '{}'. Valid types: CInt, CLong, CDouble, CCharPtr, CVoid, CBool", other);
                self.error(&error_msg);
                return Err(());
            }
        };

        Ok(extern_type)
    }

    // === Helper methods ===

    /// Advance to next token and return reference to previous
    pub(super) fn advance(&mut self) -> &Token {
        self.skip_trivia();
        self.advance_raw()
    }

    /// Peek at current token
    pub(super) fn peek(&mut self) -> &Token {
        self.skip_trivia();
        self.peek_raw()
    }

    /// Check if current token matches kind
    pub(super) fn check(&mut self, kind: TokenKind) -> bool {
        !self.is_at_end() && self.peek().kind == kind
    }

    /// Match and consume token if it matches
    pub(super) fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume token of given kind or error
    pub(super) fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token, ()> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let code = match kind {
                TokenKind::Semicolon => E_MISSING_SEMI,
                TokenKind::RightBrace | TokenKind::RightBracket | TokenKind::RightParen => {
                    E_MISSING_BRACE
                }
                _ => E_UNEXPECTED,
            };
            self.error_with_code(code, message);
            Err(())
        }
    }

    /// Check if at end of token stream
    pub(super) fn is_at_end(&mut self) -> bool {
        self.skip_trivia();
        self.is_at_end_raw()
    }

    /// Record an error
    pub(super) fn error(&mut self, message: &str) {
        self.error_with_code(E_GENERIC, message);
    }

    /// Record an error at the current token with an explicit code
    pub(super) fn error_with_code(&mut self, code: &'static str, message: &str) {
        let span = self.peek().span;
        self.error_at_with_code(code, message, span);
    }

    /// Record an error at a specific span with an explicit code
    pub(super) fn error_at_with_code(&mut self, code: &'static str, message: &str, span: Span) {
        self.diagnostics.push(
            Diagnostic::error_with_code(code, message, span)
                .with_label("syntax error")
                .with_help("check your syntax for typos or missing tokens"),
        );
    }

    /// Check if a token kind is a reserved keyword
    fn is_reserved_keyword(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Let
                | TokenKind::Mut
                | TokenKind::Fn
                | TokenKind::Type
                | TokenKind::If
                | TokenKind::Else
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Return
                | TokenKind::Break
                | TokenKind::Continue
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Import
                | TokenKind::Match
                | TokenKind::Is
        )
    }

    fn is_comment_token(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::LineComment | TokenKind::BlockComment | TokenKind::DocComment
        )
    }

    fn skip_trivia(&mut self) {
        while !self.is_at_end_raw() && Self::is_comment_token(self.peek_raw().kind) {
            self.current += 1;
        }
    }

    fn peek_raw(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn advance_raw(&mut self) -> &Token {
        if !self.is_at_end_raw() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }

    fn check_raw(&self, kind: TokenKind) -> bool {
        !self.is_at_end_raw() && self.peek_raw().kind == kind
    }

    fn is_at_end_raw(&self) -> bool {
        self.current >= self.tokens.len() || self.tokens[self.current].kind == TokenKind::Eof
    }

    /// Consume an identifier token with enhanced error message for keywords
    pub(super) fn consume_identifier(&mut self, context: &str) -> Result<&Token, ()> {
        let current = self.peek().clone();

        // Check if it's a reserved keyword
        if Self::is_reserved_keyword(current.kind) {
            let keyword_name = current.lexeme;

            // Special message for import/match (reserved for future)
            if current.kind == TokenKind::Import || current.kind == TokenKind::Match {
                self.error_with_code(
                    E_RESERVED,
                    &format!(
                    "Cannot use reserved keyword '{}' as {}. This keyword is reserved for future use",
                    keyword_name, context
                ),
                );
            } else {
                self.error_with_code(
                    E_RESERVED,
                    &format!(
                        "Cannot use reserved keyword '{}' as {}",
                        keyword_name, context
                    ),
                );
            }
            Err(())
        } else if current.kind == TokenKind::Identifier {
            Ok(self.advance())
        } else {
            self.error_with_code(
                E_UNEXPECTED,
                &format!("Expected {} but found {:?}", context, current.kind),
            );
            Err(())
        }
    }

    /// Synchronize after error
    pub(super) fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.tokens[self.current - 1].kind == TokenKind::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenKind::Fn
                | TokenKind::Type
                | TokenKind::Let
                | TokenKind::If
                | TokenKind::While
                | TokenKind::For
                | TokenKind::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn collect_doc_comments(&mut self) -> Option<String> {
        while self.check_raw(TokenKind::LineComment) || self.check_raw(TokenKind::BlockComment) {
            self.advance_raw();
        }

        if !self.check_raw(TokenKind::DocComment) {
            return None;
        }

        let mut lines = Vec::new();
        while self.check_raw(TokenKind::DocComment) {
            let token = self.advance_raw();
            let text = token
                .lexeme
                .trim_start_matches("///")
                .trim_start()
                .to_string();
            lines.push(text);

            while self.check_raw(TokenKind::LineComment) || self.check_raw(TokenKind::BlockComment)
            {
                self.advance_raw();
            }
        }

        Some(lines.join("\n"))
    }
}
