use loretta_fuzz::{fuzz_input, run_iters};

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    let version = full_moon::ast::LuaVersion::new();
    // splice = textual splice: derive an insert offset from the input itself,
    // insert a fragment at that byte offset, and reparse the spliced source.
    let original = full_moon::parse_fallible(&text, version);
    let _ = original.ast();
    let offset = (data.first().copied().unwrap_or(0) as usize * 7) % (text.len() + 1);
    let bytes = text.as_bytes();
    let mut spliced = Vec::with_capacity(bytes.len() + 22);
    spliced.extend_from_slice(&bytes[..offset]);
    spliced.extend_from_slice(b" local spliced = true ");
    spliced.extend_from_slice(&bytes[offset..]);
    let spliced = String::from_utf8_lossy(&spliced);
    let _ = full_moon::parse_fallible(&spliced, version).ast();
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("splice", iters, seed, target);
}
