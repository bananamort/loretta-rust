# Audit — Loretta-RS Port (C# reference @ `b767b4e`)

This file is the working brief for the next fixing agent. It contains two things only:

1. The **findings** from the two independent full-port audits of 2026-08-25
   (`docs/independent_audits/AUDIT_AGENT_DS.md`, `AUDIT_AGENT_OX.md`), each independently
   re-verified against both sources on HEAD `aedd440`. These are claims + a prior reviewer's
   opinion — the fixing agent must still make its own both-sides read before touching code.
2. The **binding workflow** for landing fixes.

Previous rounds are closed and live in git history: the 2026-08-15 baseline → v2 → v3 → errata
(66 resolved findings, PRs #1319–#1383) and candidates A–E from 2026-08-24 (all dispositioned,
PRs #1388–#1392). Do not treat their descriptions as open.

## Status (2026-09-06 — all five findings below are RESOLVED)

An independent review of all five findings (own both-sides reads, own probes on both harnesses
+ an independent C# span probe) confirmed the fixes landed by PRs #1393–#1397 and closed the
remaining Finding 1 work in PR #1407. Per-finding resolution:

- **Finding 1 — RESOLVED** (#1395 tiers 1+2 partial + **#1407** completion): the backtick
  diagnostics are now node-level (the C# parser replaces the token and moves the rescan error
  + the gate onto the node, `LanguageParser.InterpolatedString.cs:56-60` — the harness reports
  them once), the FiveM (HashLiteral) backtick routes through the short-string scanner
  (`Lexer.cs:622-625`), and the tree pass sorts in the C# tree-walk attachment order.
  Boundary residue (documented in the `parserdiagnostics.rs` header + the new tests): the
  parser-recovery `LUA1015/LUA1010/LUA1012` family and the statement-position ×2-vs-×1
  token-survival case are gated on full_moon parse failures.
- **Finding 2 — RESOLVED** (#1396): the `LUA1008` if-expression gate matches the C# on all
  presets for canonical Luau if-expressions (probed). NOTE: the probe input in the finding
  below (`local x = if true then 1 else 2 end`) is **not** a Luau if-expression — the C#
  `ParseIfExpression` (`LanguageParser.cs:1271-1332`) consumes **no `end`**; the canonical
  form matches. The general parser-recovery diagnostics remain unported and are now explicitly
  listed in the `parserdiagnostics.rs` header (the documented boundary).
- **Finding 3 — RESOLVED** (#1393): the header expressions resolve in the enclosing scope —
  verified via the scope op on shadowing inputs (`for i = i, 10 do end` declares the Global
  `i` on both sides). The authority resolution below is correct: the generated
  `VisitNumericForStatement` the old comment cited is in `LuaSyntaxRewriter` (not inherited);
  the walker's override is what runs.
- **Finding 4 — RESOLVED** (#1394): the label span includes the trailing `;` and the disabled
  `goto` emits the `LUA0018` pair — verified by an independent C# span probe
  (`::label::;`→`[0,10)`, `::label:: ;`→`[0,11)`, `::label:: --c\n;`→`[0,15)`,
  `::label:: --[=[c]=]\n;`→`[0,21)`, `goto x;`→`[0,4)+[5,2)`, …).
- **Finding 5 — NOT DEFECTS** (#1397 fixed the sequential panic message; the tie-break and
  keyword-table items are notes only): the C# `FindVariable` enumeration order is
  implementation-defined (`HashSet`), and the real keyword consumer covers all 26. No further
  fixes are warranted.

The 2026-09-06 review also corrected two details: the baseline is **246 tests / 0 failed**
(not 234 — the fix PRs added 12 tests; see Baselines), and the `#1395` "tiers 1+2" claim
overstated the work (its tier-2 item (c) — the statement-position `LUA1012` — was not
implemented; it is part of the documented boundary residue).

## Before anything else

- Read all four governing docs IN FULL: `docs/AGENTS.md`, `docs/PLAN.md`,
  `docs/TRANSLATION.md`, `docs/COMMIT.md`. They define verbatim parity, the Port Boundary
  (full_moon is the lexer/parser/AST; `Parser/`, `Syntax/`, `Core/Portable` and GLua operators
  are DROP), hard prohibitions, gates, and PR workflow. Where they are stricter than this file,
  they win.
- Then independently re-audit for divergences NOT on the list below. A findings list is not a
  ceiling: you are responsible for your own full read of both trees, not just confirming these.

## Baselines (re-run yourself)

Verified 2026-09-06 on the current `main` (HEAD `726494f`, after #1407):

fmt clean · clippy `-D warnings` clean · check clean · **tests 250 passed / 0 failed** (the
246 baseline at HEAD `e021c92` plus the 4 pinning tests added by #1407; the earlier `234`
figure at `aedd440` was superseded — the fix PRs #1393–#1397 added 12 tests) ·
differential **identical 1870 / pending 0 / FAILED 0** · drift **744 nodes / 1335 edges /
topo 744**. The findings below were invisible to these baselines because the corpus contains
no backtick, if-expression, shadowed-for-loop, or goto/label inputs — they are now covered by
the pinning tests added in #1393–#1397/#1407 and the probes recorded in the resolution notes
above.

## Findings (all resolved — see Status)

### Finding 1 — Backtick-string diagnostics are largely unported (DS F1) — RESOLVED (#1395 + #1407)

**Port:** `loretta/src/errors/lexerdiagnostics.rs` `scan_backtick_string` (`:568-624`) +
dispatch at `:1213`.
**C# anchors:** `Lexer.ShortString.cs:54-73` (`ScanInterpolatedStringLiteral`),
`:306-581` (`InterpolatedStringScanner`), `LanguageParser.cs:198`.

The port's scanner walks to the closing backtick/newline and emits only
`ERR_UnfinishedString` when unfinished, plus the `LUA0036` gating on the finished path under
`BacktickStringType::None` (double-reported — see (d)). It omits:

- **(a)** escape diagnostics inside the contents — C# runs `ScanEscapeSequence` per `\`
  (`Lexer.ShortString.cs:426`);
- **(b)** hole diagnostics — `LUA0034 ERR_UnclosedExpressionHole`,
  `LUA0035 ERR_DoubleBraceInInterpolation`, and `ERR_SyntaxError` with the expected-char
  argument in mismatched-delimiter holes;
- **(c)** the parser's `LUA1012 ERR_InvalidStatement` when a backtick token survives into a
  statement position;
- **(d)** for `BacktickStringType::None` presets the unfinished path skips the `LUA0036`
  gating error: C# emits LUA0003 **and then** the gate unconditionally
  (`Lexer.ShortString.cs:70-72`), so an unfinished backtick yields LUA0003+LUA0036 (probed);
  the port `return`s early on unfinished and never emits it there. Note the dispatch subtlety:
  under a `HashLiteral` preset (FiveM) C# routes backticks through the short-string/hash
  scanner (`Lexer.cs:622-625`) and never reaches the interpolated gate — probed: FiveM emits no
  LUA0036 on either side; do not add it for HashLiteral presets. Additional count detail
  (probed): on a FINISHED backtick under a None preset, C#'s net output is ONE LUA0036 — the
  parser re-attaches it to the node (`LanguageParser.InterpolatedString.cs:60`), superseding
  the lexer's token copy, and the harness op reports it once. The port emits it from the lexer
  pass, so the harness op doubles it (2×LUA0036 vs C#'s 1×). The fix should move/deduplicate
  the emission to match C#'s single report.

**Probed live on HEAD (both harnesses):** `` `abc `` @Lua51 → C# `[LUA0003, LUA0036, LUA1012]×2`,
Rust `[LUA0003]×2`. Diverges on every preset and every backtick input. Error codes already
exist in `errorcode.rs` (`ErrUnclosedExpressionHole = 34`, `ErrDoubleBraceInInterpolation = 35`,
`ErrInterpolatedStringMustStartWithBacktickCharacter`); the resx strings are ported.

### Finding 2 — If-expression gating and general parser-error diagnostics unported (DS F2) — RESOLVED (#1396); remainder documented

**Port:** `loretta/src/errors/parserdiagnostics.rs` — no `Expression::If` arm; the op gates
parser diagnostics on `full_moon::parse` succeeding (`differential/src/ops.rs:45-47`).
**C# anchor:** `LanguageParser.cs:1329-1330`
(`ERR_IfExpressionsNotSupportedInLuaVersion`, code exists in Rust as
`ErrIfExpressionsNotSupportedInLuaVersion = 1008` but is never emitted).

None of the general parser diagnostics (`LUA1012/1010/1011/1001/1014/1015/1017/1018`,
`LUA0019 ERR_CannotBeAssignedTo`, `LUA0015`) are ported either — the file's own header says
"Starts with the version-gated statement rules the differential corpus exercises."

**Probed live on HEAD (both harnesses):** `local x = if true then 1 else 2 end` @Lua51 →
C# `[LUA1008, LUA1012, LUA1012]`, Rust `[]`. Diverges on every preset: those without
`AcceptIfExpressions` add LUA1008 (Lua51 etc.), and those with it (Luau, All) still diverge
because the port never emits the LUA1012s either (probed: Luau/All C# `[LUA1012, LUA1012]` vs
Rust `[]`).

### Finding 3 — For-loop header expressions resolve through the loop's own block scope (DS F3) — RESOLVED (#1393)

**Port:** `loretta/src/script/scopeandvariablemanager/scopeandvariablewalker.rs:298-357`
(numeric + generic): creates the block scope and the iteration variables BEFORE visiting the
header expressions.
**C# anchors:** `ScopeAndVariableManager.ScopeAndVariableWalker.cs:182-203` (numeric),
`:205-227` (generic) — header expressions are visited FIRST, in the enclosing scope; the block
scope + iteration variables come after.

**Authority resolution (decided this pass):** the walker chain is
`ScopeAndVariableWalker → BaseWalker : LuaSyntaxWalker : LuaSyntaxVisitor`. The generated
method the port's comment cites (`Syntax.xml.Internal.g.cs:9989`) lives inside
**`LuaSyntaxRewriter`** — a class this walker never inherits. The visitor-side generated
`VisitNumericForStatement` (`Syntax.xml.Main.g.cs:380`) is `=> DefaultVisit(node)`. The C#
override therefore always wins; the port's in-code justification (and its pinning test
`for_loop_iteration_variables_are_recorded_before_the_header_expressions`) enshrine the wrong
order.

**Observable in the differential `scope` op itself** (contrary to DS's "not observable" note —
the corpus simply lacks a shadowing input): `for i = i, 10 do end` @Lua51 → C# declares a
**Global `i`** (header resolved against enclosing scope), Rust declares none (header bound to
the iteration variable). Also diverges: header identifiers' referencing-scope records and node-id
allocation order.

### Finding 4 — Goto/label LUA1019 span edges (OX §3.2) — RESOLVED (#1394, both sub-items)

**(a) Label span excludes a trailing `;`.**
C# `ParseGotoLabelStatement` (`LanguageParser.cs:631-648`) builds the node including
`TryMatchSemicolon()` and gates on the whole node. Probed live: `::label::;` @Luau →
C# LUA1019 span `[0..10]` = `'::label::;'`; the port's `Stmt::Label` arm
(`parserdiagnostics.rs:142-156`) ends at `right_colons()`. Same applies to any trailing trivia
(`::label:: ;` @Luau → C# span `[0..11]`). Invisible to oracles: the corpus has no `::` input.

**(b) The port's `Stmt::Goto` LUA1019 arm is reachable where C#'s is dead.**
In C#, `goto x;` under a goto-disabled preset can never reach `ParseGotoStatement`'s gate:
`SyntaxFacts.HasKeywordBeenDisabled` (`SyntaxFacts.cs:52`) demotes the keyword to an identifier
whenever `AcceptGoto` is false, so C# yields `LUA0018 ×2` (probed live @Lua51 and @Luau). C#'s
goto-statement LUA1019 arm requires simultaneously `AcceptGoto == true` (keyword enabled) and
`false` (gate) — unreachable upstream. In the port, full_moon still parses `goto x` as a
`Stmt::Goto` where its version mapping enables goto syntax (e.g. Lua51/Luau), so the arm fires
LUA1019 there where C# fires `LUA0018 ×2` (probed: `goto x;` @Lua51 → C# `[LUA0018, LUA0018]`,
Rust `[LUA1019]`; on LuaJIT20 both sides are clean — no divergence).

### Finding 5 — Minor documented-in-code divergences worth recording (DS minor notes) — NOTES ONLY (no defects; #1397 fixed the message)

These match their in-code documentation; record only so the fixing agent doesn't "fix" them
blindly:

- **FindVariable tie-break**: declaration-order vs C# `HashSet` enumeration order
  (implementation-defined upstream; documented Finding 63).
- **Sequential naming-strategy panic message**: `trim_start_matches(prefix)` strips ALL leading
  prefix chars; C# `name.Remove(0, prefixes)` removes exactly the counted run. Differs only when
  the generated name itself begins with the prefix character.
- **Keyword-kind test coverage**: the test table has 22 keywords vs the C# data source's 26
  (continue/type/typeof/export cannot dock on `Symbol::from_str`); the real consumer
  `namingstrategies::is_keyword` covers all 26.

Also verified NOT defects: OX's continue-position probe — the upstream test annotation
`(9,14)` is off-by-one vs actual C# output `(9,13)`; the port asserts `(9,13)` and matches
real behavior (re-probed this pass by reconstructing the exact raw-literal input). An upstream
issue, not ours.

## Prior reviewer's opinion per finding (input, not authority)

Formed during the 2026-08-25 verification pass. Your own both-sides read wins; say so either way.

- **Finding 1 — fix, in two tiers.**
  Tier 1 (mechanical, do first): emit `LUA0036` on the unfinished path too (matching
  `Lexer.ShortString.cs:70-72`'s unconditional post-scan gate under non-HashLiteral presets),
  and fix the finished-path double-report: the port's lexer-level emission makes the harness op
  report LUA0036 twice where C# reports it once (the parser node copy supersedes the token
  copy) — emit once from one pass only. Tier 2: extend the scanner over the string contents —
  escape scanning via the existing escape-diagnostics path (a), `{`/`}` hole tracking emitting
  LUA0034/LUA0035/ERR_SyntaxError with C# spans (b), and the parser-side LUA1012 when a
  backtick reaches statement position (c). Note the boundary: full_moon parses a backtick as a
  valid token, so (a)/(b) live in our text-scanning layer (same layer as the rest of
  lexerdiagnostics.rs) and (c) in parserdiagnostics.rs over the recovered AST. Suggested
  probes: `` `abc `` , `` `a{ }b `` , `` `{{ `` , `` local x = `abc` `` × each preset, pinned as
  tests + one corpus feature file (`features/backtick.lua`) so the differential covers it going
  forward — adding corpus inputs requires regenerating expected outputs from the C# oracle
  (never hand-edit them).

- **Finding 2 — fix incrementally, gated on what full_moon can express.**
  Add the `Expression::If` LUA1008 arm first (trivial, mirrors LanguageParser.cs:1329-1330).
  Then add parser-diagnostics for inputs full_moon recovers structurally (e.g. LUA1012 on
  malformed statement heads). General grammar errors (LUA1001/1010/1011/…) require detecting
  malformed constructs full_moon may silently accept or drop — implement only those provable to
  diverge via probe, and document the remainder as boundary-forced in the file header (replace
  the "Starts with…" phrasing with an explicit list of what is and isn't ported and why). Keep
  the `full_moon::parse` failure gate but document that parse-failed sources lose parser
  diagnostics entirely (C# recovers a tree and keeps going).

- **Finding 3 — fix; also repair the documentation and the test.**
  Reorder both arms to visit header expressions before creating the block scope/iteration
  variables, matching ScopeAndVariableWalker.cs:182-227. Delete or rewrite the incorrect
  Syntax.xml.Internal.g.cs citation (it names LuaSyntaxRewriter's order, not this walker's), and
  replace the pinning test with one asserting the C# semantics: `for i = i, 10 do end` declares
  a global `i` for the header reference while the body sees the iteration variable. Watch node-id
  allocation order (the original Finding-41 concern) — after reorder, ids allocate
  header-expressions-first like C#. Verify no other differential scope outputs shift (run the
  differential; expected: zero delta since the corpus has no such input, which is why you must
  add `for i = i, 10 do end` as a scope-op fixture/test rather than rely on the corpus).

- **Finding 4 — fix both sub-items together.**
  (a) Extend the `Stmt::Label` span end past an immediately-following `;` (and decide: C#'s span
  covers trailing trivia too because the node includes the semicolon token — replicate at least
  the semicolon; document trivia inclusion explicitly). (b) For `Stmt::Goto` under
  goto-disabled presets, emit what C# observably emits: the keyword is not a keyword there, so
  C# produces `LUA0018` on `goto` (as a bare identifier statement) and again on `x` if it forms
  a second expression statement — replicate the observable pair rather than keeping the dead
  LUA1019 arm, OR keep the arm but document that it models C#'s dead path and never fires where
  C#'s wouldn't. Recommended: replicate observable behavior (LUA0018-based), since Requirement 1
  is behavioral parity. Add fixtures (`::label::;` under Luau, `goto x;` under Lua51) to tests;
  corpus addition optional but welcome via regenerated expecteds.

- **Finding 5 — mostly leave as-is.**
  Tie-break: keep (upstream is implementation-defined). Sequential message: optionally align by
  counting prefix runs instead of trim_start_matches (one-line change; low value). Keyword-kind
  table: leave; consumer covers the full set. Do not chase the upstream (9,14) annotation.

## Coverage notes carried forward (line-read these anchors yourself)

- `Core/Portable/Compilation/ParseOptions.cs` and `Core/Portable/RealParser.cs` were
  probe-checked, not line-read, in earlier rounds; the audits cite them only indirectly. Read
  before trusting any anchor into them.
- Both audits could not exhaustively verify `format_double_r` vs .NET "R" over the whole f64
  domain, or `unicode_categories.rs` vs .NET CharUnicodeInfo beyond structural + spot checks
  (^=Sk confirmed independently this pass; table verified 4108 arms, values 0..29, full scalar
  coverage, single gap U+005F which is short-circuited by needs_escaping's ASCII list). If you
  touch either, regenerate/re-verify rather than citing this file.
- OX noted the packaged reference (0.2.14-nightly.26) vs repo commit `b767b4e` equivalence was
  assumed in its probes; the probes reproduced here used the same package. Fine for parity
  probing; flag if you find evidence of divergence.

## Binding workflow for landing fixes

All five findings above are resolved (see Status) — no further work is owed for them. This
workflow remains binding for any future finding:

- One finding per PR; inside it, small gate-green commits. Never mix findings. Never land red at
  any commit. Suggested order: Finding 3 (wrong docs + wrong test enshrined — smallest blast
  radius, highest correctness value) → Finding 4 → Finding 1 tier 1 → Finding 2 (If arm) →
  Finding 1 tier 2 → Finding 2 remainder → Finding 5 sweep.
- Gates from `loretta-rs/`: fmt, clippy `-D warnings`, check/test
  `--workspace --all-features --locked`, drift, differential byte-exact. Baselines above; recount
  yourself.
- Never push `main`: `gh pr create` → `gh pr checks --watch` → squash-merge with
  `--delete-branch`. Revert and re-queue on red.
- Protected: `references/**`, `corpus/**` (incl. expected outputs — regenerate from the C#
  oracle only), `docs/**` — never edit; spec changes go through a separate `spec:` PR. Only
  writable markdown: `loretta-rs/PROGRESS.md` (update touched rows in the same PR).
- Probes live in `.scratch/` (gitignored); never committed.
- Oracles decide correctness; compiling is not correct. If a delta appears, investigate — never
  edit reference outputs to match Rust.
