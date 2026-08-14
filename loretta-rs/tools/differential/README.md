# Differential harness

Byte-exact differential: C# reference (`references/Loretta`) vs Rust (`loretta-rs`) per `LuaVersion`.

Outputs per `corpus/*.lua`:
- diagnostics
- normalized AST dump
- scope tree
- constant-folded output
- minified output
- symbol-display samples

See `docs/PLAN.md` §2 Stage 5 and `docs/AGENTS.md` Oracles.

Status: pending — scaffold only. Run `NUGET_PACKAGES=loretta-rs/tools/differential/.packages dotnet restore` when C# harness lands.
