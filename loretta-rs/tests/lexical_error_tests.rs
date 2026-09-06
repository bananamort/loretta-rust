// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.LexicalErrorTests (b767b4e): LexicalErrorTests
// C# source: src/Compilers/Lua/Test/Portable/Lexical/LexicalErrorTests.cs
//
// The C# runner (ParsingTestsBase.ParseAndValidateAsync) parses the source and
// verifies the tree diagnostics against the expected descriptions (code +
// squiggled span text + start position + message arguments). The port's
// lexer is full_moon (the C# lexer is DROP), so the diagnostics come from the
// lexerdiagnostics module (the ported C# lexer rules over the source text).
// The round-trip check runs only for error-free parses (full_moon reconstructs
// erroring sources; the C# tree keeps the raw lexed text — documented).

use loretta::backtickstringtype::BacktickStringType;
use loretta::errors::errorcode::ErrorCode;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::lexerdiagnostics::lexer_diagnostics;

/// The expected diagnostic (the C# DiagnosticDescription).
struct Expected {
    code: ErrorCode,
    line: usize,
    col: usize,
    squiggle: &'static str,
    args: Vec<&'static str>,
}

fn exp(code: ErrorCode, line: usize, col: usize, squiggle: &'static str) -> Expected {
    Expected {
        code,
        line,
        col,
        squiggle,
        args: Vec::new(),
    }
}
fn exp_args(
    code: ErrorCode,
    line: usize,
    col: usize,
    squiggle: &'static str,
    args: Vec<&'static str>,
) -> Expected {
    Expected {
        code,
        line,
        col,
        squiggle,
        args,
    }
}

/// The C# ParsingTestsBase.ParseAndValidateAsync (ParsingTestsBase.cs:44-51):
/// the TREE diagnostics (the lexer scanner merged with the parser
/// diagnostics pass over the recovered AST, one copy — the C# tree carries
/// each diagnostic once). The full_moon parse panics on fatal-tokenizer
/// inputs such as `@\$\` (its luau-attribute path calls `current().unwrap()`,
/// ast/parsers.rs:3709) and fails on unparseable sources — those cases skip
/// the merge and assert the scanner diagnostics alone (documented: the C#
/// recovers a tree there; the general recovery diagnostics are unported).
fn parse_and_validate_lex(source: &str, options: &LuaSyntaxOptions, expected: &[Expected]) {
    let mut produced = lexer_diagnostics(source, options);
    let source_owned = source.to_string();
    let options_owned = options.clone();
    let parse_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        full_moon::parse(&source_owned).ok().map(|ast| {
            loretta::errors::parserdiagnostics::parser_diagnostics(
                &ast,
                &options_owned,
                &source_owned,
            )
        })
    }));
    if let Ok(Some(mut parser_diags)) = parse_result {
        produced.append(&mut parser_diags);
        produced.sort_by_key(|d| (d.sort_site, !d.node_level));
    }
    assert_eq!(
        produced.len(),
        expected.len(),
        "diagnostic count for {source:?}: produced={produced:?}"
    );
    for (i, (actual, exp)) in produced.iter().zip(expected.iter()).enumerate() {
        let (line, col) = actual.line_col(source);
        assert_eq!(actual.code, exp.code, "diag {i} code for {source:?}");
        assert_eq!(
            (line, col),
            (exp.line, exp.col),
            "diag {i} position for {source:?} ({:?})",
            actual.squiggle(source)
        );
        assert_eq!(
            actual.squiggle(source),
            exp.squiggle,
            "diag {i} squiggle for {source:?}"
        );
        assert_eq!(
            actual.arguments,
            exp.args.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
            "diag {i} args for {source:?}"
        );
    }
}

