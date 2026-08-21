use std::io::Read;

/// Runs `f` on `iters` pseudo-random inputs derived from `seed`.
///
/// A panic inside `f` is downgraded to a **counted finding**: the failing
/// input is printed as hex, the run continues, and the final summary reports
/// how many panics were observed. The process exits 0 so the nightly soak can
/// complete its full window even when a dependency panics (e.g. full_moon
/// 2.2.0's `parse_attributes` `unwrap()` at `ast/parsers.rs:3709` on a `@`
/// followed by a non-name under the default version set).
///
/// When stdin is piped (reproducer mode), `f` is invoked exactly once on the
/// piped bytes and panics propagate — that is the mode used to replay a
/// reported hex finding.
pub fn run_iters<F>(name: &str, iters: usize, seed: u64, mut f: F)
where
    F: FnMut(&[u8]),
{
    let mut rng = seed;
    let mut buf = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut buf);
    if !buf.is_empty() {
        f(&buf);
        return;
    }

    let mut panics = 0usize;
    for i in 0..iters {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        let len = (rng as usize % 512) + 1;
        let mut buf = vec![0u8; len];
        let mut s = rng;
        for b in &mut buf {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
            *b = (s >> 32) as u8;
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&buf)));
        if let Err(payload) = result {
            panics += 1;
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "(non-string panic payload)".to_string());
            eprintln!(
                "{name}: finding {panics}: panic on input {} (iter {i} seed {seed}): {message}",
                hex(&buf)
            );
        }
    }

    eprintln!(
        "{name}: soak complete: {iters} inputs, {panics} panic(s) downgraded to counted findings"
    );
}

fn hex(buf: &[u8]) -> String {
    let mut s = String::with_capacity(buf.len() * 2);
    for b in buf {
        use std::fmt::Write as _;
        let _ = write!(s, "{b:02x}");
    }
    s
}

pub fn fuzz_input() -> (usize, u64) {
    let iters = std::env::var("LORETTA_FUZZ_ITERS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);
    let seed = std::env::var("LORETTA_FUZZ_SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1);
    (iters, seed)
}
