// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.LexicalTestData (b767b4e): LexicalTestData
// C# source: src/Compilers/Lua/Test/Portable/Lexical/LexicalTestData.cs
//
// The data tables the lexical tests iterate. The dropped SyntaxKind /
// SyntaxFacts (Portable/Syntax/, DROP) dock on the full_moon TokenType/Symbol:
// the C# kind enumeration maps to the full_moon Symbol values with the C#
// gating (SyntaxFacts.IsTokenOrTriviaKindEnabled, SyntaxFacts.cs:90-115);
// the C# token texts are the full_moon symbol texts (the dropped
// SyntaxFacts.g.cs GetText table is the same vocabulary); the constant values
// map per SyntaxFacts.GetConstantValue (SyntaxFacts.cs:214-223); the
// separator rules port from SyntaxFacts.RequiresSeparator (SyntaxFacts.cs:125-
// 207). Documented docking differences: the C# `&&`/`||`/`!` (c boolean
// operators) and the `<<=`/`>>=`/`&=`/`|=`/`?.` (cfxlua-only) have no C#
// SyntaxKind data rows (the C# has no such kinds) and the full_moon
// tokenizer has no `!` symbol at all — those rows are absent; the C#
// `//`-comment trivia (SingleLineCommentTrivia) and the FiveM hash strings
// (HashStringLiteralToken) dock on the full_moon CStyleComment and
// InterpolatedString kinds.

use full_moon::tokenizer::{InterpolatedStringKind, StringLiteralQuoteType, Symbol, TokenType};

use crate::shorttoken::{ShortToken, TextSpan, TokenValue};
use loretta::backtickstringtype::BacktickStringType;
use loretta::integerformats::IntegerFormats;
use loretta::luasyntaxoptions::LuaSyntaxOptions;
use loretta::utilities::hexfloat::HexFloat;

/// C# ParseLong (LexicalTestData.cs:11-21).
fn parse_long(text: &str, base: u32) -> i64 {
    let clean = text.replace('_', "");
    let digits = match base {
        2 | 8 | 16 => &clean[2..],
        10 => clean.as_str(),
        _ => panic!("invalid base"),
    };
    i64::from_str_radix(digits, base).expect("the value must parse")
}

/// C# ParseDouble (LexicalTestData.cs:23-33).
fn parse_double(text: &str, base: u32) -> f64 {
    match base {
        2 | 8 => parse_long(text, base) as f64,
        10 => text
            .replace('_', "")
            .parse::<f64>()
            .expect("the value must parse"),
        16 => {
            HexFloat::double_from_hex_string(&text.replace('_', "")).expect("the value must parse")
        }
        _ => panic!("invalid base"),
    }
}

/// C# Hash.GetJenkinsOneAtATimeHashCode
/// (Compilers/Core/Portable/InternalUtilities/Hash.cs:349-363).
fn jenkins_one_at_a_time_hash(value: &str) -> u32 {
    let mut hash: u32 = 0;
    for c in value.chars() {
        hash = hash.wrapping_add(c as u32);
        hash = hash.wrapping_add(hash << 10);
        hash ^= hash >> 6;
    }
    hash = hash.wrapping_add(hash << 3);
    hash ^= hash >> 11;
    hash = hash.wrapping_add(hash << 15);
    hash
}

/// The C# GetConstantValue (SyntaxFacts.cs:214-223) applied to a symbol row
/// with `.Or(text)` (the value defaults to the token text).
fn constant_value(symbol: Symbol, text: &str) -> Option<TokenValue> {
    match symbol {
        Symbol::True => Some(TokenValue::Bool(true)),
        Symbol::False => Some(TokenValue::Bool(false)),
        Symbol::Nil => Some(TokenValue::Nil),
        _ => Some(TokenValue::String(text.to_string())),
    }
}

/// A token row gate (the C# IsTokenOrTriviaKindEnabled switch cases).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Gate {
    /// Always enabled (the C# default branch — not manufactured, not
    /// disabled, not a compound assignment operator).
    Always,
    /// The compound assignment operators: disabled when
    /// !AcceptCompoundAssignment (SyntaxFacts.cs:113).
    AcceptCompoundAssignment,
    /// Disabled when !AcceptFloorDivision (SyntaxFacts.cs:95-96).
    AcceptFloorDivision,
    /// Disabled when !AcceptBitwiseOperators (SyntaxFacts.cs:99-103).
    AcceptBitwiseOperators,
    /// The TwoColons token: disabled when !AcceptGoto && !AcceptTypedLua
    /// (SyntaxFacts.cs:94).
    GotoOrTypedLua,
    /// The GotoKeyword is disabled by HasKeywordBeenDisabled when
    /// !AcceptGoto (SyntaxFacts.cs:52); the identifier row replaces it
    /// (LexicalTestData.cs:346).
    AcceptGoto,
    /// The DoubleSlashEquals token: disabled when !AcceptFloorDivision
    /// (SyntaxFacts.cs:95-96) and, when enabled, also gated by
    /// AcceptCompoundAssignment (SyntaxFacts.cs:113).
    FloorDivisionAndCompound,
}

