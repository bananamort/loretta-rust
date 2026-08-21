use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // determinism = the same input parsed twice under the same options must
    // yield identical trees and identical diagnostics (Lua 5.3 semantics:
    // integer division, bitwise operators).
    let version = full_moon::ast::LuaVersion::new().with_lua53();
    let a = full_moon::parse_fallible(&text, version);
    let b = full_moon::parse_fallible(&text, version);
    assert_eq!(a.ast().nodes(), b.ast().nodes());
    assert_eq!(a.errors(), b.errors());
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("determinism", iters, seed, target);
}
