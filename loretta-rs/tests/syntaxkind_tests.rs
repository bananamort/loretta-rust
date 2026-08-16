// Ported from Loretta.CodeAnalysis.Lua.UnitTests.SyntaxKindTests (b767b4e): SyntaxKindTests
// C# source: src/Compilers/Lua/Test/Portable/SyntaxKindTests.cs

use full_moon::tokenizer::{Lexer, LexerResult, TokenType};
use full_moon::LuaVersion;

/// The canonical operator/keyword vocabulary of the full_moon tokenizer's
/// Symbol enum (tokenizer/structs.rs:100-178). Excludes the cfxlua-only
/// symbols (<<=, >>=, &=, |=, ?.) — LuaVersion::new() (the Default version)
/// enables Luau/Lua52/Lua53/Lua54/LuaJIT but not cfxlua (versions.rs:146-152).
const CANONICAL_SYMBOLS: &str = "and break do else elseif end false for function \
     if in local nil not or repeat return then true until while goto \
     += -= *= /= //= %= ^= ..= & -> :: @ ^ : , . .. ... = == > >= >> \
     # { [ ( < <= << - % | + ? } ] ; / // * ~ ~=";

/// Tokenizes the canonical symbol source and returns the produced symbol
/// token texts (in source order).
fn symbol_texts() -> Vec<String> {
    let lexer = Lexer::new(CANONICAL_SYMBOLS, LuaVersion::new());
    let tokens = match lexer.collect() {
        LexerResult::Ok(tokens) | LexerResult::Recovered(tokens, _) => tokens,
        LexerResult::Fatal(errors) => {
            panic!("the canonical symbol source must tokenize: {errors:?}")
        }
    };
    tokens
        .iter()
        .filter_map(|token| match token.token_type() {
            TokenType::Symbol { .. } => Some(token.to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn syntax_kind_has_no_duplicates() {
    // C# SyntaxKindHasNoDuplicates (SyntaxKindTests.cs:5-14) asserts the
    // dropped SyntaxKind enum (Portable/Syntax/SyntaxKind.cs — Locked
    // Decision 1) has no duplicate values. The port docks on the full_moon
    // tokenizer Symbol enum, whose every variant maps to a distinct fixed
    // text; the enum is #[non_exhaustive], so the kind set is exercised
    // through the canonical symbol source.
    let texts = symbol_texts();
    let mut seen = std::collections::HashSet::new();
    for text in &texts {
        assert!(
            seen.insert(text.clone()),
            "found duplicate kind text: {text}"
        );
    }
}

#[test]
fn token_kinds_have_text() {
    // C# TokenKindsHaveText (SyntaxKindTests.cs:16-36) asserts every token
    // kind except the textful literal kinds (BadToken, HashStringLiteralToken,
    // IdentifierToken, InterpolatedStringTextToken, InterpolatedStringToken,
    // NumericLiteralToken, StringLiteralToken) and EndOfFileToken has a fixed
    // (non-empty) text. The port docks on the full_moon Symbol tokens — every
    // symbol token produced over the canonical source has a non-empty fixed
    // text.
    let texts = symbol_texts();
    assert!(
        !texts.is_empty(),
        "the canonical source must contain symbol tokens"
    );
    for text in &texts {
        assert!(!text.is_empty(), "a token kind should have a fixed text");
    }
}
