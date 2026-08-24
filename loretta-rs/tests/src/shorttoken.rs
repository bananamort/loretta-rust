// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.ShortToken (b767b4e): ShortToken
// C# source: src/Compilers/Lua/Test/Portable/Lexical/ShortToken.cs

use full_moon::tokenizer::{StringLiteralQuoteType, Symbol, Token, TokenKind, TokenType};

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
    // The symbol rows carry the per-symbol C# SyntaxKind name (Finding 63
    // — the generic "SymbolToken" label made every symbol failure message
    // indistinguishable).
    if let TokenType::Symbol { symbol } = kind {
        return symbol_label(*symbol);
    }
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

/// The C# SyntaxKind name per full_moon symbol (verified in
/// Portable/Syntax/SyntaxKind.cs — e.g. AndKeyword = 517, GotoKeyword =
/// 515, AmpersandToken = 48, ColonColonToken = 52).
fn symbol_label(symbol: Symbol) -> &'static str {
    match symbol {
        Symbol::And => "AndKeyword",
        Symbol::Break => "BreakKeyword",
        Symbol::Do => "DoKeyword",
        Symbol::Else => "ElseKeyword",
        Symbol::ElseIf => "ElseIfKeyword",
        Symbol::End => "EndKeyword",
        Symbol::False => "FalseKeyword",
        Symbol::For => "ForKeyword",
        Symbol::Function => "FunctionKeyword",
        Symbol::If => "IfKeyword",
        Symbol::In => "InKeyword",
        Symbol::Local => "LocalKeyword",
        Symbol::Nil => "NilKeyword",
        Symbol::Not => "NotKeyword",
        Symbol::Or => "OrKeyword",
        Symbol::Repeat => "RepeatKeyword",
        Symbol::Return => "ReturnKeyword",
        Symbol::Then => "ThenKeyword",
        Symbol::True => "TrueKeyword",
        Symbol::Until => "UntilKeyword",
        Symbol::While => "WhileKeyword",
        Symbol::Goto => "GotoKeyword",
        Symbol::PlusEqual => "PlusEqualsToken",
        Symbol::MinusEqual => "MinusEqualsToken",
        Symbol::StarEqual => "StarEqualsToken",
        Symbol::SlashEqual => "SlashEqualsToken",
        Symbol::DoubleSlashEqual => "DoubleSlashEqualsToken",
        Symbol::PercentEqual => "PercentEqualsToken",
        Symbol::CaretEqual => "CaretEqualsToken",
        Symbol::TwoDotsEqual => "DotDotEqualsToken",
        Symbol::Ampersand => "AmpersandToken",
        Symbol::ThinArrow => "ThinArrowToken",
        Symbol::TwoColons => "ColonColonToken",
        Symbol::AtSign => "AtSignToken",
        Symbol::DoubleLessThanEqual => "LessThanLessThanEqualsToken",
        Symbol::DoubleGreaterThanEqual => "GreaterThanGreaterThanEqualsToken",
        Symbol::AmpersandEqual => "AmpersandEqualsToken",
        Symbol::PipeEqual => "PipeEqualsToken",
        Symbol::QuestionMarkDot => "QuestionMarkDotToken",
        Symbol::Caret => "CaretToken",
        Symbol::Colon => "ColonToken",
        Symbol::Comma => "CommaToken",
        Symbol::Dot => "DotToken",
        Symbol::TwoDots => "DotDotToken",
        Symbol::Ellipsis => "EllipsisToken",
        Symbol::Equal => "EqualsToken",
        Symbol::TwoEqual => "EqualsEqualsToken",
        Symbol::GreaterThan => "GreaterThanToken",
        Symbol::GreaterThanEqual => "GreaterThanEqualsToken",
        Symbol::DoubleGreaterThan => "GreaterThanGreaterThanToken",
        Symbol::Hash => "HashToken",
        Symbol::LeftBrace => "OpenBraceToken",
        Symbol::LeftBracket => "OpenBracketToken",
        Symbol::LeftParen => "OpenParenToken",
        Symbol::LessThan => "LessThanToken",
        Symbol::LessThanEqual => "LessThanEqualsToken",
        Symbol::DoubleLessThan => "LessThanLessThanToken",
        Symbol::Minus => "MinusToken",
        Symbol::Percent => "PercentToken",
        Symbol::Pipe => "PipeToken",
        Symbol::Plus => "PlusToken",
        Symbol::QuestionMark => "QuestionMarkToken",
        Symbol::RightBrace => "CloseBraceToken",
        Symbol::RightBracket => "CloseBracketToken",
        Symbol::RightParen => "CloseParenToken",
        Symbol::Semicolon => "SemicolonToken",
        Symbol::Slash => "SlashToken",
        Symbol::Star => "StarToken",
        Symbol::Tilde => "TildeToken",
        _ => "SymbolToken",
    }
}

