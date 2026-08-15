// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.TriviaRewriter (b767b4e): TriviaRewriter
// C# source: src/Compilers/Lua/Experimental/Minifying/TriviaRewriter.cs

use full_moon::tokenizer::{TokenKind, TokenReference};
use full_moon::visitors::VisitorMut;

/// Rewrites tokens to strip trivia (whitespace and comments),
/// keeping only a single space separator where required.
pub struct TriviaRewriter;

impl TriviaRewriter {
    pub const INSTANCE: Self = Self;
}

impl VisitorMut for TriviaRewriter {
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        // If the token is EOF, return as-is
        if token_ref.token().token_kind() == TokenKind::Eof {
            return token_ref;
        }

        // Strip all trivia — the minifier removes whitespace and comments
        TokenReference::new(Vec::new(), token_ref.token().to_owned(), Vec::new())
    }
}
