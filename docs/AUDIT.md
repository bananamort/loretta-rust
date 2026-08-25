# Audit — Loretta-RS Port (FINAL · C# reference @ `b767b4e`)

**Status: all 66 v3 findings RESOLVED and verified. 2 new findings (67–68) OPEN.**
Full audit history (2026-08-15 baseline → v2 → v3 → errata rounds) is preserved in git
(`git log -- docs/AUDIT.md`; key commits `c33ced4`, `89e5795`, `0441b02`). This document is the
live state only: what was fixed, what remains, and what a fixing agent needs.

## Verification basis (independent audit, 2026-08-24)

- 113 C# files + 64 Rust files read in full; every one of the 66 findings dispositioned from
  direct both-sides reads, not prior claims.
- Gates re-run from `loretta-rs/` with `--locked`: fmt clean · clippy `-D warnings` clean ·
  **tests 231 passed / 0 failed** (26 binaries + doc-tests) · differential **identical 1870 /
  pending 0 / FAILED 0** · drift nodes 744 / edges 1335 / topo 744.
- Hard prohibitions hold: zero `todo!`/`unimplemented!`/`unsafe`; allow inventory = exactly the
  documented set (`unreachable_patterns` on non_exhaustive full_moon matches ×10,
  `module_inception` script/mod.rs ×1, `too_many_arguments` luasyntaxoptions.rs ×2).
- Headers `// Ported from … (b767b4e)` on 64/64 files; `references/`, `corpus/`, `docs/`
  untouched by fix commits; PROGRESS.md updated per PR.
- Known documented residuals (spec-sanctioned, do NOT re-flag): lone-surrogate `\u{D800}` →
  U+FFFD; `f64::powf` vs `Math.Pow` last-ulp; parse-failure trees contribute nothing;
  `StringUtils::trim` anti-panic; GMod/GLua operators+comments DROP; type-parameter-default
  drops (1f15ebb).

## Resolved findings (1–66)

