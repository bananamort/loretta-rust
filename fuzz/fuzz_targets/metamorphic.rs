use full_moon::tokenizer::{Lexer, LexerResult};
use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let version = full_moon::ast::LuaVersion::new();
    // metamorphic = trivia-only mutation: for a *clean* lex, dropping all
    // trivia and re-lexing the rendered token stream must reproduce the same
    // non-trivia tokens. Recovered lexes are skipped: their partial token
    // streams are not guaranteed to be re-lexable into themselves.
    let tokens = match Lexer::new(&text, version).collect() {
        LexerResult::Ok(tokens) => tokens,
        LexerResult::Recovered(_, _) | LexerResult::Fatal(_) => return,
    };
    let significant: Vec<String> = tokens
        .iter()
        .filter(|token| !token.token_type().is_trivia())
        .map(|token| token.to_string())
        .collect();
    if significant.is_empty() {
        return;
    }
    let variant = significant.join(" ");
    let variant_tokens = match Lexer::new(&variant, version).collect() {
        LexerResult::Ok(tokens) => tokens,
        LexerResult::Recovered(_, _) | LexerResult::Fatal(_) => return,
    };
    let variant_significant: Vec<String> = variant_tokens
        .iter()
        .filter(|token| !token.token_type().is_trivia())
        .map(|token| token.to_string())
        .collect();
    assert_eq!(significant, variant_significant);
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("metamorphic", iters, seed, target);
}
