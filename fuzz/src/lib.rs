use std::io::Read;
pub fn run_iters<F>(name: &str, iters: usize, seed: u64, mut f: F) where F: FnMut(&[u8]) {
    let mut rng = seed;
    let mut buf = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut buf);
    if !buf.is_empty() { f(&buf); return; }
    for i in 0..iters {
        rng ^= rng << 13; rng ^= rng >> 7; rng ^= rng << 17;
        let len = (rng as usize % 512) + 1;
        let mut buf = vec![0u8; len];
        let mut s = rng;
        for b in &mut buf { s = s.wrapping_mul(6364136223846793005).wrapping_add(1); *b = (s >> 32) as u8; }
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&buf)));
        if let Err(payload) = result { eprintln!("{}: panic on input {} (iter {} seed {}):", name, hex(&buf), i, seed); std::panic::resume_unwind(payload); }
    }
}
fn hex(buf: &[u8]) -> String { let mut s = String::with_capacity(buf.len()*2); for b in buf { use std::fmt::Write as _; let _ = write!(s, "{:02x}", b); } s }
pub fn fuzz_input() -> (usize, u64) {
    let iters = std::env::var("LORETTA_FUZZ_ITERS").ok().and_then(|v| v.parse().ok()).unwrap_or(5000);
    let seed = std::env::var("LORETTA_FUZZ_SEED").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    (iters, seed)
}