#[test]
fn lexer_emits_diagnostics_on_invalid_escapes() {
    let source = r#"local str = "some\ltext"
local str = 'some\ltext'
local str = "some\xGtext"
local str = 'some\xGtext'
local str = "some\300text"
local str = 'some\300text'"#;
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp(ErrorCode::ErrInvalidStringEscape, 1, 18, r"\l"),
            exp(ErrorCode::ErrInvalidStringEscape, 2, 18, r"\l"),
            exp(ErrorCode::ErrInvalidStringEscape, 3, 18, r"\x"),
            exp(ErrorCode::ErrInvalidStringEscape, 4, 18, r"\x"),
            exp(ErrorCode::ErrInvalidStringEscape, 5, 18, r"\300"),
            exp(ErrorCode::ErrInvalidStringEscape, 6, 18, r"\300"),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_on_string_with_line_break_but_lexes_rest_properly() {
    let source =
        "local str1 = \"some\nlocal str2 = 'some\r\nlocal str3 = \"some\rlocal str4 = 'some";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp(ErrorCode::ErrUnfinishedString, 1, 14, "\"some"),
            exp(ErrorCode::ErrUnfinishedString, 2, 14, "'some"),
            exp(ErrorCode::ErrUnfinishedString, 3, 14, "\"some"),
            exp(ErrorCode::ErrUnfinishedString, 4, 14, "'some"),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_on_interpolated_string_with_line_break_but_lexes_rest_properly() {
    let source = "local str1 = `some\nlocal str2 = `some\r\nlocal str3 = `some\rlocal str4 = `some";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp(ErrorCode::ErrUnfinishedString, 1, 18, "e"),
            exp(ErrorCode::ErrUnfinishedString, 2, 18, "e"),
            exp(ErrorCode::ErrUnfinishedString, 3, 18, "e"),
            exp(ErrorCode::ErrUnfinishedString, 4, 18, "e"),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_on_unterminated_short_string() {
    let cases: &[&str] = &["\"text", "'text", "\"text'", "'text\""];
    for text in cases {
        let source = format!("local str = {text}");
        parse_and_validate_lex(
            &source,
            &LuaSyntaxOptions::ALL,
            &[exp(ErrorCode::ErrUnfinishedString, 1, 13, text)],
        );
    }
}

#[test]
fn lexer_emits_diagnostics_on_invalid_numbers() {
    let source = "local num1 = 0b\nlocal num2 = 0b_\nlocal num3 = 0o\nlocal num4 = 0o_";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp(ErrorCode::ErrInvalidNumber, 1, 14, "0b"),
            exp(ErrorCode::ErrInvalidNumber, 2, 14, "0b_"),
            exp(ErrorCode::ErrInvalidNumber, 3, 14, "0o"),
            exp(ErrorCode::ErrInvalidNumber, 4, 14, "0o_"),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_on_large_numbers_and_overflows() {
    let source =
        "local num1 = 0b10000000000000000000000000000000000000000000000000000000000000000\n\
local num2 = 0o1000000000000000000000\n\
local num3 = 1e999999\n\
local num4 = 0x1p999999";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                1,
                14,
                "0b10000000000000000000000000000000000000000000000000000000000000000",
            ),
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                2,
                14,
                "0o1000000000000000000000",
            ),
            exp(ErrorCode::ErrDoubleOverflow, 3, 14, "1e999999"),
            exp(ErrorCode::ErrDoubleOverflow, 4, 14, "0x1p999999"),
        ],
    );
}

#[test]
fn lexer_gates_only_shift_operators_not_single_bitwise_chars() {
    // Finding 22: the C# lexer errors ONLY on '<<' (Lexer.cs:501-507);
    // the single '&'/'|' binary operators are the parser's rule
    // (LanguageParser.cs:908-912 — ported in parserdiagnostics.rs) and
    // '>>' gets no error at all (the parser combines two '>' tokens
    // silently, LanguageParser.cs:840-845). The C# TREE carries all three
    // gates (probed @Lua51 via the harness: 3 unique LUA0021s — '<<' at
    // (1,13), '&' at (3,15), '|' at (4,15); the harness op doubles each),
    // so this test asserts through the combined tree diagnostics.
    parse_and_validate_lex(
        "local a = 1 << 2\n\
         local b = 3 >> 1\n\
         local c = x & y\n\
         local d = p | q\n",
        &LuaSyntaxOptions::LUA51,
        &[
            exp(
                ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion,
                1,
                13,
                "<<",
            ),
            exp(
                ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion,
                3,
                13,
                "&",
            ),
            exp(
                ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion,
                4,
                13,
                "|",
            ),
        ],
    );
}

#[test]
fn lexer_complex_hex_suffix_reports_only_double_overflow() {
    // Finding 21: the complex 'i' suffix is a double value — the C#
    // HexFloat path (Lexer.Numbers.cs:380-394) reports only
    // DoubleOverflow on a real overflow, never the integer TooLarge
    // (0x10000000000000000i fits the double — the old code reported
    // TooLarge for the 17-digit builder). Every case pinned against the
    // C# oracle on AllWithIntegers.
    parse_and_validate_lex(
        "local num1 = 0xffi\n\
         local num2 = 0x10000000000000000i\n\
         local num3 = 0x1p10i\n\
         local num4 = 0x1p1024i",
        &LuaSyntaxOptions::ALL_WITH_INTEGERS,
        &[exp(ErrorCode::ErrDoubleOverflow, 4, 14, "0x1p1024i")],
    );
}

#[test]
fn lexer_reports_toolarge_for_hex_values_wider_than_64_bits() {
    // Finding 20: the C# TryParse paths fail only on values wider than 64
    // bits — the 64-bit patterns parse as two's-complement longs (no
    // error) and the ull suffix covers the full u64 range.
    // Every case pinned against the C# oracle on AllWithIntegers.
    parse_and_validate_lex(
        "local num1 = 0xffffffffffffffff\n\
         local num2 = 0x8000000000000000\n\
         local num3 = 0xffffffffffffffffull\n\
         local num4 = 0x10000000000000000\n\
         local num5 = 0x10000000000000000ull",
        &LuaSyntaxOptions::ALL_WITH_INTEGERS,
        &[
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                4,
                14,
                "0x10000000000000000",
            ),
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                5,
                14,
                "0x10000000000000000ull",
            ),
        ],
    );
}

#[test]
fn lexer_does_not_report_invalid_number_for_digitless_hex() {
    // Finding 18: the C# hex parser has no digit-less ErrInvalidNumber
    // rule — only the binary and octal parsers do
    // (Lexer.Numbers.cs:81-85, 156-160). The Int64-format presets report
    // ErrNumericLiteralTooLarge instead (the C# long.TryParse("")
    // failure, Lexer.Numbers.cs:417-418).
    parse_and_validate_lex(
        "local num1 = 0x\nlocal num2 = 0x_\n",
        &LuaSyntaxOptions::ALL_WITH_INTEGERS,
        &[
            exp(ErrorCode::ErrNumericLiteralTooLarge, 1, 14, "0x"),
            exp(ErrorCode::ErrNumericLiteralTooLarge, 2, 14, "0x_"),
        ],
    );
    // The double-only presets route the integer hex through
    // HexFloat.DoubleFromHexString — no error for valid hex (the C#
    // throws FormatException on the digit-less builder — the port's
    // anti-crash silence).
    parse_and_validate_lex(
        "local num1 = 0xff\nlocal num2 = 0x\n",
        &LuaSyntaxOptions::ALL,
        &[],
    );
}