impl Gate {
    fn enabled(self, options: &LuaSyntaxOptions) -> bool {
        match self {
            Gate::Always => true,
            Gate::AcceptCompoundAssignment => options.accept_compound_assignment,
            Gate::AcceptFloorDivision => options.accept_floor_division,
            Gate::AcceptBitwiseOperators => options.accept_bitwise_operators,
            Gate::GotoOrTypedLua => options.accept_goto || options.accept_typed_lua,
            Gate::AcceptGoto => options.accept_goto,
            Gate::FloorDivisionAndCompound => {
                options.accept_floor_division && options.accept_compound_assignment
            }
        }
    }
}

/// The keyword + symbol token rows (the C# SyntaxKind enumeration over the
/// enabled kinds). The `&&`/`||`/`!` C# rows have no full_moon symbols
/// (documented above).
const SYMBOL_ROWS: &[(Symbol, Gate)] = &[
    (Symbol::And, Gate::Always),
    (Symbol::Break, Gate::Always),
    (Symbol::Do, Gate::Always),
    (Symbol::Else, Gate::Always),
    (Symbol::ElseIf, Gate::Always),
    (Symbol::End, Gate::Always),
    (Symbol::False, Gate::Always),
    (Symbol::For, Gate::Always),
    (Symbol::Function, Gate::Always),
    (Symbol::If, Gate::Always),
    (Symbol::In, Gate::Always),
    (Symbol::Local, Gate::Always),
    (Symbol::Nil, Gate::Always),
    (Symbol::Not, Gate::Always),
    (Symbol::Or, Gate::Always),
    (Symbol::Repeat, Gate::Always),
    (Symbol::Return, Gate::Always),
    (Symbol::Then, Gate::Always),
    (Symbol::True, Gate::Always),
    (Symbol::Until, Gate::Always),
    (Symbol::While, Gate::Always),
    (Symbol::Goto, Gate::AcceptGoto),
    (Symbol::PlusEqual, Gate::AcceptCompoundAssignment),
    (Symbol::MinusEqual, Gate::AcceptCompoundAssignment),
    (Symbol::StarEqual, Gate::AcceptCompoundAssignment),
    (Symbol::SlashEqual, Gate::AcceptCompoundAssignment),
    (Symbol::DoubleSlashEqual, Gate::FloorDivisionAndCompound),
    (Symbol::PercentEqual, Gate::AcceptCompoundAssignment),
    (Symbol::CaretEqual, Gate::AcceptCompoundAssignment),
    (Symbol::TwoDotsEqual, Gate::AcceptCompoundAssignment),
    (Symbol::Ampersand, Gate::AcceptBitwiseOperators),
    (Symbol::ThinArrow, Gate::Always),
    (Symbol::TwoColons, Gate::GotoOrTypedLua),
    // The C# has no AtSign kind — the `@` is the C# bad character
    // (LexicalErrorTests.cs:299-323), so there is no data row.
    (Symbol::Caret, Gate::Always),
    (Symbol::Colon, Gate::Always),
    (Symbol::Comma, Gate::Always),
    (Symbol::Dot, Gate::Always),
    (Symbol::TwoDots, Gate::Always),
    (Symbol::Ellipsis, Gate::Always),
    (Symbol::Equal, Gate::Always),
    (Symbol::TwoEqual, Gate::Always),
    (Symbol::GreaterThan, Gate::Always),
    // The C# gates GreaterThanEqualsToken with AcceptBitwiseOperators
    // (SyntaxFacts.cs:102) — mirrored.
    (Symbol::GreaterThanEqual, Gate::AcceptBitwiseOperators),
    (Symbol::DoubleGreaterThan, Gate::AcceptBitwiseOperators),
    (Symbol::Hash, Gate::Always),
    (Symbol::LeftBrace, Gate::Always),
    (Symbol::LeftBracket, Gate::Always),
    (Symbol::LeftParen, Gate::Always),
    (Symbol::LessThan, Gate::Always),
    (Symbol::LessThanEqual, Gate::Always),
    (Symbol::DoubleLessThan, Gate::AcceptBitwiseOperators),
    (Symbol::Minus, Gate::Always),
    (Symbol::Percent, Gate::Always),
    (Symbol::Pipe, Gate::AcceptBitwiseOperators),
    (Symbol::Plus, Gate::Always),
    (Symbol::QuestionMark, Gate::Always),
    (Symbol::RightBrace, Gate::Always),
    (Symbol::RightBracket, Gate::Always),
    (Symbol::RightParen, Gate::Always),
    (Symbol::Semicolon, Gate::Always),
    (Symbol::Slash, Gate::Always),
    (Symbol::DoubleSlash, Gate::AcceptFloorDivision),
    (Symbol::Star, Gate::Always),
    (Symbol::Tilde, Gate::AcceptBitwiseOperators),
    (Symbol::TildeEqual, Gate::Always),
];

