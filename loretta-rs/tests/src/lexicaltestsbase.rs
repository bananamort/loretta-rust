// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.LexicalTestsBase (b767b4e): LexicalTestsBase
// C# source: src/Compilers/Lua/Test/Portable/Lexical/LexicalTestsBase.cs
//
// The C# SyntaxFactory.ParseTokens (the dropped Syntax infrastructure) maps
// to the full_moon Lexer over the source text. The C# token stream excludes
// the trivia (attached to the tokens instead) — the port filters the
// full_moon stream with TokenType::is_trivia. The dropped token values
// (Option<object?>) dock on the ShortToken TokenValue, computed per the C#
// Lexer.Numbers.cs / Lexer.ShortString.cs rules (the integer-format options
// decide long vs double; the C# spans are UTF-16, the port's are bytes — the
// test sources are ASCII).

use full_moon::tokenizer::{Lexer, LexerResult, Token, TokenType};

use crate::luatestbase::options_to_version;
use crate::shorttoken::{ShortToken, TextSpan, TokenValue};
use loretta::integerformats::IntegerFormats;
use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;
use loretta::utilities::hexfloat::HexFloat;

/// C# LexicalTestsBase (LexicalTestsBase.cs:5-23): the lexical test base.
pub struct LexicalTestsBase;

impl LexicalTestsBase {
    /// C# Lex (LexicalTestsBase.cs:7-8): the token stream of the text
    /// (excluding the trivia, which the C# attaches to the tokens).
    pub fn lex(text: &str, options: Option<&LuaSyntaxOptions>) -> Vec<ShortToken> {
        let options = options.unwrap_or(&LuaSyntaxOptions::ALL);
        let parse_options = LuaParseOptions::new(options.clone());
        let lexer = Lexer::new(text, options_to_version(&parse_options));
        let tokens = match lexer.collect() {
            LexerResult::Ok(tokens) | LexerResult::Recovered(tokens, _) => tokens,
            LexerResult::Fatal(errors) => panic!("lex failed: {errors:?}"),
        };
        tokens
            .iter()
            .filter(|token| !token.token_type().is_trivia())
            .map(|token| Self::token_to_short_token(token, options))
            .collect()
    }

    /// C# LexToken (LexicalTestsBase.cs:10-23): the first token; any further
    /// non-EOF token fails the assertion.
    pub fn lex_token(text: &str, options: Option<&LuaSyntaxOptions>) -> ShortToken {
        let tokens = Self::lex(text, options);
        let mut iter = tokens.iter();
        let first = iter
            .next()
            .expect("the lexer must produce at least the EOF token");
        for rest in iter {
            assert!(
                matches!(rest.kind, TokenType::Eof),
                "more than one token was lexed: {rest}"
            );
        }
        first.clone()
    }

    /// The C# SyntaxToken -> ShortToken mapping (ShortToken.cs:19): kind,
    /// text, span, value.
    fn token_to_short_token(token: &Token, options: &LuaSyntaxOptions) -> ShortToken {
        let text = token.to_string();
        let start = token.start_position().bytes();
        let span = TextSpan::new(start, text.len());
        let value = token_value(token, options);
        ShortToken::new(token.token_type().clone(), text, span, value)
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
/// decoding shared with the constant folder's literal decoding).
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
