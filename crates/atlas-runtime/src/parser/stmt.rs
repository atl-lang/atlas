//! Statement parsing

use crate::ast::*;
use crate::diagnostic::error_codes::{
    FOREIGN_SYNTAX_CLASS, FOREIGN_SYNTAX_ECHO, FOREIGN_SYNTAX_FUNCTION_KW,
    FOREIGN_SYNTAX_IMPORT_FROM, FOREIGN_SYNTAX_INCREMENT, FOREIGN_SYNTAX_VAR,
    INVALID_ASSIGN_TARGET, INVALID_ASSIGN_TARGET_CALL, INVALID_ASSIGN_TARGET_MEMBER,
    INVALID_ASSIGN_TARGET_RANGE,
};
use crate::diagnostic::Diagnostic;
use crate::parser::Parser;
use crate::token::TokenKind;

impl Parser {
    /// Parse a statement
    pub(super) fn parse_statement(&mut self) -> Result<Stmt, ()> {
        // Cross-language pattern detection: catch common foreign syntax before main dispatch.
        if self.peek().kind == TokenKind::Identifier {
            // Capture span NOW — before any advance() — so all errors point to the foreign token.
            let token_span = self.peek().span;
            let lexeme = self.peek().lexeme.clone();
            match lexeme.as_str() {
                "echo" => {
                    self.emit_descriptor(FOREIGN_SYNTAX_ECHO.emit(token_span));
                    return Err(());
                }
                "var" => {
                    self.emit_descriptor(FOREIGN_SYNTAX_VAR.emit(token_span));
                    return Err(());
                }
                "function" => {
                    self.emit_descriptor(FOREIGN_SYNTAX_FUNCTION_KW.emit(token_span));
                    return Err(());
                }
                "class" => {
                    self.emit_descriptor(FOREIGN_SYNTAX_CLASS.emit(token_span));
                    return Err(());
                }
                // "console" is a valid Atlas namespace (B21) — no foreign syntax trap
                _ => {}
            }

            // Detect `x++` and `x--` increment/decrement patterns
            let next1 = self.peek_nth_nontrivia(1).map(|t| t.kind);
            let next2 = self.peek_nth_nontrivia(2).map(|t| t.kind);
            let is_increment = next1 == Some(TokenKind::Plus) && next2 == Some(TokenKind::Plus);
            let is_decrement = next1 == Some(TokenKind::Minus) && next2 == Some(TokenKind::Minus);
            if is_increment || is_decrement {
                let op = if is_increment { "++" } else { "--" };
                self.emit_descriptor(FOREIGN_SYNTAX_INCREMENT.emit(token_span).arg("op", op));
                return Err(());
            }
        }

        match self.peek().kind {
            TokenKind::Let => self.parse_var_decl(),
            TokenKind::If => self.parse_if_stmt(),
            TokenKind::While => self.parse_while_stmt(),
            TokenKind::For => self.parse_for_in_stmt(),
            TokenKind::Return => self.parse_return_stmt(),
            TokenKind::Break => self.parse_break_stmt(),
            TokenKind::Continue => self.parse_continue_stmt(),
            TokenKind::LeftBrace => {
                // Standalone block statement - wrap as Expr::Block
                let block = self.parse_block()?;
                let span = block.span;
                Ok(Stmt::Expr(ExprStmt {
                    expr: Expr::Block(block),
                    span,
                }))
            }
            TokenKind::Match => {
                // Match at statement position — no trailing semicolon required
                // (consistent with if/while/for which also end with `}`)
                let expr = self.parse_expression()?;
                let span = expr.span();
                Ok(Stmt::Expr(ExprStmt { expr, span }))
            }
            TokenKind::Fn => Ok(Stmt::FunctionDecl(self.parse_function()?)),
            TokenKind::Import => {
                let import_span = self.peek().span;
                self.emit_descriptor(FOREIGN_SYNTAX_IMPORT_FROM.emit(import_span));
                Err(())
            }
            _ => self.parse_assign_or_expr_stmt(),
        }
    }

