use full_moon::tokenizer::{Lexer, LexerResult, TokenType};
use loretta::utilities::hexfloat::HexFloat;
use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // number = numeric literal fuzzing under Luau (hex-float literals are
    // Luau/5.2+ syntax): lex every Number token and run the ported C#
    // HexFloat parser over its text, round-tripping the bit pattern.
    let version = full_moon::ast::LuaVersion::new().with_luau();
    let tokens = match Lexer::new(&text, version).collect() {
        LexerResult::Ok(tokens) | LexerResult::Recovered(tokens, _) => tokens,
        LexerResult::Fatal(_) => return,
    };
    let mut literals = 0usize;
    let mut parsed = 0usize;
    let mut rejected = 0usize;
    for token in tokens {
        if let TokenType::Number { text } = token.token_type() {
            literals += 1;
            match HexFloat::double_from_hex_string(&text.to_string()) {
                Ok(value) => {
                    parsed += 1;
                    let _ = HexFloat::double_to_hex_string(value);
                }
                Err(_) => rejected += 1,
            }
        }
    }
    let _ = (literals, parsed, rejected);
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("number", iters, seed, target);
}
