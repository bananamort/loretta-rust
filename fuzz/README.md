# Loretta-RS Fuzz
Standalone fuzz harness. Detached workspace (`publish = false`).
- **16 targets:** `compile`, `run`, `typeck`, `number`, `structured`, `typeck_defs`, `determinism`, `roundtrip`, `splice`, `optdiff`, `metamorphic`, `spans`, `api`, `gcstress`, `host`, `serde_roundtrip`.
- **Oracle:** never panic, abort, or hang — only `Ok` or structured `Err`.
## Standalone (CI / local smoke, no toolchain)
```sh
cargo build --bins
LORETTA_FUZZ_ITERS=5000 LORETTA_FUZZ_SEED=1 ./target/debug/compile
echo -n "local x=1" | ./target/debug/compile
```
## Nightly soak
Runs via `.github/workflows/fuzz.yml` (`schedule: 17 3 * * *`) as `cargo build --bins` per target with `LORETTA_FUZZ_SEED=${run_id}`.
