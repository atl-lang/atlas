//! Token types for lexical analysis
//!
//! Defines all token types recognized by the Atlas lexer.

use crate::span::Span;
use serde::{Deserialize, Serialize};

/// Token type produced by the lexer
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    /// The kind of token
    pub kind: TokenKind,
    /// The source text of this token
    pub lexeme: String,
    /// Source location
    pub span: Span,
}

impl Token {
    /// Create a new token
    pub fn new(kind: TokenKind, lexeme: impl Into<String>, span: Span) -> Self {
        Self {
            kind,
            lexeme: lexeme.into(),
            span,
        }
    }
}

/// Classification of token types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenKind {
    // Literals
    /// Number literal (42, 3.14)
    Number,
    /// String literal ("hello")
    String,
    /// Template string literal segment (`hello ${name}`)
    TemplateString,
    /// `true` keyword
    True,
    /// `false` keyword
    False,
    /// `null` keyword
    Null,
    /// Identifier
    Identifier,

    // Keywords
    /// `let` keyword (immutable variable)
    Let,
    /// `mut` keyword (mutable modifier for let)
    Mut,
    /// `fn` keyword (function declaration)
    Fn,
    /// `type` keyword (type alias declaration)
    Type,
    /// `if` keyword
    If,
    /// `else` keyword
    Else,
    /// `while` keyword
    While,
    /// `for` keyword
    For,
    /// `in` keyword (used in for-in loops)
    In,
    /// `return` keyword
    Return,
    /// `break` keyword
    Break,
    /// `continue` keyword
    Continue,

    // Module system (v0.2+)
    /// `import` keyword
    Import,
    /// `export` keyword
    Export,
    /// `from` keyword (used in import statements)
    From,
    /// `extern` keyword (FFI declarations)
    Extern,

    // Pattern matching (v0.2+)
    /// `match` keyword
    Match,
    /// `as` keyword (used in imports and patterns)
    As,
    /// `is` keyword (type predicates)
    Is,

    // Ownership annotations (v0.3+)
    /// `own` keyword (owned parameter annotation)
    Own,
    /// `borrow` keyword (borrowed parameter annotation)
    Borrow,
    /// `share` keyword (shared parameter annotation)
    Share,
    // Async/await (B8)
    /// `async` keyword
    Async,
    /// `await` keyword
    Await,
    // Trait system (v0.3+)
    /// `trait` keyword
    Trait,
    /// `impl` keyword
    Impl,
    /// `extends` keyword (supertrait bounds and generic bounds — TypeScript style)
    Extends,

    // Type declarations (v0.3+)
    /// `struct` keyword
    Struct,
    /// `enum` keyword
    Enum,
    /// `record` keyword
    Record,

    // Visibility modifiers (v0.3+ B37)
    /// `pub` keyword — public visibility
    Pub,
    /// `private` keyword — file-private visibility
    Private,
    /// `internal` keyword — module-internal visibility
    Internal,

    // Static modifier (v0.3+ B39)
    /// `static` keyword — static method modifier
    Static,

    // Const (v0.3+ B39)
    /// `const` keyword — compile-time constant declaration
    Const,

    // Advanced features (v0.3+ B41)
    /// `defer` keyword — deferred execution (LIFO on function exit)
    Defer,

    // Constructor syntax (H-374)
    /// `new` keyword — collection/object constructor: `new Map<K,V>()`
    New,

    // Operators
    /// `+` (addition)
    Plus,
    /// `-` (subtraction or negation)
    Minus,
    /// `*` (multiplication)
    Star,
    /// `/` (division)
    Slash,
    /// `%` (modulo)
    Percent,
    /// `!` (logical not)
    Bang,
    /// `==` (equality)
    EqualEqual,
    /// `!=` (inequality)
    BangEqual,
    /// `<` (less than)
    Less,
    /// `<=` (less than or equal)
    LessEqual,
    /// `>` (greater than)
    Greater,
    /// `>=` (greater than or equal)
    GreaterEqual,
    /// `&&` (logical and)
    AmpAmp,
    /// `||` (logical or)
    PipePipe,
    /// `&` (type intersection)
    Ampersand,
    /// `@` (attribute prefix)
    At,
    /// `|` (type union)
    Pipe,

    // Compound assignment operators
    /// `+=` (add and assign)
    PlusEqual,
    /// `-=` (subtract and assign)
    MinusEqual,
    /// `*=` (multiply and assign)
    StarEqual,
    /// `/=` (divide and assign)
    SlashEqual,
    /// `%=` (modulo and assign)
    PercentEqual,

    // Punctuation
    /// `=` (assignment)
    Equal,
    /// `(` (left parenthesis)
    LeftParen,
    /// `)` (right parenthesis)
    RightParen,
    /// `{` (left brace)
    LeftBrace,
    /// `}` (right brace)
    RightBrace,
    /// `[` (left bracket)
    LeftBracket,
    /// `]` (right bracket)
    RightBracket,
    /// `;` (semicolon)
    Semicolon,
    /// `,` (comma)
    Comma,
    /// `.` (dot for member access)
    Dot,
    /// `..` (range operator)
    Range,
    /// `..=` (inclusive range operator)
    RangeInclusive,
    /// `...` (rest/spread operator for variadic parameters)
    DotDotDot,
    /// `:` (colon)
    Colon,
    /// `::` (double colon for enum variant paths)
    ColonColon,
    /// `->` (arrow for function return type)
    Arrow,
    /// `=>` (fat arrow for match arms)
    FatArrow,
    /// `_` (underscore for wildcard patterns)
    Underscore,
    /// `?` (error propagation operator)
    Question,
    /// Start of string interpolation
    InterpolationStart,
    /// `}` end of string interpolation (only in interpolated strings)
    InterpolationEnd,

    // Comments (emitted in comment-preserving mode)
    /// Single-line comment (// ...)
    LineComment,
    /// Block comment (/* ... */)
    BlockComment,
    /// Doc comment (/// ...)
    DocComment,

    // Special
    /// End of file
    Eof,
    /// Lexer error
    Error,
}

