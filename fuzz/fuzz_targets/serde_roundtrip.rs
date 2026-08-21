use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // serde_roundtrip = the full_moon serde (default feature) contract: an
    // AST serialized to JSON under Luau must deserialize back to a tree that
    // prints identically.
    let version = full_moon::ast::LuaVersion::new().with_luau();
    let ast = full_moon::parse_fallible(&text, version).into_ast();
    let json = serde_json::to_string(&ast).expect("serialize ast");
    let back: full_moon::ast::Ast = serde_json::from_str(&json).expect("deserialize ast");
    assert_eq!(ast.to_string(), back.to_string());
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("serde_roundtrip", iters, seed, target);
}
