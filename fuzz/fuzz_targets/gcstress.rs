use full_moon::tokenizer::{Lexer, LexerResult};
use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // gcstress = allocation churn under Lua 5.4: repeatedly parse, print and
    // drop full trees plus token streams so allocator/GC pressure patterns
    // (arena growth, drop-order bugs) get exercised.
    let version = full_moon::ast::LuaVersion::new().with_lua54();
    let mut churn = 0usize;
    for _ in 0..16 {
        let result = full_moon::parse_fallible(&text, version);
        let printed = result.ast().to_string();
        let _ = result.errors();
        let tokens = match Lexer::new(&printed, version).collect() {
            LexerResult::Ok(tokens) | LexerResult::Recovered(tokens, _) => tokens,
            LexerResult::Fatal(_) => return,
        };
        churn += tokens.len();
    }
    let _ = churn;
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("gcstress", iters, seed, target);
}
