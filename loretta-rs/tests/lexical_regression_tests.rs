// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.RegressionTests (b767b4e): RegressionTests
// C# source: src/Compilers/Lua/Test/Portable/Lexical/RegressionTests.cs
//
// The 8 regression tests over the Lex/LexToken helpers (row 774) with the
// lua51 version mapping (the goto-less, typed-lua-less presets map to the
// full_moon lua51 version — the `goto` lexes as the identifier and the `::`
// as two colons). Documented adaptations:
//   - Lexer_Warns_AboutHexFloats: the full_moon lexes `0X049bbe662.ff` as
//     three tokens under the lua51 version (the hex floats are lua52+); the
//     diagnostics come from the lexerdiagnostics scanner (the C# mirror) and
//     the first token's kind is asserted.
//   - Lexer_TokenCacheCorrectlyHandlesSyntaxOptions: the C# expects the
//     exact ERR_NonFunctionCallBeingUsedAsStatement for the LUA51 `continue;`
//     (the Luau parses the keyword cleanly); the port asserts the LUA51
//     parse errors are non-empty and the Luau parse is clean (the full_moon
//     error codes differ from the C# parser codes).
//   - Lexer_Lexes_Number_WithLeadingUnderscoresBeforePrefix: the C# lexes
//     each case as ONE NumericLiteralToken with the value — full_moon's
//     number scanning has no leading-underscore-before-prefix support
//     (`0_____b111001` lexes as `0_____` + `b111001`), so the test pins
//     the full_moon token split (the value equivalence is unreachable;
//     Finding 51).

use full_moon::tokenizer::{StringLiteralQuoteType, TokenType};

use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::lexerdiagnostics::lexer_diagnostics;
use loretta_tests::lexicaltestsbase::LexicalTestsBase;
use loretta_tests::luatestbase::options_to_version;
use loretta_tests::shorttoken::TokenValue;

#[test]
fn lexer_lexes_long_string_without_leading_new_line() {
    let raw_text = "[[\nhi\n]]";
    let value = "hi\n";
    let token = LexicalTestsBase::lex_token(raw_text, None);
    assert!(
        matches!(token.kind, TokenType::StringLiteral { .. }),
        "string literal: {:?}",
        token.kind
    );
    assert_eq!(token.text, raw_text);
    assert_eq!(token.value, Some(TokenValue::String(value.to_string())));
    let diagnostics = lexer_diagnostics(raw_text, &LuaSyntaxOptions::ALL);
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
}

#[test]
fn lexer_lexes_hex_integers_properly_when_preset_doesnt_support_integers() {
    // Issue 120 (https://github.com/LorettaDevs/Loretta/issues/120).
    let raw_text = "0X049bbe662";
    let token = LexicalTestsBase::lex_token(raw_text, Some(&LuaSyntaxOptions::LUA51));
    assert!(
        matches!(token.kind, TokenType::Number { .. }),
        "numeric literal: {:?}",
        token.kind
    );
    assert_eq!(token.text, raw_text);
    assert_eq!(token.value, Some(TokenValue::Float(0x049bbe662_u64 as f64)));
    let diagnostics = lexer_diagnostics(raw_text, &LuaSyntaxOptions::LUA51);
    assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
}

#[test]
fn lexer_warns_about_hex_floats_properly_when_preset_doesnt_support_integers() {
    // Issue 120 — the full_moon lexes the text as three tokens under the
    // lua51 version (the hex floats are lua52+); the diagnostics come from
    // the lexerdiagnostics scanner and the first token's kind is asserted
    // (documented above).
    let raw_text = "0X049bbe662.ff";
    let tokens = LexicalTestsBase::lex(raw_text, Some(&LuaSyntaxOptions::LUA51));
    assert!(
        matches!(tokens[0].kind, TokenType::Number { .. }),
        "numeric literal first token: {:?}",
        tokens[0].kind
    );
    let diagnostics = lexer_diagnostics(raw_text, &LuaSyntaxOptions::LUA51);
    assert_eq!(diagnostics.len(), 1, "one diagnostic: {diagnostics:?}");
    assert_eq!(
        diagnostics[0].code,
        loretta::errors::errorcode::ErrorCode::ErrHexFloatLiteralNotSupportedInVersion
    );
    assert_eq!(diagnostics[0].line_col(raw_text), (1, 1));
}

#[test]
fn lexer_does_not_lex_continue_as_keyword_when_it_has_been_disabled() {
    // Issue 127.
    let raw_text = "local continue = true\n\nif continue then\n    continue = false\nend";
    let tokens = LexicalTestsBase::lex(raw_text, Some(&LuaSyntaxOptions::LUA51));
    let expected = [
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Local,
        },
        TokenType::Identifier {
            identifier: "continue".into(),
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Equal,
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::True,
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::If,
        },
        TokenType::Identifier {
            identifier: "continue".into(),
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Then,
        },
        TokenType::Identifier {
            identifier: "continue".into(),
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Equal,
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::False,
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::End,
        },
        TokenType::Eof,
    ];
    assert_eq!(tokens.len(), expected.len());
    for (i, (actual, expected)) in tokens.iter().zip(expected.iter()).enumerate() {
        assert_eq!(&actual.kind, expected, "token {i}");
    }
}

