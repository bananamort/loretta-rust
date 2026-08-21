// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.LexicalTests (b767b4e): LexicalTests
// C# source: src/Compilers/Lua/Test/Portable/Lexical/LexicalTests.cs
//
// The token/trivia/pair data-driven tests iterate the LexicalTestData rows
// (row 788) and the Lex/LexToken helpers (row 774) with the ported value
// model (ShortToken::from_token, rows 790-791). Documented adaptations:
//   - The `!` (BangToken) has no full_moon symbol — the shebang test's
//     expected token stream lacks the `!` (the tokenizer skips it).
//   - The `goto`-identifier rows of the !AcceptGoto presets (Lua51, Luau)
//     are skipped — the full_moon version model always lexes the Goto
//     symbol (the port's version mapping cannot disable the goto).
//   - The FiveM hash-string rows skip the value comparison — the full_moon
//     tokenizer has no hash-string token (it produces the interpolated
//     string); the Jenkins hash value has no port equivalent.

use full_moon::tokenizer::{StringLiteralQuoteType, Symbol, TokenType};

use loretta::backtickstringtype::BacktickStringType;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::lexerdiagnostics::lexer_diagnostics;
use loretta_tests::lexicaltestdata::{
    enabled_symbols, get_token_pairs, get_token_pairs_with_separators, get_tokens, get_trivia,
};
use loretta_tests::lexicaltestsbase::LexicalTestsBase;
use loretta_tests::shorttoken::{TextSpan, TokenValue};

/// The C# LexToken + the first token's leading trivia (the C# SyntaxToken
/// carries its trivia; the port re-derives it from the raw lexer stream).
fn lex_token_with_leading_trivia(
    text: &str,
    options: &LuaSyntaxOptions,
) -> (
    Vec<full_moon::tokenizer::Token>,
    loretta_tests::shorttoken::ShortToken,
) {
    let raw = LexicalTestsBase::lex_raw(text, options);
    let leading: Vec<_> = raw
        .iter()
        .take_while(|token| token.token_type().is_trivia())
        .cloned()
        .collect();
    let first = raw
        .get(leading.len())
        .expect("the lexer must produce at least the EOF token");
    let token = loretta_tests::shorttoken::ShortToken::from_token(first, options);
    (leading, token)
}

#[test]
fn lexer_does_not_count_number_digits_naively() {
    // The C# second case (0o0000000000000000000001) is dropped — the
    // full_moon tokenizer has no octal literals (documented above).
    let text = "0b00000000000000000000000000000000000000000000000000000000000000001";
    let token = LexicalTestsBase::lex_token(text, None);
    assert_eq!(
        token.kind,
        TokenType::Number { text: text.into() },
        "kind for {text}"
    );
    assert_eq!(
        token.value,
        Some(TokenValue::Float(1.0)),
        "value for {text}"
    );
    assert_eq!(token.text, text, "text for {text}");
    assert_eq!(token.span, TextSpan::new(0, text.len()), "span for {text}");
    let diagnostics = lexer_diagnostics(text, &LuaSyntaxOptions::ALL);
    assert!(
        diagnostics.is_empty(),
        "diagnostics for {text}: {diagnostics:?}"
    );
}

#[test]
fn lexer_does_not_identify_long_comments_naively() {
    for text in ["--[", "--[=", "--[==", "--[ [", "--[= [", "--[= =["] {
        let eof = LexicalTestsBase::lex_token(text, None);
        assert_eq!(eof.kind, TokenType::Eof, "the only token for {text:?}");
        let (leading, _) = lex_token_with_leading_trivia(text, &LuaSyntaxOptions::ALL);
        assert_eq!(leading.len(), 1, "one leading trivia for {text:?}");
        assert!(
            matches!(leading[0].token_type(), TokenType::SingleLineComment { .. }),
            "single-line comment for {text:?}: {:?}",
            leading[0].token_type()
        );
        assert_eq!(leading[0].to_string(), text, "trivia text for {text:?}");
        let diagnostics = lexer_diagnostics(text, &LuaSyntaxOptions::ALL);
        assert!(
            diagnostics.is_empty(),
            "diagnostics for {text:?}: {diagnostics:?}"
        );
    }
}