#[test]
fn lexer_emits_diagnostic_on_decimal_integer_overflow() {
    // Finding 17: the decimal path never accumulated the value, so the C#
    // long/ulong.TryParse failures (Lexer.Numbers.cs:248-251, 256-259,
    // 280-283) were never reported.
    parse_and_validate_lex(
        "local num1 = 9223372036854775808\n\
         local num2 = 18446744073709551615\n\
         local num3 = 18446744073709551615ULL\n\
         local num4 = 18446744073709551616ULL\n\
         local num5 = 9223372036854775808ll",
        &LuaSyntaxOptions::ALL_WITH_INTEGERS,
        &[
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                1,
                14,
                "9223372036854775808",
            ),
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                2,
                14,
                "18446744073709551615",
            ),
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                4,
                14,
                "18446744073709551616ULL",
            ),
            exp(
                ErrorCode::ErrNumericLiteralTooLarge,
                5,
                14,
                "9223372036854775808ll",
            ),
        ],
    );
    // The double-only presets parse the integer as a double
    // (RealParser) — 2^63 fits, so there is no error (Lexer.Numbers.cs:
    // 272-279); the suffix paths stay silent when the value fits the
    // ulong/long.
    parse_and_validate_lex(
        "local num1 = 9223372036854775808\n\
         local num2 = 18446744073709551615ULL",
        &LuaSyntaxOptions::ALL,
        &[],
    );
}

#[test]
fn lexer_emits_diagnostic_on_unfinished_long_comment() {
    let cases: &[&str] = &["/* hi", "--[[ hi", "--[=[ hi"];
    for text in cases {
        parse_and_validate_lex(
            text,
            &LuaSyntaxOptions::ALL,
            &[exp(ErrorCode::ErrUnfinishedLongComment, 1, 1, text)],
        );
    }
}

#[test]
fn lexer_shebang_guard_follows_the_csharp_trivia_semantics() {
    // Finding 25: the C# guard starts true at each trivia run (after
    // every token) and clears on '\v'/'\f'/comments; the newline and the
    // space/tab fast path keep it (Lexer.cs:725-798). The port's
    // scan-global flag re-armed by newlines and never cleared by
    // '\v'/'\f' diverged. Every case pinned against the C# oracle on
    // Lua51 (the oracle's LUA1012 for the first two inputs is the C#
    // parser's statement rule, not ported here — only the recognition
    // flip is asserted).
    let options = LuaSyntaxOptions {
        accept_shebang: false,
        ..LuaSyntaxOptions::ALL
    };
    // '\v' clears the guard: not a shebang — no shebang diagnostic.
    parse_and_validate_lex("\u{0B}#! foo\n", &options, &[]);
    // A comment clears the guard and the newline does NOT re-arm it:
    // not a shebang.
    parse_and_validate_lex("-- c\n#! foo\n", &options, &[]);
    // The token re-arms the guard and the tab keeps it: a shebang.
    parse_and_validate_lex(
        "1\t#! foo\n",
        &options,
        &[exp(
            ErrorCode::ErrShebangNotSupportedInLuaVersion,
            1,
            3,
            "#! foo",
        )],
    );
}

#[test]
fn lexer_emits_diagnostic_when_shebang_is_found_and_lua_syntax_options_accept_shebang_is_false() {
    let source = "#!/bin/bash";
    let options = LuaSyntaxOptions {
        accept_shebang: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[exp(
            ErrorCode::ErrShebangNotSupportedInLuaVersion,
            1,
            1,
            source,
        )],
    );
}

#[test]
fn lexer_emits_diagnostic_when_binary_number_is_found_and_lua_syntax_options_accept_binary_numbers_is_false(
) {
    let source = "local num = 0b1010";
    let options = LuaSyntaxOptions {
        accept_binary_numbers: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[exp(
            ErrorCode::ErrBinaryNumericLiteralNotSupportedInVersion,
            1,
            13,
            "0b1010",
        )],
    );
}

#[test]
fn lexer_emits_diagnostic_when_octal_number_is_found_and_lua_syntax_options_accept_octal_numbers_is_false(
) {
    let source = "local num = 0o77";
    let options = LuaSyntaxOptions {
        accept_octal_numbers: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[exp(
            ErrorCode::ErrOctalNumericLiteralNotSupportedInVersion,
            1,
            13,
            "0o77",
        )],
    );
}

#[test]
fn lexer_emits_diagnostic_when_hex_float_is_found_and_lua_syntax_options_accept_hex_float_is_false()
{
    let source = "local num1 = 0xff.ff\nlocal num2 = 0xffp10\nlocal num3 = 0xff.ffp10";
    let options = LuaSyntaxOptions {
        accept_hex_float_literals: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrHexFloatLiteralNotSupportedInVersion,
                1,
                14,
                "0xff.ff",
            ),
            exp(
                ErrorCode::ErrHexFloatLiteralNotSupportedInVersion,
                2,
                14,
                "0xffp10",
            ),
            exp(
                ErrorCode::ErrHexFloatLiteralNotSupportedInVersion,
                3,
                14,
                "0xff.ffp10",
            ),
        ],
    );
}