fn short_string_token(quote: char, text: &str, value: String) -> ShortToken {
    ShortToken::from_text(
        TokenType::StringLiteral {
            literal: text.into(),
            multi_line_depth: 0,
            quote_type: if quote == '\'' {
                StringLiteralQuoteType::Single
            } else {
                StringLiteralQuoteType::Double
            },
        },
        format!("{quote}{text}{quote}"),
        Some(TokenValue::String(value)),
    )
}

fn long_string_token(separator_count: usize, content: &str) -> ShortToken {
    let sep = "=".repeat(separator_count);
    ShortToken::from_text(
        TokenType::StringLiteral {
            literal: content.into(),
            multi_line_depth: separator_count,
            quote_type: StringLiteralQuoteType::Brackets,
        },
        format!("[{sep}[{content}]{sep}]"),
        Some(TokenValue::String(content.to_string())),
    )
}

/// C# GetDecimalNumberValue (LexicalTestData.cs:349-368).
fn decimal_number_value(options: &LuaSyntaxOptions, text: &str) -> Option<TokenValue> {
    if options.decimal_integer_format != IntegerFormats::NotSupported
        && !text.contains('.')
        && !text.contains('e')
    {
        match options.decimal_integer_format {
            IntegerFormats::Double => Some(TokenValue::Float(parse_long(text, 10) as f64)),
            IntegerFormats::Int64 => Some(TokenValue::Integer(parse_long(text, 10))),
            _ => panic!("invalid integer format"),
        }
    } else {
        Some(TokenValue::Float(parse_double(text, 10)))
    }
}

/// C# GetHexFloatValue (LexicalTestData.cs:370-389).
fn hex_float_value(options: &LuaSyntaxOptions, text: &str) -> Option<TokenValue> {
    if options.hex_integer_format != IntegerFormats::NotSupported
        && !text.contains('.')
        && !text.contains('p')
    {
        match options.hex_integer_format {
            IntegerFormats::Double => Some(TokenValue::Float(parse_long(text, 16) as f64)),
            IntegerFormats::Int64 => Some(TokenValue::Integer(parse_long(text, 16))),
            _ => panic!("invalid integer format"),
        }
    } else {
        Some(TokenValue::Float(parse_double(text, 16)))
    }
}