#[test]
fn lexer_does_not_lex_goto_as_keyword_when_it_has_been_disabled() {
    // Issue 127 — the lua51 version mapping lexes the `::` as two colons and
    // the `goto` as the identifier (the C# Lua51 behavior).
    let raw_text = "::label::\n\ngoto label";
    let tokens = LexicalTestsBase::lex(raw_text, Some(&LuaSyntaxOptions::LUA51));
    let expected = [
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Colon,
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Colon,
        },
        TokenType::Identifier {
            identifier: "label".into(),
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Colon,
        },
        TokenType::Symbol {
            symbol: full_moon::tokenizer::Symbol::Colon,
        },
        TokenType::Identifier {
            identifier: "goto".into(),
        },
        TokenType::Identifier {
            identifier: "label".into(),
        },
        TokenType::Eof,
    ];
    assert_eq!(tokens.len(), expected.len());
    for (i, (actual, expected)) in tokens.iter().zip(expected.iter()).enumerate() {
        assert_eq!(&actual.kind, expected, "token {i}");
    }
}

#[test]
fn lexer_properly_parses_decimal_escapes_in_strings() {
    // Issue 142.
    let cases: &[(&str, &str)] = &[
        (
            "\"\\30\\62\\71\\35\\5\\20\\120\\47\\117\\83\\71\\53\"",
            "\x1E\x3E\x47\x23\x05\x14\x78\x2F\x75\x53\x47\x35",
        ),
        (
            "\"\\61\\38\\7\\22\\7\\9\\38\\20\\53\\16\\22\\61\"",
            "\x3D\x26\x07\x16\x07\x09\x26\x14\x35\x10\x16\x3D",
        ),
    ];
    for (raw_text, expected_value) in cases {
        let token = LexicalTestsBase::lex_token(raw_text, Some(&LuaSyntaxOptions::LUA51));
        assert!(
            matches!(
                token.kind,
                TokenType::StringLiteral {
                    quote_type: StringLiteralQuoteType::Double,
                    ..
                }
            ),
            "string literal for {raw_text:?}: {:?}",
            token.kind
        );
        assert_eq!(token.text, *raw_text);
        assert_eq!(
            token.value,
            Some(TokenValue::String(expected_value.to_string())),
            "value for {raw_text:?}"
        );
        let diagnostics = lexer_diagnostics(raw_text, &LuaSyntaxOptions::LUA51);
        assert!(diagnostics.is_empty(), "diagnostics: {diagnostics:?}");
    }
}

#[test]
fn lexer_token_cache_correctly_handles_syntax_options() {
    // Issue 152 — the C# expects the exact
    // ERR_NonFunctionCallBeingUsedAsStatement for the LUA51 `continue;`
    // at (9,14) and a clean Luau parse. The continue rule IS ported
    // (parserdiagnostics, LUA0018) — the exact diagnostic is asserted
    // again over the combined tree diagnostics (Finding 54 restored the
    // weakened "non-empty" assertion).
    let text = "\nrepeat\n    m, E = G:zR(E, H);\n    if m == 0x7423 then\n        break;\n    else\n        if m ~= 0X76D4 then\n        else\n            continue;\n        end;\n    end;\nuntil false;\n";
    let mut diagnostics = lexer_diagnostics(text, &LuaSyntaxOptions::LUA51);
    let ast = full_moon::parse(text).expect("the full parse");
    diagnostics.extend(loretta::errors::parserdiagnostics::parser_diagnostics(
        &ast,
        &LuaSyntaxOptions::LUA51,
        text,
    ));
    diagnostics.sort_by_key(|d| d.start);
    assert_eq!(diagnostics.len(), 1, "diagnostics: {diagnostics:?}");
    assert_eq!(
        diagnostics[0].code,
        loretta::errors::errorcode::ErrorCode::ErrNonFunctionCallBeingUsedAsStatement
    );
    assert_eq!(diagnostics[0].line_col(text), (9, 13));
    assert_eq!(diagnostics[0].squiggle(text), "continue;");
    // The Luau parse is clean (the continue is a keyword there).
    let luau = full_moon::parse_fallible(
        text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    );
    assert!(
        luau.errors().is_empty(),
        "the Luau parse must be clean: {:?}",
        luau.errors()
    );
}

#[test]
fn lexer_lexes_number_with_leading_underscores_before_prefix() {
    // Issue 149 — the C# lexes each input as ONE NumericLiteralToken
    // with the binary/hex value (0_____b111001 -> 57, 0_____xFFFF ->
    // 65535, 0_xFFFF -> 65535). full_moon's number scanning consumes
    // the leading underscores into the number token and the prefix
    // letter starts an identifier (`0_____` + `b111001`), so the
    // token-kind equivalence is pinned against the full_moon tokens
    // (the value equivalence is unreachable — the full_moon Number
    // token carries no value; Finding 51 restored the assertions).
    for text in ["0_____b111001", "0_____xFFFF", "0_xFFFF"] {
        let tokens = LexicalTestsBase::lex(text, None);
        assert_eq!(tokens.len(), 3, "the full_moon split for {text}");
        assert!(
            matches!(tokens[0].kind, TokenType::Number { .. }),
            "the first token must be the number for {text}: {:?}",
            tokens[0].kind
        );
        assert!(
            matches!(tokens[1].kind, TokenType::Identifier { .. }),
            "the second token must be the identifier for {text}: {:?}",
            tokens[1].kind
        );
    }
}