/// The C# token.Value (the dropped lexer's typed literal values). The C#
/// lexer gives the non-literal tokens the constant value or the token text
/// (the data rows mirror it); the InterpolatedString value is the full token
/// text (the FiveM hash rows are skipped by the row-863 tests — the full_moon
/// tokenizer does not hash).
fn token_value(token: &Token, options: &LuaSyntaxOptions) -> Option<TokenValue> {
    match token.token_type() {
        TokenType::Eof => Some(TokenValue::String(String::new())),
        TokenType::Identifier { .. } => Some(TokenValue::String(token.to_string())),
        TokenType::Number { .. } => number_value(&token.to_string(), options),
        TokenType::StringLiteral {
            literal,
            quote_type,
            ..
        } => {
            let text = literal.as_str();
            let decoded = if *quote_type == StringLiteralQuoteType::Brackets {
                // Long strings do not process escapes; the C# skips the
                // leading new line of the content (TryScanLongString,
                // Lexer.cs:926-927) — the full_moon keeps it in the literal.
                let content = if let Some(rest) = text.strip_prefix("\r\n") {
                    rest
                } else if let Some(rest) = text.strip_prefix('\n') {
                    rest
                } else if let Some(rest) = text.strip_prefix('\r') {
                    rest
                } else {
                    text
                };
                content.to_string()
            } else {
                unescape_lua_string(text, options)
            };
            Some(TokenValue::String(decoded))
        }
        TokenType::InterpolatedString { .. } => Some(TokenValue::String(token.to_string())),
        TokenType::Symbol { symbol } => symbol_value(*symbol, &token.to_string()),
        _ => None,
    }
}

/// The C# GetConstantValue.Or(text) (SyntaxFacts.cs:214-223) applied to a
/// lexed symbol token — the keyword constants (true/false/nil) and the token
/// text otherwise.
fn symbol_value(symbol: Symbol, text: &str) -> Option<TokenValue> {
    match symbol {
        Symbol::True => Some(TokenValue::Bool(true)),
        Symbol::False => Some(TokenValue::Bool(false)),
        Symbol::Nil => Some(TokenValue::Nil),
        _ => Some(TokenValue::String(text.to_string())),
    }
}

/// The C# Lexer.Numbers.cs value computation — the integer-format options
/// decide long vs double; the LuaJIT suffixes decide the value kind
/// regardless of the formats (the C# ParseBinaryNumber/ParseDecimalNumber/
/// ParseHexadecimalNumber suffix handling).
fn number_value(text: &str, options: &LuaSyntaxOptions) -> Option<TokenValue> {
    let clean: String = text.chars().filter(|c| *c != '_').collect();
    let lower = clean.to_lowercase();
    if lower.ends_with("ull") {
        let digits = &clean[..clean.len() - 3];
        return Some(TokenValue::Unsigned(parse_unsigned(digits)?));
    }
    if lower.ends_with("ll") {
        let digits = &clean[..clean.len() - 2];
        return Some(TokenValue::Integer(parse_signed(digits)?));
    }
    if lower.ends_with('i') {
        let digits = &clean[..clean.len() - 1];
        return Some(TokenValue::Complex(parse_complex(digits)?));
    }
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

/// The C# `ulong.Parse(text[..^3])` / `Convert.ToUInt64(text[2..^3], base)`
/// of the suffix digits.
fn parse_unsigned(text: &str) -> Option<u64> {
    if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        u64::from_str_radix(rest, 16).ok()
    } else if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        u64::from_str_radix(rest, 2).ok()
    } else if let Some(rest) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
        u64::from_str_radix(rest, 8).ok()
    } else {
        text.parse::<u64>().ok()
    }
}