/// C# LexicalTestData.GetTokens (LexicalTestData.cs:35-390).
pub fn get_tokens(options: &LuaSyntaxOptions) -> Vec<ShortToken> {
    let mut tokens = Vec::new();

    // The keyword and symbol rows (the C# SyntaxKind enumeration with the
    // enabled filter + the constant values).
    for (symbol, gate) in SYMBOL_ROWS {
        if !gate.enabled(options) {
            continue;
        }
        let text = symbol.to_string();
        tokens.push(ShortToken::from_text(
            TokenType::Symbol { symbol: *symbol },
            text.clone(),
            constant_value(*symbol, &text),
        ));
    }

    // Numbers.

    // Binary.
    if options.accept_binary_numbers {
        for text in ["0b10", "0B10"] {
            let value = if options.binary_integer_format == IntegerFormats::Int64 {
                Some(TokenValue::Integer(parse_long(text, 2)))
            } else {
                Some(TokenValue::Float(parse_double(text, 2)))
            };
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                value,
            ));
        }
        if options.accept_underscore_in_number_literals {
            for text in ["0b10_10", "0B10_10"] {
                let value = if options.binary_integer_format == IntegerFormats::Int64 {
                    Some(TokenValue::Integer(parse_long(text, 2)))
                } else {
                    Some(TokenValue::Float(parse_double(text, 2)))
                };
                tokens.push(ShortToken::from_text(
                    TokenType::Number { text: text.into() },
                    text.to_string(),
                    value,
                ));
            }
        }
    }

    // Octal.
    if options.accept_octal_numbers {
        for text in ["0o77", "0O77"] {
            let value = if options.octal_integer_format == IntegerFormats::Int64 {
                Some(TokenValue::Integer(parse_long(text, 8)))
            } else {
                Some(TokenValue::Float(parse_double(text, 8)))
            };
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                value,
            ));
        }
        if options.accept_underscore_in_number_literals {
            for text in ["0o77_77", "0O77_77"] {
                let value = if options.octal_integer_format == IntegerFormats::Int64 {
                    Some(TokenValue::Integer(parse_long(text, 8)))
                } else {
                    Some(TokenValue::Float(parse_double(text, 8)))
                };
                tokens.push(ShortToken::from_text(
                    TokenType::Number { text: text.into() },
                    text.to_string(),
                    value,
                ));
            }
        }
    }

    // Decimal.
    for text in ["1", "1e10", "1.1", "1.1e10", ".1", ".1e10"] {
        tokens.push(ShortToken::from_text(
            TokenType::Number { text: text.into() },
            text.to_string(),
            decimal_number_value(options, text),
        ));
    }
    if options.accept_underscore_in_number_literals {
        for text in [
            "1_1",
            "1_1e1_0",
            "1_1.1_1",
            "1_1.1_1e1_0",
            ".1_1",
            ".1_1e1_0",
        ] {
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                decimal_number_value(options, text),
            ));
        }
    }

    // LuaJIT suffixes.
    if options.accept_lua_jit_number_suffixes {
        for text in [
            "10ULL",
            "20ULL",
            "200005ULL",
            "18446744073709551615ULL",
            "10uLL",
            "20uLL",
            "200005uLL",
            "18446744073709551615uLL",
        ] {
            let value = text[..text.len() - 3]
                .parse::<u64>()
                .expect("the value must parse");
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                Some(TokenValue::Unsigned(value)),
            ));
        }
        for text in [
            "10LL",
            "20LL",
            "200005LL",
            "9223372036854775807LL",
            "10lL",
            "20lL",
            "200005lL",
            "9223372036854775807lL",
        ] {
            let value = text[..text.len() - 2]
                .parse::<i64>()
                .expect("the value must parse");
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                Some(TokenValue::Integer(value)),
            ));
        }
        for text in ["100i", "999999999999999i", "100I", "999999999999999I"] {
            let value = parse_double(&text[..text.len() - 1], 10);
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                Some(TokenValue::Complex(value)),
            ));
        }

        // Binary.
        if options.accept_binary_numbers {
            for text in [
                "0b0001LL",
                "0b000111LL",
                "0b0111111111111111111111111111111111111111111111111111111111111111LL",
                "0b0001lL",
                "0b000111lL",
                "0b0111111111111111111111111111111111111111111111111111111111111111lL",
            ] {
                let value =
                    i64::from_str_radix(&text[2..text.len() - 2], 2).expect("the value must parse");
                tokens.push(ShortToken::from_text(
                    TokenType::Number { text: text.into() },
                    text.to_string(),
                    Some(TokenValue::Integer(value)),
                ));
            }
            for text in [
                "0b0001ULL",
                "0b000111ULL",
                "0b1111111111111111111111111111111111111111111111111111111111111111ULL",
                "0b0001uLl",
                "0b000111uLl",
                "0b1111111111111111111111111111111111111111111111111111111111111111uLl",
            ] {
                let value =
                    u64::from_str_radix(&text[2..text.len() - 3], 2).expect("the value must parse");
                tokens.push(ShortToken::from_text(
                    TokenType::Number { text: text.into() },
                    text.to_string(),
                    Some(TokenValue::Unsigned(value)),
                ));
            }
            for text in ["0b0001i", "0b111111i"] {
                let value = parse_double(&text[..text.len() - 1], 2);
                tokens.push(ShortToken::from_text(
                    TokenType::Number { text: text.into() },
                    text.to_string(),
                    Some(TokenValue::Complex(value)),
                ));
            }
        }

        // Hexadecimal.
        for text in [
            "0x11000013d077020LL",
            "0x7FFFFFFFFFFFFFFFLL",
            "0x11000013d077020lL",
            "0x7FFFFFFFFFFFFFFFlL",
        ] {
            let value =
                i64::from_str_radix(&text[2..text.len() - 2], 16).expect("the value must parse");
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                Some(TokenValue::Integer(value)),
            ));
        }
        for text in [
            "0x11000013d077020ULL",
            "0xFFFFFFFFFFFFFFFFULL",
            "0x11000013d077020uLl",
            "0xFFFFFFFFFFFFFFFFuLl",
        ] {
            let value =
                u64::from_str_radix(&text[2..text.len() - 3], 16).expect("the value must parse");
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                Some(TokenValue::Unsigned(value)),
            ));
        }
        for text in ["0x11i", "0x1020i", "0x11I", "0x1020I"] {
            let value = parse_double(&text[..text.len() - 1], 16);
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                Some(TokenValue::Complex(value)),
            ));
        }
    }

    // Hexadecimal.
    if options.accept_hex_float_literals {
        for text in ["0xf", "0xfp10", "0xf.f", "0xf.fp10", "0x.f", "0x.fp10"] {
            tokens.push(ShortToken::from_text(
                TokenType::Number { text: text.into() },
                text.to_string(),
                hex_float_value(options, text),
            ));
        }
        if options.accept_underscore_in_number_literals {
            for text in [
                "0xf_f",
                "0xf_f.f_f",
                "0xf_f.f_fp1_0",
                "0x.f_f",
                "0x.f_fp1_0",
                "0xf_fp1_0",
            ] {
                tokens.push(ShortToken::from_text(
                    TokenType::Number { text: text.into() },
                    text.to_string(),
                    hex_float_value(options, text),
                ));
            }
        }
    }

    // Strings.
    let mut short_string_content_text =
        String::from("hi\\n\\r\\b\\f\\n\\v\\1\\11\\111\\\n\\u{D800}\\u{10FFFF}\\xF\\xFF\\z ");
    let mut short_string_content_value =
        String::from("hi\n\r\u{8}\u{C}\n\u{B}\u{1}\u{B}\u{6F}\nu{D800}u{10FFFF}xFxFFz ");

    if options.accept_hex_escapes_in_strings {
        short_string_content_value = short_string_content_value.replace("xFxFF", "\u{F}\u{FF}");
    } else if !options.accept_invalid_escapes {
        short_string_content_text = short_string_content_text.replace("\\xF\\xFF", "");
        short_string_content_value = short_string_content_value.replace("xFxFF", "");
    }

    if options.accept_unicode_escape {
        // The C# replaces with the lone surrogate \uD800 + \u{10FFFF}; the
        // port's string model is valid UTF-8 (the C# UTF-16 lone surrogate
        // has no valid UTF-8 encoding) — the surrogate code points are
        // dropped, matching the string decoding in lexicaltestsbase.rs.
        short_string_content_value =
            short_string_content_value.replace("u{D800}u{10FFFF}", "\u{10FFFF}");
    } else if !options.accept_invalid_escapes {
        short_string_content_text = short_string_content_text.replace("\\u{D800}\\u{10FFFF}", "");
        short_string_content_value = short_string_content_value.replace("u{D800}u{10FFFF}", "");
    }

    if options.accept_whitespace_escape {
        short_string_content_value = short_string_content_value.replace("z ", "");
    } else if !options.accept_invalid_escapes {
        short_string_content_text = short_string_content_text.replace("\\z ", "");
        short_string_content_value = short_string_content_value.replace("z ", "");
    }

    if options.accept_invalid_escapes {
        short_string_content_text.push_str("\\l");
        short_string_content_value.push('l');
    }

    // Short strings.
    for quote in ['\'', '"'] {
        tokens.push(short_string_token(
            quote,
            &short_string_content_text,
            short_string_content_value.clone(),
        ));
    }

    // Long strings.
    const LONG_STRING_CONTENT: &str =
        "first line \n\nsecond line \r\n\nthird line \r\nfourth line \u{FF}.\n";
    for separator_count in 0..6 {
        tokens.push(long_string_token(separator_count, LONG_STRING_CONTENT));
    }

    if options.backtick_string_type == BacktickStringType::HashLiteral {
        // The C# HashStringLiteralToken (FiveM hash strings) docks on the
        // full_moon InterpolatedString kind (the full_moon tokenizer has no
        // hash-string token — documented above).
        let value = jenkins_one_at_a_time_hash(&short_string_content_value.to_lowercase());
        tokens.push(ShortToken::from_text(
            TokenType::InterpolatedString {
                literal: short_string_content_text.clone().into(),
                kind: InterpolatedStringKind::Simple,
            },
            format!("`{short_string_content_text}`"),
            Some(TokenValue::Unsigned(value as u64)),
        ));
    } else if options.backtick_string_type == BacktickStringType::InterpolatedStringLiteral {
        let text = format!("`{short_string_content_text}`");
        tokens.push(ShortToken::from_text(
            TokenType::InterpolatedString {
                literal: short_string_content_text.clone().into(),
                kind: InterpolatedStringKind::Simple,
            },
            text.clone(),
            Some(TokenValue::String(text)),
        ));
    }

    // Identifiers.
    if options.use_lua_jit_identifier_rules {
        for identifier in [
            "a", "abc", "_", "🅱", "\u{FEFF}", "\u{206B}", "\u{202A}", "\u{206A}", "\u{FEFF}",
            "\u{206A}", "\u{200E}", "\u{200C}", "\u{200E}",
        ] {
            tokens.push(ShortToken::from_text(
                TokenType::Identifier {
                    identifier: identifier.into(),
                },
                identifier.to_string(),
                Some(TokenValue::String(identifier.to_string())),
            ));
        }
    }

    if options.continue_type == loretta::continuetype::ContinueType::None {
        // The C# tail row: the `continue` identifier when the keyword is
        // disabled (LexicalTestData.cs:345).
        tokens.push(ShortToken::from_text(
            TokenType::Identifier {
                identifier: "continue".into(),
            },
            "continue".to_string(),
            Some(TokenValue::String("continue".to_string())),
        ));
    } else if options.continue_type == loretta::continuetype::ContinueType::Keyword {
        // The C# ContinueKeyword row (the keyword enabled for the non-
        // contextual continue types — LexicalTestData.cs:35-46); the full_moon
        // tokenizer has no continue symbol — the row docks on the identifier.
        tokens.push(ShortToken::from_text(
            TokenType::Identifier {
                identifier: "continue".into(),
            },
            "continue".to_string(),
            Some(TokenValue::String("continue".to_string())),
        ));
    }
    if !options.accept_goto {
        tokens.push(ShortToken::from_text(
            TokenType::Identifier {
                identifier: "goto".into(),
            },
            "goto".to_string(),
            Some(TokenValue::String("goto".to_string())),
        ));
    }

    tokens
}

