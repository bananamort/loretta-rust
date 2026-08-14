use loretta_fuzz::{fuzz_input, run_iters};
fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let _ = full_moon::parse_fallible(&text, full_moon::ast::LuaVersion::new());
}
fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("structured", iters, seed, target);
}
