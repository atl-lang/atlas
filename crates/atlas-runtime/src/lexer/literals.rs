//! Literal parsing for the lexer

use crate::lexer::Lexer;
use crate::lexer::{InterpolationContext, InterpolationKind};
use crate::span::Span;
use crate::token::{Token, TokenKind};

enum StringScan {
    Complete(Token),
    Interpolation { part: Token, interp_span: Span },
    Error(Token),
}

impl Lexer {
    /// Scan a string literal
    pub(super) fn string(&mut self) -> Token {
        let part_start_pos = self.start_pos;
        match self.scan_string_part(part_start_pos) {
            StringScan::Complete(token) => token,
            StringScan::Interpolation { part, interp_span } => {
                self.pending_tokens.push_back(Token::new(
                    TokenKind::InterpolationStart,
                    "${",
                    interp_span,
                ));
                self.interpolation_stack.push(InterpolationContext {
                    depth: 1,
                    kind: InterpolationKind::DoubleQuote,
                });
                part
            }
            StringScan::Error(token) => token,
        }
    }

    pub(super) fn queue_string_continuation(&mut self) {
        let part_start_pos = self.current;
        self.start_pos = part_start_pos;
        self.start_line = self.line;
        self.start_column = self.column;

        let next = match self.scan_string_part(part_start_pos) {
            StringScan::Complete(token) => {
                self.pending_tokens.push_back(token);
                return;
            }
            StringScan::Interpolation { part, interp_span } => {
                self.pending_tokens.push_back(part);
                self.pending_tokens.push_back(Token::new(
                    TokenKind::InterpolationStart,
                    "${",
                    interp_span,
                ));
                self.interpolation_stack.push(InterpolationContext {
                    depth: 1,
                    kind: InterpolationKind::DoubleQuote,
                });
                return;
            }
            StringScan::Error(token) => token,
        };

        self.pending_tokens.push_back(next);
    }

    pub(super) fn template_string(&mut self) -> Token {
        let part_start_pos = self.start_pos;
        match self.scan_template_string_part(part_start_pos) {
            StringScan::Complete(token) => token,
            StringScan::Interpolation { part, interp_span } => {
                self.pending_tokens.push_back(Token::new(
                    TokenKind::InterpolationStart,
                    "{",
                    interp_span,
                ));
                self.interpolation_stack.push(InterpolationContext {
                    depth: 1,
                    kind: InterpolationKind::Template,
                });
                part
            }
            StringScan::Error(token) => token,
        }
    }

    pub(super) fn queue_template_string_continuation(&mut self) {
        let part_start_pos = self.current;
        self.start_pos = part_start_pos;
        self.start_line = self.line;
        self.start_column = self.column;

        let next = match self.scan_template_string_part(part_start_pos) {
            StringScan::Complete(token) => {
                self.pending_tokens.push_back(token);
                return;
            }
            StringScan::Interpolation { part, interp_span } => {
                self.pending_tokens.push_back(part);
                self.pending_tokens.push_back(Token::new(
                    TokenKind::InterpolationStart,
                    "{",
                    interp_span,
                ));
                self.interpolation_stack.push(InterpolationContext {
                    depth: 1,
                    kind: InterpolationKind::Template,
                });
                return;
            }
            StringScan::Error(token) => token,
        };

        self.pending_tokens.push_back(next);
    }

    fn scan_string_part(&mut self, part_start_pos: usize) -> StringScan {
        let mut value = String::new();
        let mut has_error = false;
        let mut error_token = None;

        while !self.is_at_end() {
            let current_char = self.peek();

            if current_char == '"' {
                self.advance(); // Closing "
                let span = Span::new_in(part_start_pos, self.current, self.file);
                let token = Token::new(TokenKind::String, value, span);
                return if let Some(err) = error_token {
                    StringScan::Error(err)
                } else {
                    StringScan::Complete(token)
                };
            }

            if !has_error && current_char == '$' && self.peek_next() == Some('{') {
                let span = Span::new_in(part_start_pos, self.current, self.file);
                let part = Token::new(TokenKind::String, value, span);
                let interp_start = self.current;
                self.advance(); // $
                self.advance(); // {
                let interp_span = Span::new_in(interp_start, self.current, self.file);
                return StringScan::Interpolation { part, interp_span };
            }

            if current_char == '\n' {
                self.line += 1;
                self.column = 1;
            }

            if current_char == '\\' {
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return StringScan::Error(self.error_unterminated_string());
                }

                let escape_char = self.peek();
                match escape_char {
                    'n' => {
                        self.advance();
                        value.push('\n');
                    }
                    'r' => {
                        self.advance();
                        value.push('\r');
                    }
                    't' => {
                        self.advance();
                        value.push('\t');
                    }
                    '\\' => {
                        self.advance();
                        value.push('\\');
                    }
                    '"' => {
                        self.advance();
                        value.push('"');
                    }
                    '0' => {
                        self.advance();
                        value.push('\0');
                    }
                    'x' => {
                        self.advance(); // consume 'x'
                        match self.scan_hex_digits(2) {
                            Some(ch) => value.push(ch),
                            None => {
                                if !has_error {
                                    error_token = Some(self.error_invalid_escape('x'));
                                    has_error = true;
                                }
                            }
                        }
                        continue;
                    }
                    'u' => {
                        self.advance(); // consume 'u'
                        match self.scan_hex_digits(4) {
                            Some(ch) => value.push(ch),
                            None => {
                                if !has_error {
                                    error_token = Some(self.error_invalid_escape('u'));
                                    has_error = true;
                                }
                            }
                        }
                        continue;
                    }
                    _ => {
                        if !has_error {
                            error_token = Some(self.error_invalid_escape(escape_char));
                            has_error = true;
                        }
                        self.advance(); // consume invalid char
                    }
                }
                continue;
            } else {
                value.push(self.advance());
            }
        }