/// The enabled symbol rows for the options (the C# IsTokenOrTriviaKindEnabled
/// enumeration — used by the row-863 Lexer_Covers_AllTokens check).
pub fn enabled_symbols(options: &LuaSyntaxOptions) -> Vec<Symbol> {
    SYMBOL_ROWS
        .iter()
        .filter(|(_, gate)| gate.enabled(options))
        .map(|(symbol, _)| *symbol)
        .collect()
}

/// C# GetSeparators (LexicalTestData.cs:400-418).
fn get_separators(options: &LuaSyntaxOptions) -> Vec<ShortToken> {
    let mut rows = Vec::new();
    for ws in [" ", "  ", "\t"] {
        rows.push(ShortToken::from_text(
            TokenType::Whitespace {
                characters: ws.into(),
            },
            ws.to_string(),
            None,
        ));
    }
    // The C# EndOfLineTrivia docks on the full_moon Whitespace kind (the
    // full_moon has no separate end-of-line kind — documented).
    for eol in ["\r", "\n", "\r\n"] {
        rows.push(ShortToken::from_text(
            TokenType::Whitespace {
                characters: eol.into(),
            },
            eol.to_string(),
            None,
        ));
    }
    if options.accept_c_comment_syntax {
        // The full_moon CStyleComment stores the content between the `/*`
        // and `*/` (not the full text).
        for (content, full) in [("", "/**/"), ("\naaa\n", "/*\naaa\n*/")] {
            rows.push(ShortToken::from_text(
                TokenType::CStyleComment {
                    comment: content.into(),
                },
                full.to_string(),
                None,
            ));
        }
    }
    // The full_moon MultiLineComment stores the content between the `--[[`
    // and `]]` and the number of `=` signs in the opener (the blocks).
    for (content, equals, full) in [
        ("", 0, "--[[]]"),
        ("\naaa\n", 0, "--[[\naaa\n]]"),
        ("", 1, "--[=[]=]"),
        ("\naaa\n", 1, "--[=[\naaa\n]=]"),
        ("", 4, "--[====[]====]"),
        ("\naaa\n", 4, "--[====[\naaa\n]====]"),
    ] {
        rows.push(ShortToken::from_text(
            TokenType::MultiLineComment {
                comment: content.into(),
                blocks: equals,
            },
            full.to_string(),
            None,
        ));
    }
    rows
}

