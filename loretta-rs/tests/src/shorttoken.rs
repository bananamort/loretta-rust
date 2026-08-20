// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.ShortToken (b767b4e): ShortToken
// C# source: src/Compilers/Lua/Test/Portable/Lexical/ShortToken.cs

use full_moon::tokenizer::{Token, TokenKind, TokenType};

use loretta::integerformats::IntegerFormats;
use loretta::luasyntaxoptions::LuaSyntaxOptions;
use loretta::utilities::hexfloat::HexFloat;

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
    /// An unsigned long value (the C# ulong).
    Unsigned(u64),
    /// A double value (the C# double).
    Float(f64),
    /// An imaginary number (the C# Complex — the suffix-`i` literals; the
    /// real part is always zero in the test data).
    Complex(f64),
    /// A string value (the C# string — the decoded literal).
    String(String),
    /// A boolean value (the C# bool — the true/false keyword constants).
    Bool(bool),
    /// The null value (the C# nil keyword constant).
    Nil,
}

impl std::fmt::Display for TokenValue {
    /// The C# ToString of the boxed value.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenValue::Integer(v) => write!(f, "{v}"),
            TokenValue::Unsigned(v) => write!(f, "{v}"),
            TokenValue::Float(v) => write!(f, "{v}"),
            TokenValue::Complex(v) => write!(f, "(0, {v})"),
            TokenValue::String(v) => write!(f, "{v}"),
            TokenValue::Bool(v) => write!(f, "{v}"),
            TokenValue::Nil => write!(f, ""),
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

    /// C# ctor (ShortToken.cs:19): `ShortToken(SyntaxToken token)` — the
    /// conversion of a lexed token (the dropped SyntaxToken docks on the
    /// full_moon Token) into the expected row: kind, text, span, and the
    /// typed literal value (per the C# Lexer.Numbers.cs / Lexer.ShortString.cs
    /// rules — the integer-format options decide long vs double).
    pub fn from_token(token: &Token, options: &LuaSyntaxOptions) -> Self {
        let text = token.to_string();
        let span = TextSpan::new(token.start_position().bytes(), text.len());
        let value = token_value(token, options);
        Self::new(token.token_type().clone(), text, span, value)
    }

    /// C# ctor (ShortToken.cs:21): `ShortToken(SyntaxTrivia trivia)` — the
    /// conversion of a trivia token (the dropped SyntaxTrivia docks on the
    /// full_moon trivia Token): the full text and span, no value.
    pub fn from_trivia(token: &Token) -> Self {
        let text = token.to_string();
        let span = TextSpan::new(token.start_position().bytes(), text.len());
        Self::new(token.token_type().clone(), text, span, None)
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

/// The C# token.Value (the dropped lexer's typed literal values).
fn token_value(token: &Token, options: &LuaSyntaxOptions) -> Option<TokenValue> {
    match token.token_type() {
        TokenType::Number { .. } => number_value(&token.to_string(), options),
        TokenType::StringLiteral {
            literal,
            multi_line_depth,
            ..
        } => {
            let text = literal.as_str();
            let decoded = if *multi_line_depth > 0 {
                // Long strings do not process escapes.
                text.to_string()
            } else {
                unescape_lua_string(text)
            };
            Some(TokenValue::String(decoded))
        }
        _ => None,
    }
}

/// The C# Lexer.Numbers.cs value computation — the integer-format options
/// decide long vs double.
fn number_value(text: &str, options: &LuaSyntaxOptions) -> Option<TokenValue> {
    let clean: String = text.chars().filter(|c| *c != '_').collect();
    if let Some(rest) = clean
        .strip_prefix("0x")
        .or_else(|| clean.strip_prefix("0X"))
    {
        if is_hex_float(&clean) {
            let value = HexFloat::double_from_hex_string(&clean).ok()?;
            return Some(TokenValue::Float(value));
        }
        let value = i64::from_str_radix(rest, 16).ok()?;
        return Some(match options.hex_integer_format {
            IntegerFormats::Int64 => TokenValue::Integer(value),
            _ => TokenValue::Float(value as f64),
        });
    }
    if let Some(rest) = clean
        .strip_prefix("0b")
        .or_else(|| clean.strip_prefix("0B"))
    {
        let value = i64::from_str_radix(rest, 2).ok()?;
        return Some(match options.binary_integer_format {
            IntegerFormats::Int64 => TokenValue::Integer(value),
            _ => TokenValue::Float(value as f64),
        });
    }
    if let Some(rest) = clean
        .strip_prefix("0o")
        .or_else(|| clean.strip_prefix("0O"))
    {
        let value = i64::from_str_radix(rest, 8).ok()?;
        return Some(match options.octal_integer_format {
            IntegerFormats::Int64 => TokenValue::Integer(value),
            _ => TokenValue::Float(value as f64),
        });
    }
    let is_float = clean.contains('.') || clean.contains('e') || clean.contains('E');
    if is_float {
        let value = clean.parse::<f64>().ok()?;
        return Some(TokenValue::Float(value));
    }
    let value = clean.parse::<i64>().ok()?;
    Some(match options.decimal_integer_format {
        IntegerFormats::Int64 => TokenValue::Integer(value),
        _ => TokenValue::Float(value as f64),
    })
}

/// Whether the hex text is a hex float (has a '.' or a 'p' exponent).
fn is_hex_float(text: &str) -> bool {
    text.contains('.') || text.contains('p') || text.contains('P')
}

/// The C# token.Value<string> — the decoded string literal (the Lua escape
/// decoding shared with the constant folder's literal decoding). The C# UTF-16
/// lone surrogates (e.g. `\u{D800}`) have no valid UTF-8 encoding and are
/// dropped (documented in the row-788 port).
fn unescape_lua_string(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            None => out.push('\\'),
            Some('a') => out.push('\x07'),
            Some('b') => out.push('\x08'),
            Some('f') => out.push('\x0C'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('v') => out.push('\x0B'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('z') => loop {
                match chars.clone().next() {
                    Some(n) if n.is_ascii_whitespace() => {
                        chars.next();
                    }
                    _ => break,
                }
            },
            Some('x') => {
                let mut hex = String::new();
                for _ in 0..2 {
                    match chars.next() {
                        Some(h) if h.is_ascii_hexdigit() => hex.push(h),
                        other => {
                            if let Some(o) = other {
                                out.push(o);
                            }
                            break;
                        }
                    }
                }
                if let Ok(v) = u8::from_str_radix(&hex, 16) {
                    out.push(v as char);
                }
            }
            Some('u') => {
                let mut digits = String::new();
                if chars.next() == Some('{') {
                    for c in chars.by_ref() {
                        if c == '}' {
                            break;
                        }
                        digits.push(c);
                    }
                }
                if let Ok(v) = u32::from_str_radix(&digits, 16) {
                    if let Some(c) = char::from_u32(v) {
                        out.push(c);
                    }
                }
            }
            Some(d) if d.is_ascii_digit() => {
                let mut digits = String::new();
                digits.push(d);
                for _ in 0..2 {
                    match chars.next() {
                        Some(n) if n.is_ascii_digit() => digits.push(n),
                        other => {
                            if let Some(o) = other {
                                out.push(o);
                            }
                            break;
                        }
                    }
                }
                if let Ok(v) = digits.parse::<u32>() {
                    out.push(char::from_u32(v).unwrap_or('\u{FFFD}'));
                }
            }
            // The C# AcceptInvalidEscapes keeps the escaped character itself.
            Some(other) => out.push(other),
        }
    }
    out
}