/// The C# `long.Parse(text[..^2])` / `Convert.ToInt64(text[2..^2], base)`.
fn parse_signed(text: &str) -> Option<i64> {
    if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        i64::from_str_radix(rest, 16).ok()
    } else if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        i64::from_str_radix(rest, 2).ok()
    } else if let Some(rest) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
        i64::from_str_radix(rest, 8).ok()
    } else {
        text.parse::<i64>().ok()
    }
}

/// The C# `new Complex(0, ParseDouble(text[..^1], @base))` of the suffix
/// digits.
fn parse_complex(text: &str) -> Option<f64> {
    if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        let value = i64::from_str_radix(rest, 16).ok()?;
        Some(value as f64)
    } else if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        let value = i64::from_str_radix(rest, 2).ok()?;
        Some(value as f64)
    } else if let Some(rest) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
        let value = i64::from_str_radix(rest, 8).ok()?;
        Some(value as f64)
    } else {
        text.parse::<f64>().ok()
    }
}

/// Whether the hex text is a hex float (has a '.' or a 'p' exponent).
fn is_hex_float(text: &str) -> bool {
    text.contains('.') || text.contains('p') || text.contains('P')
}

/// The C# token.Value<string> — the decoded string literal (the Lua escape
/// decoding shared with the constant folder's literal decoding). The C# UTF-16
/// lone surrogates (e.g. `\u{D800}`) have no valid UTF-8 encoding and are
/// dropped (documented in the row-788 port). The invalid-escape presets keep
/// the escape text minus the backslash (the C# ScanEscapeSequence `goto
/// default` — Lexer.ShortString.cs:138-139, 167-171, 185-186).
/// The C# Lexer.ShortString escape processing for a short-string token's
/// literal content (the dropped lexer's decoded value). Exposed for the
/// interpolated-string tests (the C# InterpolatedStringTextToken value is
/// the decoded text — Finding 60).
pub fn unescape_lua_string(text: &str, options: &LuaSyntaxOptions) -> String {
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
            Some('z') if options.accept_invalid_escapes && !options.accept_whitespace_escape => {
                // The C# `goto default` — the escape char kept as-is.
                out.push('z');
            }
            Some('z') => loop {
                match chars.clone().next() {
                    Some(n) if n.is_ascii_whitespace() => {
                        chars.next();
                    }
                    _ => break,
                }
            },
            Some('x')
                if options.accept_invalid_escapes && !options.accept_hex_escapes_in_strings =>
            {
                // The C# `goto default` — the escape char kept as-is.
                out.push('x');
            }
            Some('x') => {
                let mut hex = String::new();
                for _ in 0..2 {
                    match chars.clone().next() {
                        Some(h) if h.is_ascii_hexdigit() => {
                            chars.next();
                            hex.push(h);
                        }
                        _ => break,
                    }
                }
                if let Ok(v) = u8::from_str_radix(&hex, 16) {
                    out.push(v as char);
                }
            }
            Some('u') if options.accept_invalid_escapes && !options.accept_unicode_escape => {
                // The C# `goto default` — the escape char kept as-is.
                out.push('u');
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
                    match chars.clone().next() {
                        Some(n) if n.is_ascii_digit() => {
                            chars.next();
                            digits.push(n);
                        }
                        _ => break,
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