#[test]
fn lexer_emits_underscore_diagnostic_twice_for_prefixed_hex_literals() {
    // Finding 24: the C# reports the underscore gating twice for the
    // prefixed literals with an underscore at the prefix position —
    // once at the dispatch (Lexer.cs:562-591, the '0'..prefix span) and
    // once in the parser (Lexer.Numbers.cs:359-360, the full token
    // text). The binary/octal in-parser checks use the digit-loop flag
    // (Lexer.Numbers.cs:76-77, 153-154), so their prefix underscore
    // stays single.
    let source = "local num1 = 0_b101\nlocal num2 = 0_xF\nlocal num3 = 0b1_01";
    let options = LuaSyntaxOptions {
        accept_underscore_in_number_literals: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                1,
                14,
                "0_b",
            ),
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                2,
                14,
                "0_x",
            ),
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                2,
                14,
                "0_xF",
            ),
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                3,
                14,
                "0b1_01",
            ),
        ],
    );
}

#[test]
fn lexer_emits_diagnostic_when_underscore_in_number_is_found_and_lua_syntax_options_accept_underscores_in_numbers_is_false(
) {
    let source = "local num1 = 0b1010_1010\nlocal num2 = 0o7070_7070\nlocal num3 = 10_10.10_10\nlocal num4 = 0xf_f";
    let options = LuaSyntaxOptions {
        accept_underscore_in_number_literals: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                1,
                14,
                "0b1010_1010",
            ),
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                2,
                14,
                "0o7070_7070",
            ),
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                3,
                14,
                "10_10.10_10",
            ),
            exp(
                ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                4,
                14,
                "0xf_f",
            ),
        ],
    );
}

#[test]
fn lexer_emits_diagnostic_when_identifiers_with_characters_above_0x7f_are_found_and_lua_syntax_options_use_luajit_identifier_rules_is_false(
) {
    let source = "local 🅱 = 1\r\n\
local \u{FEFF} = 1 -- ZERO WIDTH NO-BREAK SPACE\r\n\
local \u{206B} = 1 -- ACTIVATE SYMMETRIC SWAPPING\r\n\
local \u{202A} = 1 -- LEFT-TO-RIGHT EMBEDDING\r\n\
local \u{206A} = 1 -- INHIBIT SYMMETRIC SWAPPING\r\n\
local \u{200E} = 1 -- LEFT-TO-RIGHT MARK\r\n\
local \u{200C} = 1 -- ZERO WIDTH NON-JOINER";
    let options = LuaSyntaxOptions {
        use_lua_jit_identifier_rules: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
                1,
                7,
                "🅱",
            ),
            exp(
                ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
                2,
                7,
                "\u{FEFF}",
            ),
            exp(
                ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
                3,
                7,
                "\u{206B}",
            ),
            exp(
                ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
                4,
                7,
                "\u{202A}",
            ),
            exp(
                ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
                5,
                7,
                "\u{206A}",
            ),
            exp(
                ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
                6,
                7,
                "\u{200E}",
            ),
            exp(
                ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
                7,
                7,
                "\u{200C}",
            ),
        ],
    );
}

#[test]
fn lexer_absorbs_bad_character_runaways_past_200_tokens() {
    // Finding 26: the C# absorbs the rest of the input when the bad-token
    // count passes 200 (Lexer.cs:700-713) — the 202nd bad token is one
    // BadCharacter + one InvalidStatement over the remainder instead of
    // per-character errors.
    let source = "@".repeat(205);
    let mut expected: Vec<Expected> = Vec::new();
    for i in 1..=201 {
        expected.push(exp_args(ErrorCode::ErrBadCharacter, 1, i, "@", vec!["@"]));
        expected.push(exp(ErrorCode::ErrInvalidStatement, 1, i, "@"));
    }
    // The 202nd token absorbs the remaining four characters (the C#
    // BadCharacter argument is the absorbed text).
    expected.push(exp_args(
        ErrorCode::ErrBadCharacter,
        1,
        202,
        "@@@@",
        vec!["@@@@"],
    ));
    expected.push(exp(ErrorCode::ErrInvalidStatement, 1, 202, "@@@@"));
    parse_and_validate_lex(&source, &LuaSyntaxOptions::ALL, &expected);
}

#[test]
fn lexer_emits_diagnostic_when_bad_characters_are_found() {
    let source = "@$\\";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp_args(ErrorCode::ErrBadCharacter, 1, 1, "@", vec!["@"]),
            exp(ErrorCode::ErrInvalidStatement, 1, 1, "@"),
            exp_args(ErrorCode::ErrBadCharacter, 1, 2, "$", vec!["$"]),
            exp(ErrorCode::ErrInvalidStatement, 1, 2, "$"),
            exp_args(ErrorCode::ErrBadCharacter, 1, 3, "\\", vec!["\\"]),
            exp(ErrorCode::ErrInvalidStatement, 1, 3, "\\"),
        ],
    );
}