/// C# LexicalTestData.GetTrivia (LexicalTestData.cs:392-398).
pub fn get_trivia(options: &LuaSyntaxOptions) -> Vec<ShortToken> {
    let mut trivia = get_separators(options);
    // The full_moon SingleLineComment stores the content after the `--`.
    trivia.push(ShortToken::from_text(
        TokenType::SingleLineComment {
            comment: " hi".into(),
        },
        "-- hi".to_string(),
        None,
    ));
    if options.accept_c_comment_syntax {
        // The C# `// hi` row is a SingleLineCommentTrivia; the full_moon
        // tokenizes C-style comments as CStyleComment (documented); the
        // comment field holds the content after the `//`.
        trivia.push(ShortToken::from_text(
            TokenType::CStyleComment {
                comment: " hi".into(),
            },
            "// hi".to_string(),
            None,
        ));
    }
    trivia.push(ShortToken::from_text(
        TokenType::Shebang {
            line: "#!/bin/bash".into(),
        },
        "#!/bin/bash".to_string(),
        None,
    ));
    trivia
}

/// The C# SyntaxFacts.IsKeyword (the keyword kinds).
fn is_keyword(kind: &TokenType) -> bool {
    matches!(
        kind,
        TokenType::Symbol {
            symbol: Symbol::And
                | Symbol::Break
                | Symbol::Do
                | Symbol::Else
                | Symbol::ElseIf
                | Symbol::End
                | Symbol::False
                | Symbol::For
                | Symbol::Function
                | Symbol::Goto
                | Symbol::If
                | Symbol::In
                | Symbol::Local
                | Symbol::Nil
                | Symbol::Not
                | Symbol::Or
                | Symbol::Repeat
                | Symbol::Return
                | Symbol::Then
                | Symbol::True
                | Symbol::Until
                | Symbol::While
        }
    )
}

