use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // compile = parse under the full dialect set, materializing both the
    // reconstructed tree and the collected diagnostics.
    let result = full_moon::parse_fallible(&text, full_moon::ast::LuaVersion::new());
    let _ = result.ast();
    let _ = result.errors();
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("compile", iters, seed, target);
}