impl TokenKind {
    /// Check if a string is a keyword and return its token kind
    pub fn is_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "fn" => Some(TokenKind::Fn),
            "type" => Some(TokenKind::Type),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "while" => Some(TokenKind::While),
            "for" => Some(TokenKind::For),
            "in" => Some(TokenKind::In),
            "return" => Some(TokenKind::Return),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "null" => Some(TokenKind::Null),
            "import" => Some(TokenKind::Import),
            "export" => Some(TokenKind::Export),
            "from" => Some(TokenKind::From),
            "extern" => Some(TokenKind::Extern),
            "match" => Some(TokenKind::Match),
            "as" => Some(TokenKind::As),
            "is" => Some(TokenKind::Is),
            "async" => Some(TokenKind::Async),
            "await" => Some(TokenKind::Await),
            "own" => Some(TokenKind::Own),
            "borrow" => Some(TokenKind::Borrow),
            "share" => Some(TokenKind::Share),
            "trait" => Some(TokenKind::Trait),
            "impl" => Some(TokenKind::Impl),
            "extends" => Some(TokenKind::Extends),
            "struct" => Some(TokenKind::Struct),
            "enum" => Some(TokenKind::Enum),
            "record" => Some(TokenKind::Record),
            "pub" => Some(TokenKind::Pub),
            "private" => Some(TokenKind::Private),
            "internal" => Some(TokenKind::Internal),
            "static" => Some(TokenKind::Static),
            "const" => Some(TokenKind::Const),
            "defer" => Some(TokenKind::Defer),
            "new" => Some(TokenKind::New),
            _ => None,
        }
    }

    /// Get the string representation of this token kind
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenKind::Number => "number",
            TokenKind::String => "string",
            TokenKind::TemplateString => "template string",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::Null => "null",
            TokenKind::Identifier => "identifier",
            TokenKind::Let => "let",
            TokenKind::Mut => "mut",
            TokenKind::Fn => "fn",
            TokenKind::Type => "type",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::While => "while",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::Return => "return",
            TokenKind::Break => "break",
            TokenKind::Continue => "continue",
            TokenKind::Import => "import",
            TokenKind::Export => "export",
            TokenKind::From => "from",
            TokenKind::Extern => "extern",
            TokenKind::Match => "match",
            TokenKind::As => "as",
            TokenKind::Is => "is",
            TokenKind::Async => "async",
            TokenKind::Await => "await",
            TokenKind::Own => "own",
            TokenKind::Borrow => "borrow",
            TokenKind::Share => "share",
            TokenKind::Trait => "trait",
            TokenKind::Impl => "impl",
            TokenKind::Extends => "extends",
            TokenKind::Struct => "struct",
            TokenKind::Enum => "enum",
            TokenKind::Record => "record",
            TokenKind::Pub => "pub",
            TokenKind::Private => "private",
            TokenKind::Internal => "internal",
            TokenKind::Static => "static",
            TokenKind::Const => "const",
            TokenKind::Defer => "defer",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Bang => "!",
            TokenKind::EqualEqual => "==",
            TokenKind::BangEqual => "!=",
            TokenKind::Less => "<",
            TokenKind::LessEqual => "<=",
            TokenKind::Greater => ">",
            TokenKind::GreaterEqual => ">=",
            TokenKind::AmpAmp => "&&",
            TokenKind::PipePipe => "||",
            TokenKind::Ampersand => "&",
            TokenKind::At => "@",
            TokenKind::Pipe => "|",
            TokenKind::PlusEqual => "+=",
            TokenKind::MinusEqual => "-=",
            TokenKind::StarEqual => "*=",
            TokenKind::SlashEqual => "/=",
            TokenKind::PercentEqual => "%=",
            TokenKind::Equal => "=",
            TokenKind::LeftParen => "(",
            TokenKind::RightParen => ")",
            TokenKind::LeftBrace => "{",
            TokenKind::RightBrace => "}",
            TokenKind::LeftBracket => "[",
            TokenKind::RightBracket => "]",
            TokenKind::Semicolon => ";",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",
            TokenKind::Range => "..",
            TokenKind::RangeInclusive => "..=",
            TokenKind::DotDotDot => "...",
            TokenKind::Colon => ":",
            TokenKind::ColonColon => "::",
            TokenKind::Arrow => "->",
            TokenKind::FatArrow => "=>",
            TokenKind::Underscore => "_",
            TokenKind::Question => "?",
            TokenKind::InterpolationStart => "${",
            TokenKind::InterpolationEnd => "}",
            TokenKind::LineComment => "// comment",
            TokenKind::BlockComment => "/* comment */",
            TokenKind::DocComment => "/// comment",
            TokenKind::Eof => "EOF",
            TokenKind::Error => "error",
            TokenKind::New => "new",
        }
    }
}
#[cfg(test)]
mod test_mut_keyword {
    use super::TokenKind;

    #[test]
    fn test_mut_is_keyword() {
        assert_eq!(TokenKind::is_keyword("mut"), Some(TokenKind::Mut));
    }

    #[test]
    fn test_mut_as_str() {
        assert_eq!(TokenKind::Mut.as_str(), "mut");
    }
}
