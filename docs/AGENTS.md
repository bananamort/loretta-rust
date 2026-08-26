# Loretta to Rust Port

Port Loretta (`../references/Loretta/src/Compilers/Lua/Portable/`) verbatim from C# to Rust using `full-moon`.

- C# source: `../references/Loretta/` (`b767b4e`)
- Rust AST: `full_moon` 2.2.0 (`../references/full-moon/` `47d4bf9`)

## Rationale

We are using `full-moon` because it is maintained on crates.io and stays automatically up-to-date with Luau syntax updates. This replaces the need to port Loretta's equivalent functionality, allowing us to port Loretta's remaining logic directly onto `full-moon`.

## Decision Rationale

Every decision is made to achieve a **verbatim** port of C# Loretta to Rust using `full_moon` — correctly and byte-for-byte — **not** to satisfy documentation. If a Loretta feature has a `full_moon` equivalent, it is ported verbatim via mechanical adaptation (e.g., `TextSpan` → `Position.bytes`). Documentation is updated to reflect the correct verbatim decision, not vice versa — a `spec:` PR amends docs when the port’s correctness requires it.

**Why `GMod` (GLua) is the only `DROP` dialect — and why this does not authorize doc edits:**
`LuaSyntaxOptions.GMod` (`acceptCCommentSyntax:true` `acceptCBooleanOperators:true`) adds `//`/`/* */` and `&&`/`||`/`!=`/`!`. `full_moon` `2.2.0` `src/tokenizer/structs.rs:94` `Symbol` and `src/tokenizer/structs.rs:318` `TokenType::CStyleComment` have **no** `Symbol` for `&&`/`||`/`!=`/`!` and no `TokenType` for `//` outside `[cfxlua]` `CStyleComment`. Supporting them would require forking `full_moon`’s lexer (`lexer.rs`) and `Symbol::from_str` — local parser upkeep that directly violates `Rationale` (`full_moon` stays up-to-date on `crates.io` so we port remaining logic, not the parser). Therefore `GMod` is explicitly `DROP` per `Port Boundary` and `Locked Decision 2`. **All other** `LuaSyntaxOptions` presets (`Lua51`/`Lua52`/`Lua53`/`Lua54`/`LuaJIT20`/`LuaJIT21`/`Luau`/`FiveM`/`All`) map 1:1 to `full_moon` `LuaVersion` `luau`/`lua52`/`lua53`/`lua54`/`luajit`/`cfxlua` (`src/ast/versions.rs:3` bitfield). An agent must **not** infer from this that docs are mutable for convenience — `docs/` stays read-only per `Locked Decision 7`; only a `spec:` PR that proves `full_moon` has no equivalent may amend `Port Boundary`.

## Requirements

1. **Logic Parity.** Port Loretta's logic verbatim without omitting or skipping code. Same types, same member names (snake_cased), same control flow, same constants, same error codes and messages.

2. **`full-moon` Integration.** Use `full-moon` in Rust instead of Loretta's equivalent functionality. Never port Loretta's lexer, parser, or syntax-tree model. If a Loretta capability has no full-moon equivalent, it is dropped per Port Boundary.

3. **Verbatim Translation.** Translate each graph node verbatim into Rust. The port must stay diffable against the C#.

## Locked Decisions

1. **full-moon is the lexer, parser, and AST.** Do not port Loretta's `GreenNode`, `SyntaxNode`, `SyntaxToken`, `SyntaxTrivia`, visitors, `SyntaxFactory`, `Syntax.xml`, `SourceText`, `TextSpan`, or pooling/caching. If no full-moon equivalent exists, the capability is dropped.

2. **Dialect support = full-moon's feature set.** `luau`, `lua52`, `lua53`, `lua54`, `luajit`, `cfxlua`. Everything else is dropped — including GLua operators (`&&`, `||`, `!=`, `!`) and C-style comments (`//`, `/* */`) from `LuaSyntaxOptions.GMod`.

3. **Everything else is ported verbatim.** No idiomatic rewrites. No redesign around full-moon beyond mechanical node mapping.

4. **Two oracles prove correctness.** See Oracles below.

5. **Port lives in `loretta-rs/`.** Workspace with `loretta` lib, `loretta-cli` bin, `tests/`, `corpus/`, `tools/differential`.

6. **`references/` is read-only.** Never edit sources under `references/`.

7. **`docs/` is read-only during the port.** Never edit any file in `docs/` (`AGENTS.md`, `PLAN.md`, `COMMIT.md`, `TRANSLATION.md`) while porting. If the spec needs to change, stop and open a `spec:` PR separately. To track progress, create and write only to `loretta-rs/PROGRESS.md` — do not create or write to any other `.md` file, and do not reuse or overwrite `docs/`.

## Port Boundary

C# paths are under `../references/Loretta/src/`.