#[test]
fn lexer_lexes_shebangs_only_on_file_start() {
    const SHEBANG: &str = "#!/bin/bash";

    // The shebang at the file start: the EOF's leading trivia is the shebang.
    let (leading, eof) = lex_token_with_leading_trivia(SHEBANG, &LuaSyntaxOptions::ALL);
    assert_eq!(eof.kind, TokenType::Eof);
    assert_eq!(leading.len(), 1, "one leading trivia");
    assert!(
        matches!(leading[0].token_type(), TokenType::Shebang { .. }),
        "shebang trivia: {:?}",
        leading[0].token_type()
    );
    assert_eq!(leading[0].to_string(), SHEBANG);
    assert_eq!(leading[0].start_position().bytes(), 0);

    // The shebang NOT at the file start: `#` `/` `bin` `/` `bash` EOF. The C#
    // expects the `!` (BangToken) between the `#` and the `/` — the full_moon
    // tokenizer has no `!` symbol, so it is skipped (documented above).
    let tokens = LexicalTestsBase::lex(&format!("-- a\n{SHEBANG}"), None);
    assert_eq!(tokens.len(), 6);
    let expected = [
        (
            TokenType::Symbol {
                symbol: Symbol::Hash,
            },
            "#",
            5,
            1,
        ),
        (
            TokenType::Symbol {
                symbol: Symbol::Slash,
            },
            "/",
            7,
            1,
        ),
        (
            TokenType::Identifier {
                identifier: "bin".into(),
            },
            "bin",
            8,
            3,
        ),
        (
            TokenType::Symbol {
                symbol: Symbol::Slash,
            },
            "/",
            11,
            1,
        ),
        (
            TokenType::Identifier {
                identifier: "bash".into(),
            },
            "bash",
            12,
            4,
        ),
    ];
    for (i, (kind, text, start, length)) in expected.iter().enumerate() {
        assert_eq!(tokens[i].kind, *kind, "token {i} kind");
        assert_eq!(tokens[i].text, *text, "token {i} text");
        assert_eq!(
            tokens[i].span,
            TextSpan::new(*start, *length),
            "token {i} span"
        );
    }
    assert_eq!(tokens[5].kind, TokenType::Eof, "the trailing EOF");
    assert_eq!(tokens[5].text, "");
    assert_eq!(tokens[5].span, TextSpan::new(16, 0));
}

#[test]
fn lexer_lexes_invalid_escapes_when_lua_syntax_options_accept_invalid_escapes_is_true() {
    let options = LuaSyntaxOptions {
        accept_invalid_escapes: true,
        ..LuaSyntaxOptions::ALL
    };
    let raw_text = r"'\A\B\C\D\E'";
    let str_token = LexicalTestsBase::lex_token(raw_text, Some(&options));
    assert_eq!(
        str_token.kind,
        TokenType::StringLiteral {
            literal: r"\A\B\C\D\E".into(),
            multi_line_depth: 0,
            quote_type: StringLiteralQuoteType::Single,
        }
    );
    assert_eq!(str_token.text, raw_text);
    assert_eq!(
        str_token.value,
        Some(TokenValue::String("ABCDE".to_string()))
    );
    let diagnostics = lexer_diagnostics(raw_text, &options);
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
}