    /// Parse a variable declaration
    pub(super) fn parse_var_decl(&mut self) -> Result<Stmt, ()> {
        let keyword_span = self.peek().span;
        self.advance(); // consume 'let'

        // Check for `let mut`
        let mutable = if self.peek().kind == TokenKind::Mut {
            self.advance(); // consume 'mut'
            true // let mut x = ... (mutable)
        } else {
            false // let x = ... (immutable)
        };

        // Tuple destructuring: `let (a, b) = expr;` or `let mut (a, b) = expr;`
        if self.check(TokenKind::LeftParen) {
            self.advance(); // consume '('
            let mut names = Vec::new();
            loop {
                if self.check(TokenKind::RightParen) {
                    break;
                }
                let tok = self.consume_identifier("a variable name in tuple pattern")?;
                names.push(Identifier {
                    name: tok.lexeme.clone(),
                    span: tok.span,
                });
                if self.check(TokenKind::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
            self.consume(TokenKind::RightParen, "Expected ')' after tuple pattern")?;
            self.consume(
                TokenKind::Equal,
                "Expected '=' in destructuring declaration",
            )?;
            let init = self.parse_expression()?;
            let end_span = self
                .consume(
                    TokenKind::Semicolon,
                    "Expected ';' after destructuring declaration",
                )?
                .span;
            return Ok(Stmt::LetDestructure(LetDestructure {
                mutable,
                names,
                init,
                span: keyword_span.merge(end_span),
            }));
        }

        let name_token = self.consume_identifier("a variable name")?;
        let name = Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        let type_ref = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };

        self.consume(TokenKind::Equal, "Expected '=' in variable declaration")?;
        let init = self.parse_expression()?;
        let end_span = self
            .consume(
                TokenKind::Semicolon,
                "Expected ';' after variable declaration",
            )?
            .span;

        Ok(Stmt::VarDecl(VarDecl {
            mutable,
            uses_deprecated_var: false, // var keyword removed (D-001)
            name,
            type_ref,
            init,
            span: keyword_span.merge(end_span),
            needs_drop: std::cell::RefCell::new(None),
        }))
    }

    /// Parse assignment or expression statement
    pub(super) fn parse_assign_or_expr_stmt(&mut self) -> Result<Stmt, ()> {
        let expr = self.parse_expression()?;
        let expr_span = expr.span();

        // Check what follows the expression
        let next_kind = self.peek().kind;

        match next_kind {
            // Regular assignment: x = value
            TokenKind::Equal => {
                self.advance(); // consume =
                let target = self.expr_to_assign_target(expr)?;
                let value = self.parse_expression()?;
                let end_span = self
                    .consume(TokenKind::Semicolon, "Expected ';' after assignment")?
                    .span;

                Ok(Stmt::Assign(Assign {
                    target,
                    value,
                    span: expr_span.merge(end_span),
                }))
            }

            // Compound assignment: x += value, x -= value, etc.
            TokenKind::PlusEqual
            | TokenKind::MinusEqual
            | TokenKind::StarEqual
            | TokenKind::SlashEqual
            | TokenKind::PercentEqual => {
                let op_token = self.advance();
                let op = match op_token.kind {
                    TokenKind::PlusEqual => CompoundOp::AddAssign,
                    TokenKind::MinusEqual => CompoundOp::SubAssign,
                    TokenKind::StarEqual => CompoundOp::MulAssign,
                    TokenKind::SlashEqual => CompoundOp::DivAssign,
                    TokenKind::PercentEqual => CompoundOp::ModAssign,
                    _ => unreachable!(),
                };

                let target = self.expr_to_assign_target(expr)?;
                let value = self.parse_expression()?;
                let end_span = self
                    .consume(
                        TokenKind::Semicolon,
                        "Expected ';' after compound assignment",
                    )?
                    .span;

                Ok(Stmt::CompoundAssign(CompoundAssign {
                    target,
                    op,
                    value,
                    span: expr_span.merge(end_span),
                }))
            }

            // Expression statement
            _ => {
                let end_span = self
                    .consume(TokenKind::Semicolon, "Expected ';' after expression")?
                    .span;
                Ok(Stmt::Expr(ExprStmt {
                    expr,
                    span: expr_span.merge(end_span),
                }))
            }
        }
    }

    /// Convert an expression to an assignment target
    pub(super) fn expr_to_assign_target(&mut self, expr: Expr) -> Result<AssignTarget, ()> {
        match expr {
            Expr::Identifier(ident) => Ok(AssignTarget::Name(ident)),
            Expr::Index(idx) => match idx.index {
                IndexValue::Single(index) => {
                    if matches!(index.as_ref(), Expr::Range { .. }) {
                        let span = self.peek().span;
                        self.emit_descriptor(INVALID_ASSIGN_TARGET_RANGE.emit(span));
                        return Err(());
                    }
                    Ok(AssignTarget::Index {
                        target: idx.target,
                        index,
                        span: idx.span,
                    })
                }
            },
            Expr::Member(member) => {
                if member.args.is_some() {
                    let span = self.peek().span;
                    self.emit_descriptor(INVALID_ASSIGN_TARGET_CALL.emit(span));
                    return Err(());
                }
                if !matches!(
                    member.target.as_ref(),
                    Expr::Identifier(_) | Expr::Member(_) | Expr::Index(_)
                ) {
                    let span = self.peek().span;
                    self.emit_descriptor(INVALID_ASSIGN_TARGET_MEMBER.emit(span));
                    return Err(());
                }
                Ok(AssignTarget::Member {
                    target: member.target,
                    member: member.member,
                    span: member.span,
                })
            }
            _ => {
                let span = self.peek().span;
                self.emit_descriptor(INVALID_ASSIGN_TARGET.emit(span));
                Err(())
            }
        }
    }

    /// Parse if statement
    pub(super) fn parse_if_stmt(&mut self) -> Result<Stmt, ()> {
        let if_span = self.consume(TokenKind::If, "Expected 'if'")?.span;

        let cond = self.parse_condition("if")?;

        let then_block = self.parse_block()?;
        let then_span = then_block.span;

        let else_block = if self.match_token(TokenKind::Else) {
            // Handle `else if` chains: wrap the nested if-statement in a synthetic block
            if self.check(TokenKind::If) {
                let nested_if = self.parse_if_stmt()?;
                let nested_span = match &nested_if {
                    Stmt::If(if_stmt) => if_stmt.span,
                    _ => unreachable!(),
                };
                Some(Block {
                    statements: vec![nested_if],
                    tail_expr: None,
                    span: nested_span,
                })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };

        let end_span = else_block.as_ref().map_or(then_span, |b| b.span);

        Ok(Stmt::If(IfStmt {
            cond,
            then_block,
            else_block,
            span: if_span.merge(end_span),
        }))
    }

    /// Parse while statement
    pub(super) fn parse_while_stmt(&mut self) -> Result<Stmt, ()> {
        let while_span = self.consume(TokenKind::While, "Expected 'while'")?.span;

        let cond = self.parse_condition("while")?;

        let body = self.parse_block()?;
        let body_span = body.span;

        Ok(Stmt::While(WhileStmt {
            cond,
            body,
            span: while_span.merge(body_span),
        }))
    }

    /// Parse for-in statement
    ///
    /// Syntax: `for item in array { body }`
    pub(super) fn parse_for_in_stmt(&mut self) -> Result<Stmt, ()> {
        let for_span = self.consume(TokenKind::For, "Expected 'for'")?.span;
        let paren_span = self.peek().span;
        let has_parens = self.match_token(TokenKind::LeftParen);
        if has_parens {
            self.diagnostics.push(
                Diagnostic::warning(
                    "Unnecessary parentheses around `for` clause. Atlas uses Rust-style syntax: `for x in iter { }`",
                    paren_span,
                )
                .with_help("Remove the parentheses: `for <var> in <iter> { }`"),
            );
        }

        // Parse variable name
        let name_token = self.consume_identifier("variable name after 'for'")?;
        let variable = Identifier {
            name: name_token.lexeme.clone(),
            span: name_token.span,
        };

        // Expect 'in' keyword
        self.consume(TokenKind::In, "Expected 'in' after variable name")?;

        // Parse iterable expression
        let iterable = Box::new(self.parse_expression()?);

        if has_parens {
            self.consume(TokenKind::RightParen, "Expected ')' after for-in clause")?;
        }

        // Parse body block
        let body = self.parse_block()?;
        let body_span = body.span;

        Ok(Stmt::ForIn(ForInStmt {
            variable,
            iterable,
            body,
            span: for_span.merge(body_span),
        }))
    }

    fn parse_condition(&mut self, keyword: &str) -> Result<Expr, ()> {
        let paren_span = self.peek().span;
        let has_parens = self.match_token(TokenKind::LeftParen);
        if has_parens {
            self.diagnostics.push(
                Diagnostic::warning(
                    format!("Unnecessary parentheses around `{keyword}` condition. Atlas uses Rust-style syntax: `{keyword} expr {{ }}`"),
                    paren_span,
                )
                .with_help(format!("Remove the parentheses: `{keyword} <condition> {{ }}`")),
            );
        }
        // Prevent `Identifier {` from being parsed as a struct literal inside conditions.
        // e.g. `if FOO { ... }` — FOO is a variable, `{` is the then-block, not a struct.
        let prev = self.no_struct_literal;
        self.no_struct_literal = true;
        let cond = self.parse_expression();
        self.no_struct_literal = prev;
        let cond = cond?;
        if has_parens {
            self.consume(
                TokenKind::RightParen,
                &format!("Expected ')' after {keyword} condition"),
            )?;
        }
        Ok(cond)
    }

    /// Parse return statement
    pub(super) fn parse_return_stmt(&mut self) -> Result<Stmt, ()> {
        let return_span = self.consume(TokenKind::Return, "Expected 'return'")?.span;

        let value = if !self.check(TokenKind::Semicolon) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after return")?
            .span;

        Ok(Stmt::Return(ReturnStmt {
            value,
            span: return_span.merge(end_span),
        }))
    }

    /// Parse break statement
    pub(super) fn parse_break_stmt(&mut self) -> Result<Stmt, ()> {
        let break_span = self.consume(TokenKind::Break, "Expected 'break'")?.span;
        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after break")?
            .span;
        Ok(Stmt::Break(break_span.merge(end_span)))
    }

    /// Parse continue statement
    pub(super) fn parse_continue_stmt(&mut self) -> Result<Stmt, ()> {
        let continue_span = self
            .consume(TokenKind::Continue, "Expected 'continue'")?
            .span;
        let end_span = self
            .consume(TokenKind::Semicolon, "Expected ';' after continue")?
            .span;
        Ok(Stmt::Continue(continue_span.merge(end_span)))
    }

    /// Parse a block with support for implicit returns (Rust-style tail expressions)
    ///
    /// If the last item in a block is an expression without a trailing semicolon,
    /// it becomes the block's value (tail expression) rather than a statement.
    pub(super) fn parse_block(&mut self) -> Result<Block, ()> {
        let start_span = self.consume(TokenKind::LeftBrace, "Expected '{'")?.span;
        let mut statements = Vec::new();
        let mut tail_expr: Option<Box<Expr>> = None;

        while !self.check(TokenKind::RightBrace) && !self.is_at_end() {
            // Try to detect tail expression: if this could be an expression and
            // is followed by `}` (no semicolon), it's the block's return value
            if let Some(expr) = self.try_parse_tail_expression()? {
                tail_expr = Some(Box::new(expr));
                break;
            }

            // Otherwise, parse a regular statement
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => self.synchronize(),
            }
        }

        let end_span = self.consume(TokenKind::RightBrace, "Expected '}'")?.span;

        Ok(Block {
            statements,
            tail_expr,
            span: start_span.merge(end_span),
        })
    }

    /// Try to parse a tail expression (expression followed by `}` with no semicolon)
    ///
    /// Returns Some(expr) if successful, None if this isn't a tail expression context.
    /// Uses backtracking to avoid consuming tokens if it's not a tail expression.
    fn try_parse_tail_expression(&mut self) -> Result<Option<Expr>, ()> {
        // Skip tokens that definitely start statements, not expressions
        match self.peek().kind {
            TokenKind::Let
            | TokenKind::Const
            | TokenKind::If
            | TokenKind::While
            | TokenKind::For
            | TokenKind::Return
            | TokenKind::Break
            | TokenKind::Continue
            | TokenKind::Fn => return Ok(None),
            _ => {}
        }

        // Save position for potential backtrack
        let saved_pos = self.current;
        let saved_diag_len = self.diagnostics.len();

        // Try to parse an expression
        let expr = match self.parse_expression() {
            Ok(e) => e,
            Err(_) => {
                // Not an expression, restore and let parse_statement handle it
                self.current = saved_pos;
                self.diagnostics.truncate(saved_diag_len);
                return Ok(None);
            }
        };

        // Check what follows the expression
        match self.peek().kind {
            TokenKind::RightBrace => {
                // Expression followed by `}` = tail expression (implicit return)
                Ok(Some(expr))
            }
            TokenKind::Semicolon => {
                // Expression followed by `;` = regular statement, backtrack
                self.current = saved_pos;
                self.diagnostics.truncate(saved_diag_len);
                Ok(None)
            }
            TokenKind::Equal
            | TokenKind::PlusEqual
            | TokenKind::MinusEqual
            | TokenKind::StarEqual
            | TokenKind::SlashEqual
            | TokenKind::PercentEqual => {
                // Assignment or compound assignment, backtrack
                self.current = saved_pos;
                self.diagnostics.truncate(saved_diag_len);
                Ok(None)
            }
            _ => {
                // Unexpected token after expression - let parse_statement produce error
                self.current = saved_pos;
                self.diagnostics.truncate(saved_diag_len);
                Ok(None)
            }
        }
    }
}