        StringScan::Error(self.error_unterminated_string())
    }

    fn scan_template_string_part(&mut self, part_start_pos: usize) -> StringScan {
        let mut value = String::new();
        let mut has_error = false;
        let mut error_token = None;

        while !self.is_at_end() {
            let current_char = self.peek();

            if current_char == '`' {
                self.advance(); // Closing `
                let span = Span::new_in(part_start_pos, self.current, self.file);
                let token = Token::new(TokenKind::TemplateString, value, span);
                return if let Some(err) = error_token {
                    StringScan::Error(err)
                } else {
                    StringScan::Complete(token)
                };
            }

            if !has_error && current_char == '$' && self.peek_next() == Some('{') {
                let span = Span::new_in(part_start_pos, self.current, self.file);
                let part = Token::new(TokenKind::TemplateString, value, span);
                let interp_start = self.current;
                self.advance(); // $
                self.advance(); // {
                let interp_span = Span::new_in(interp_start, self.current, self.file);
                return StringScan::Interpolation { part, interp_span };
            }

            if current_char == '\n' {
                self.line += 1;
                self.column = 1;
            }

            if current_char == '\\' {
                self.advance(); // consume backslash
                if self.is_at_end() {
                    return StringScan::Error(self.error_unterminated_string());
                }

                let escape_char = self.peek();
                match escape_char {
                    'n' => {
                        self.advance();
                        value.push('\n');
                    }
                    'r' => {
                        self.advance();
                        value.push('\r');
                    }
                    't' => {
                        self.advance();
                        value.push('\t');
                    }
                    '\\' => {
                        self.advance();
                        value.push('\\');
                    }
                    '"' => {
                        self.advance();
                        value.push('"');
                    }
                    '`' => {
                        self.advance();
                        value.push('`');
                    }
                    '{' => {
                        self.advance();
                        value.push('{');
                    }
                    '}' => {
                        self.advance();
                        value.push('}');
                    }
                    '0' => {
                        self.advance();
                        value.push('\0');
                    }
                    'x' => {
                        self.advance(); // consume 'x'
                        match self.scan_hex_digits(2) {
                            Some(ch) => value.push(ch),
                            None => {
                                if !has_error {
                                    error_token = Some(self.error_invalid_escape('x'));
                                    has_error = true;
                                }
                            }
                        }
                        continue;
                    }
                    'u' => {
                        self.advance(); // consume 'u'
                        match self.scan_hex_digits(4) {
                            Some(ch) => value.push(ch),
                            None => {
                                if !has_error {
                                    error_token = Some(self.error_invalid_escape('u'));
                                    has_error = true;
                                }
                            }
                        }
                        continue;
                    }
                    _ => {
                        if !has_error {
                            error_token = Some(self.error_invalid_escape(escape_char));
                            has_error = true;
                        }
                        self.advance(); // consume invalid char
                    }
                }
                continue;
            } else {
                value.push(self.advance());
            }
        }

        StringScan::Error(self.error_unterminated_string())
    }

    /// Scan exactly `count` hex digits and return the corresponding Unicode character.
    /// Returns `None` if fewer than `count` hex digits follow or if the codepoint is invalid.
    fn scan_hex_digits(&mut self, count: usize) -> Option<char> {
        let mut codepoint: u32 = 0;
        for _ in 0..count {
            if self.is_at_end() {
                return None;
            }
            let ch = self.peek();
            let digit = ch.to_digit(16)?;
            codepoint = codepoint * 16 + digit;
            self.advance();
        }
        char::from_u32(codepoint)
    }

    /// Scan a number literal (integer, float, or scientific notation)
    pub(super) fn number(&mut self) -> Token {
        let start = self.current - 1; // -1 because we already advanced past first digit

        // Consume all digits
        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        // Check for decimal point
        if !self.is_at_end() && self.peek() == '.' {
            // Look ahead to ensure there's a digit after the dot
            if let Some(c) = self.peek_next() {
                if c.is_ascii_digit() {
                    self.advance(); // consume .

                    // Consume fractional digits
                    while !self.is_at_end() && self.peek().is_ascii_digit() {
                        self.advance();
                    }
                }
            }
        }

        // Check for scientific notation (e or E)
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            self.advance(); // consume e/E

            // Optional + or - sign
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                self.advance();
            }

            // Must have at least one digit in exponent
            if self.is_at_end() || !self.peek().is_ascii_digit() {
                return self.error_invalid_number("exponent requires digits after `e`/`E`");
            }

            // Consume exponent digits
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let lexeme: String = self.chars[start..self.current].iter().collect();
        self.make_token(TokenKind::Number, &lexeme)
    }

    /// Scan an identifier or keyword
    pub(super) fn identifier(&mut self) -> Token {
        let start = self.current - 1; // -1 because we already advanced past first char

        while !self.is_at_end() {
            let c = self.peek();
            if c.is_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let lexeme: String = self.chars[start..self.current].iter().collect();

        // Check for standalone underscore (wildcard pattern)
        if lexeme == "_" {
            return self.make_token(TokenKind::Underscore, "_");
        }

        // Check if it's a keyword
        let kind = TokenKind::is_keyword(&lexeme).unwrap_or(TokenKind::Identifier);

        self.make_token(kind, &lexeme)
    }
}