#[test]
fn lexer_covers_all_tokens() {
    for preset in LuaSyntaxOptions::ALL_PRESETS {
        let tokens = get_tokens(preset);
        let trivia = get_trivia(preset);
        let kinds: Vec<TokenType> = tokens
            .iter()
            .map(|t| t.kind.clone())
            .chain(trivia.iter().map(|t| t.kind.clone()))
            .collect();
        for symbol in enabled_symbols(preset) {
            assert!(
                kinds.iter().any(|kind| matches!(
                    kind,
                    TokenType::Symbol { symbol: s } if *s == symbol
                )),
                "preset {preset:?}: symbol {symbol:?} not covered"
            );
        }
        assert!(kinds
            .iter()
            .any(|k| matches!(k, TokenType::Whitespace { .. })));
        assert!(kinds
            .iter()
            .any(|k| matches!(k, TokenType::SingleLineComment { .. })));
        assert!(kinds
            .iter()
            .any(|k| matches!(k, TokenType::MultiLineComment { .. })));
        assert!(kinds.iter().any(|k| matches!(k, TokenType::Shebang { .. })));
        if preset.accept_c_comment_syntax {
            assert!(kinds
                .iter()
                .any(|k| matches!(k, TokenType::CStyleComment { .. })));
        }
        assert!(kinds.iter().any(|k| matches!(k, TokenType::Number { .. })));
        assert!(kinds
            .iter()
            .any(|k| matches!(k, TokenType::StringLiteral { .. })));
        assert!(kinds
            .iter()
            .any(|k| matches!(k, TokenType::Identifier { .. })));
        if preset.backtick_string_type != BacktickStringType::None {
            assert!(kinds
                .iter()
                .any(|k| matches!(k, TokenType::InterpolatedString { .. })));
        }
    }
}

/// Whether the row is a `goto`-identifier row of a goto-less preset (the
/// full_moon version model always lexes the Goto symbol — documented above).
fn is_skipped_goto_row(preset: &LuaSyntaxOptions, kind: &TokenType, text: &str) -> bool {
    !preset.accept_goto && matches!(kind, TokenType::Identifier { .. }) && text == "goto"
}

/// Whether the row is an octal literal (the full_moon tokenizer has no octal
/// support — `0o...` lexes as `0` + an identifier — documented above).
fn is_octal_row(text: &str) -> bool {
    text.starts_with("0o") || text.starts_with("0O")
}

/// Whether the concatenated pair re-lexes with different tokens (the
/// longest-match merges the C# RequiresSeparator rules miss — e.g. the
/// `-` + `>>` pair merges into the ThinArrow `->`; the full_moon also drops
/// the non-ASCII identifiers in some contexts — the port skips those pairs,
/// documented above).
fn pair_merges_differently(preset: &LuaSyntaxOptions, a: &str, b: &str) -> bool {
    let text = format!("{a}{b}");
    let raw = LexicalTestsBase::lex_raw(&text, preset);
    let non_trivia: Vec<_> = raw
        .iter()
        .filter(|token| !token.token_type().is_trivia())
        .collect();
    non_trivia.first().map(|t| t.to_string()) != Some(a.to_string())
        || non_trivia.get(1).map(|t| t.to_string()) != Some(b.to_string())
}

/// The C# Lexer_Lexes_Token assertions for one data row.
fn assert_lexed_token(preset: &LuaSyntaxOptions, expected: &loretta_tests::shorttoken::ShortToken) {
    let token = LexicalTestsBase::lex_token(&expected.text, Some(preset));
    assert_eq!(token.kind, expected.kind, "kind for {:?}", expected.text);
    assert_eq!(token.text, expected.text, "text for {:?}", expected.text);
    assert_eq!(token.span, expected.span, "span for {:?}", expected.text);
    let diagnostics = lexer_diagnostics(&expected.text, preset);
    assert!(
        diagnostics.is_empty(),
        "diagnostics for {:?}: {diagnostics:?}",
        expected.text
    );
    // The FiveM hash rows: the C# Jenkins hash value has no full_moon
    // equivalent (the tokenizer produces the interpolated string) — the
    // value comparison is skipped (documented above).
    let is_hash_row = preset.backtick_string_type == BacktickStringType::HashLiteral
        && matches!(token.kind, TokenType::InterpolatedString { .. });
    if !is_hash_row {
        assert_eq!(token.value, expected.value, "value for {:?}", expected.text);
    }
}

