use full_moon::tokenizer::{Lexer, LexerResult};
use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // spans = byte-span invariants under Lua 5.4: every token's
    // [start, end) span (via Position::bytes) must be in bounds, ordered,
    // and non-overlapping with the previous token.
    let version = full_moon::ast::LuaVersion::new().with_lua54();
    let tokens = match Lexer::new(&text, version).collect() {
        LexerResult::Ok(tokens) | LexerResult::Recovered(tokens, _) => tokens,
        LexerResult::Fatal(_) => return,
    };
    let mut previous_end = 0usize;
    for token in &tokens {
        let start = token.start_position().bytes();
        let end = token.end_position().bytes();
        assert!(start <= end, "inverted span");
        assert!(end <= text.len(), "span out of bounds");
        assert!(start >= previous_end, "overlapping or out-of-order span");
        previous_end = end;
    }
    assert_eq!(previous_end, text.len(), "tokens do not cover the input");
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("spans", iters, seed, target);
}
