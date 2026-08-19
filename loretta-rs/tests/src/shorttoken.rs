// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.ShortToken (b767b4e): ShortToken
// C# source: src/Compilers/Lua/Test/Portable/Lexical/ShortToken.cs

use full_moon::tokenizer::{TokenKind, TokenType};

/// C# TextSpan (Compilers/Core/Portable/Text/TextSpan.cs:13-213): the dropped
/// span type, ported minimally for the test rows (Start/Length/End and the
/// C# ToString `[Start..End)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSpan {
    start: usize,
    length: usize,
}

impl TextSpan {
    /// The C# ctor (TextSpan.cs:19).
    pub fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }

    /// C# Start (TextSpan.cs:39).
    pub fn start(&self) -> usize {
        self.start
    }

    /// C# Length (TextSpan.cs:50).
    pub fn length(&self) -> usize {
        self.length
    }

    /// C# End => Start + Length (TextSpan.cs:44).
    pub fn end(&self) -> usize {
        self.start + self.length
    }
}

impl std::fmt::Display for TextSpan {
    /// C# ToString (TextSpan.cs:213): `[{Start}..{End})`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}..{})", self.start, self.end())
    }
}

/// A literal token value (the C# `object?` Value — the dropped lexer's
/// typed literal values).
#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    /// A long integer value (the C# long).
    Integer(i64),
    /// A double value (the C# double).
    Float(f64),
    /// A string value (the C# string — the decoded literal).
    String(String),
}

impl std::fmt::Display for TokenValue {
    /// The C# ToString of the boxed value.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Integer(v) => write!(f, "{v}"),
            TokenValue::Float(v) => write!(f, "{v}"),
            TokenValue::String(v) => write!(f, "{v}"),
        }
    }
}

/// C# ShortToken (ShortToken.cs:6-26): the record struct the lexical tests use
/// for expected token rows. The dropped SyntaxKind enum (Portable/Syntax/,
/// Locked Decision 1) docks on the full_moon [`TokenType`]; the dropped
/// `Option<object?>` token value docks on `Option<TokenValue>`.
#[derive(Debug, Clone, PartialEq)]
pub struct ShortToken {
    pub kind: TokenType,
    pub text: String,
    pub span: TextSpan,
    pub value: Option<TokenValue>,
}

impl ShortToken {
    /// C# primary record ctor (ShortToken.cs:6-11): kind, text, span, value.
    pub fn new(kind: TokenType, text: String, span: TextSpan, value: Option<TokenValue>) -> Self {
        Self {
            kind,
            text,
            span,
            value,
        }
    }

    /// C# convenience ctor (ShortToken.cs:13-17): `new TextSpan(0, text.Length)`.
    pub fn from_text(kind: TokenType, text: String, value: Option<TokenValue>) -> Self {
        let span = TextSpan::new(0, text.len());
        Self {
            kind,
            text,
            span,
            value,
        }
    }

    /// C# WithSpan (ShortToken.cs:23): `this with { Span = span }`.
    pub fn with_span(&self, span: TextSpan) -> Self {
        Self {
            span,
            ..self.clone()
        }
    }
}

impl std::fmt::Display for ShortToken {
    /// C# ToString (ShortToken.cs:25): `{Kind}<{Text}> ({Span})` plus
    /// `" = {Value}"` when the value is some. The C# Kind label is the dropped
    /// SyntaxKind enum name — the port uses the C#-established names for the
    /// trivia/EOF/literal kinds (see [`kind_label`]); the per-symbol C#
    /// SyntaxKind names land with the lexical test-kind surface.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}<{}> ({})",
            kind_label(&self.kind),
            self.text,
            self.span
        )?;
        if let Some(value) = &self.value {
            write!(f, " = {value}")?;
        }
        Ok(())
    }
}

/// Formats the dropped SyntaxKind enum name for the token type. The names are
/// the C# values verified in Portable/Syntax/SyntaxKind.cs (e.g. EndOfFileToken
/// = 110, NumericLiteralToken = 1001, StringLiteralToken = 1002,
/// IdentifierToken = 1003, InterpolatedStringToken = 1005).
fn kind_label(kind: &TokenType) -> &'static str {
    match kind.kind() {
        TokenKind::Eof => "EndOfFileToken",
        TokenKind::Identifier => "IdentifierToken",
        TokenKind::Number => "NumericLiteralToken",
        TokenKind::StringLiteral => "StringLiteralToken",
        TokenKind::MultiLineComment => "MultiLineCommentTrivia",
        TokenKind::SingleLineComment => "SingleLineCommentTrivia",
        TokenKind::Whitespace => "WhitespaceTrivia",
        TokenKind::Shebang => "ShebangTrivia",
        TokenKind::Symbol => "SymbolToken",
        TokenKind::InterpolatedString => "InterpolatedStringToken",
        TokenKind::CStyleComment => "CStyleCommentTrivia",
        // TokenKind is #[non_exhaustive].
        _ => "UnknownToken",
    }
}