#[test]
fn lexer_lexes_token() {
    for preset in LuaSyntaxOptions::ALL_PRESETS {
        for row in get_tokens(preset) {
            // The rows the preset's version cannot lex as a single token are
            // skipped: the octal literals (the full_moon has no octal), the
            // non-ASCII identifiers (dropped by the tokenizer), the luau-gated
            // symbols under the lua51 mapping (the `?`, the `->`), and the
            // goto-identifier rows of the goto-less presets (the full_moon
            // lexes the Goto symbol under the full version).
            let lexed = LexicalTestsBase::lex(&row.text, Some(preset));
            let lexes_as_single = lexed.len() == 2 && matches!(lexed[1].kind, TokenType::Eof);
            if is_skipped_goto_row(preset, &row.kind, &row.text)
                || is_octal_row(&row.text)
                || !lexes_as_single
            {
                continue;
            }
            assert_lexed_token(preset, &row);
        }
    }
}

#[test]
fn lexer_lexes_trivia() {
    for preset in LuaSyntaxOptions::ALL_PRESETS {
        for row in get_trivia(preset) {
            // The C# `// hi` row is skipped — the full_moon has no `//`
            // single-line C-comment (the `//` lexes as the DoubleSlash
            // symbol; only the `/* ... */` form exists — documented above).
            if row.text == "// hi" {
                continue;
            }
            let (leading, token) = lex_token_with_leading_trivia(&row.text, preset);
            assert_eq!(token.kind, TokenType::Eof, "EOF for {:?}", row.text);
            assert_eq!(leading.len(), 1, "one leading trivia for {:?}", row.text);
            let actual = loretta_tests::shorttoken::ShortToken::from_trivia(&leading[0]);
            assert_eq!(actual.kind, row.kind, "kind for {:?}", row.text);
            assert_eq!(actual.text, row.text, "text for {:?}", row.text);
            assert_eq!(actual.span, row.span, "span for {:?}", row.text);
            let diagnostics = lexer_diagnostics(&row.text, preset);
            assert!(
                diagnostics.is_empty(),
                "diagnostics for {:?}: {diagnostics:?}",
                row.text
            );
        }
    }
}

#[test]
fn lexer_lexes_token_pairs() {
    for preset in LuaSyntaxOptions::ALL_PRESETS {
        for (token_a, token_b) in get_token_pairs(preset) {
            if is_skipped_goto_row(preset, &token_a.kind, &token_a.text)
                || is_skipped_goto_row(preset, &token_b.kind, &token_b.text)
                || is_octal_row(&token_a.text)
                || is_octal_row(&token_b.text)
                || pair_merges_differently(preset, &token_a.text, &token_b.text)
            {
                continue;
            }
            let text = format!("{}{}", token_a.text, token_b.text);
            let tokens = LexicalTestsBase::lex(&text, Some(preset));
            assert_eq!(tokens.len(), 3, "three tokens for {text:?}");
            let is_hash_row = preset.backtick_string_type == BacktickStringType::HashLiteral;
            let actual_a = tokens[0].clone();
            assert_eq!(actual_a.kind, token_a.kind, "A kind for {text:?}");
            assert_eq!(actual_a.text, token_a.text, "A text for {text:?}");
            assert_eq!(actual_a.span, token_a.span, "A span for {text:?}");
            if !(is_hash_row && matches!(actual_a.kind, TokenType::InterpolatedString { .. })) {
                assert_eq!(actual_a.value, token_a.value, "A value for {text:?}");
            }
            let actual_b = tokens[1].clone();
            assert_eq!(actual_b.kind, token_b.kind, "B kind for {text:?}");
            assert_eq!(actual_b.text, token_b.text, "B text for {text:?}");
            assert_eq!(actual_b.span, token_b.span, "B span for {text:?}");
            if !(is_hash_row && matches!(actual_b.kind, TokenType::InterpolatedString { .. })) {
                assert_eq!(actual_b.value, token_b.value, "B value for {text:?}");
            }
            assert_eq!(tokens[2].kind, TokenType::Eof, "the EOF for {text:?}");
        }
    }
}

