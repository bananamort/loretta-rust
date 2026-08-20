// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Lexical.LexicalTestsBase (b767b4e): LexicalTestsBase
// C# source: src/Compilers/Lua/Test/Portable/Lexical/LexicalTestsBase.cs
//
// The C# SyntaxFactory.ParseTokens (the dropped Syntax infrastructure) maps
// to the full_moon Lexer over the source text. The C# token stream excludes
// the trivia (attached to the tokens instead) — the port filters the
// full_moon stream with TokenType::is_trivia. The token rows (kind, text,
// span, value) come from the ShortToken::from_token / from_trivia ctors
// (ShortToken.cs:19-21, rows 790-791 — the dropped SyntaxToken/SyntaxTrivia
// dock on the full_moon Token; the C# spans are UTF-16, the port's are bytes —
// the test sources are ASCII).

use full_moon::tokenizer::{Lexer, LexerResult, TokenType};

use crate::luatestbase::options_to_version;
use crate::shorttoken::ShortToken;
use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

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
            .map(|token| ShortToken::from_token(token, options))
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
}