#[test]
fn hex_escape_missing_digits_are_silent_under_the_csharp_goto_default() {
    // Finding 43: under AcceptInvalidEscapes && !AcceptHexEscapesInStrings
    // the C# \x jumps to the default case (the silent echo) BEFORE the
    // hex-digit parsing (ShortString.cs:166-171), so the missing-digit
    // ErrInvalidStringEscape never fires. The Lua51 preset has exactly
    // that option pair; the LuaJIT21 preset parses the digits.
    parse_and_validate_lex("local s = \"\\xG\"", &LuaSyntaxOptions::LUA51, &[]);
    parse_and_validate_lex(
        "local s = \"\\xG\"",
        &LuaSyntaxOptions::LUAJIT21,
        &[exp(ErrorCode::ErrInvalidStringEscape, 1, 12, r"\x")],
    );
}

#[test]
fn lexer_emits_diagnostics_when_hex_escapes_are_found_and_lua_syntax_options_accept_hex_escapes_is_false(
) {
    let source = "local str1 = \"hello\\xAthere\"\n\
local str2 = 'hello\\xAthere'\n\
local str3 = \"hello\\xFFthere\"\n\
local str4 = 'hello\\xFFthere'";
    let options = LuaSyntaxOptions {
        accept_hex_escapes_in_strings: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                1,
                20,
                r"\xA",
            ),
            exp(
                ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                2,
                20,
                r"\xA",
            ),
            exp(
                ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                3,
                20,
                r"\xFF",
            ),
            exp(
                ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                4,
                20,
                r"\xFF",
            ),
        ],
    );
}

#[test]
fn lexer_emits_multiple_diagnostics_when_multiple_hex_escapes_are_found_and_lua_syntax_options_accept_hex_escapes_is_false(
) {
    let source = r"local str = 'hello\xAFthere\xBFgood\xCFfriend'";
    let options = LuaSyntaxOptions {
        accept_hex_escapes_in_strings: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                1,
                19,
                r"\xAF",
            ),
            exp(
                ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                1,
                28,
                r"\xBF",
            ),
            exp(
                ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                1,
                36,
                r"\xCF",
            ),
        ],
    );
}

#[test]
fn lexer_emits_warning_for_exotic_line_break() {
    let source = "local a = 1\n\rlocal b = 2\n\rlocal c = 3";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp(
                ErrorCode::WrnLineBreakMayAffectErrorReporting,
                1,
                12,
                "\n\r",
            ),
            exp(
                ErrorCode::WrnLineBreakMayAffectErrorReporting,
                3,
                12,
                "\n\r",
            ),
        ],
    );
}

