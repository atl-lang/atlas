//! Lexical analysis (tokenization)
//!
//! The lexer converts Atlas source code into a stream of tokens with accurate span information.

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

mod literals;

/// Lexer state for tokenizing source code
pub struct Lexer {
    /// Original source code
    pub(super) source: String,
    /// Characters of source code
    pub(super) chars: Vec<char>,
    /// Current position in chars
    pub(super) current: usize,
    /// Current line number (1-indexed)
    pub(super) line: u32,
    /// Current column number (1-indexed)
    pub(super) column: u32,
    /// Start position of current token
    pub(super) start_pos: usize,
    /// Start line of current token
    pub(super) start_line: u32,
    /// Start column of current token
    pub(super) start_column: u32,
    /// Collected diagnostics
    pub(super) diagnostics: Vec<Diagnostic>,
    /// Whether to emit comment tokens
    emit_comments: bool,
    /// Pending comment tokens to emit
    pending_comments: Vec<Token>,
}

impl Lexer {
    /// Create a new lexer for the given source code
    pub fn new(source: impl Into<String>) -> Self {
        let source = source.into();
        let chars: Vec<char> = source.chars().collect();
        Self {
            source,
            chars,
            current: 0,
            line: 1,
            column: 1,
            start_pos: 0,
            start_line: 1,
            start_column: 1,
            diagnostics: Vec::new(),
            emit_comments: false,
            pending_comments: Vec::new(),
        }
    }

    /// Tokenize the source code, returning tokens and any diagnostics
    pub fn tokenize(&mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        (tokens, std::mem::take(&mut self.diagnostics))
    }

    /// Tokenize preserving comment tokens in the stream
    pub fn tokenize_with_comments(&mut self) -> (Vec<Token>, Vec<Diagnostic>) {
        self.emit_comments = true;
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token();
            let is_eof = token.kind == TokenKind::Eof;

            // Drain any pending comment tokens collected before this token
            tokens.append(&mut self.pending_comments);
            tokens.push(token);

            if is_eof {
                break;
            }
        }