fn is_identifier(kind: &TokenType) -> bool {
    matches!(kind, TokenType::Identifier { .. })
}

fn is_number(kind: &TokenType) -> bool {
    matches!(kind, TokenType::Number { .. })
}

fn is_string(kind: &TokenType) -> bool {
    matches!(kind, TokenType::StringLiteral { .. })
}

fn is_comment(kind: &TokenType) -> bool {
    matches!(
        kind,
        TokenType::SingleLineComment { .. }
            | TokenType::MultiLineComment { .. }
            | TokenType::CStyleComment { .. }
    )
}

fn is_symbol(kind: &TokenType, symbol: Symbol) -> bool {
    matches!(kind, TokenType::Symbol { symbol: s } if *s == symbol)
}

/// C# SyntaxFacts.RequiresSeparator (SyntaxFacts.cs:125-207). The C# rules
/// reference the kinds by the dropped SyntaxKind names; the port maps them to
/// the full_moon TokenType values. The `!` (BangToken) rules have no full_moon
/// symbol (the tokenizer has no `!` — documented above).
pub fn requires_separator(
    kind_a: &TokenType,
    text_a: &str,
    kind_b: &TokenType,
    text_b: &str,
) -> bool {
    let a_is_keyword = is_keyword(kind_a);
    let b_is_keyword = is_keyword(kind_b);

    if is_identifier(kind_a) && is_identifier(kind_b) {
        return true;
    }
    if a_is_keyword && b_is_keyword {
        return true;
    }
    if a_is_keyword && is_identifier(kind_b) {
        return true;
    }
    if is_identifier(kind_a) && b_is_keyword {
        return true;
    }
    if is_identifier(kind_a) && is_number(kind_b) {
        return true;
    }
    if is_number(kind_a) && is_identifier(kind_b) {
        return true;
    }
    if is_number(kind_a) && b_is_keyword {
        return true;
    }
    if is_number(kind_a)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Dot | Symbol::TwoDots | Symbol::Ellipsis | Symbol::TwoDotsEqual
            }
        )
    {
        return true;
    }
    if a_is_keyword && is_number(kind_b) {
        return true;
    }
    if is_number(kind_a) && is_number(kind_b) {
        return true;
    }
    if is_symbol(kind_a, Symbol::LeftBracket) && is_symbol(kind_b, Symbol::LeftBracket) {
        return true;
    }
    if is_symbol(kind_a, Symbol::LeftBracket) && is_string(kind_b) && text_b.starts_with('[') {
        return true;
    }
    if is_symbol(kind_a, Symbol::Colon)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Colon | Symbol::TwoColons
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Plus)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Minus)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Minus) && is_comment(kind_b) && text_b.starts_with('-') {
        return true;
    }
    if is_symbol(kind_a, Symbol::Minus)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Minus | Symbol::MinusEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Star)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Slash)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::SlashEqual | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Slash)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Slash | Symbol::Star | Symbol::StarEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Slash) && is_comment(kind_b) && text_b.starts_with('/') {
        return true;
    }
    if is_symbol(kind_a, Symbol::Caret)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Percent)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::TwoDots)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if matches!(
        kind_a,
        TokenType::Symbol {
            symbol: Symbol::Dot | Symbol::TwoDots
        }
    ) && matches!(
        kind_b,
        TokenType::Symbol {
            symbol: Symbol::Dot | Symbol::TwoDots | Symbol::Ellipsis | Symbol::TwoDotsEqual
        }
    ) {
        return true;
    }
    if is_symbol(kind_a, Symbol::Equal)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::LessThan)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::LessThan
                    | Symbol::LessThanEqual
                    | Symbol::Equal
                    | Symbol::TwoEqual
                    | Symbol::DoubleLessThan
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::GreaterThan)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::GreaterThan
                    | Symbol::GreaterThanEqual
                    | Symbol::Equal
                    | Symbol::TwoEqual
                    | Symbol::DoubleGreaterThan
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Ampersand)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Ampersand
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Pipe)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Pipe
            }
        )
    {
        return true;
    }
    // Dot can be the start of a number.
    if matches!(
        kind_a,
        TokenType::Symbol {
            symbol: Symbol::Dot | Symbol::TwoDots | Symbol::Ellipsis
        }
    ) && is_number(kind_b)
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Tilde)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::Equal | Symbol::TwoEqual
            }
        )
    {
        return true;
    }
    if is_symbol(kind_a, Symbol::Minus)
        && matches!(
            kind_b,
            TokenType::Symbol {
                symbol: Symbol::ThinArrow | Symbol::GreaterThan | Symbol::GreaterThanEqual
            }
        )
    {
        return true;
    }
    if matches!(
        kind_a,
        TokenType::Symbol {
            symbol: Symbol::Slash | Symbol::DoubleSlash
        }
    ) && matches!(
        kind_b,
        TokenType::Symbol {
            symbol: Symbol::Slash
                | Symbol::DoubleSlash
                | Symbol::SlashEqual
                | Symbol::DoubleSlashEqual
                | Symbol::Equal
                | Symbol::TwoEqual
        }
    ) {
        return true;
    }
    let _ = text_a;
    false
}

