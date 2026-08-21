use loretta::experimental::constantfolder::ConstantFolder;
use loretta::experimental::constantfoldingoptions::ConstantFoldingOptions;
use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // optdiff = option diff: the same source under the default dialect set
    // vs cfxlua (C-style comments) must not crash either parser; materialize
    // both printed trees to surface option-sensitive differences.
    let base = full_moon::ast::LuaVersion::new();
    let csharp = base.with_cfxlua();
    let a = full_moon::parse_fallible(&text, base);
    let b = full_moon::parse_fallible(&text, csharp);
    if a.errors().is_empty() && b.errors().is_empty() {
        let _ = (a.ast().to_string(), b.ast().to_string());
    }
    // The ported constant folder must be idempotent: folding twice equals
    // folding once (the C# ConstantFolder contract).
    let mut folder = ConstantFolder::new(ConstantFoldingOptions::ALL);
    let once = folder.fold(a.into_ast());
    let twice = folder.fold(once.clone());
    assert_eq!(once.to_string(), twice.to_string());
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("optdiff", iters, seed, target);
}
