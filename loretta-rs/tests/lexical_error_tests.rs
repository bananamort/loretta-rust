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

/// The C# ParsingTestsBase.ParseAndValidateAsync (ParsingTestsBase.cs:44-51).
/// The port's diagnostics are produced by the lexer-diagnostics scanner (the
/// C# lexer is DROP; the full_moon parse panics on fatal-tokenizer inputs
/// such as `@$\` — its luau-attribute path calls `current().unwrap()`,
/// ast/parsers.rs:3709), so the C# parse + round-trip step is not performed
/// here (the parser round trips are covered by the parser test suites).
fn parse_and_validate_lex(source: &str, options: &LuaSyntaxOptions, expected: &[Expected]) {
    let produced = lexer_diagnostics(source, options);
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
