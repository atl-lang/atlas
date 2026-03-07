//! Expression parsing (Pratt parsing)

use super::E_BAD_NUMBER;
use crate::ast::*;
use crate::diagnostic::error_codes;
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
            TokenKind::If => self.parse_if_expr(),
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::Fn => self.parse_anon_fn(),
            TokenKind::Range | TokenKind::RangeInclusive => self.parse_range_prefix(),
            _ => {
                self.error("Expected expression");
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
                self.error_at_with_code(
                    E_BAD_NUMBER,
                    &format!("Invalid number literal: '{}'", lexeme),
                    span,
                );
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
        let expr = self.parse_expression()?;
        let end_span = self.consume(TokenKind::RightParen, "Expected ')'")?.span;

        Ok(Expr::Group(GroupExpr {
            expr: Box::new(expr),
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
            self.diagnostics.push(Diagnostic::error(
                "expected expression after `await`",
                await_span,
            ));
            return Err(());
        }

        let expr = self.parse_precedence(Precedence::Unary)?;
        let end_span = expr.span();
        Ok(Expr::Await {
            expr: Box::new(expr),
            span: await_span.merge(end_span),
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
            span: callee_span.merge(end_span),
        }))
    }

    /// Parse index expression
    fn parse_index(&mut self, target: Expr) -> Result<Expr, ()> {
        let target_span = target.span();
        self.consume(TokenKind::LeftBracket, "Expected '['")?;
        if self.check(TokenKind::RightBracket) {
            self.error("Expected expression inside '[]'");
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
            self.error("Inclusive range requires an end expression");
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
            self.error("Inclusive range requires an end expression");
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
        )
    }

    /// Parse member expression (method call or property access)
    fn parse_member(&mut self, target: Expr) -> Result<Expr, ()> {
        let target_span = target.span();
        self.consume(TokenKind::Dot, "Expected '.'")?;

        // Member name must be an identifier
        let member_token = self.consume_identifier("a method or property name")?;
        let member = Identifier {
            name: member_token.lexeme.clone(),
            span: member_token.span,
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
            type_tag: std::cell::Cell::new(None),
            trait_dispatch: std::cell::RefCell::new(None),
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

    /// Parse primary type (named, generic, grouped, or function), plus array suffixes.
    fn parse_type_primary(&mut self) -> Result<TypeRef, ()> {
        let mut type_ref = if self.check(TokenKind::LeftParen) {
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

        // Handle array type syntax: type[]
        loop {
            if !self.match_token(TokenKind::LeftBracket) {
                break;
            }
            let rbracket_token = self.consume(
                TokenKind::RightBracket,
                "Expected ']' after '[' in array type",
            )?;
            let end_span = rbracket_token.span;
            let full_span = type_ref.span().merge(end_span);
            type_ref = TypeRef::Array(Box::new(type_ref), full_span);
        }

        Ok(type_ref)
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
            self.error("Structural type must include at least one member");
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

        if self.check(TokenKind::RightParen) {
            self.consume(TokenKind::RightParen, "Expected ')'")?;
            if self.match_token(TokenKind::Arrow) {
                let return_type = self.parse_type_ref()?;
                let full_span = start_span.merge(return_type.span());
                return Ok(TypeRef::Function {
                    params: Vec::new(),
                    return_type: Box::new(return_type),
                    span: full_span,
                });
            }
            self.error("Unexpected empty type group");
            return Err(());
        }

        let mut params = Vec::new();
        params.push(self.parse_type_ref()?);

        while self.match_token(TokenKind::Comma) {
            params.push(self.parse_type_ref()?);
        }

        self.consume(TokenKind::RightParen, "Expected ')' after type list")?;

        if self.match_token(TokenKind::Arrow) {
            let return_type = self.parse_type_ref()?;
            let full_span = start_span.merge(return_type.span());
            return Ok(TypeRef::Function {
                params,
                return_type: Box::new(return_type),
                span: full_span,
            });
        }

        if params.len() == 1 {
            return Ok(params.remove(0));
        }

        self.error("Expected '->' after function type parameters");
        Err(())
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
            self.error("Generic type requires at least one type argument");
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

    // parse_function_type removed in favor of parse_paren_type

    /// Parse anonymous function expression: `fn(params) -> ReturnType { body }`
    ///
    /// Anonymous functions allow optional type annotations on parameters.
    /// This is the unified syntax after arrow functions were removed.
    fn parse_anon_fn(&mut self) -> Result<Expr, ()> {
        let start_span = self.consume(TokenKind::Fn, "Expected 'fn'")?.span;
        self.consume(TokenKind::LeftParen, "Expected '(' after 'fn'")?;
        let params = self.parse_anon_fn_params()?;
        self.consume(TokenKind::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_token(TokenKind::Arrow) {
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

            let param_name_tok = self.consume_identifier("a parameter name")?;
            let param_name = param_name_tok.lexeme.clone();
            let param_name_span = param_name_tok.span;

            // Type annotation is required: `param: Type`
            self.consume(TokenKind::Colon, "Expected ':' after parameter name")?;
            let type_ref = self.parse_type_ref()?;
            let param_span_end = type_ref.span();

            params.push(Param {
                name: Identifier {
                    name: param_name,
                    span: param_name_span,
                },
                type_ref,
                ownership,
                mutable: false,
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
            self.error("Anonymous struct literal requires at least one field");
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
                            Diagnostic::warning_with_code(
                                error_codes::GENERIC_WARNING,
                                format!(
                                    "Redundant field value for '{}'; shorthand is available",
                                    field_name.name
                                ),
                                field_name.span,
                            )
                            .with_label("shorthand available")
                            .with_help(format!("use `{{ {} }}` instead", field_name.name)),
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
                self.error("Expected ',' or ';' after match arm");
                return Err(());
            }
        }

        let end_span = self
            .consume(TokenKind::RightBrace, "Expected '}' after match arms")?
            .span;

        if arms.is_empty() {
            self.error("Match expression must have at least one arm");
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
        let body = if self.check(TokenKind::Return) {
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
                        self.error_at_with_code(
                            E_BAD_NUMBER,
                            &format!("Invalid number literal: '{}'", lexeme),
                            span,
                        );
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
                } else if self.check(TokenKind::LeftParen) {
                    // Check if this is a constructor pattern (has arguments)
                    self.parse_constructor_pattern(id)
                } else {
                    // Check if this is a zero-argument constructor (None, unit-like variants)
                    // For now, recognize built-in constructors: None
                    if id.name == "None" {
                        // Zero-argument constructor
                        Ok(Pattern::Constructor {
                            name: id.clone(),
                            args: Vec::new(),
                            span: id.span,
                        })
                    } else {
                        // Variable binding pattern
                        Ok(Pattern::Variable(id))
                    }
                }
            }

            _ => {
                self.error("Expected pattern");
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
