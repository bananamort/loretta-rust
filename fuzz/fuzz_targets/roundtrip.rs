use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // roundtrip = parse -> print -> parse under Lua 5.2 (goto/labels,
    // hex floats): a clean parse must reproduce the same tree when its
    // printed form is parsed again.
    let version = full_moon::ast::LuaVersion::new().with_lua52();
    let a = full_moon::parse_fallible(&text, version);
    if a.errors().is_empty() {
        let printed = a.ast().to_string();
        let b = full_moon::parse_fallible(&printed, version);
        assert_eq!(a.ast().nodes(), b.ast().nodes());
        assert_eq!(a.errors(), b.errors());
    }
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("roundtrip", iters, seed, target);
}