#[test]
fn lexer_z_escape_skips_newlines_and_carriage_returns() {
    // Finding 23: the `\z` skip uses the C# CharUtils.IsWhitespace set
    // ([ \t\n\v\f\r] — Lexer.ShortString.cs:141) — the port's helper
    // omitted '\n' and '\r', so the skip stopped before them and the
    // diagnostic span was too short.
    let source = "local a = \"aaa\\z\nbbbb\"\nlocal b = 'aaa\\z\rbbbb'";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions {
            accept_whitespace_escape: false,
            ..LuaSyntaxOptions::ALL
        },
        &[
            exp(
                ErrorCode::ErrWhitespaceEscapeNotSupportedInVersion,
                1,
                15,
                "\\z\n",
            ),
            exp(
                ErrorCode::ErrWhitespaceEscapeNotSupportedInVersion,
                3,
                15,
                "\\z\r",
            ),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_when_whitespace_escapes_are_found_and_lua_syntax_options_accept_whitespace_escape_is_false(
) {
    let source = "local a = \"aaa\\z    aaaa\"\nlocal b = 'aaa\\z    aaaa'";
    let options = LuaSyntaxOptions {
        accept_whitespace_escape: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrWhitespaceEscapeNotSupportedInVersion,
                1,
                15,
                r"\z    ",
            ),
            exp(
                ErrorCode::ErrWhitespaceEscapeNotSupportedInVersion,
                2,
                15,
                r"\z    ",
            ),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_when_invalid_unicode_escapes_are_found() {
    let source = "local a = '\\u{}'\n\
local b = '\\uFEBF}'\n\
local c = '\\u{FEBF'\n\
local d = '\\uFEBF'\n\
local e = '\\u{1100000}'";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[
            exp(ErrorCode::ErrHexDigitExpected, 1, 12, r"\u{"),
            exp(
                ErrorCode::ErrUnicodeEscapeMissingOpenBrace,
                2,
                12,
                r"\uFEBF}",
            ),
            exp(
                ErrorCode::ErrUnicodeEscapeMissingCloseBrace,
                3,
                12,
                r"\u{FEBF",
            ),
            exp(
                ErrorCode::ErrUnicodeEscapeMissingOpenBrace,
                4,
                12,
                r"\uFEBF",
            ),
            exp(
                ErrorCode::ErrUnicodeEscapeMissingCloseBrace,
                4,
                12,
                r"\uFEBF",
            ),
            exp_args(
                ErrorCode::ErrEscapeTooLarge,
                5,
                12,
                r"\u{1100000}",
                vec!["10FFFF"],
            ),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_when_unicode_escapes_are_found_and_lua_syntax_options_accept_unicode_escape_is_false(
) {
    let source = "local a = \"\\u{FEBE}\"\nlocal b = '\\u{FEBE}'";
    let options = LuaSyntaxOptions {
        accept_unicode_escape: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrUnicodeEscapesNotSupportedLuaInVersion,
                1,
                12,
                r"\u{FEBE}",
            ),
            exp(
                ErrorCode::ErrUnicodeEscapesNotSupportedLuaInVersion,
                2,
                12,
                r"\u{FEBE}",
            ),
        ],
    );
}

#[test]
fn lexer_emits_diagnostics_when_interpolated_or_hash_strings_are_found_and_lua_syntax_options_backtick_string_type_is_none(
) {
    // The finished-path LUA0036 is the C# PARSER's node-level error
    // (LanguageParser.InterpolatedString.cs:59-60) — the tree carries it
    // once (the parser supersedes the token copy), so this test asserts
    // through the combined tree diagnostics.
    let source = "local a = `hello`\nlocal b = `hi!`";
    let options = LuaSyntaxOptions {
        backtick_string_type: BacktickStringType::None,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[
            exp(
                ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
                1,
                11,
                "`hello`",
            ),
            exp(
                ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
                2,
                11,
                "`hi!`",
            ),
        ],
    );
}

#[test]
fn lexer_emits_the_unfinished_path_gate_too_when_backtick_string_type_is_none() {
    // AUDIT.md Finding 1(d): the C# gate (Lexer.ShortString.cs:70-72)
    // fires UNCONDITIONALLY after the scan — the unfinished path emits
    // LUA0003 AND the LUA0036 gate (probed @Lua51 '`abc': [LUA0003,
    // LUA0036, LUA1012]; the port's scanner carries both; the statement-
    // position LUA1012 belongs to the unported general parser-recovery
    // family, documented in the parserdiagnostics header).
    let text = "local x = `abc";
    let options = LuaSyntaxOptions {
        backtick_string_type: BacktickStringType::None,
        ..LuaSyntaxOptions::ALL
    };
    // The unfinished source fails the full_moon parse (no recovered AST),
    // so this asserts the scanner's pair: the last-char LUA0003 plus the
    // unconditional unfinished-path gate (the statement-position LUA1012
    // belongs to the unported general parser-recovery family, documented
    // in the parserdiagnostics header).
    let produced = lexer_diagnostics(text, &options);
    assert_eq!(produced.len(), 2, "two diagnostics: {produced:?}");
    let (first_line, first_col) = produced[0].line_col(text);
    assert_eq!(produced[0].code, ErrorCode::ErrUnfinishedString);
    assert_eq!((first_line, first_col), (1, 14));
    assert_eq!(produced[0].squiggle(text), "c");
    assert_eq!(
        produced[1].code,
        ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
        "the unfinished-path gate"
    );
    assert_eq!(produced[1].squiggle(text), "`abc");
}

#[test]
fn parser_emits_the_finished_path_gate_once() {
    // AUDIT.md Finding 1(d) count detail: in an expression context the C#
    // reports LUA0036 ONCE (the parser node copy supersedes the token copy
    // — LanguageParser.InterpolatedString.cs:59-60). Probed @Lua51
    // 'local x = `ab`': C# [LUA0036] (was Rust [LUA0036, LUA0036]).
    let text = "local x = `ab`";
    let options = LuaSyntaxOptions {
        backtick_string_type: BacktickStringType::None,
        ..LuaSyntaxOptions::ALL
    };
    let mut produced = lexer_diagnostics(text, &options);
    if let Ok(ast) = full_moon::parse(text) {
        produced.extend(loretta::errors::parserdiagnostics::parser_diagnostics(
            &ast, &options, text,
        ));
        produced.sort_by_key(|d| (d.sort_site, !d.node_level));
    }
    assert_eq!(produced.len(), 1, "one diagnostic: {produced:?}");
    assert_eq!(
        produced[0].code,
        ErrorCode::ErrInterpolatedStringsNotSupportedInVersion
    );
    assert_eq!(produced[0].line_col(text), (1, 11));
    assert_eq!(produced[0].squiggle(text), "`ab`");
}

#[test]
fn lexer_emits_diagnostics_when_lua_jit_suffix_is_malformed() {
    let source = "local a = 2000e5LL";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::ALL,
        &[exp(ErrorCode::ErrLuajitSuffixInFloat, 1, 11, "2000e5LL")],
    );
}

#[test]
fn lexer_emits_diagnostics_when_lua_jit_suffix_and_lua_syntax_options_accept_lua_jit_number_suffixes_is_false(
) {
    let source = "local a = 2000ULL";
    let options = LuaSyntaxOptions {
        accept_lua_jit_number_suffixes: false,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(
        source,
        &options,
        &[exp(
            ErrorCode::ErrNumberSuffixNotSupportedInVersion,
            1,
            11,
            "2000ULL",
        )],
    );
}

#[test]
fn lexer_emits_no_diagnostics_when_an_invalid_escape_is_found() {
    let source = r"local a = '\A\B\C\D\E'";
    let options = LuaSyntaxOptions {
        accept_invalid_escapes: true,
        ..LuaSyntaxOptions::ALL
    };
    parse_and_validate_lex(source, &options, &[]);
}

#[test]
fn lexer_emits_diagnostics_when_nesting_long_strings() {
    let source = "local a = [[[[\"]\"]];";
    parse_and_validate_lex(
        source,
        &LuaSyntaxOptions::LUA51,
        &[exp(ErrorCode::ErrLua51NestingInLongString, 1, 11, "[[[[")],
    );
}

#[test]
fn lexer_emits_escape_diagnostics_inside_backtick_strings() {
    // AUDIT.md Finding 1(a): the C# contents loop runs ScanEscapeSequence
    // per '\' (Lexer.ShortString.cs:424-427) and its AddError calls land
    // on the token. Probed @Lua54 'local x = `a\qb`': C#
    // [LUA0036, LUA0001, LUA0001] (tree pass: node gate + text-token
    // escape; token pass: the escape again) — the port's combined set
    // matches; silent under AcceptInvalidEscapes (@Luau: [] both).
    let text = "local x = `a\\qb`";
    let options = LuaSyntaxOptions::LUA54;
    let mut produced = lexer_diagnostics(text, &options);
    if let Ok(ast) = full_moon::parse(text) {
        produced.extend(loretta::errors::parserdiagnostics::parser_diagnostics(
            &ast, &options, text,
        ));
        produced.sort_by_key(|d| (d.sort_site, !d.node_level));
    }
    assert_eq!(produced.len(), 2, "two diagnostics: {produced:?}");
    assert_eq!(
        produced[0].code,
        ErrorCode::ErrInterpolatedStringsNotSupportedInVersion
    );
    assert_eq!(produced[0].squiggle(text), "`a\\qb`");
    assert_eq!(produced[1].code, ErrorCode::ErrInvalidStringEscape);
    assert_eq!(produced[1].line_col(text), (1, 13));
    assert_eq!(produced[1].squiggle(text), "\\q");

    // Silent when invalid escapes are accepted.
    let luau_produced = lexer_diagnostics(text, &LuaSyntaxOptions::LUAU);
    assert!(
        luau_produced.is_empty(),
        "no diagnostics under Luau: {luau_produced:?}"
    );
}

#[test]
fn lexer_emits_hole_diagnostics_inside_backtick_strings() {
    // AUDIT.md Finding 1(b): '{{' reports LUA0035 over the two braces
    // minus one (MakeError(openBracePosition - 1, width: 2),
    // Lexer.ShortString.cs:444-445); an unclosed hole reports LUA0034 the
    // same way (:458-459); a mismatched closer reports ERR_SyntaxError
    // with the expected-char argument (:498-503); TrySetError keeps the
    // FIRST error only.
    let options = LuaSyntaxOptions::LUAU;

    let text = "local x = `a{{b}`";
    let produced = lexer_diagnostics(text, &options);
    assert_eq!(produced.len(), 1, "one diagnostic: {produced:?}");
    assert_eq!(produced[0].code, ErrorCode::ErrDoubleBraceInInterpolation);
    assert_eq!(produced[0].squiggle(text), "a{");

    let text = "local x = `a{b`";
    let produced = lexer_diagnostics(text, &options);
    // The nested scan emits its own unfinished error during the contents;
    // the hole's LUA0034 wins this level's first-error slot (the End
    // error is TrySetError'd after it and suppressed).
    assert_eq!(
        produced.len(),
        2,
        "the nested unfinished + the unclosed hole: {produced:?}"
    );
    assert_eq!(produced[0].code, ErrorCode::ErrUnfinishedString);
    assert_eq!(produced[1].code, ErrorCode::ErrUnclosedExpressionHole);
    assert_eq!(produced[1].squiggle(text), "a{");

    let text = "local x = `a{(b]}`";
    // The ']' inside the '(' hole raises ERR_SyntaxError (expecting ')');
    // the scan then runs off the string's closing backtick, so the nested
    // level also reports its own unfinished error FIRST (the recursion's
    // AddError fires during the contents, before this level's slot flush —
    // the C# emission chronology). The parser-recovery LUA0015/LUA1003s
    // the C# adds on top are unported (documented).
    let produced = lexer_diagnostics(text, &options);
    assert_eq!(produced.len(), 2, "two diagnostics: {produced:?}");
    assert_eq!(produced[0].code, ErrorCode::ErrUnfinishedString);
    assert_eq!(produced[1].code, ErrorCode::ErrSyntaxError);
    assert_eq!(produced[1].arguments, vec![")".to_string()]);
    assert_eq!(produced[1].squiggle(text), "]");
}

/// The harness's DiagnosticsOp shape (differential/src/ops.rs
/// compute_diagnostics): the tree pass (parser + scanner diagnostics merged
/// in the C# tree-walk order) plus the tokens pass (the token-level scanner
/// diagnostics again). The node-level backtick diagnostics appear once.
fn harness_diagnostics(
    source: &str,
    options: &LuaSyntaxOptions,
) -> Vec<loretta::errors::lexerdiagnostics::LexerDiagnostic> {
    let scanner = lexer_diagnostics(source, options);
    let parser = full_moon::parse(source)
        .map(|ast| loretta::errors::parserdiagnostics::parser_diagnostics(&ast, options, source))
        .unwrap_or_default();
    let mut tree: Vec<_> = parser.iter().chain(scanner.iter()).collect();
    tree.sort_by_key(|d| (d.sort_site, !d.node_level));
    let mut produced: Vec<_> = tree.into_iter().cloned().collect();
    produced.extend(scanner.iter().filter(|d| !d.node_level).cloned());
    produced
}

#[test]
fn backtick_hole_diagnostics_are_node_level_and_match_the_reference_counts() {
    // AUDIT.md Finding 1(b) — the fixed shape: the C# parser replaces the
    // backtick token with the node and moves the rescan error + the gate
    // onto it (LanguageParser.InterpolatedString.cs:56-60), so the harness
    // reports each ONCE (the token pass must not double them). Probed on
    // the C# oracle (my span probe): 'local x = `a{{b}`' @Lua51 ->
    // [LUA0035|11|2, LUA0036|10|7]; @Luau -> [LUA0035|11|2];
    // @FiveM -> [] (the hash string treats braces as content).
    let options = LuaSyntaxOptions::LUA51;
    let text = "local x = `a{{b}`";
    let produced = harness_diagnostics(text, &options);
    assert_eq!(produced.len(), 2, "two diagnostics: {produced:?}");
    assert_eq!(
        produced[0].code,
        ErrorCode::ErrDoubleBraceInInterpolation,
        "the rescan error first (the C# attachment order)"
    );
    assert_eq!(produced[0].line_col(text), (1, 12));
    assert_eq!(produced[0].squiggle(text), "a{");
    assert_eq!(
        produced[1].code,
        ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
        "the gate second"
    );
    assert_eq!(produced[1].line_col(text), (1, 11));
    assert_eq!(produced[1].squiggle(text), "`a{{b}`");

    let luau = harness_diagnostics(text, &LuaSyntaxOptions::LUAU);
    assert_eq!(luau.len(), 1, "one diagnostic under Luau: {luau:?}");
    assert_eq!(luau[0].code, ErrorCode::ErrDoubleBraceInInterpolation);

    let fivem = harness_diagnostics(text, &LuaSyntaxOptions::FIVEM);
    assert!(
        fivem.is_empty(),
        "no hole diagnostics under the FiveM hash string: {fivem:?}"
    );
}

#[test]
fn backtick_hole_error_in_expression_keeps_the_node_shape() {
    // Probed @Lua51 'local x = `a{b`': C# [LUA0034|11|2, LUA0036|10|5,
    // LUA0015|13|1, LUA0003|14|1, LUA0036|14|1, LUA0003|14|1, LUA0036|14|1]
    // — the LUA0015 is the unported parser-recovery family (documented in
    // the parserdiagnostics header); the rest is the port's shape: the
    // outer node's [LUA0034, LUA0036] once each, the plain-unfinished
    // NESTED string's [LUA0003, LUA0036] twice (its token survives).
    let text = "local x = `a{b`";
    let produced = harness_diagnostics(text, &LuaSyntaxOptions::LUA51);
    let codes: Vec<ErrorCode> = produced.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        vec![
            ErrorCode::ErrUnclosedExpressionHole,
            ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
            ErrorCode::ErrUnfinishedString,
            ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
            ErrorCode::ErrUnfinishedString,
            ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
        ],
        "the port shape (minus the unported LUA0015): {produced:?}"
    );
    assert_eq!(produced[0].line_col(text), (1, 12));
    assert_eq!(produced[0].squiggle(text), "a{");
    assert_eq!(produced[2].line_col(text), (1, 15));
    assert_eq!(produced[2].squiggle(text), "`");
    assert_eq!(produced[2].code, ErrorCode::ErrUnfinishedString);
}

#[test]
fn backtick_unfinished_in_expression_emits_the_scanner_pair_once() {
    // Probed @Lua51 'local x = `abc': C# [LUA0003|13|1, LUA0036|10|4] once
    // each (the local assignment parses; the node carries the pair).
    let text = "local x = `abc";
    let produced = harness_diagnostics(text, &LuaSyntaxOptions::LUA51);
    assert_eq!(produced.len(), 2, "the scanner pair: {produced:?}");
    assert_eq!(produced[0].code, ErrorCode::ErrUnfinishedString);
    assert_eq!(produced[0].line_col(text), (1, 14));
    assert_eq!(produced[0].squiggle(text), "c");
    assert_eq!(
        produced[1].code,
        ErrorCode::ErrInterpolatedStringsNotSupportedInVersion
    );
    assert_eq!(produced[1].line_col(text), (1, 11));
    assert_eq!(produced[1].squiggle(text), "`abc");
}

#[test]
fn five_m_hash_string_backticks_scan_as_short_strings() {
    // AUDIT.md Finding 1(d) — the FiveM (HashLiteral) preset routes the
    // backtick through the short-string/hash scanner (Lexer.cs:622-625):
    // braces are plain content (no hole diagnostics) and the unfinished
    // span is the whole lexeme. Probed on the C# oracle: 'local x = `a{b`'
    // @FiveM -> []; '`abc' @FiveM -> [LUA0003|0|4] twice (the token pass
    // doubles the token-level unfinished) + LUA1012 (unported).
    let text = "local x = `a{b`";
    let produced = harness_diagnostics(text, &LuaSyntaxOptions::FIVEM);
    assert!(
        produced.is_empty(),
        "the hash string scans the braces as content: {produced:?}"
    );

    let text = "`abc";
    let produced = harness_diagnostics(text, &LuaSyntaxOptions::FIVEM);
    assert_eq!(produced.len(), 2, "the token-level pair: {produced:?}");
    assert_eq!(produced[0].code, ErrorCode::ErrUnfinishedString);
    assert_eq!(produced[0].line_col(text), (1, 1));
    assert_eq!(produced[0].squiggle(text), "`abc");
    assert_eq!(produced[1].code, ErrorCode::ErrUnfinishedString);
    assert_eq!(produced[1].squiggle(text), "`abc");
}
