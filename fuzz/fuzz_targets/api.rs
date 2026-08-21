use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;
use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // api = exercise the ported loretta options API directly (not raw
    // full_moon): every syntax-options preset must construct parse options
    // and parse the input under the full dialect set.
    for preset in LuaSyntaxOptions::ALL_PRESETS {
        let parse_options = LuaParseOptions::new(preset.clone());
        let _ = (parse_options.language(), parse_options.documentation_mode());
        let _ = preset.accept_hash_strings();
        let version = full_moon::ast::LuaVersion::new();
        let result = full_moon::parse_fallible(&text, version);
        let _ = (result.ast(), result.errors());
    }
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("api", iters, seed, target);
}