/// C# LexicalTestData.GetTokenPairs (LexicalTestData.cs:420-425).
pub fn get_token_pairs(options: &LuaSyntaxOptions) -> Vec<(ShortToken, ShortToken)> {
    let tokens = get_tokens(options);
    let mut pairs = Vec::new();
    for token_a in &tokens {
        for token_b in &tokens {
            if requires_separator(&token_a.kind, &token_a.text, &token_b.kind, &token_b.text) {
                continue;
            }
            let mut token_b = token_b.clone();
            token_b.span = TextSpan::new(token_a.span.end(), token_b.span.length());
            pairs.push((token_a.clone(), token_b));
        }
    }
    pairs
}

/// C# LexicalTestData.GetTokenPairsWithSeparators (LexicalTestData.cs:427-437).
pub fn get_token_pairs_with_separators(
    options: &LuaSyntaxOptions,
) -> Vec<(ShortToken, ShortToken, ShortToken)> {
    let tokens = get_tokens(options);
    let separators = get_separators(options);
    let mut triples = Vec::new();
    for token_a in &tokens {
        for token_b in &tokens {
            if requires_separator(&token_a.kind, &token_a.text, &token_b.kind, &token_b.text) {
                continue;
            }
            for separator in &separators {
                if requires_separator(
                    &token_a.kind,
                    &token_a.text,
                    &separator.kind,
                    &separator.text,
                ) || requires_separator(
                    &separator.kind,
                    &separator.text,
                    &token_b.kind,
                    &token_b.text,
                ) {
                    continue;
                }
                let mut separator = separator.clone();
                separator.span = TextSpan::new(token_a.span.end(), separator.span.length());
                let mut token_b = token_b.clone();
                token_b.span = TextSpan::new(separator.span.end(), token_b.span.length());
                triples.push((token_a.clone(), separator, token_b));
            }
        }
    }
    triples
}