All verified fixed on main @ `0441b02` (PRs #1319–#1383). One line each; full detail in git
history of this file.

| # | Summary |
|---|---|
| 1 | vararg Ellipsis creates `"..."` parameter (was `unreachable!()`) |
| 2 | overflow literals fold to 0 + scanner LUA0005 (was panic) |
| 3 | `%0` Long→NaN no-fold instead of panic |
| 4 | InterpolatedString arm implemented (dead-arm cleanup, repro invalid) |
| 5 | multi-tree state accumulates via shared id counter (was last-tree-wins) |
| 6 | forward gotos bind same-scope placeholder (no orphaned jump) |
| 7 | labels carry statement syntax (`set_label_syntax`) |
| 8 | function scopes capture outer variables (override wired) |
| 9 | local-function node maps to its FunctionScope |
| 10 | interpolated-string expressions visited by scope walker |
| 11 | non-ASCII rename gated per-tree LuaJIT rules |
| 12 | tree attribution via node-id bases (not substring search) |
| 13 | referenced/captured variables dedupe (ISet parity) |
| 14 | location store separate from State.Scopes (GetScope null parity) |
| 15 | one Parameter node per parameter |
| 16 | RESOLVED — not a defect (C# `AddReferencedVariable` early-return identical) |
| 17–26 | scanner: decimal/hex/binary/octal LUA0005 paths; hex invented-rule removed; e/E-in-hex classification; ull overflow; complex DoubleOverflow-only; bitwise split lexer-`<<`/parser-`&`\|`; `\z` set `[ \t-\r]`; underscore double-emission; shebang per-run guard; runaway absorption |
| 27–38 | folder: operand trivia; unanchored decFloat; sign→0.0; overflow→no-extract; decFloat-first `"0x1.8p10"`→0.0; UTF-16 `#` length; `\ddd>255` skipped; astral/lone-surrogate handling; folder `\z` includes `\v`; option-gated echo/skip; chained bottom-up const-table fold; concat throw-parity |
| 39–41 | minify: descendant-of-last-use slot release; prefix schedule per-slot ceiling; loop-var records before header expressions |
| 42–46 | Eq/Hash omit two escape fields; `\x` missing-digit silence gate; empty-name rename validation flow; `is_hidden` false-for-all; parserdiagnostics citation corrected |
| 47–50 | `'^'`=Sk(27); astral 4-byte UTF-8; surrogate-half escapes; `is_identifier` whitespace guard |
| 51–62 | test restorations: stub test filled; long-string literal bytes; dup block removed; exact TokenCache assert; exact diagnostics restored group-wide; LUA1016/LUA1019 gates implemented + expectations restored; dead scaffolding + normalizer stub deleted; ContextualKind/goto-row dims restored; enum-wide token sweep; decoded interp values; FindScope/RenameVariable dims restored; redundant sweep trimmed |
| 63 | stale/false comments refreshed (mod headers, false equivalence/interp comments, SYMBOL_ROWS `!=`, ShortToken labels, tie-break note) |
| 64 | validate_options emits LUA2000 BadDocumentationMode (+unit test) |
| 65 | features OrdinalIgnoreCase lookup/dedup semantics |
| 66 | powf-vs-Math.Pow residual documented at `constantfolder.rs:948-952` |

## OPEN findings (for the fixing agent — confirm first, then fix)

### Finding 67 — Dot access never folds expression-keyed table fields

- **Rust:** `experimental/constantfolder.rs` Dot arm (~`:577-581`) calls
  `lookup_table_field(&table, Some(&name), None)` — with `brackets=None`, the
  `ExpressionKey` + `is_str_with_value` arm is unreachable.
- **C#:** `ConstantFolder.cs:348-357` (VisitMemberAccessExpression) checks BOTH
  IdentifierKeyed (Text == MemberName) AND ExpressionKeyed
  (`HasEFlag(key, IsStr) && GetValue<string>(key) == MemberName`).
- **Observable:** `({ ["x"] = 1 }).x` folds to `1` in C#, stays unfolded in Rust.
  Verified live via differential probes on both harnesses.
- **Fix:** in the Dot arm pass the folder through so ExpressionKey fields are checked:
  `lookup_table_field(&table, Some(&name.token().to_string()), Some((/*key-less*/), self))` is
  not directly expressible — mirror the C# by extending `lookup_table_field`'s ExpressionKey
  arm to compare `is_str_with_value(key, name, accept_invalid_escapes)` using the folder's
  syntax options when `brackets` is None but `name` is Some. Add a pinned test:
  `fold_sample(r#"local t = { ["x"] = 1 } print(t.x)"#)` → `print(1 )`.
- **Note:** bracket form `({["x"]=1})["x"]` and name-key form `({x=1}).x` already fold
  identically to C# (probed). Only expression-key-table-via-dot is affected.

### Finding 68 — find_variable panic message diverges from C#

- **Rust:** `scoping/iscope.rs:146` panics `format!("'{name}' must be a valid identifier.")`
  — interpolates the actual invalid value (empty rename → `'' must be…`).
- **C#:** `IScope.cs:167` throws `$"'{nameof(name)}' must be a valid identifier."` —
  `nameof(name)` compiles to the literal `"name"`, so the message is ALWAYS
  `'name' must be a valid identifier.` regardless of input.
- **Fix:** hardcode the message:
  `panic!("'name' must be a valid identifier.")`. No test currently asserts the string;
  add one alongside (empty-rename path already exists in `script.rs` tests).

## Workflow for the fixing agent (binding — COMMIT.md/PLAN.md/TRANSLATION.md)

- Confirm each finding against both sides BEFORE editing (Source Protocol: read the full C#
  file, its deps, the full_moon APIs, the ported Rust deps).
- One finding per PR; inside it, small gate-green commits. Never push `main`;
  `gh pr create` → `gh pr checks --watch` → squash-merge. Never land red.
- Gates: fmt, clippy `-D warnings`, check/test `--workspace --all-features --locked`,
  drift, differential byte-exact (baseline 1870/0/0; tests ≥231/0). Oracles decide.
- Never edit `references/**`, `corpus/**`, or `docs/**`; only writable markdown is
  `loretta-rs/PROGRESS.md` (update touched rows in the same PR).
- Probes live in `.scratch/` (gitignored); never committed.

## Verdict

The port meets AGENTS.md Requirements 1–3 (Logic Parity, full-moon Integration, Verbatim
Translation) and the Definition of Done, **except** for Findings 67–68 above — narrow,
non-corpus-visible divergences that close out with the two listed fixes. After those land and
both oracles stay green, the verbatim-port goal is fully met with zero open items.