        self.emit_comments = false;
        (tokens, std::mem::take(&mut self.diagnostics))
    }

    /// Scan the next token
    fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        // Mark start of token
        self.start_pos = self.current;
        self.start_line = self.line;
        self.start_column = self.column;

        if self.is_at_end() {
            return self.make_token(TokenKind::Eof, "");
        }

        let c = self.advance();

        match c {
            // Single-character tokens
            '(' => self.make_token(TokenKind::LeftParen, "("),
            ')' => self.make_token(TokenKind::RightParen, ")"),
            '{' => self.make_token(TokenKind::LeftBrace, "{"),
            '}' => self.make_token(TokenKind::RightBrace, "}"),
            '[' => self.make_token(TokenKind::LeftBracket, "["),
            ']' => self.make_token(TokenKind::RightBracket, "]"),
            ';' => self.make_token(TokenKind::Semicolon, ";"),
            ',' => self.make_token(TokenKind::Comma, ","),
            ':' => self.make_token(TokenKind::Colon, ":"),
            '?' => self.make_token(TokenKind::Question, "?"),

            // Operators with potential compound forms
            '+' => {
                if self.match_char('+') {
                    self.make_token(TokenKind::PlusPlus, "++")
                } else if self.match_char('=') {
                    self.make_token(TokenKind::PlusEqual, "+=")
                } else {
                    self.make_token(TokenKind::Plus, "+")
                }
            }
            '-' => {
                if self.match_char('-') {
                    self.make_token(TokenKind::MinusMinus, "--")
                } else if self.match_char('=') {
                    self.make_token(TokenKind::MinusEqual, "-=")
                } else if self.match_char('>') {
                    self.make_token(TokenKind::Arrow, "->")
                } else {
                    self.make_token(TokenKind::Minus, "-")
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::StarEqual, "*=")
                } else {
                    self.make_token(TokenKind::Star, "*")
                }
            }
            '/' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::SlashEqual, "/=")
                } else {
                    self.make_token(TokenKind::Slash, "/")
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::PercentEqual, "%=")
                } else {
                    self.make_token(TokenKind::Percent, "%")
                }
            }

            // Two-character tokens
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::EqualEqual, "==")
                } else if self.match_char('>') {
                    self.make_token(TokenKind::FatArrow, "=>")
                } else {
                    self.make_token(TokenKind::Equal, "=")
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::BangEqual, "!=")
                } else {
                    self.make_token(TokenKind::Bang, "!")
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::LessEqual, "<=")
                } else {
                    self.make_token(TokenKind::Less, "<")
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenKind::GreaterEqual, ">=")
                } else {
                    self.make_token(TokenKind::Greater, ">")
                }
            }
            '&' => {
                if self.match_char('&') {
                    self.make_token(TokenKind::AmpAmp, "&&")
                } else {
                    self.make_token(TokenKind::Ampersand, "&")
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.make_token(TokenKind::PipePipe, "||")
                } else {
                    self.make_token(TokenKind::Pipe, "|")
                }
            }

            // String literals
            '"' => self.string(),

            // Numbers
            c if c.is_ascii_digit() => self.number(),

            // Dot (member access) or start of decimal number
            '.' => {
                // Check if this is the start of a decimal number (e.g., .5)
                // NOTE: Atlas doesn't support .5 syntax, only 0.5
                // So . is always a member access operator
                self.make_token(TokenKind::Dot, ".")
            }

            // Identifiers and keywords
            c if c.is_alphabetic() || c == '_' => self.identifier(),

            // Unexpected character
            _ => self.error_token(&format!("Unexpected character '{}'", c)),
        }
    }

    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) {
        loop {
            if self.is_at_end() {
                return;
            }

            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        let comment_start = self.current;
                        // Check for doc comment (///)
                        let is_doc = self.current + 2 < self.chars.len()
                            && self.chars[self.current + 2] == '/'
                            && (self.current + 3 >= self.chars.len()
                                || self.chars[self.current + 3] != '/');

                        // Single-line comment
                        while !self.is_at_end() && self.peek() != '\n' {
                            self.advance();
                        }

                        if self.emit_comments {
                            let text: String =
                                self.chars[comment_start..self.current].iter().collect();
                            let span = Span::new(comment_start, self.current);
                            let kind = if is_doc {
                                TokenKind::DocComment
                            } else {
                                TokenKind::LineComment
                            };
                            self.pending_comments.push(Token::new(kind, text, span));
                        }
                    } else if self.peek_next() == Some('*') {
                        // Multi-line comment
                        let comment_start = self.current;
                        let comment_start_line = self.line;
                        self.advance(); // /
                        self.advance(); // *

                        let mut terminated = false;
                        while !self.is_at_end() {
                            if self.peek() == '*' && self.peek_next() == Some('/') {
                                self.advance(); // *
                                self.advance(); // /
                                terminated = true;
                                break;
                            }
                            if self.peek() == '\n' {
                                self.line += 1;
                                self.column = 1;
                            }
                            self.advance();
                        }

                        if self.emit_comments && terminated {
                            let text: String =
                                self.chars[comment_start..self.current].iter().collect();
                            let span = Span::new(comment_start, self.current);
                            self.pending_comments.push(Token::new(
                                TokenKind::BlockComment,
                                text,
                                span,
                            ));
                        }

                        // Report error if comment was not terminated
                        if !terminated {
                            let span = Span {
                                start: self.start_pos,
                                end: self.current,
                            };
                            let snippet = self.get_line_snippet(comment_start_line);
                            self.diagnostics.push(
                                Diagnostic::error_with_code(
                                    "AT1004",
                                    "Unterminated multi-line comment",
                                    span,
                                )
                                .with_line(comment_start_line as usize)
                                .with_snippet(snippet)
                                .with_label("comment starts here")
                                .with_help("add '*/' to close the multi-line comment"),
                            );
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    /// Scan a string literal
    // === Character navigation ===
    /// Advance to next character and return it
    pub(super) fn advance(&mut self) -> char {
        let c = self.chars[self.current];
        self.current += 1;
        self.column += 1;
        c
    }

    /// Peek at current character without advancing
    pub(super) fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.chars[self.current]
        }
    }

    /// Peek at next character (current + 1)
    pub(super) fn peek_next(&self) -> Option<char> {
        if self.current + 1 >= self.chars.len() {
            None
        } else {
            Some(self.chars[self.current + 1])
        }
    }

    /// Check if current character matches expected, and advance if so
    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.chars[self.current] != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    /// Check if we've reached the end of source
    pub(super) fn is_at_end(&self) -> bool {
        self.current >= self.chars.len()
    }

    // === Token creation ===

    /// Create a token with the given kind and lexeme
    pub(super) fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        let span = Span {
            start: self.start_pos,
            end: self.current,
        };

        Token {
            kind,
            lexeme: lexeme.to_string(),
            span,
        }
    }

    /// Create an error token and record a diagnostic with a specific code
    pub(super) fn error_token_with_code(&mut self, code: &str, message: &str) -> Token {
        let span = Span {
            start: self.start_pos,
            end: self.current.max(self.start_pos + 1),
        };

        // Extract snippet from source for this line
        let snippet = self.get_line_snippet(self.start_line);

        // Record diagnostic
        self.diagnostics.push(
            Diagnostic::error_with_code(code, message, span)
                .with_line(self.start_line as usize)
                .with_snippet(snippet)
                .with_label("lexer error"),
        );

        Token {
            kind: TokenKind::Error,
            lexeme: message.to_string(),
            span,
        }
    }

    /// Create an error token for invalid/unexpected characters (AT1001)
    pub(super) fn error_token(&mut self, message: &str) -> Token {
        self.error_token_with_code("AT1001", message)
    }

    /// Create an error token for unterminated strings (AT1002)
    pub(super) fn error_unterminated_string(&mut self) -> Token {
        self.error_token_with_code("AT1002", "Unterminated string literal")
    }

    /// Create an error token for invalid escape sequences (AT1003)
    pub(super) fn error_invalid_escape(&mut self, escape_char: char) -> Token {
        self.error_token_with_code(
            "AT1003",
            &format!("Invalid escape sequence '\\{}'", escape_char),
        )
    }

    /// Get the source line for a given line number
    fn get_line_snippet(&self, line: u32) -> String {
        self.source
            .lines()
            .nth((line - 1) as usize)
            .unwrap_or("")
            .to_string()
    }
}
