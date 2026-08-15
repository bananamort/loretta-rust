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

Expected committed at `loretta-rs/corpus/expected/<preset>/<file>/<operation>.json` (88 files for `anim.lua` + 7 `features/*.lua` × 11 presets, `rustic.lua` omitted for size). `loretta-rs/corpus/features/` are tiny per-feature inputs that hit every `PORT` node via its feature group.

Rust oracle: agents port `diagnostics`→`loretta/src/errors`, `scope`→`loretta/src/scoping`/`script`, `fold`→`loretta/src/experimental`, etc., and implement `loretta-rs/tools/differential` Rust side that writes same JSON for `cargo test` Oracle 1 + differential Oracle 2. Drift is byte-exact `diff` of Rust vs C# `expected`.

Run: `NUGET_PACKAGES=/tmp/loretta-differential-packages dotnet run --project loretta-rs/tools/differential/Differential.csproj -c Release -- <op> <preset> <code>`
See `docs/PLAN.md` §2 Stage 5 and `docs/AGENTS.md` Oracles.
