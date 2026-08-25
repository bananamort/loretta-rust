# Audit — Loretta-RS Port (C# reference @ `b767b4e`)

This file is the working brief for the next fixing agent. It contains two things only:

1. The **unresolved candidate findings** reported by the most recent independent audit
   (2026-08-24), reproduced verbatim as reported. These are CLAIMS, not verdicts — the agent
   must independently confirm or refute each one against both sources before touching code.
2. The **binding workflow** for landing fixes.

Past audit rounds (2026-08-15 baseline → v2 → v3 → errata; 66 resolved findings) live in git
history (`git log -- docs/AUDIT.md`, key commits `c33ced4`, `89e5795`, `0441b02`) and in the
merged PRs (#1319–#1383). Do not treat their descriptions as current state.

## Before anything else

- Read all four governing docs IN FULL: `docs/AGENTS.md`, `docs/PLAN.md`,
  `docs/TRANSLATION.md`, `docs/COMMIT.md`. They define verbatim parity, the Port Boundary
  (full_moon is the lexer/parser/AST; Parser//Syntax//Core/Portable and GLua operators are
  DROP), hard prohibitions, gates, and PR workflow. Where they are stricter than this file,
  they win.
- Then independently re-audit for divergences NOT on the list below. A candidates list is not
  a ceiling: you are responsible for your own full read of both trees, not just confirming
  these claims.

## Unresolved candidate findings (verbatim from the 2026-08-24 independent audit)

**Candidate A — Dot access never folds expression-keyed table fields**
`experimental/constantfolder.rs` Dot arm (~`:577-581`) calls
`lookup_table_field(&table, Some(&name), None)` — with `brackets=None`, the
`ExpressionKey` + `is_str_with_value` arm is unreachable.
C# anchor: `ConstantFolder.cs:348-357` (VisitMemberAccessExpression) checks BOTH
IdentifierKeyed (`Text == MemberName`) AND ExpressionKeyed
(`HasEFlag(key, IsStr) && GetValue<string>(key) == MemberName`).
Reported observable: `({ ["x"] = 1 }).x` folds to `1` in C#, stays unfolded in Rust
(differential-probed on both harnesses during the audit). Bracket form and name-key form
reported folding identically.

**Candidate B — find_variable panic message diverges**
`scoping/iscope.rs:146` panics `format!("'{name}' must be a valid identifier.")` —
interpolates the actual invalid value.
C# anchor: `IScope.cs:167`
`throw new ArgumentException($"'{nameof(name)}' must be a valid identifier.")`.
Note: `nameof(name)` compiles to the literal `"name"`, so the C# message is always
`'name' must be a valid identifier.` regardless of input.

**Candidate C — get_or_create_label_in accepts None label_syntax**
`scoping/iscope.rs:289-300` takes `label_syntax: Option<lua52::Label>` with no non-null check;
C# `IScope.cs:218` has `LorettaDebug.AssertNotNull(labelSyntax)` before use.
Context to verify yourself: the only null-syntax caller is GotoWalker (forward gotos);
`LorettaDebug.AssertNotNull` is `[Conditional("DEBUG")]`; C# debug builds would trip this
assert on any forward goto — decide for yourself whether replicating that assert is parity
or replicating an upstream defect.

**Candidate D — empty-name rename with zero locations returns Ok**
`script/script.rs` rename flow returns Ok unchanged when the variable has no locations; C#
constructs `new RenameRewriter(...)` whose ctor throws ArgumentException on
`string.IsNullOrEmpty(newName)` (`Script.RenameRewriter.cs:14-17`) even when the variable has
no locations. Context to verify yourself: whether any reachable variable can have zero
locations (locals/params carry declaration locations; unreferenced globals do not enter the
map).

**Candidate E — parse-failed trees contribute nothing**
`manager.rs add_tree` drops trees where `full_moon::parse_fallible` errors; C#
`LuaSyntaxTree` always yields an error-recovery tree. This was previously classified as a
Port-Boundary structural consequence — verify that classification yourself against AGENTS.md
Locked Decision 1 / Rationale before accepting it.

For every candidate: read the full C# file and the full Rust file, probe behavior where two
readings could disagree (`.scratch/` probes, existing tests, differential), and reach YOUR OWN
conclusion — fix it if it's real parity loss, refute it with evidence if it isn't, or document
it as a spec-forced boundary if that's what it turns out to be.

### Prior reviewer's opinion per candidate (input, not authority)

The 2026-08-24 verification pass also formed opinions on each candidate. These are recorded so
you can weigh the reasoning — they carry no authority; your own both-sides read does. Where you
agree or disagree, say so in your report either way.

- **A** — Believed REAL and worth fixing. Reasoning: `({["x"]=1}).x` is valid Lua on which the
  outputs visibly diverge (probed live: C# folds to `print(1 )`, Rust leaves unfolded), and the
  fix stays inside the boundary (~10 lines mirroring `ConstantFolder.cs:348-357`'s
  ExpressionKeyed check). Suggested fix shape: pass the folder context through the Dot arm and
  extend `lookup_table_field`'s ExpressionKey arm with an `is_str_with_value`-style comparison;
  pin with a test asserting `({ ["x"] = 1 }) print(t.x)` folds.
- **B** — Believed REAL but de minimis severity. Reasoning: Logic Parity says "same messages,"
  and C#'s message is the constant `'name' must be a valid identifier.` (`nameof` compiles to
  the literal); the fix is hardcoding that string at `iscope.rs:146`. No current test asserts
  the panic text, which is why nothing catches it. Suggested: hardcode + add a message
  assertion to the empty-rename test.
- **C** — Believed NOT worth replicating verbatim. Reasoning: the C#
  `AssertNotNull(labelSyntax)` fires debug-only, and C# itself passes null syntax for forward
  gotos from GotoWalker (`GotoWalker.cs:26`, single-arg call) — so debug builds of upstream
  would trip their own assert on any forward goto; Loretta's tests never combine rename+goto,
  so upstream never noticed. The port's Option + `set_label_syntax` design avoids that defect.
  Recommendation: keep the port's design, document the divergence from the assert at the call
  site.
- **D** — Believed UNOBSERVABLE for reachable variables. Reasoning: locals/params always carry
  declaration locations and unreferenced globals never enter the map, so the zero-location path
  cannot be reached through public API. If you find a reachable zero-location case, this
  becomes real; otherwise recommend documenting rather than replicating the ArgumentException.
- **E** — Believed correctly classified as Port-Boundary structural. Reasoning: full_moon
  returns no AST on failure and any recovery layer would be a local Lua parser (Locked Decision
  1 / Rationale). Recommendation: keep the drop, keep the in-code comment current.

If your independent read contradicts any of these opinions, your read wins — record why.

## Coverage limits of the 2026-08-24 audit (line-read these anchors yourself)

The independent audit's pass was full-read for nearly everything, but it self-declared three
coverage limits. They do not invalidate its findings, but they mean specific anchors were
probe-checked rather than line-read — so YOUR Source Protocol read is the first true line-read
for them:

- **`Core/Portable/Compilation/ParseOptions.cs`** — probe-checked only (Candidate B/64's fix
  anchors to `ParseOptions.cs:47-55`). Read the file itself before trusting the anchor.
- **`Core/Portable/RealParser.cs`** — probe-checked only; Findings 28-31's C# behaviors
  (sign→0.0, overflow→false, trailing-garbage tolerance) rest on probes plus the code read in
  `NumberParsing.cs`. Read `RealParser.cs:15-17,30-37,288-368,384-392` yourself.
- **DROP-side infrastructure received no review** — everything inside `Core/Portable`,
  `Parser/`, `Syntax/`, and `Generated/` was skipped per Port Boundary (correct per docs), but
  that also means nothing verifies those trees stayed byte-stable at b767b4e beyond git.
  Spot-verify via `git -C references/Loretta status/log` if you want belt-and-braces.

Additionally, two table-level "exact" verifications (unicode_categories vs Unicode data;
LuaResources vs resx) trace back to mechanical diffs from an earlier session, re-cited rather
than repeated. If you touch those tables, re-run the diff rather than citing this file.

Count-decomposition caveat: the run's 231 passed decomposes as ~197 integration + ~35
lib-unit by attribute count, while AGENTS.md frames it as 172 Oracle-1 + internals. Both end
at 231/0; use raw cargo output as truth, not either decomposition.

## Binding workflow for landing fixes

- One finding per PR; inside it, small gate-green commits (reorder/stub first, behavior flips
  after). Never mix findings. Never land red at any commit.
- Gates from `loretta-rs/`: fmt, clippy `-D warnings`, check/test
  `--workspace --all-features --locked`, drift, differential byte-exact. Baselines:
  tests 231 passed / 0 failed · differential identical 1870 / pending 0 / FAILED 0.
  Recount yourself rather than trusting these numbers.
- Never push `main`: `gh pr create` → `gh pr checks --watch` → squash-merge with
  `--delete-branch`. Revert and re-queue on red.
- Protected: `references/**`, `corpus/**` (incl. expected outputs), `docs/**` — never edit;
  spec changes go through a separate `spec:` PR. Only writable markdown:
  `loretta-rs/PROGRESS.md` (update touched rows in the same PR).
- Probes live in `.scratch/` (gitignored); never committed.
- Oracles decide correctness; compiling is not correct. If a delta appears, investigate —
  never edit reference outputs to match Rust.
