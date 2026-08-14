# Loretta to Rust Port

Port Loretta (`../references/Loretta/src/Compilers/Lua/Portable/`) verbatim from C# to Rust using `full-moon`.

- C# source: `../references/Loretta/` (`b767b4e`)
- Rust AST: `full_moon` 2.2.0 (`../references/full-moon/` `47d4bf9`)

## Rationale

We are using `full-moon` because it is maintained on crates.io and stays automatically up-to-date with Luau syntax updates. This replaces the need to port Loretta's equivalent functionality, allowing us to port Loretta's remaining logic directly onto `full-moon`.

## Requirements

1. **Logic Parity.** Port Loretta's logic verbatim without omitting or skipping code. Same types, same member names (snake_cased), same control flow, same constants, same error codes and messages.

2. **`full-moon` Integration.** Use `full-moon` in Rust instead of Loretta's equivalent functionality. Never port Loretta's lexer, parser, or syntax-tree model. If a Loretta capability has no full-moon equivalent, it is dropped per Port Boundary.

3. **Verbatim Translation.** Translate each graph node verbatim into Rust. The port must stay diffable against the C#.

## Locked Decisions

1. **full-moon is the lexer, parser, and AST.** Do not port Loretta's `GreenNode`, `SyntaxNode`, `SyntaxToken`, `SyntaxTrivia`, visitors, `SyntaxFactory`, `Syntax.xml`, `SourceText`, `TextSpan`, or pooling/caching. If no full-moon equivalent exists, the capability is dropped.

2. **Dialect support = full-moon's feature set.** `luau`, `lua52`, `lua53`, `lua54`, `luajit`, `cfxlua`. Everything else is dropped — including GLua operators (`&&`, `||`, `!=`, `!`) and C-style comments (`//`, `/* */`) from `LuaSyntaxOptions.GMod`.

3. **Everything else is ported verbatim.** No idiomatic rewrites. No redesign around full-moon beyond mechanical node mapping.

4. **Two oracles prove correctness.** See Oracles below.

5. **Port lives in `loretta-rs/`.** Workspace with `loretta` lib, `loretta-cli` bin, `tests/`, `corpus/`, `tools/golden-dumper`.

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
| `Compilers/Lua/Portable/LuaSyntaxOptions` + `LuaParseOptions` + enums + `Operations/` | ADAPT | `loretta/src/options.rs` |
| `Compilers/Lua/Portable/SymbolDisplay/` | PORT | `loretta/src/symbol_display/` |
| `Compilers/Lua/Experimental/` | PORT | `loretta/src/experimental/` |
| `Compilers/Lua/CommandLine/` | PORT | `loretta-cli/src/main.rs` |
| `Compilers/Lua/Test/Portable/` + `Test.Utilities` | PORT | `loretta-rs/tests/` |
| `InternalBenchmarks/samples/benchies/*.lua` | REUSE | `loretta-rs/corpus/` |

- Counts: `Lua/Portable` 99 hand + 6 generated (108 with `obj/`), `Experimental` 14, `Core/Portable` 217, `Test/Portable` 31 (30 hand + 1 generated, 637 `[Test]`s excluding generated).
- When a dropped type is needed (e.g. `TextSpan`, `SourceText`), use the full-moon equivalent (`Position.bytes`, `&str`).
- `LuaExtensions.cs` in `Portable/` is syntax helpers over dropped nodes — DROP. `Experimental/LuaExtensions.cs` (`ConstantFold`/`Minify`) is PORT.
- Full table with file-level detail is in `PLAN.md` §3.

## The Method

1. **A codebase as a graph.** Parse C# into a typed graph. One node per type, method, property, field, enum, etc. Edges: `declares`, `calls`, `type-uses`, `inherits`, `implements`, `overrides`, `contains-nested`. The graph is the unit of work, not the file tree.

2. **One item per file, bottom-up.** Each node becomes one Rust file with an explicit `use` header. Nodes are topo-sorted. A node is translated only after its dependencies exist, so the prompt sees the real Rust types, not guesses. Cycles are withheld as hand-designed clusters whose stub shape is a contract.

3. **Gate every landing.** A node must compile in-tree and pass a drift check before it lands. Drift means no dropped declarations, no fake green `mod` deletions, and no stubs. Failures are reverted and re-queued. Drift is checked immediately after every change, not at the end.

4. **Two oracles, not spot checks.** Equivalence is proven twice. The same inputs must produce identical results in C# and Rust.

Details for each step, including the C# constructs covered and the per-node prompt layout, are in `PLAN.md` §2. The full pipeline diagram is there.

## Oracles

- **Oracle 1 — Ported test suite.** 637 `[Test]`s translated to `#[test]` case tables. See `PLAN.md` §3 for the subsystem breakdown.
- **Oracle 2 — Byte-exact differential.** `loretta-rs/tools/golden-dumper` (C#) and the Rust port dump the same outputs per input and per `LuaVersion` preset. Compared byte-for-byte on `corpus/`. Outputs: diagnostics, normalized AST dump, scope tree, constant-folded output, minified output, symbol-display samples. CLI is covered by `loretta-cli` integration tests.

A mismatch is a bug in Rust. Never edit golden files.

- **Rule:** Compiling is not correct. Oracles decide correctness.

## Definition of Done

- **Item:** all gates and both oracles pass, committed via PR, file starts with `// Ported from <C# path> (b767b4e): <names>`.
- **Project:** every `PORT`/`ADAPT` row landed; full test suite green; differential byte-identical across corpus.

## Where to Look Next

- Pipeline detail and Dropped-vs-Replaced table: `PLAN.md`
- How to translate correctly: `TRANSLATION.md`
- Git, commits, and PR workflow: `COMMIT.md`