| C# source | Disposition | Rust destination |
|---|---|---|
| `Compilers/Core/Portable/` (red/green trees, `SourceText`, diagnostics, pooling) | DROP | — |
| `Compilers/Lua/Portable/Parser/` (lexer, `LanguageParser`, `SlidingTextWindow`) | DROP | — |
| `Compilers/Lua/Portable/Syntax/` (nodes, visitors, `Syntax.xml`, `Generated`) | DROP | — |
| `Compilers/Lua/Portable/Errors/` | PORT | `loretta/src/errors/` |
| `Compilers/Lua/Portable/Scoping/` | PORT | `loretta/src/scoping/` |
| `Compilers/Lua/Portable/Script/` | PORT | `loretta/src/script/` |
| `Compilers/Lua/Portable/Utilities/` | PORT | `loretta/src/utilities/` |
| `Compilers/Lua/Portable/LuaSyntaxOptions` + `LuaParseOptions` + enums + `Operations/` | ADAPT | `loretta/src/{luasyntaxoptions,luaparseoptions,backtickstringtype,continuetype,integerformats}.rs` + `operations/{binaryoperatorkind,unaryoperatorkind}.rs` |
| `Compilers/Lua/Portable/SymbolDisplay/` | PORT | `loretta/src/symbol_display/` |
| `Compilers/Lua/Experimental/` | PORT | `loretta/src/experimental/` |
| `Compilers/Lua/CommandLine/` | PORT | `loretta-cli/src/main.rs` |
| `Compilers/Lua/Test/Portable/` + `Test.Utilities` | PORT | `loretta-rs/tests/` |
| `InternalBenchmarks/samples/benchies/*.lua` | REUSE | `loretta-rs/corpus/` |

- Counts: `Lua/Portable` 99 hand + 6 generated (108 with `obj/`), `Experimental` 14, `Core/Portable` 217, `Test/Portable` 31 (30 hand + 1 generated, 157 hand `[Test]`s + 480 Generated = 637 total; Rust has 172 `#[test]`s (157 strict + 15 data-driven splits: parsing_regression +9, type_parsing_regression +7, etc.), Generated is DROP).
- When a dropped type is needed (e.g. `TextSpan`, `SourceText`), use the full-moon equivalent (`Position.bytes`, `&str`).
- `LuaExtensions.cs` in `Portable/` is syntax helpers over dropped nodes — DROP. `Experimental/LuaExtensions.cs` (`ConstantFold`/`Minify`) is PORT.
- Full table with file-level detail is in `PLAN.md` §3.

## The Method

1. **A codebase as a graph.** The C# codebase is parsed into a typed semantic graph — one node per class, interface, enum, record, delegate, method, property, field, etc. — with `declares`, `calls`, `type-uses`, `inherits`, `implements`, `overrides`, `contains-nested` edges. The graph, not the file tree, is the unit of work.

2. **One item per file, bottom-up.** Each node becomes its own Rust file with an explicit `use` header. Nodes are topo-sorted and translated only once their dependencies exist, so each prompt sees the real translated types it depends on — not guesses. Cycles are withheld as hand-designed clusters whose stub shape is a contract.

3. **Gate every landing.** A translated node must compile in-tree and pass a drift check — it may not silently drop declarations, fake green by deleting `mod` entries, or stub out logic — before it lands. Failures are reverted and re-queued. Drift is checked immediately after every change, not at the end.

4. **Two oracles, not spot checks.** Equivalence is proven twice: Loretta's own test suite, ported to 172 Rust `#[test]`s (the 157 hand `[Test]`s of the 637 total; Generated is DROP), plus a byte-exact differential — the same inputs run on the C# reference and on the Rust port must produce identical results.

Details for each step, including the C# constructs covered and the per-node prompt layout, are in `PLAN.md` §2. The full pipeline diagram is there.

## Oracles

- **Oracle 1 — Ported test suite.** The 157 hand `[Test]`s land as 172 `#[test]` case tables (637 total including Generated; Generated is DROP). See `PLAN.md` §3 for the subsystem breakdown.
- **Oracle 2 — Byte-exact differential.** `loretta-rs/tools/differential` (C# reference) and the Rust port produce the same outputs per input and per `LuaVersion` preset. Compared byte-for-byte on `corpus/`. Outputs: diagnostics, normalized AST dump, scope tree, constant-folded output, minified output, symbol-display samples. CLI is covered by `loretta-cli` integration tests.

A mismatch is a bug in Rust. Never edit reference outputs to match Rust.

- **Rule:** Compiling is not correct. Oracles decide correctness.

## Definition of Done

- **Item:** all gates and both oracles pass, committed via PR, file starts with `// Ported from <C# path> (b767b4e): <names>`.
- **Project:** every `PORT`/`ADAPT` row landed; full test suite green; differential byte-identical across corpus.

## Where to Look Next

- Pipeline detail and Dropped-vs-Replaced table: `PLAN.md`
- How to translate correctly: `TRANSLATION.md`
- Git, commits, and PR workflow: `COMMIT.md`