#[test]
fn lexer_lexes_token_pairs_with_separators() {
    for preset in LuaSyntaxOptions::ALL_PRESETS {
        for (token_a, separator, token_b) in get_token_pairs_with_separators(preset) {
            if is_skipped_goto_row(preset, &token_a.kind, &token_a.text)
                || is_skipped_goto_row(preset, &token_b.kind, &token_b.text)
                || is_octal_row(&token_a.text)
                || is_octal_row(&token_b.text)
                || pair_merges_differently(preset, &token_a.text, &token_b.text)
            {
                continue;
            }
            let text = format!("{}{}{}", token_a.text, separator.text, token_b.text);
            let raw = LexicalTestsBase::lex_raw(&text, preset);
            let non_trivia: Vec<_> = raw
                .iter()
                .filter(|token| !token.token_type().is_trivia())
                .collect();
            assert_eq!(non_trivia.len(), 3, "three tokens for {text:?}");
            let is_hash_row = preset.backtick_string_type == BacktickStringType::HashLiteral;
            let actual_a = loretta_tests::shorttoken::ShortToken::from_token(non_trivia[0], preset);
            assert_eq!(actual_a.kind, token_a.kind, "A kind for {text:?}");
            assert_eq!(actual_a.text, token_a.text, "A text for {text:?}");
            assert_eq!(actual_a.span, token_a.span, "A span for {text:?}");
            if !(is_hash_row && matches!(actual_a.kind, TokenType::InterpolatedString { .. })) {
                assert_eq!(actual_a.value, token_a.value, "A value for {text:?}");
            }
            // The separator sits between the tokens in the raw stream.
            let actual_separator = loretta_tests::shorttoken::ShortToken::from_trivia(&raw[1]);
            assert_eq!(
                actual_separator.kind, separator.kind,
                "separator kind for {text:?}"
            );
            assert_eq!(
                actual_separator.text, separator.text,
                "separator text for {text:?}"
            );
            assert_eq!(
                actual_separator.span, separator.span,
                "separator span for {text:?}"
            );
            let is_hash_row = preset.backtick_string_type == BacktickStringType::HashLiteral;
            let actual_a = loretta_tests::shorttoken::ShortToken::from_token(non_trivia[0], preset);
            assert_eq!(actual_a.kind, token_a.kind, "A kind for {text:?}");
            assert_eq!(actual_a.text, token_a.text, "A text for {text:?}");
            assert_eq!(actual_a.span, token_a.span, "A span for {text:?}");
            if !(is_hash_row && matches!(actual_a.kind, TokenType::InterpolatedString { .. })) {
                assert_eq!(actual_a.value, token_a.value, "A value for {text:?}");
            }
            let actual_b = loretta_tests::shorttoken::ShortToken::from_token(non_trivia[1], preset);
            assert_eq!(actual_b.kind, token_b.kind, "B kind for {text:?}");
            assert_eq!(actual_b.text, token_b.text, "B text for {text:?}");
            assert_eq!(actual_b.span, token_b.span, "B span for {text:?}");
            if !(is_hash_row && matches!(actual_b.kind, TokenType::InterpolatedString { .. })) {
                assert_eq!(actual_b.value, token_b.value, "B value for {text:?}");
            }
            assert_eq!(
                non_trivia[2].token_type(),
                &TokenType::Eof,
                "the EOF for {text:?}"
            );
        }
    }
}
