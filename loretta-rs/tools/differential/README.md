# Differential harness — strict per-feature C# oracle

Byte-exact differential: C# reference (`references/Loretta` `b767b4e` `Loretta.CodeAnalysis.Lua` + `Experimental`) vs Rust (`loretta-rs` `full_moon` `2.2.0`) per `LuaVersion` preset.

C# oracle (`Differential.csproj` `net10.0`): `dotnet run -- <operation> <preset> <code|file> [--out <dir>]`
- `operation`: `options` `diagnostics` `lex` `parse` `scope` `rename` `constantfold` `minify` (covers `charutils`/`stringutils`/`hexfloat`/`objectdisplay`/`operator` via `lex`/`parse`)
- `preset`: `Lua51` `Lua52` `Lua53` `Lua54` `LuaJIT20` `LuaJIT21` `GMod` `Luau` `FiveM` `All` `AllWithIntegers` (11 presets, `GMod` is expected to error where `//`/`&&` not in `All`)
- `all` mode: `dotnet run -- all All <file.lua> --out loretta-rs/corpus/expected` → `expected/<preset>/<file>/<operation>.json` (strict per-feature, not per-node)

Outputs per `corpus/*.lua` + `corpus/features/*.lua` (small, covers every `PORT` node via feature group, not 744 files):
- `diagnostics` `{diagnostics[], hasErrors}`
- `lex` `{tokens[], count, roundTrip}`
- `parse` `{treeText, rootKind, hasErrors}`
- `scope` `{rootScope, scopeCount}` (`Script` + `ScopeAndVariableManager`)
- `constantfold` `{original, withoutExtraction, withExtraction}` (`ConstantFoldingOptions`)
- `minify` `{minified}` (`NamingStrategies.Alphabetical`)

Expected committed at `loretta-rs/corpus/expected/<preset>/<file>/<operation>.json` (1848 files for `anim.lua` + 13 `features/*.lua` × 11 presets × 12 ops; `rustic.lua` (6.1 MB) is throttled to `diagnostics` + `parse` only (>500 KB) — 1870 files total). `loretta-rs/corpus/features/` are tiny per-feature inputs that hit every `PORT` node via its feature group.

Rust oracle: agents port `diagnostics`→`loretta/src/errors`, `scope`→`loretta/src/scoping`/`script`, `fold`→`loretta/src/experimental`, etc., and implement `loretta-rs/tools/differential` Rust side that writes same JSON for `cargo test` Oracle 1 + differential Oracle 2. Drift is byte-exact `diff` of Rust vs C# `expected`.

Rust side lives in `loretta-rs/differential/` (workspace member `differential`, no new dependencies; hand-rolled System.Text.Json-compatible writer in `src/json.rs`):

- `cargo run -p differential -- <operation> <preset> <code|file> [--out <dir>]` — same CLI as the C# oracle. `operation`: `options` `diagnostics` `lex` `parse` (implemented); `scope` `rename` `constantfold` `minify` land with their subsystems. `all <preset> <file> --out <dir>` writes `dir/<preset>/<stem>/<op>.json` exactly like the C# reference.
- `cargo run -p differential -- check corpus/expected --out <tmp>` — Oracle 2 gate (run by CI). Compares every implemented op against the committed C# reference for `corpus/anim.lua` + `corpus/features/*.lua` + `corpus/rustic.lua` × 11 presets, byte-for-byte (rustic runs `diagnostics`/`parse` only — throttled >500 KB). Pairs whose reference output has `hasErrors: true` are reported as pending coverage until the per-preset version-gating diagnostics land; any other difference is a hard failure (drift = bug in Rust).
- Token kind names are oracle data: symbol→kind table from `Portable/Syntax/SyntaxKind.cs`, keyword/literal naming verified against the expected `lex.json` files.

Run: `NUGET_PACKAGES=/tmp/loretta-differential-packages dotnet run --project loretta-rs/tools/differential/Differential.csproj -c Release -- <op> <preset> <code>`
See `docs/PLAN.md` §2 Stage 5 and `docs/AGENTS.md` Oracles.
