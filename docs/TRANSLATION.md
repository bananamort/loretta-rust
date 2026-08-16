# Translation Rules

How to translate C# to Rust for this port. Apply mechanically. If a case is not covered, revert and re-queue with evidence instead of inventing a mapping; if the spec is wrong, open a `spec:` PR.

## Source Protocol

Before writing any Rust, you must read the sources. Code written from memory is treated as fabricated.

1. `read` the full C# file for the item.
2. `read` every C# dependency it touches.
3. `grep` then `read` every full-moon API you will call (including `#[cfg(feature=...)]` gates).
4. `read` the already-ported Rust of your dependencies.

If you have not made these calls this session, make them now. When the file and your expectation disagree, the file wins. When the oracle and your reasoning disagree, the oracle wins.

See `PLAN.md` §2 for the per-node prompt layout and `full-moon` API examples.

## Rules

- **Text is bytes.** C# `string`/`char` are UTF-16; Rust and full-moon are UTF-8 byte offsets (`Position.bytes`). Work on `&str`/`&[u8]`; never index strings.
- **Logical vs bitwise.** `!x` is logical. `&&`/`||` short-circuit; `&`/`|` on `bool` do not — preserve evaluation order.
- **Nullability.** `null` → `Option`; `??` → `unwrap_or`; `?.` → `map`/`and_then`.
- **Exceptions.** `throw` → `Result`/`Option`; filters → `match` on error enums. Preserve exact error codes and messages.
- **Numbers.** `int`→`i32`, `long`→`i64`, `double`→`f64`. `unchecked` wraps → `wrapping_*`. Shifts mask the count (`&31`); Rust panics on oversize — mask explicitly.
- **LINQ.** → iterator chains; `collect()` where C# materialized (`ToList`/`ToArray`). Preserve laziness count.
- **Type system.** Interface → trait; `partial` → one `impl`; extension `this` → trait fn; `out`/`ref` → tuple/`&mut`; properties → `fn name(&self)`; `switch` → `match`; `yield` → `impl Iterator` or `Vec`.
- **`dynamic` (ConstantFolder).** → explicit `enum Numeric { Long(i64), Double(f64) }` reproducing the promotion rules.
- **Reflection.** → static tables from the committed generated output and English `LuaResources.resx` strings.
- **Immutability.** `ImmutableArray` → `Vec` + clones; pooling/caching → dropped.

Full C#→Rust mapping with file references is in `PLAN.md` §2.

## Hard Prohibitions

Violating any of these is a revert.

1. No stubs: `todo!()`, `unimplemented!()`, dummy returns, `// Logic elided`.
2. No skipped surface: every member lands or is documented as intentionally dropped.
3. No test tampering: never edit or ignore a ported test to pass; never edit reference outputs.
4. No warning suppression: no `#[allow(...)]`, no `unsafe` — except `#[allow(unreachable_patterns)]` on `#[non_exhaustive]` `full_moon` `TokenType`/`Symbol` matches (wildcard ` _ => unreachable!()` required), single `#[allow(clippy::module_inception)]` for `loretta/src/script/mod.rs` (`pub mod script`), and `#[allow(clippy::too_many_arguments)]` for `loretta/src/luasyntaxoptions.rs` `LuaSyntaxOptions::new`/`with` (27 args, C# `28` fields verbatim).
5. No redesign: no renames beyond snake_case, no new dependencies.
6. No scope creep: never port anything marked DROP; never edit `references/`.
7. No translation from memory: you must `read`/`grep` per Source Protocol first.
8. One item per commit. No batching.

## Details

- `Text is bytes:` `TextSpan.cs:229` and `SourceText.cs:1026` are UTF-16; `full-moon` `Position.bytes` (`tokenizer/structs.rs:852`) is UTF-8. Never index strings. `CharUtils.cs:219` and `ObjectDisplay.cs:427` character tables must match exactly — use the oracle as arbiter.
- `Numbers:` `HexFloat.cs:334` and `IntegerFormats` `NotSupported`/`Double`/`Int64` require exact `wrapping_*` and `&31` handling.
- `dynamic:` `ConstantFolder.NumberParsing.cs:68` and `ExpressionFlags.cs:125` define the promotion rules for `Numeric`.
