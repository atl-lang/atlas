//! Expression parsing (Pratt parsing)

use crate::ast::*;
use crate::diagnostic::error_codes::{GENERIC_WARNING, INVALID_NUMBER, SYNTAX_ERROR};
use crate::diagnostic::Diagnostic;
use crate::parser::{Parser, Precedence};
use crate::span::Span;
use crate::token::TokenKind;

impl Parser {
    /// Parse an expression
    pub(super) fn parse_expression(&mut self) -> Result<Expr, ()> {
        self.parse_precedence(Precedence::Lowest)
    }

    /// Parse expression with given precedence
    pub(super) fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr, ()> {
        let mut left = self.parse_prefix()?;

        while precedence < self.current_precedence() {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    /// Parse prefix expression
    fn parse_prefix(&mut self) -> Result<Expr, ()> {
        match self.peek().kind {
            TokenKind::Number => self.parse_number(),
            TokenKind::String => self.parse_string(),
            TokenKind::TemplateString => self.parse_template_string(),
            TokenKind::True | TokenKind::False => self.parse_bool(),
            TokenKind::Null => self.parse_null(),
            TokenKind::Identifier => self.parse_identifier(),
            TokenKind::LeftParen => self.parse_group(),
            TokenKind::LeftBracket => self.parse_array_literal(),
            TokenKind::Record => self.parse_record_literal(),
            TokenKind::LeftBrace => self.parse_block_or_anon_struct(),
            TokenKind::Minus | TokenKind::Bang => self.parse_unary(),
            TokenKind::Await => self.parse_await(),
            TokenKind::New => self.parse_new(),
            TokenKind::If => self.parse_if_expr(),
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::Fn => self.parse_anon_fn(),
            TokenKind::Range | TokenKind::RangeInclusive => self.parse_range_prefix(),
            _ => {
                let span = self.peek().span;
                self.emit_descriptor(
                    SYNTAX_ERROR
                        .emit(span)
                        .arg("detail", "expected an expression")
                        .with_help("expressions include literals, identifiers, function calls, operators, `if`, `match`, `fn`, and `new`")
                        .with_note("if you meant to write a statement, check for a missing semicolon on the previous line"),
                );
                Err(())
            }
        }
    }

    /// Parse infix expression
    fn parse_infix(&mut self, left: Expr) -> Result<Expr, ()> {
        match self.peek().kind {
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqualEqual
            | TokenKind::BangEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::AmpAmp
            | TokenKind::PipePipe => self.parse_binary(left),
            TokenKind::LeftParen => self.parse_call(left),
            TokenKind::LeftBracket => self.parse_index(left),
            TokenKind::Dot => self.parse_member(left),
            TokenKind::Question => self.parse_try(left),
            TokenKind::Range | TokenKind::RangeInclusive => self.parse_range_infix(left),
            _ => Ok(left),
        }
    }

    /// Get current token precedence
    pub(super) fn current_precedence(&mut self) -> Precedence {
        let kind = self.peek().kind;
        self.token_precedence_kind(kind)
    }

    /// Get precedence for a token kind
    pub(super) fn token_precedence_kind(&self, kind: TokenKind) -> Precedence {
        match kind {
            TokenKind::PipePipe => Precedence::Or,
            TokenKind::AmpAmp => Precedence::And,
            TokenKind::EqualEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Range | TokenKind::RangeInclusive => Precedence::Range,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::LeftParen
            | TokenKind::LeftBracket
            | TokenKind::Dot
            | TokenKind::Question => Precedence::Call,
            _ => Precedence::Lowest,
        }
    }

    /// Parse number literal
    fn parse_number(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        let lexeme = token.lexeme.clone();
        let value: f64 = match lexeme.parse::<f64>() {
            Ok(value) if value.is_finite() => value,
            _ => {
                self.emit_descriptor(INVALID_NUMBER.emit(span).arg("literal", &lexeme).with_note(
                    "use `math:nan()` or `math:inf()` to represent special numeric values",
                ));
                0.0
            }
        };
        Ok(Expr::Literal(Literal::Number(value), span))
    }

    /// Parse string literal
    fn parse_string(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        let first = Expr::Literal(Literal::String(token.lexeme.clone()), span);

        if !self.check(TokenKind::InterpolationStart) {
            return Ok(first);
        }

        let mut parts = Vec::new();
        parts.push(first);

        while self.check(TokenKind::InterpolationStart) {
            self.advance(); // consume ${
            let expr = self.parse_expression()?;
            self.consume(
                TokenKind::InterpolationEnd,
                "Expected '}' to close string interpolation",
            )?;
            let next = self.consume(
                TokenKind::String,
                "Expected string segment after interpolation",
            )?;
            let next_expr = Expr::Literal(Literal::String(next.lexeme.clone()), next.span);
            parts.push(expr);
            parts.push(next_expr);
        }

        let mut iter = parts.into_iter();
        let mut combined = iter
            .next()
            .expect("string interpolation must have at least one part");

        for part in iter {
            let span = combined.span().merge(part.span());
            combined = Expr::Binary(BinaryExpr {
                op: BinaryOp::Add,
                left: Box::new(combined),
                right: Box::new(part),
                span,
            });
        }

        Ok(combined)
    }

    fn parse_template_string(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let mut parts = Vec::new();
        let mut span = token.span;
        parts.push(TemplatePart::Literal(token.lexeme.clone()));

        while self.check(TokenKind::InterpolationStart) {
            self.advance(); // consume {
            let expr = self.parse_expression()?;
            let expr_span = expr.span();
            self.consume(
                TokenKind::InterpolationEnd,
                "Expected '}' to close string interpolation",
            )?;
            let next = self.consume(
                TokenKind::TemplateString,
                "Expected string segment after interpolation",
            )?;
            span = span.merge(expr_span).merge(next.span);
            parts.push(TemplatePart::Expression(Box::new(expr)));
            parts.push(TemplatePart::Literal(next.lexeme.clone()));
        }

        Ok(Expr::TemplateString { parts, span })
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ()> {
        let stmt = self.parse_if_stmt()?;
        let Stmt::If(if_stmt) = stmt else {
            return Err(());
        };
        let span = if_stmt.span;
        Ok(Expr::Block(Block {
            statements: vec![Stmt::If(if_stmt)],
            tail_expr: None,
            span,
        }))
    }

    /// Parse boolean literal
    fn parse_bool(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        let value = token.kind == TokenKind::True;
        Ok(Expr::Literal(Literal::Bool(value), span))
    }

    /// Parse null literal
    fn parse_null(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let span = token.span;
        Ok(Expr::Literal(Literal::Null, span))
    }

    /// Parse identifier, struct expression, or enum variant expression
    fn parse_identifier(&mut self) -> Result<Expr, ()> {
        let token = self.advance();
        let ident = Identifier {
            name: token.lexeme.clone(),
            span: token.span,
        };

        // Check for enum variant expression: `EnumName::VariantName` or `EnumName::VariantName(args)`
        if self.check(TokenKind::ColonColon) {
            return self.parse_enum_variant(ident);
        }

        // Check for struct expression: `TypeName { field: value, ... }`
        // This requires the identifier to start with an uppercase letter (type name convention).
        // Disabled inside conditions (if/while) to avoid ambiguity with the then/body block.
        if !self.no_struct_literal
            && self.check(TokenKind::LeftBrace)
            && ident.name.chars().next().is_some_and(|c| c.is_uppercase())
        {
            return self.parse_struct_expr(ident);
        }

        Ok(Expr::Identifier(ident))
    }

    /// Parse enum variant expression: `EnumName::VariantName` or `EnumName::VariantName(args)`
    fn parse_enum_variant(&mut self, enum_name: Identifier) -> Result<Expr, ()> {
        let start_span = enum_name.span;
        self.consume(TokenKind::ColonColon, "Expected '::' after enum name")?;

        // Parse variant name
        let variant_token = self.consume_identifier("an enum variant name")?;
        let variant_name = Identifier {
            name: variant_token.lexeme.clone(),
            span: variant_token.span,
        };

        // Check for optional arguments: EnumName::VariantName(arg1, arg2, ...)
        let args = if self.check(TokenKind::LeftParen) {
            self.advance(); // consume '('
            let mut args = Vec::new();

            if !self.check(TokenKind::RightParen) {
                loop {
                    args.push(self.parse_expression()?);

                    if !self.check(TokenKind::Comma) {
                        break;
                    }
                    self.advance(); // consume ','
                }
            }

            self.consume(
                TokenKind::RightParen,
                "Expected ')' after enum variant arguments",
            )?;
            Some(args)
        } else {
            None
        };

        let end_span = args
            .as_ref()
            .and_then(|a| a.last())
            .map(|e| e.span())
            .unwrap_or(variant_name.span);

        Ok(Expr::EnumVariant(crate::ast::EnumVariantExpr {
            enum_name,
            variant_name,
            args,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse struct expression: `TypeName { field: value, ... }`
    fn parse_struct_expr(&mut self, struct_name: Identifier) -> Result<Expr, ()> {
        let start_span = struct_name.span;
        self.consume(TokenKind::LeftBrace, "Expected '{' after struct type name")?;

        let mut fields = Vec::new();

        // Handle empty struct: `Point {}`
        if !self.check(TokenKind::RightBrace) {
            loop {
                let field_start = self.peek().span;

                // Parse field name
                let field_name_token = self.consume_identifier("a field name")?;
                let field_name = Identifier {
                    name: field_name_token.lexeme.clone(),
                    span: field_name_token.span,
                };

                // Expect ':'
                self.consume(TokenKind::Colon, "Expected ':' after field name")?;

                // Parse field value
                let value = self.parse_expression()?;
                let field_end = value.span();

                fields.push(StructFieldInit {
                    name: field_name,
                    value,
                    span: field_start.merge(field_end),
                });

                // Check for comma or end
                if !self.match_token(TokenKind::Comma) {
                    break;
                }

                // Allow trailing comma
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after struct fields")?
            .span;

        Ok(Expr::StructExpr(StructExpr {
            name: struct_name,
            fields,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse grouped expression
    fn parse_group(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::LeftParen, "Expected '('")?.span;

        // Unit tuple: ()
        if self.check(TokenKind::RightParen) {
            let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;
            return Ok(Expr::TupleLiteral {
                elements: Vec::new(),
                span: start_span.merge(end_span),
            });
        }

        let first = self.parse_expression()?;

        // Comma after first element → tuple literal
        if self.match_token(TokenKind::Comma) {
            let mut elements = vec![first];
            // Trailing comma on single element: (expr,) → 1-tuple
            // Multi-element: (e1, e2, ...) with optional trailing comma
            while !self.check(TokenKind::RightParen) {
                elements.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
            let end_span = self
                .consume(TokenKind::RightParen, "Expected ')' after tuple elements")?
                .span;
            return Ok(Expr::TupleLiteral {
                elements,
                span: start_span.merge(end_span),
            });
        }

        // No comma → grouping expression
        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;
        Ok(Expr::Group(GroupExpr {
            expr: Box::new(first),
            span: start_span.merge(end_span),
        }))
    }

    /// Parse array literal
    fn parse_array_literal(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::LeftBracket, "Expected '['")?.span;
        let mut elements = Vec::new();

        if !self.check(TokenKind::RightBracket) {
            loop {
                elements.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?.span;

        Ok(Expr::ArrayLiteral(ArrayLiteral {
            elements,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse a record literal: `record { key: value, key2: value2 }`.
    ///
    /// Record literal syntax:
    /// - `record { }` - empty record
    /// - `record { key: value }` - single entry
    /// - `record { key: value, key2: value2 }` - multiple entries
    /// - Trailing comma is allowed: `record { key: value, }`
    fn parse_record_literal(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::Record, "Expected 'record'")?.span;
        self.consume(TokenKind::LeftBrace, "Expected '{' after record")?;

        let mut entries = Vec::new();

        if !self.check(TokenKind::RightBrace) {
            loop {
                let entry_start = self.peek().span;

                let key_token = self.consume_identifier("record key")?;
                let key = Identifier {
                    name: key_token.lexeme.clone(),
                    span: key_token.span,
                };

                self.consume(TokenKind::Colon, "Expected ':' after record key")?;

                let value = self.parse_expression()?;
                let entry_end = value.span();

                entries.push(ObjectEntry {
                    key,
                    value,
                    span: entry_start.merge(entry_end),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }

                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after record literal")?
            .span;

        Ok(Expr::ObjectLiteral(ObjectLiteral {
            entries,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse await expression: `await <expr>`
    fn parse_await(&mut self) -> Result<Expr, ()> {
        let await_span = self.advance().span; // consume `await`

        // `await` must be followed by an expression
        if self.is_at_end() || self.check(TokenKind::Semicolon) {
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(await_span)
                    .arg("detail", "expected expression after `await`")
                    .with_help("write `await some_async_call()` — `await` requires an expression to suspend on")
                    .with_note("`await` can only be used inside `async fn` functions"),
            );
            return Err(());
        }

        let expr = self.parse_precedence(Precedence::Unary)?;
        let end_span = expr.span();
        Ok(Expr::Await {
            expr: Box::new(expr),
            span: await_span.merge(end_span),
        })
    }

    /// Parse `new TypeName<TypeArgs>(args)` constructor expression (H-374)
    ///
    /// Syntax: `new Map<K, V>()` | `new Set<T>()` | `new Queue<T>()` | `new Stack<T>()`
    fn parse_new(&mut self) -> Result<Expr, ()> {
        let new_span = self.advance().span; // consume `new`

        // Parse type name (must be an identifier)
        let name_token = self.consume_identifier("a type name after `new`")?;
        let type_name = crate::ast::Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        // Parse optional type arguments: `<K, V>`
        let type_args = self.try_parse_call_type_args()?;

        // Parse argument list: `()`  or  `(arg1, arg2, ...)`
        self.consume(
            TokenKind::LeftParen,
            "Expected '(' after type name in `new` expression",
        )?;
        let mut args = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        let end_span = self
            .consume(
                TokenKind::RightParen,
                "Expected ')' to close `new` expression",
            )?
            .span;

        Ok(Expr::New {
            type_name,
            type_args,
            args,
            span: new_span.merge(end_span),
        })
    }

    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expr, ()> {
        let op_token = self.advance();
        let op_span = op_token.span;
        let op = match op_token.kind {
            TokenKind::Minus => UnaryOp::Negate,
            TokenKind::Bang => UnaryOp::Not,
            _ => unreachable!(),
        };

        let operand = self.parse_precedence(Precedence::Unary)?;
        let operand_span = operand.span();

        Ok(Expr::Unary(UnaryExpr {
            op,
            expr: Box::new(operand),
            span: op_span.merge(operand_span),
        }))
    }

    /// Parse binary expression
    fn parse_binary(&mut self, left: Expr) -> Result<Expr, ()> {
        let left_span = left.span();
        let op_token = self.advance();
        let op_kind = op_token.kind;

        let op = match op_kind {
            TokenKind::Plus => BinaryOp::Add,
            TokenKind::Minus => BinaryOp::Sub,
            TokenKind::Star => BinaryOp::Mul,
            TokenKind::Slash => BinaryOp::Div,
            TokenKind::Percent => BinaryOp::Mod,
            TokenKind::EqualEqual => BinaryOp::Eq,
            TokenKind::BangEqual => BinaryOp::Ne,
            TokenKind::Less => BinaryOp::Lt,
            TokenKind::LessEqual => BinaryOp::Le,
            TokenKind::Greater => BinaryOp::Gt,
            TokenKind::GreaterEqual => BinaryOp::Ge,
            TokenKind::AmpAmp => BinaryOp::And,
            TokenKind::PipePipe => BinaryOp::Or,
            _ => unreachable!(),
        };

        // Get precedence from the operator kind
        let precedence = match op_kind {
            TokenKind::PipePipe => Precedence::Or,
            TokenKind::AmpAmp => Precedence::And,
            TokenKind::EqualEqual | TokenKind::BangEqual => Precedence::Equality,
            TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            _ => Precedence::Lowest,
        };

        let right = self.parse_precedence(precedence)?;
        let right_span = right.span();

        Ok(Expr::Binary(BinaryExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
            span: left_span.merge(right_span),
        }))
    }

    /// Parse call expression
    fn parse_call(&mut self, callee: Expr) -> Result<Expr, ()> {
        let callee_span = callee.span();
        self.consume(TokenKind::LeftParen, "Expected '('")?;
        let mut args = Vec::new();

        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;

        Ok(Expr::Call(CallExpr {
            callee: Box::new(callee),
            args,
            type_args: vec![],
            span: callee_span.merge(end_span),
        }))
    }

    /// Parse index expression
    fn parse_index(&mut self, target: Expr) -> Result<Expr, ()> {
        let target_span = target.span();
        self.consume(TokenKind::LeftBracket, "Expected '['")?;
        if self.check(TokenKind::RightBracket) {
            let span = self.peek().span;
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(span)
                    .arg("detail", "expected an expression inside `[]`")
                    .with_help("write an index expression: `array[0]` or `map[key]`")
                    .with_note("empty brackets `[]` are not a valid index — provide an expression"),
            );
            return Err(());
        }
        let index_expr = self.parse_expression()?;
        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?.span;

        Ok(Expr::Index(IndexExpr {
            target: Box::new(target),
            index: IndexValue::Single(Box::new(index_expr)),
            span: target_span.merge(end_span),
        }))
    }

    fn parse_range_prefix(&mut self) -> Result<Expr, ()> {
        let (token_span, inclusive) = {
            let token = self.advance();
            (token.span, matches!(token.kind, TokenKind::RangeInclusive))
        };

        let next_kind = self.peek().kind;
        let end_expr = if Self::can_start_expression(next_kind) {
            Some(self.parse_precedence(Precedence::Range)?)
        } else {
            None
        };

        if inclusive && end_expr.is_none() {
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(token_span)
                    .arg("detail", "inclusive range `..=` requires an end expression")
                    .with_help("write `start..=end` — inclusive ranges must have both a start and an end value")
                    .with_note("use `start..` for an open-ended range without an upper bound"),
            );
            return Err(());
        }

        let end_span = end_expr
            .as_ref()
            .map(|expr| expr.span())
            .unwrap_or(token_span);

        Ok(Expr::Range {
            start: None,
            end: end_expr.map(Box::new),
            inclusive,
            span: token_span.merge(end_span),
        })
    }

    fn parse_range_infix(&mut self, left: Expr) -> Result<Expr, ()> {
        let left_span = left.span();
        let (token_span, inclusive) = {
            let token = self.advance();
            (token.span, matches!(token.kind, TokenKind::RangeInclusive))
        };

        let next_kind = self.peek().kind;
        let end_expr = if Self::can_start_expression(next_kind) {
            Some(self.parse_precedence(Precedence::Range)?)
        } else {
            None
        };

        if inclusive && end_expr.is_none() {
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(token_span)
                    .arg("detail", "inclusive range `..=` requires an end expression")
                    .with_help("write `start..=end` — inclusive ranges must have both a start and an end value")
                    .with_note("use `start..` for an open-ended range without an upper bound"),
            );
            return Err(());
        }

        let end_span = end_expr
            .as_ref()
            .map(|expr| expr.span())
            .unwrap_or(token_span);

        Ok(Expr::Range {
            start: Some(Box::new(left)),
            end: end_expr.map(Box::new),
            inclusive,
            span: left_span.merge(end_span),
        })
    }

    fn can_start_expression(kind: TokenKind) -> bool {
        matches!(
            kind,
            TokenKind::Number
                | TokenKind::String
                | TokenKind::True
                | TokenKind::False
                | TokenKind::Null
                | TokenKind::Identifier
                | TokenKind::LeftParen
                | TokenKind::LeftBracket
                | TokenKind::Record
                | TokenKind::LeftBrace
                | TokenKind::Minus
                | TokenKind::Bang
                | TokenKind::Match
                | TokenKind::Fn
                | TokenKind::Range
                | TokenKind::RangeInclusive
                | TokenKind::New
        )
    }

    /// Parse member expression (method call or property access)
    fn parse_member(&mut self, target: Expr) -> Result<Expr, ()> {
        let target_span = target.span();
        self.consume(TokenKind::Dot, "Expected '.'")?;

        // Member name: identifier OR integer literal (for tuple element access: .0, .1, ...)
        let member = if self.check(TokenKind::Number) {
            let tok = self.advance();
            // Only accept non-negative integer literals as tuple indices
            let lexeme = tok.lexeme.clone();
            let is_integer = lexeme.parse::<u64>().is_ok();
            if !is_integer {
                let span = tok.span;
                self.emit_descriptor(
                    SYNTAX_ERROR
                        .emit(span)
                        .arg(
                            "detail",
                            "tuple element index must be a non-negative integer",
                        )
                        .with_help(
                            "use `.0`, `.1`, `.2`, etc. to access tuple elements by position",
                        )
                        .with_note(
                            "floating-point or negative literals are not valid tuple indices",
                        ),
                );
                return Err(());
            }
            Identifier {
                name: lexeme,
                span: tok.span,
            }
        } else {
            let member_token = self.consume_member_name()?;
            Identifier {
                name: member_token.lexeme.clone(),
                span: member_token.span,
            }
        };

        // Check for generic type arguments: method<T>(args)
        // Only parse as type args if we see < followed by type-like content
        let type_args = if self.check(TokenKind::Less) {
            self.try_parse_call_type_args()?
        } else {
            vec![]
        };

        // Check for method call (with parentheses)
        let (args, end_span) = if self.check(TokenKind::LeftParen) {
            self.consume(TokenKind::LeftParen, "Expected '('")?;
            let mut args_vec = Vec::new();

            if !self.check(TokenKind::RightParen) {
                loop {
                    args_vec.push(self.parse_expression()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }

            let end = self.consume(TokenKind::RightParen, "Expected ')'")?.span;
            (Some(args_vec), end)
        } else {
            // No parentheses - property access (for now, treat as method call with no args)
            // Phase 16 only supports method calls, but parser can handle both
            (None, member.span)
        };

        Ok(Expr::Member(MemberExpr {
            target: Box::new(target),
            member,
            args,
            type_args,
            type_tag: std::cell::Cell::new(None),
            trait_dispatch: std::cell::RefCell::new(None),
            static_dispatch: std::cell::RefCell::new(None),
            span: target_span.merge(end_span),
        }))
    }

    /// Parse try expression (error propagation operator ?)
    fn parse_try(&mut self, expr: Expr) -> Result<Expr, ()> {
        let expr_span = expr.span();
        let question_span = self.consume(TokenKind::Question, "Expected '?'")?.span;

        Ok(Expr::Try(TryExpr {
            expr: Box::new(expr),
            target_kind: std::cell::RefCell::new(None),
            span: expr_span.merge(question_span),
        }))
    }

    /// Parse type reference
    pub(super) fn parse_type_ref(&mut self) -> Result<TypeRef, ()> {
        self.parse_union_type()
    }

    /// Parse union type: A | B
    fn parse_union_type(&mut self) -> Result<TypeRef, ()> {
        let mut members = vec![self.parse_intersection_type()?];

        while self.match_token(TokenKind::Pipe) {
            members.push(self.parse_intersection_type()?);
        }

        if members.len() == 1 {
            Ok(members.remove(0))
        } else {
            let start = members.first().unwrap().span();
            let end = members.last().unwrap().span();
            Ok(TypeRef::Union {
                members,
                span: start.merge(end),
            })
        }
    }

    /// Parse intersection type: A & B
    fn parse_intersection_type(&mut self) -> Result<TypeRef, ()> {
        let mut members = vec![self.parse_type_primary()?];

        while self.match_token(TokenKind::Ampersand) {
            members.push(self.parse_type_primary()?);
        }

        if members.len() == 1 {
            Ok(members.remove(0))
        } else {
            let start = members.first().unwrap().span();
            let end = members.last().unwrap().span();
            Ok(TypeRef::Intersection {
                members,
                span: start.merge(end),
            })
        }
    }

    /// Parse primary type (named, generic, grouped, or function), plus postfix array suffix.
    fn parse_type_primary(&mut self) -> Result<TypeRef, ()> {
        // Detect old prefix array syntax `[]Type` and emit a clear migration error.
        // The inner type name must follow `[]` immediately (no space between `]` and name).
        if self.check(TokenKind::LeftBracket) {
            let next2_is_rbracket = self
                .peek_nth_nontrivia(1)
                .map(|t| t.kind == TokenKind::RightBracket)
                .unwrap_or(false);
            let next3_is_ident = self
                .peek_nth_nontrivia(2)
                .map(|t| {
                    matches!(t.kind, TokenKind::Identifier)
                        || TokenKind::is_keyword(t.lexeme.as_str()).is_some()
                })
                .unwrap_or(false);
            if next2_is_rbracket && next3_is_ident {
                let lbracket_span = self.peek().span;
                self.advance(); // consume `[`
                self.advance(); // consume `]`
                let name_token = self.peek().clone();
                let name = &name_token.lexeme;
                let full_span = lbracket_span.merge(name_token.span);
                self.emit_descriptor(
                    SYNTAX_ERROR
                        .emit(full_span)
                        .arg(
                            "detail",
                            format!(
                                "array types use postfix syntax — write `{}[]`, not `[]{}`",
                                name, name
                            ),
                        )
                        .with_help(format!("change `[]{}` to `{}[]`", name, name))
                        .with_note(
                            "Atlas uses TypeScript-style postfix array syntax: `T[]` not `[]T`",
                        ),
                );
                return Err(());
            }
        }

        let type_ref = if self.check(TokenKind::LeftParen) {
            self.parse_paren_type()?
        } else if self.check(TokenKind::LeftBrace) {
            self.parse_structural_type()?
        } else {
            let token = if self.check(TokenKind::Null) {
                self.advance()
            } else {
                self.consume_identifier("a type name")?
            };
            let span = token.span;
            let name = token.lexeme.clone();

            // Check for generic type: Type<T1, T2>
            if self.check(TokenKind::Less) {
                self.parse_generic_type(name, span)?
            } else {
                TypeRef::Named(name, span)
            }
        };

        // Handle postfix array type syntax: T[], T[][], generic<T>[], etc.
        // Consume all trailing `[]` suffixes, wrapping outward each time.
        let mut result = type_ref;
        loop {
            let next_is_lbracket = self.check(TokenKind::LeftBracket);
            let next2_is_rbracket = self
                .peek_nth_nontrivia(1)
                .map(|t| t.kind == TokenKind::RightBracket)
                .unwrap_or(false);
            if next_is_lbracket && next2_is_rbracket {
                let lbracket_span = self.peek().span;
                self.advance(); // consume `[`
                let rbracket_span = self.peek().span;
                self.advance(); // consume `]`
                let full_span = result.span().merge(lbracket_span).merge(rbracket_span);
                result = TypeRef::Array(Box::new(result), full_span);
            } else {
                break;
            }
        }

        Ok(result)
    }

    /// Parse structural type: { field: type, method: (params) -> return }
    fn parse_structural_type(&mut self) -> Result<TypeRef, ()> {
        use crate::ast::StructuralMember;

        let start_span = self
            .consume(
                TokenKind::LeftBrace,
                "Expected '{' at start of structural type",
            )?
            .span;

        let mut members = Vec::new();
        if !self.check(TokenKind::RightBrace) {
            loop {
                let member_start = self.peek().span;
                let name_tok = self.consume_identifier("a structural member name")?;
                let member_name = name_tok.lexeme.clone();
                let member_name_span = name_tok.span;
                self.consume(TokenKind::Colon, "Expected ':' after member name")?;
                let type_ref = self.parse_type_ref()?;
                let member_span = member_start.merge(type_ref.span());
                members.push(StructuralMember {
                    name: member_name,
                    type_ref,
                    span: member_span.merge(member_name_span),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after structural type")?
            .span;

        if members.is_empty() {
            let span = self.peek().span;
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(span)
                    .arg("detail", "structural type must include at least one member")
                    .with_help("write `{ field: Type, ... }` — structural types require at least one named field")
                    .with_note("use a named struct declaration for types with no fields"),
            );
            return Err(());
        }

        Ok(TypeRef::Structural {
            members,
            span: start_span.merge(end_span),
        })
    }

    /// Parse parenthesized type or function type.
    fn parse_paren_type(&mut self) -> Result<TypeRef, ()> {
        let start_token = self.consume(TokenKind::LeftParen, "Expected '(' at start of type")?;
        let start_span = start_token.span;

        // Unit type () — either `(): T` or `() => T` (function) or `()` (unit tuple)
        if self.check(TokenKind::RightParen) {
            let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;
            if self.match_token(TokenKind::FatArrow) || self.match_token(TokenKind::Colon) {
                let return_type = self.parse_type_ref()?;
                let full_span = start_span.merge(return_type.span());
                return Ok(TypeRef::Function {
                    params: Vec::new(),
                    return_type: Box::new(return_type),
                    span: full_span,
                });
            }
            // Unit tuple type
            return Ok(TypeRef::Tuple {
                elements: Vec::new(),
                span: start_span.merge(end_span),
            });
        }

        let mut params = Vec::new();
        // TypeScript allows named params in function types: (x: number) => void
        // Detect `ident :` and skip the name — only the type matters for TypeRef::Function.
        self.skip_fn_type_param_name();
        params.push(self.parse_type_ref()?);

        while self.match_token(TokenKind::Comma) {
            // Allow trailing comma: (T,) is a 1-tuple type
            if self.check(TokenKind::RightParen) {
                break;
            }
            self.skip_fn_type_param_name();
            params.push(self.parse_type_ref()?);
        }

        let end_span = self
            .consume(TokenKind::RightParen, "Expected ')' after type list")?
            .span;

        if self.match_token(TokenKind::FatArrow) || self.match_token(TokenKind::Colon) {
            let return_type = self.parse_type_ref()?;
            let full_span = start_span.merge(return_type.span());
            return Ok(TypeRef::Function {
                params,
                return_type: Box::new(return_type),
                span: full_span,
            });
        }

        // No arrow or colon → tuple type. Single element without trailing comma stays as grouping
        // (already consumed trailing comma above, so params.len() == 1 here means
        // the user wrote `(T)` with no comma — treat as grouped type, not a tuple).
        if params.len() == 1 {
            return Ok(params.remove(0));
        }

        // Multi-element or single with trailing comma → tuple type
        let full_span = start_span.merge(end_span);
        Ok(TypeRef::Tuple {
            elements: params,
            span: full_span,
        })
    }

    /// Skip optional `name:` or `name?:` prefix in a function type parameter position.
    ///
    /// TypeScript allows named params: `(x: number) => void`.
    /// Atlas only uses the type — the name is discarded. We peek two tokens ahead
    /// to avoid consuming tokens that are actually part of a type expression.
    fn skip_fn_type_param_name(&mut self) {
        let is_ident = self.peek().kind == TokenKind::Identifier
            || TokenKind::is_keyword(self.peek().lexeme.as_str()).is_some();
        if !is_ident {
            return;
        }
        // Peek at next non-trivia token — if it's `:` then this is `name: Type`
        let next_is_colon = self
            .peek_nth_nontrivia(1)
            .map(|t| t.kind == TokenKind::Colon)
            .unwrap_or(false);
        if next_is_colon {
            self.advance(); // consume name
            self.advance(); // consume `:`
        }
    }

    /// Parse generic type: Type<T1, T2, ...>
    fn parse_generic_type(&mut self, name: String, start: Span) -> Result<TypeRef, ()> {
        // Consume '<'
        self.consume(TokenKind::Less, "Expected '<'")?;

        // Parse type arguments
        let mut type_args = vec![];
        loop {
            type_args.push(self.parse_type_ref()?);

            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        // Ensure at least one type argument
        if type_args.is_empty() {
            let span = self.peek().span;
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(span)
                    .arg("detail", "generic type requires at least one type argument")
                    .with_help(
                        "write `Type<T>` — provide at least one type inside the angle brackets",
                    )
                    .with_note("use a non-generic type if no type parameters are needed"),
            );
            return Err(());
        }

        // Consume '>'
        let end_token = self.consume(TokenKind::Greater, "Expected '>' after type arguments")?;
        let span = start.merge(end_token.span);

        // Future<T> is a first-class type — produce TypeRef::Future directly
        if name == "Future" && type_args.len() == 1 {
            return Ok(TypeRef::Future {
                inner: Box::new(type_args.remove(0)),
                span,
            });
        }

        Ok(TypeRef::Generic {
            name,
            type_args,
            span,
        })
    }

    /// Try to parse type arguments for a generic call: `<T1, T2, ...>`
    /// Returns empty vec if `<` is not present.
    /// Used for: `Json.parse<User>(str)`, `fn<T>(x)`
    fn try_parse_call_type_args(&mut self) -> Result<Vec<TypeRef>, ()> {
        if !self.match_token(TokenKind::Less) {
            return Ok(vec![]);
        }

        // Parse type arguments
        let mut type_args = vec![];
        loop {
            type_args.push(self.parse_type_ref()?);
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }

        // Consume '>'
        self.consume(TokenKind::Greater, "Expected '>' after type arguments")?;

        Ok(type_args)
    }

    // parse_function_type removed in favor of parse_paren_type

    /// Parse anonymous function expression: `fn(params): ReturnType { body }`
    ///
    /// Anonymous functions allow optional type annotations on parameters.
    /// This is the unified syntax after arrow functions were removed.
    fn parse_anon_fn(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::Fn, "Expected 'fn'")?.span;
        self.consume(TokenKind::LeftParen, "Expected '(' after 'fn'")?;
        let params = self.parse_anon_fn_params()?;
        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };

        let body = self.parse_block_expr()?;
        let body_span = body.span();
        let span = start_span.merge(body_span);

        Ok(Expr::AnonFn {
            params,
            return_type,
            body: Box::new(body),
            span,
        })
    }

    /// Parse parameters for anonymous functions (type annotations optional).
    ///
    /// Syntax: `[ownership] name [: type]`
    fn parse_anon_fn_params(&mut self) -> Result<Vec<Param>, ()> {
        let mut params = Vec::new();
        if self.check(TokenKind::RightParen) {
            return Ok(params);
        }
        loop {
            let param_span_start = self.peek().span;

            // Optional ownership annotation: own | borrow | share (D-040: borrow is implicit)
            let ownership_from_source = if self.match_token(TokenKind::Own) {
                Some(OwnershipAnnotation::Own)
            } else if self.match_token(TokenKind::Borrow) {
                Some(OwnershipAnnotation::Borrow)
            } else if self.match_token(TokenKind::Share) {
                Some(OwnershipAnnotation::Share)
            } else {
                None
            };
            let ownership_explicit = ownership_from_source.is_some();

            let (param_name, param_name_span) = {
                let tok = self.consume_identifier("a parameter name")?;
                (tok.lexeme.clone(), tok.span)
            };

            // D-040: borrow is the implicit default for closure params too.
            let ownership = if ownership_from_source.is_none() {
                Some(OwnershipAnnotation::Borrow)
            } else {
                ownership_from_source
            };

            // Type annotation is optional for anonymous function params: `fn(req, res) { ... }`
            // When omitted, the param type is inferred as `any` (H-403).
            let (type_ref, type_span_end) = if self.check(TokenKind::Colon) {
                self.advance(); // consume `:`
                let tr = self.parse_type_ref()?;
                let end = tr.span();
                (tr, end)
            } else {
                // No annotation — use `any` so the body type-checks without errors
                (
                    TypeRef::Named("any".to_string(), param_name_span),
                    param_name_span,
                )
            };

            // Parse optional default value (B39-P05): `= expr`
            let (default_value, param_span_end) = if self.match_token(TokenKind::Equal) {
                let expr = self.parse_expression()?;
                let end = expr.span();
                (Some(Box::new(expr)), end)
            } else {
                (None, type_span_end)
            };

            params.push(Param {
                name: Identifier {
                    name: param_name,
                    span: param_name_span,
                },
                type_ref,
                ownership,
                ownership_explicit,
                mutable: false,
                default_value,
                is_rest: false,
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

    /// Parse a block as an expression: `{ stmt* }`
    fn parse_block_expr(&mut self) -> Result<Expr, ()> {
        let block = self.parse_block()?;
        Ok(Expr::Block(block))
    }

    /// Parse either an anonymous struct literal or a block expression.
    ///
    /// Disambiguation rule:
    /// - If `{` is followed by IDENTIFIER and then `:` or `,` or `}`, parse as anonymous struct.
    /// - Otherwise, parse as block expression.
    fn parse_block_or_anon_struct(&mut self) -> Result<Expr, ()> {
        if self.is_anonymous_struct_start() {
            self.parse_anonymous_struct_literal()
        } else {
            self.parse_block_expr()
        }
    }

    fn is_anonymous_struct_start(&self) -> bool {
        let next = self.peek_nth_nontrivia(1);
        let after = self.peek_nth_nontrivia(2);
        matches!(
            (next, after),
            (Some(next), Some(after))
                if next.kind == TokenKind::Identifier
                    && matches!(
                        after.kind,
                        TokenKind::Colon | TokenKind::Comma | TokenKind::RightBrace
                    )
        )
    }

    /// Parse an anonymous struct literal: `{ field: value, field }`
    fn parse_anonymous_struct_literal(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::LeftBrace, "Expected '{'")?.span;

        // H-086: Deprecate anonymous struct syntax — use `record { }` instead
        self.diagnostics.push(
            Diagnostic::warning(
                "Anonymous struct syntax `{ ... }` is deprecated. Use `record { ... }` instead.",
                start_span,
            )
            .with_help("Add the `record` keyword: `record { field: value, ... }`"),
        );

        if self.check(TokenKind::RightBrace) {
            let span = self.peek().span;
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(span)
                    .arg("detail", "anonymous struct literal requires at least one field")
                    .with_help("write `record { field: value, ... }` — provide at least one field inside the braces")
                    .with_note("use `()` or a named struct for zero-field types"),
            );
            self.consume(
                TokenKind::RightBrace,
                "Expected '}' after anonymous struct literal",
            )?;
            return Err(());
        }

        let mut entries = Vec::new();

        loop {
            let field_start = self.peek().span;
            let field_token = self.consume_identifier("a struct field name")?;
            let field_name = Identifier {
                name: field_token.lexeme.clone(),
                span: field_token.span,
            };

            let value = if self.match_token(TokenKind::Colon) {
                let expr = self.parse_expression()?;
                if let Expr::Identifier(id) = &expr {
                    if id.name == field_name.name {
                        self.diagnostics.push(
                            GENERIC_WARNING
                                .emit(field_name.span)
                                .arg(
                                    "detail",
                                    format!(
                                        "redundant field value for `{}`; shorthand is available",
                                        field_name.name
                                    ),
                                )
                                .with_help(format!("use `{{ {} }}` instead", field_name.name))
                                .build()
                                .with_label("shorthand available"),
                        );
                    }
                }
                expr
            } else {
                Expr::Identifier(field_name.clone())
            };

            let field_end = value.span();
            entries.push(ObjectEntry {
                key: field_name,
                value,
                span: field_start.merge(field_end),
            });

            if !self.match_token(TokenKind::Comma) {
                break;
            }
            if self.check(TokenKind::RightBrace) {
                break;
            }
        }

        let end_span = self
            .consume(
                TokenKind::RightBrace,
                "Expected '}' after anonymous struct literal",
            )?
            .span;

        Ok(Expr::ObjectLiteral(ObjectLiteral {
            entries,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse match expression
    fn parse_match_expr(&mut self) -> Result<Expr, ()> {
        use crate::ast::MatchExpr;

        let start_span = self.consume(TokenKind::Match, "Expected 'match'")?.span;

        // Parse scrutinee (the expression being matched)
        let scrutinee = self.parse_expression()?;

        // Parse match block
        self.consume(TokenKind::LeftBrace, "Expected '{' after match expression")?;

        // Parse match arms
        let mut arms = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);

            if self.check(TokenKind::RightBrace) {
                break;
            }

            // Arms are separated by commas or semicolons; trailing separator optional.
            if self.match_token(TokenKind::Comma) || self.match_token(TokenKind::Semicolon) {
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            } else {
                let span = self.peek().span;
                self.emit_descriptor(
                    SYNTAX_ERROR
                        .emit(span)
                        .arg("detail", "expected `,` or `;` after match arm")
                        .with_help("separate match arms with `,` or `;`: `pattern => expr,`")
                        .with_note("a trailing separator before `}` is optional"),
                );
                return Err(());
            }
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after match arms")?
            .span;

        if arms.is_empty() {
            let span = self.peek().span;
            self.emit_descriptor(
                SYNTAX_ERROR
                    .emit(span)
                    .arg("detail", "match expression must have at least one arm")
                    .with_help(
                        "write `match x { pattern => expr, ... }` — at least one arm is required",
                    )
                    .with_note("a match with no arms cannot produce a value or handle any case"),
            );
            return Err(());
        }

        Ok(Expr::Match(MatchExpr {
            scrutinee: Box::new(scrutinee),
            arms,
            span: start_span.merge(end_span),
        }))
    }

    /// Parse match arm (pattern => expression)
    fn parse_match_arm(&mut self) -> Result<MatchArm, ()> {
        use crate::ast::MatchArm;

        let pattern = self.parse_or_pattern()?;
        let pattern_span = pattern.span();

        // Parse optional guard clause: `pattern if <expr> => body`
        let guard = if self.match_token(TokenKind::If) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };

        self.consume(TokenKind::FatArrow, "Expected '=>' after pattern")?;

        // H-114: allow `return expr` as a match arm body by wrapping it in a Block expression
        // H-407: also allow `continue` and `break` as match arm bodies
        let body = if self.check(TokenKind::Continue) {
            let span = self.peek().span;
            self.advance(); // consume `continue`
                            // No semicolon: match arm delimiter is `,` or `}`
            Expr::Block(Block {
                statements: vec![Stmt::Continue(span)],
                tail_expr: None,
                span,
            })
        } else if self.check(TokenKind::Break) {
            let span = self.peek().span;
            self.advance(); // consume `break`
                            // No semicolon: match arm delimiter is `,` or `}`
            Expr::Block(Block {
                statements: vec![Stmt::Break(span)],
                tail_expr: None,
                span,
            })
        } else if self.check(TokenKind::Return) {
            let ret_start = self.peek().span;
            self.advance(); // consume `return`
            let value = if !self.check(TokenKind::Comma) && !self.check(TokenKind::RightBrace) {
                Some(self.parse_expression()?)
            } else {
                None
            };
            let ret_span = value
                .as_ref()
                .map(|e: &Expr| ret_start.merge(e.span()))
                .unwrap_or(ret_start);
            Expr::Block(Block {
                statements: vec![Stmt::Return(ReturnStmt {
                    value,
                    span: ret_span,
                })],
                tail_expr: None,
                span: ret_span,
            })
        } else {
            self.parse_expression()?
        };
        let body_span = body.span();

        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: pattern_span.merge(body_span),
        })
    }

    /// Parse OR pattern: primary | primary | ...
    fn parse_or_pattern(&mut self) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        let first = self.parse_pattern()?;
        let start_span = first.span();

        // If no `|` follows, return single pattern (no wrapping)
        if !self.check(TokenKind::Pipe) {
            return Ok(first);
        }

        let mut alternatives = vec![first];
        while self.match_token(TokenKind::Pipe) {
            alternatives.push(self.parse_pattern()?);
        }

        let end_span = alternatives.last().unwrap().span();
        Ok(Pattern::Or(alternatives, start_span.merge(end_span)))
    }

    /// Parse pattern
    fn parse_pattern(&mut self) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        match self.peek().kind {
            // Literal patterns: numbers, strings, bools, null
            TokenKind::Number => {
                let token = self.advance();
                let span = token.span;
                let lexeme = token.lexeme.clone();
                let value: f64 = match lexeme.parse::<f64>() {
                    Ok(value) if value.is_finite() => value,
                    _ => {
                        self.emit_descriptor(INVALID_NUMBER.emit(span).arg("literal", &lexeme));
                        0.0
                    }
                };
                Ok(Pattern::Literal(Literal::Number(value), span))
            }
            TokenKind::String => {
                let token = self.advance();
                Ok(Pattern::Literal(
                    Literal::String(token.lexeme.clone()),
                    token.span,
                ))
            }
            TokenKind::True => {
                let token = self.advance();
                Ok(Pattern::Literal(Literal::Bool(true), token.span))
            }
            TokenKind::False => {
                let token = self.advance();
                Ok(Pattern::Literal(Literal::Bool(false), token.span))
            }
            TokenKind::Null => {
                let token = self.advance();
                Ok(Pattern::Literal(Literal::Null, token.span))
            }

            // Wildcard pattern: _
            TokenKind::Underscore => {
                let token = self.advance();
                Ok(Pattern::Wildcard(token.span))
            }

            // Tuple pattern: (p1, p2, ...)
            TokenKind::LeftParen => {
                let start_span = self.peek().span;
                self.advance(); // consume '('
                let mut elements = Vec::new();
                while !self.check(TokenKind::RightParen) && !self.is_at_end() {
                    elements.push(self.parse_pattern()?);
                    if self.check(TokenKind::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                let end_tok =
                    self.consume(TokenKind::RightParen, "Expected ')' after tuple pattern")?;
                Ok(Pattern::Tuple {
                    elements,
                    span: start_span.merge(end_tok.span),
                })
            }

            // Array pattern: [...]
            TokenKind::LeftBracket => self.parse_array_pattern(),

            // Constructor pattern or variable binding: Identifier or Identifier(...)
            // Also handles enum variant patterns: EnumName::VariantName
            TokenKind::Identifier => {
                let id_token = self.advance();
                let id = Identifier {
                    name: id_token.lexeme.clone(),
                    span: id_token.span,
                };

                // Check for enum variant pattern: EnumName::VariantName
                if self.check(TokenKind::ColonColon) {
                    self.advance(); // consume ::
                    let variant_token =
                        self.consume(TokenKind::Identifier, "Expected variant name after '::'")?;
                    let variant_id = Identifier {
                        name: variant_token.lexeme.clone(),
                        span: variant_token.span,
                    };

                    // Check for arguments: EnumName::VariantName(args)
                    if self.check(TokenKind::LeftParen) {
                        // Parse tuple variant pattern with arguments
                        self.advance(); // consume (
                        let mut args = Vec::new();
                        if !self.check(TokenKind::RightParen) {
                            loop {
                                args.push(self.parse_pattern()?);
                                if !self.match_token(TokenKind::Comma) {
                                    break;
                                }
                            }
                        }
                        let end_span =
                            self.consume(TokenKind::RightParen, "Expected ')' after arguments")?;
                        let span = id.span.merge(end_span.span);
                        Ok(Pattern::EnumVariant {
                            enum_name: id,
                            variant_name: variant_id,
                            args,
                            span,
                        })
                    } else {
                        // Unit enum variant pattern (no arguments)
                        let span = id.span.merge(variant_id.span);
                        Ok(Pattern::EnumVariant {
                            enum_name: id,
                            variant_name: variant_id,
                            args: Vec::new(),
                            span,
                        })
                    }
                } else if self.check(TokenKind::LeftBrace) {
                    // Struct pattern: TypeName { field, field: sub_pattern, ... }
                    self.parse_struct_pattern(Some(id))
                } else if self.check(TokenKind::LeftParen) {
                    // Built-in constructors (Ok, Err, Some) stay as Pattern::Constructor.
                    // User-defined uppercase variants with args become Pattern::BareVariant.
                    if matches!(id.name.as_str(), "Ok" | "Err" | "Some") {
                        self.parse_constructor_pattern(id)
                    } else if id.name.starts_with(|c: char| c.is_uppercase()) {
                        // Bare user enum variant with args: Pending(msg), Success(x)
                        let name_span = id.span;
                        self.advance(); // consume (
                        let mut args = Vec::new();
                        if !self.check(TokenKind::RightParen) {
                            loop {
                                args.push(self.parse_or_pattern()?);
                                if !self.match_token(TokenKind::Comma) {
                                    break;
                                }
                            }
                        }
                        let end_span = self
                            .consume(
                                TokenKind::RightParen,
                                "Expected ')' after variant arguments",
                            )?
                            .span;
                        Ok(Pattern::BareVariant {
                            name: id,
                            args,
                            span: name_span.merge(end_span),
                        })
                    } else {
                        self.parse_constructor_pattern(id)
                    }
                } else {
                    // Built-in zero-arg constructors and uppercase user variants
                    if id.name == "None" {
                        Ok(Pattern::Constructor {
                            name: id.clone(),
                            args: Vec::new(),
                            span: id.span,
                        })
                    } else if id.name.starts_with(|c: char| c.is_uppercase()) {
                        // Bare user enum variant (unit): Active, Inactive, Running
                        let span = id.span;
                        Ok(Pattern::BareVariant {
                            name: id,
                            args: Vec::new(),
                            span,
                        })
                    } else {
                        // Lowercase → variable binding
                        Ok(Pattern::Variable(id))
                    }
                }
            }

            _ => {
                let span = self.peek().span;
                self.emit_descriptor(
                    SYNTAX_ERROR
                        .emit(span)
                        .arg("detail", "expected a pattern")
                        .with_help("valid patterns: literals, identifiers, `_`, tuple patterns `(a, b)`, struct patterns `Point { x, y }`, or enum variants")
                        .with_note("patterns appear on the left side of `=>` in match arms"),
                );
                Err(())
            }
        }
    }

    /// Parse array pattern: [pattern, pattern, ...]
    fn parse_array_pattern(&mut self) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        let start_span = self.consume(TokenKind::LeftBracket, "Expected '['")?.span;
        let mut elements = Vec::new();

        if !self.check(TokenKind::RightBracket) {
            loop {
                elements.push(self.parse_pattern()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightBracket, "Expected ']'")?.span;

        Ok(Pattern::Array {
            elements,
            span: start_span.merge(end_span),
        })
    }

    /// Parse struct pattern: `TypeName { field, field: sub_pattern }` or anonymous `{ field }`.
    ///
    /// `type_name` is `Some(id)` for named structs, `None` for anonymous record patterns.
    fn parse_struct_pattern(
        &mut self,
        type_name: Option<Identifier>,
    ) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::{Pattern, StructFieldPattern};

        let start_span = type_name
            .as_ref()
            .map(|id| id.span)
            .unwrap_or_else(|| self.peek().span);

        self.consume(TokenKind::LeftBrace, "Expected '{' in struct pattern")?;

        let mut fields = Vec::new();

        if !self.check(TokenKind::RightBrace) {
            loop {
                let field_tok = self.consume_identifier("a struct field name in pattern")?;
                let field_name = Identifier {
                    name: field_tok.lexeme.clone(),
                    span: field_tok.span,
                };
                let field_span = field_tok.span;

                // `field: sub_pattern` or shorthand `field`
                let sub_pattern = if self.check(TokenKind::Colon) {
                    self.advance(); // consume ':'
                    Some(self.parse_or_pattern()?)
                } else {
                    None
                };

                let end_span = sub_pattern.as_ref().map(|p| p.span()).unwrap_or(field_span);

                fields.push(StructFieldPattern {
                    name: field_name,
                    pattern: sub_pattern,
                    span: field_span.merge(end_span),
                });

                if !self.match_token(TokenKind::Comma) {
                    break;
                }
                // Allow trailing comma
                if self.check(TokenKind::RightBrace) {
                    break;
                }
            }
        }

        let end_tok = self.consume(TokenKind::RightBrace, "Expected '}' after struct pattern")?;
        let span = start_span.merge(end_tok.span);

        Ok(Pattern::Struct {
            type_name,
            fields,
            span,
        })
    }

    /// Parse constructor pattern: Name(pattern, pattern, ...)
    fn parse_constructor_pattern(&mut self, name: Identifier) -> Result<crate::ast::Pattern, ()> {
        use crate::ast::Pattern;

        let name_span = name.span;
        self.consume(TokenKind::LeftParen, "Expected '('")?;

        let mut args = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_or_pattern()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }

        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;

        Ok(Pattern::Constructor {
            name,
            args,
            span: name_span.merge(end_span),
        })
    }
}
