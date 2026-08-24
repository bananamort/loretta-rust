# Audit — Loretta-RS Port (v3, 2026-08-23 · C# reference @ `b767b4e`)

This audit **supersedes** both the 2026-08-15 audit and the same-day v2 pass circulated earlier
on 2026-08-23. After v2 was challenged, **every remaining finding was re-verified by direct
reads of both sides** (C# `references/Loretta` @ `b767b4e`, Rust working tree, vendored
`references/full-moon` where relevant), cross-checked by an independent implementing agent, and
— critically — checked against the governing specs (`docs/AGENTS.md`, `docs/PLAN.md`,
`docs/TRANSLATION.md`, `docs/COMMIT.md`). Several v2 items were **withdrawn** as wrong or as
spec-sanctioned behavior; several others carry corrected citations/directions. The v2 numbering
is retired; the mapping is in the appendix. Do not reuse v2 numbers.

## Checks as run (2026-08-23)

| Check | Result |
|---|---|
| `cargo fmt --check --all` | clean |
| `cargo clippy --all-targets -- -D warnings` | clean |
| `cargo check --all-targets` | clean |
| `cargo test` | 181 passed, 0 failed (= the 172 Oracle-1 `#[test]`s + ~9 internal unit tests across binaries) |
| `differential check corpus/expected` | identical: 1870 · pending: 0 · FAILED: 0 |

## Errata (independent verification pass, 2026-08-23)

Four findings were re-checked against both sides by direct reads and corrected:

- **Finding 16** — RESOLVED as a non-issue: the C# `AddReferencedVariable` early-returns
  identically when the variable is declared in the scope (`IScope.cs:203-209`), and
  `GetOrCreateVariable` also adds only locally. The Rust matches; the remaining delta in that
  area is Finding 13 (Vec vs ISet dedupe).
- **Finding 40** — arithmetic corrected: the C# start value is
  `(getMaxDigits + MaxPrefixCount − digitCount) − minPrefixCount` and is always ≥ 4 with the
  real constants, so the draft's "throws immediately at ≤0 / `getMaxDigits ≤1` ⇒ zero
  attempts" case cannot occur.
- **Finding 57** — mechanism corrected: the `default_visit_*` helpers re-dispatch through
  `full_moon::visitors::Visitor::visit_stmt(self, …)` back into the overrides — trait-qualified
  calls dispatch to the concrete type's override in Rust (verified by a minimal probe:
  `visit_stmt → default_visit_stmt → visit_stmt → …` recurses past 1000 frames), so the
  collector infinitely recurses if ever driven; full_moon's trait defaults are empty no-ops
  regardless, so it neither terminates nor descends. The v3 errata's "no recursion" wording is
  withdrawn; the defect stands as dead, non-functional scaffolding.
- **Finding 63** — the missing `!=` row is boundary-forced (GLua-only syntax, GMod DROP, no
  full_moon Symbol); only the `SYMBOL_ROWS` header comment's drop enumeration is incomplete.

## Artifact counts (derived by direct inspection)

`nodes.json` 744 · `topo.json` 744 · `edges.json` 1335 · `test-nodes.json` 319 ·
`test-nodes-topo.json` 319 · `test-nodes-edges.json` 532 · `test-missing.json` 0 ·
`PROGRESS.md` rows 1–1063 all `done` · `corpus/expected` 1870 `.json` files in 176 folders + root
(= the differential's identical count exactly).

## Spec conformance notes (things that are NOT defects)

These were flagged in earlier passes and are **wrong**: all are fixed by the governing specs.

- `Portable/LuaExtensions.cs` (tree-typed helpers) — **DROP**, documented (`AGENTS.md:65`,
  `PLAN.md:141`). Nothing missing.
- `SyntaxNormalizer` + its 1902-line test file — inside `Portable/Syntax/` = **DROP** (Port
  Boundary). Never reimplement. Residual issue is only the misleading leftover Rust stub test;
  its removal is tracked in Finding 57.
- `loretta/src/options.rs` being a placeholder while the ADAPT cluster landed as one-file-per-node
  (`luasyntaxoptions.rs`, …) — `options.rs` is the Port Boundary's *designated* destination and
  The Method §2 sanctions per-node files. Process choice, not a defect.
- `experimental/syntaxextensions.rs` placeholder — documented intentional drop of the obsolete
  `[Obsolete] FoldConstants` wrapper (Prohibition #2 satisfied).
- Missing GLua `&&`/`||`/`!=`/`!` and `//`,`/* */` support — documented DROP
  (`Locked Decision 2`, PLAN Dialect Support). Only cosmetic residue: local comment headers may
  enumerate the dropped set incompletely (see Finding 60).
- Type-parameter-default parse "inversions" in type tests — documented intentional drop of a
  Loretta extension beyond the Luau RFC (commit 1f15ebb).
- `StringUtils::trim` returning `""` on degenerate input instead of throwing — documented
  anti-panic adaptation; noted, not a defect.

## What is exact (verified)

ErrorCode variant-for-variant (57 codes, gaps at 2/29 preserved); all 66 resource strings
byte-identical (mechanical diff; resx's other 4 entries are designer blobs); severity/category/
prefix/warning-level behavior; all 12 `LuaSyntaxOptions` presets value-for-value (incl. GMod =
LuaJIT20.With(cCommentSyntax, cBooleanOperators, ContinueType::Keyword) — re-verified), `With()`
semantics, ctor validation, ToString preset names; ContinueType/IntegerFormats/BacktickStringType/
ScopeKind/VariableKind/BinaryOperatorKind(21)/UnaryOperatorKind(5); HexFloat bit math;
ObjectDisplay for BMP chars (double formatting empirically identical to .NET `ToString("R")`);
CharUtils tables incl. quirks; MinifyingUtils, both slot allocators, NamingStrategies character
sets; TriviaRewriter separator table arm-for-arm; scoping interface layer; ConstantFolder core
(flags, wrapping arithmetic, comparison/concat order, shift masking, upstream `TryGetInt32`
endpoint quirk); the 20×20 precedence table; all 25 lexical-error tests with exact
IDs/severities/positions/squiggles/args; `\u{}` scanner arm (HexDigitExpected / brace errors /
EscapeTooLarge arg / gating) matches C#. `loretta` depends only on `full_moon`.

## Why the green checks hide the findings below

The corpus (anim.lua, 13 feature files, rustic.lua ≈ 6 MB) contains **zero `...` varargs and
zero gotos**, and the harness has **no rename operation**; the differential `pending` bucket only
fires when expected output contains `"hasErrors": true`. None of Findings 1–24 is reachable by
the harness today.

---

## Findings

One bullet per issue. Grouped by area; numbering is authoritative for fix tracking.

### A. Crashes

1. Vararg parameters panic the whole scope/minify/rename pipeline: the `Ellipsis` match arm
   produces `"..."` then falls through to unconditional `unreachable!()` —
   `loretta/src/script/scopeandvariablemanager/scopeandvariablewalker.rs:143-158`; C#
   `CreateParameter` maps `VarArgParameter => "..."` (`ScopeAndVariableWalker.cs:71-82`).
2. Integer literals overflowing `i64/u64` panic the constant folder
   (`constantfolder.rs:704-711`) instead of C#'s fold-as-0 + `ERR_NumericLiteralTooLarge`
   (`Lexer.Numbers.cs:250-259,280-283,364-367`).
3. `%` by zero on Long×Long panics (`constantfolder.rs:874-881`, `wrapping_rem`) even on
   double-only presets where C# evaluates doubles (NaN → no-fold). Int64 presets crash-parity
   holds; Lua51/Lua52-class presets diverge.
4. Dead defensive arm, misfiled as reachable in v2: `set_first_leading`'s
   `InterpolatedString => unreachable!()` (`constantfolder.rs:1386-1391`) — the v2 repro
   `({k=\`x{1}\`}).k` does not reach it (verified by probe; C# does not fold it either).
   Action: prove reachability or remove/justify the arm; the in-code comment is currently false.

### B. Scoping / Script

5. Multi-tree state destroyed: `manager.rs:79-96` walks each tree into fresh maps then
   **overwrites** `variables`/`labels`/`scopes` (last tree wins) vs C#'s shared accumulating
   builders (`ScopeAndVariableManager.cs:35-47`). Parse failure additionally skips the whole
   tree silently (`:73-77`).
6. Forward gotos lose their jump: single-pass ordering means `goto top` binds to a placeholder
   label (`gotowalker.rs:44-47`), and the later `::top::` creates a **new** label
   (`gotolabelwalker.rs:43-45`) leaving the jump orphaned; C# runs GotoLabelWalker before
   GotoWalker (`ScopeAndVariableManager.cs:64-72`).
7. `label_syntax` always `None` (`gotolabelwalker.rs:43-45` → `GotoLabel::new(name, None)`)
   vs C# `CreateLabel(name, node)` (`GotoLabelWalker.cs:24`).
8. Captured variables permanently empty: C# `IFunctionScope.AddReferencedVariable` override
   missing; `Variable::add_capturing_scope` (`ivariable.rs:152`) has zero call sites.
9. A local-function statement's scope-map entry is overwritten with the enclosing scope: walker
   records `record_statement_scope(..., &self.scope())` before creating the function scope
   (`scopeandvariablewalker.rs:420-450`), then `manager.rs:92-95` merges `location_scopes`
   after walked scopes; C# stores the FunctionScope itself (`ScopeAndVariableWalker.cs:326`).
10. Luau interpolated strings skipped entirely by the scope walker — no `InterpolatedString`
    arm in `visit_expr_children` (grep-verified); identifiers inside `` `{}` `` get no
    registration/read-locations/rename coverage.
11. Renaming to non-ASCII names rejected unconditionally for every tree (`script.rs:188-197`)
    vs C# per-tree `UseLuaJitIdentifierRules` gate (`Script.cs:158-165`).
12. Tree attribution by substring search defaulting to tree 0 (`script.rs:139-143`) vs C#
    `location.SyntaxTree`.
13. `referenced_variables` is a `Vec` with unconditional pushes (`iscope.rs:184-186,224`) vs
    C# `ISet` dedupe (`IScope.cs:124`).
14. `State.Scopes` pollution: identifier/statement entries merged into the map
    (`manager.rs:92-95`) flip C#'s `GetScope(identifierNode) == null` into a scope result.
    (`find_scope` itself climbs correctly via containing scopes once a node is recorded,
    `script.rs:87-105` — v2's stronger claim withdrawn.)
15. Two distinct `Parameter` nodes inserted per named parameter (`create_parameter` inserts at
    `:147-151`; callers insert again at `:447,:467,:541,:606`); C# has exactly one per node.
16. RESOLVED — NOT A DEFECT (checked against the C#): `add_referenced_variable` early-returns
    when the variable is declared in this scope (`iscope.rs:216-228`) — the C#
    `AddReferencedVariable` does exactly the same (`if (_declaredVariables.Contains(variable))
    return;`, `IScope.cs:203-209`), and `GetOrCreateVariable` also adds only locally (the
    parent walk happens in the lookup, `TryGetVariable`). The Rust matches; the remaining delta
    in this area is Finding 13 (Vec vs ISet dedupe).

### C. Lexer-diagnostics scanner (vs the deleted C# lexer's observable diagnostics)

17. Decimal `ERR_NumericLiteralTooLarge` never reported — the decimal path never accumulates
    the value (`lexerdiagnostics.rs:866-901`) vs C#'s ulong/long/plain-int TryParse failures
    (`Lexer.Numbers.cs:248-251,256-259,280-283`).
18. Invented `ErrInvalidNumber` for digit-less hex (`:713-715`); C# hex parser has no such rule
    (binary/octal do).
19. Number classification treats any `e`/`E` as double (`number_is_double`, `:895-903`) → hex
    integers like `0xE5` mis-typed Double; C# types by lexer Value (Long,
    `Lexer.Numbers.cs:306-435`); visible in i64-exact comparisons beyond 2^53.
20. Hex `ull` past `u64::MAX` unchecked (`:724` excludes `is_unsigned_long`); C# reports TooLarge
    (`Lexer.Numbers.cs:364-367`).
21. Complex `i` suffix gets an integer > i64 test (`:716-726`); C# converts to double and
    reports only `DoubleOverflow` on real overflow (`Lexer.Numbers.cs:380-390`).
22. Bitwise gating inverted: per-character `&`/`|` errors whenever `accept_bitwise_operators`
    is false (`:1042-1053`) and never on `<<`; C# lexer errors only on `<<` (`Lexer.cs:501-507`)
    and the parser errors only for single `&`/`|` binary operators
    (`LanguageParser.cs:908-912`) — port that rule into `parserdiagnostics.rs` (precedent:
    LUA0018, PR #1311).
23. `\z` whitespace-skip set omits `\n`,`\r`: local helper matches only `' ' | '\t' | '\u{0B}' |
    '\u{0C}'` (`:983-985`, used `:389-395`); C# `CharUtils.IsWhitespace` = `' ' | '\t'..'\r'`.
24. Prefix underscores emit once; C# emits twice for prefixed literals — dispatch-time check
    (`Lexer.cs:562,573,584`) plus in-parser `IndexOf('_')`
    (`Lexer.Numbers.cs:243-244,359-360`).
25. Shebang guard semantics differ: C# re-initializes `onlyShebangsAndNewLines=true` per trivia
    run and clears on `\v`/`\f` (`Lexer.cs:727,741-750`); Rust keeps a scan-global flag
    re-armed by newlines (`:1026`) and cleared by many constructs but not `\v`/`\f`.
26. The >200-char bad-token runaway absorption (`Lexer.cs:703-711`) is absent.

### D. ConstantFolder

27. Unary folds take trivia from the operator token — `visit_expression` captures
    `first_leading(&node)` for the whole unary node (`:568-570`) feeding `visit_unary`
    (`:235-278`); C# passes the **operand** as trivia container (`ConstantFolder.cs:25-46`).
28. String-number extraction is fully anchored (`is_dec_float` must consume everything,
    `:991-1041`); C#'s decFloat regex is unanchored (`NumberParsing.cs:16-18`) and RealParser
    ignores trailing garbage → `"v1.5"`, `"1.5x"` extract in C#, nothing in Rust.
29. Signed strings: `"-1.5"` parses as −1.5 (`parse_double_literal` → `f64::parse`,
    `:916-923`); C#'s RealParser rejects signs → NoDigits → returns **true with 0.0**
    (`RealParser.cs:15-17,384-392`).
30. Overflow strings: `"1e400"` yields `Ok(inf)` → invalid `-Infinity` output; C# returns false
    on Overflow and leaves the expression untouched (`RealParser.cs:376-380`).
31. Hex-float strings like `"0x1.8p10"` yield **0.0** in C# (unanchored decFloat matches the
    leading `"0"` first, TryParseDouble stops at `x` → NoDigits → 0.0-true; outcome pinned by
    runtime probe) — Rust routes to HexFloat and returns the real value. Replicate C# ordering.
32. `#` length counts UTF-8 bytes (`get_string_value(...).len()`, `:269`) vs C# UTF-16 units
    (`ConstantFolder.cs:43`): `#"é"` → 2 vs 1; `#"😀"` → 4 vs 2.
33. `\ddd` >255 pushes U+FFFD (`:1201-1203`) vs C# error + skip character entirely
    (`ShortString.cs:223-226`).
34. Astral `\u{…}` emits one char and lone surrogates are dropped (`:1169-1185`) vs C#
    UTF-16 surrogate pair / raw code unit (`ShortString.cs:285-297`).
35. Folder `\z` skip uses `is_ascii_whitespace()` excluding `\v` (`:1141-1151`); C# includes it.
36. Escape processing is option-unplumbed: ungated processing plus invalid escapes always echo
    the character (`:1205`) vs C#'s preset-dependent echo/skip+error (`ShortString.cs:199-205`).
37. Chained member/element access on const tables never folds — lookup gated on
    `suffixes.len() == 1` (`:522-556`) vs C# immediate-base bottom-up folding
    (`ConstantFolder.cs:336-407`).
38. Concat strips parentheses via `get_inner_expression` (`:365-386`) where C# checks the direct
    Kind and throws (`ConstantFolder.cs:122-135`) — crash/success asymmetry.

### E. Minifying

39. Slot release checks self-identity only (`renametable.rs:156-161`) vs C# release when the
    visited node equals **or descends from** the last-use node
    (`RenamingRewriter.RenameTable.cs:78-80`, `AncestorsAndSelf().Any(...)`); the in-code
    comment claiming equivalence is false. Changes minified output with the default allocator.
40. Alphabetic strategy tries ascending `min_prefix_count..=5` (`namingstrategies.rs:79-86`);
    C# tries DESCENDING from `firstNameChar − minPrefixCount` (where
    `firstNameChar = (getMaxDigits(slot) + MaxPrefixCount) − digitCount`,
    `NamingStrategies.cs:20-34`) to 1, naming the suffix slices `fullName[prefixes..]` — i.e.
    prefix counts 0..(firstNameChar−1) tried in descending order before throwing. With the
    real constants the starting value is always ≥ 4, so the C# always makes attempts before
    throwing (the v3 draft's "throws immediately at ≤0 / `getMaxDigits ≤1` ⇒ zero attempts"
    case cannot occur).
41. Numeric/generic-for loop-variable records created AFTER header expressions
    (`scopeandvariablewalker.rs:287-341`) vs C#'s generated visitor visiting identifiers first
    (`Syntax.xml.Internal.g.cs:9991-9995`) — different (valid) allocation orders.

### F. Options / infra / display / utilities

42. `LuaSyntaxOptions` derives Eq/Hash over ALL fields (`luasyntaxoptions.rs:12`); C#'s
    hand-written Equals/GetHashCode deliberately omit `AcceptUnicodeEscape` +
    `AcceptInvalidEscapes` (`LuaSyntaxOptions.cs:660-721`).
43. Scanner emits `ErrInvalidStringEscape` for `\x` + non-hex digit unconditionally
    (`:441-448`); under `AcceptInvalidEscapes && !AcceptHexEscapesInStrings` C# silently echoes
    (goto-default before digit parsing, `ShortString.cs:166-171` + `:199-205`). Gate the
    emission on `!(accept_invalid_escapes && !accept_hex_escapes_in_strings)`; keep both-errors
    behavior when `!accept_invalid_escapes`. (Corrected direction from v2 #26.)
44. `rename_variable("")` panics (`script.rs:128-130`); C# throws ArgumentNullException/
    ArgumentException through `Script.cs:143-144` + location-handling `FindVariable`.
45. `errorfacts::is_hidden` true for Void/Unknown (`errorfacts.rs:33-35`) vs generated
    all-false (`ErrorFacts.g.cs:34-41`); benign today, align anyway.
46. `parserdiagnostics.rs:4` cites nonexistent `Syntax/LuaParser.cs` (actual emitters:
    `Parser/LanguageParser.cs`).
64. **RESTORED** (lost in v3 renumbering; numbered out of sequence to keep all references
    stable): `LuaParseOptions::validate_options` has an empty
    body (`luaparseoptions.rs:68`) where C# delegates to base
    `ParseOptions.ValidateOptions`, which emits `ERR_BadDocumentationMode` for an invalid
    documentation mode (`Core/Portable/Compilation/ParseOptions.cs:47-55`). Restore the
    validation path.
65. **RESTORED** (lost in v3 renumbering): `features` stored as `Vec<(String, String)>`
    (`luaparseoptions.rs:13`) loses C#'s case-insensitive dictionary semantics
    (`ImmutableDictionary<string,string>` with OrdinalIgnoreCase keys,
    `LuaParseOptions.cs:57`), and C# equality/hash include Features
    (`LuaParseOptions.cs:107-115`). Restore lookup/equality semantics.

### G. Display / utilities

47. `'^'` (U+005E) stored as Math Symbol 25 (`unicode_categories.rs:42`); Unicode/.NET say
    Modifier Symbol 27 → escaping diverges under `EscapeNonPrintableCharacters`.
48. `encode_char_to_utf8` handles only 1–3-byte forms; ≥U+10000 falls into the 3-byte branch
    and `(n >> 12) & 0xF` drops bits 16–20 (`charutils.rs:136-142`) — corrupt bytes for astral
    chars (input class unreachable in C# `char`, but reachable here).
49. ObjectDisplay escapes astral chars as one combined `\u{…}` (`objectdisplay.rs:63-69`) vs
    C# surrogate halves (`\u{D83D}\u{DE00}`).
50. `StringUtils::is_identifier` lacks the `IsNullOrWhiteSpace` guard variant
    (`stringutils.rs:11-28` vs `StringUtils.cs:41-45`) — names starting with ≥U+007F
    whitespace pass in Rust, fail in C#.

### H. Tests

51. Empty stub test inflating pass counts:
    `lexer_lexes_number_with_leading_underscores_before_prefix`
    (`lexical_regression_tests.rs:232-238`) is comment-only with zero assertions (3 C# cases
    dropped). Either restore equivalents within full_moon's grammar or remove the test and note
    the drop.
52. Long-string data rows use REAL control characters instead of C#'s literal `\n`/`\r\n`/
    `\r`/`\xFF` sequences (`lexicaltestdata.rs:567-573` vs `LexicalTestData.cs:297-306`) —
    silent data drift from the C# oracle inputs.
53. Verbatim duplicated assertion block (`A` token asserted twice):
    `lexical_tests.rs:402-409` ≡ `:424-430` — delete the repetition.
54. TokenCache regression weakened: exact code/squiggle/location
    (`ERR_NonFunctionCallBeingUsedAsStatement` @(9,14)) reduced to "lua51 errors non-empty /
    luau clean" (`lexical_regression_tests.rs:206-229`). The continue rule IS ported
    (parserdiagnostics, LUA0018) — assert the exact diagnostic again.
55. Diagnostic assertions downgraded group-wide (substring/"non-empty" instead of exact
    codes+squiggles+locations): all 7 TypeParsingErrorTests, Parsing RegressionTests #6/#8,
    LocalVariableAttributeTests #1/#2 — restore exact assertions against the ported scanners.
56. Typed-Lua LUA1016 and Luau-goto LUA1019 gating inversions — tests expect success where C#
    expects gating errors (`type_parsing_error_tests.rs:106-121`;
    `parsing_regression_tests.rs:243-258`). Errors are PORT scope (PLAN §3); implement the
    gates (LUA0018 precedent), then restore C# expectations.
57. Dead/misleading test infrastructure: `tests/src/parsingtestsbase.rs` is imported by nothing
    and its `default_visit_*` helpers re-dispatch through
    `full_moon::visitors::Visitor::visit_stmt(self, …)` back into the overrides — in Rust,
    trait-qualified calls dispatch to the concrete type's override, so the chain
    `visit_stmt → default_visit_stmt → visit_stmt → …` is infinite recursion if ever driven
    (verified by probe) — and full_moon's trait defaults are empty no-ops regardless, so the
    collector neither terminates nor descends; it is dead, non-functional scaffolding
    (`:196-204`); `tests/src/syntaxextensions.rs` likewise unused — delete or repair-and-wire.
    Also delete
    the misleading `tests/syntax_normalizer_tests.rs` stub: the normalizer is DROP surface
    (Port Boundary), so removal plus a `PROGRESS.md` note is the only honest disposition.
58. Assertion dimensions dropped: `ContextualKind` tuples unchecked in two lexical regression
    tests; goto-identifier data rows over-skipped even where the lua51 mapping would lex them
    exactly as C#.
59. `lexer_covers_all_tokens` checks a hardcoded enabled-symbol vocabulary
    (`lexical_tests.rs:198-245`) vs C#'s enum-wide sweep — strictly weaker guarantee.
60. Interpolated-string tests assert RAW segment literals where C# asserts DECODED values, plus
    a dead `let _ = TokenValue::String(String::new());` keeping an import alive
    (`interpolated_string_tests.rs:61-69,84-90`).
61. Scoping-test weakenings: FindScope test 2 docks at the file node instead of an inner
    expression node (`scope_find_scope_tests.rs:58-70`); RenameVariable test 1 ignores the
    `tree_without_support` payload although the enum carries it
    (`scope_rename_variable_tests.rs:52-64`).
62. Redundant extra test with no C# counterpart: `parser_parses_typed_lua_structures`
    re-sweeps all 48 CASES inputs already covered individually
    (`type_parsing_tests.rs:90-113`). Harmless; trim or document.

### I. Cosmetics

63. Stale/false comments: nearly every `mod.rs` says "Pending port …"; `constantfolder.rs`
    interp-comment claim (Finding 4); `renametable.rs` ancestor-equivalence claim (Finding 39);
    `SYMBOL_ROWS` header omits `!=` from the documented GLua drop list
    (`lexicaltestdata.rs:125-129`) — the missing `!=` row itself is boundary-forced (it is
    GLua-only syntax, GMod is DROP, and full_moon has no `!`/`!=` Symbol), so only the header
    comment's drop enumeration is incomplete; ShortToken Display labels every symbol generically
    `"SymbolToken"` (failure-message cosmetics only); find_variable deterministic tie-break vs
    C# arbitrary HashSet order (note only).
66. Minor numeric note: folder exponentiation uses `f64::powf` where C# uses `Math.Pow` —
    last-ulp differences are possible on some inputs (corpus-visible cases agree; numbered out
    of sequence to preserve references). Align or document during the folder fixes.

---

## Appendix — v2 → v3 mapping (traceability)

| v2 # | Disposition | Where it went |
|---|---|---|
| 1 | kept | Finding 1 |
| 2 | kept | Finding 2 |
| 3 | corrected/downgraded | Finding 4 (dead-arm cleanup, repro invalid) |
| 4 | kept | Finding 3 |
| 5 | kept | Finding 5 |
| 6 | kept | Finding 5 (second half: the parse-failure skip) |
| 7 | kept | Finding 8 |
| 8 | kept | Finding 6 |
| 9 | kept | Finding 7 |
| 10 | kept | Finding 9 |
| 11 | kept | Finding 10 |
| 12 | kept | Finding 11 |
| 13 | kept | Finding 12 |
| 14 | kept | Finding 13 |
| 15 | kept | Finding 14 (reworded: climb happens once recorded) |
| 16 | kept | Finding 15 |
| 17 | kept (citation refined) | Finding 44 (throws from `FindVariable` during location handling) |
| 18 | kept | Finding 17 |
| 19 | kept | Finding 18 |
| 20 | kept | Finding 19 |
| 21 | kept | Finding 20 |
| 22 | kept | Finding 21 |
| 23 | kept | Finding 22 |
| 24 | **WITHDRAWN — INVALID** | C# squiggles the char BEFORE the newline in both stop modes; Rust already correct; pinned by C# test expecting `e` @(1,18) |
| 25 | kept | Finding 23 |
| 26 | **CORRECTED — direction reversed** | Finding 43 (Rust over-emits where C# is silent) |
| 27 | kept | Finding 24 |
| 28 | kept | Finding 25 |
| 29 | kept | Finding 26 |
| 30 | kept | Finding 27 |
| 31 | kept (extended) | Finding 28, with the hex-string `"0x1.8p10"` → 0.0 case split out as Finding 31 |
| 32 | kept | Finding 29 |
| 33 | kept | Finding 30 |
| 34 | kept | Finding 32 |
| 35 | kept | Finding 33 |
| 36 | kept | Finding 34 |
| 37 | kept | Finding 35 |
| 38 | kept | Finding 36 |
| 39 | kept | Finding 37 |
| 40 | kept | Finding 38 (refined: descending, unbounded, throws at ≤0) |
| 41 | kept (minor) | Finding 66 |
| 39 | kept | Finding 37 |
| 40 | kept | Finding 38 (refined: descending, unbounded, throws at ≤0) |
| 41 | kept | Finding 41 |
| 42 | kept | Finding 39 |
| 43 | kept | Finding 40 |
| 44 | kept | Finding 41 |
| 45 | kept | Finding 42 |
| 46 | **RESTORED** | Finding 64 (lost in the v3 renumbering) |
| 47 | **RESTORED** | Finding 65 (lost in the v3 renumbering) |
| 48 | kept (label fixed — never withdrawn) | Finding 45 |
| 49 | kept | Finding 46 |
| 50 | **WITHDRAWN** | `options.rs` is the designated ADAPT destination; process choice |
| 51 | **WITHDRAWN** | `Portable/LuaExtensions.cs` is documented DROP |
| 52 | **WITHDRAWN** | documented intentional drop (Prohibition #2 satisfied) |
| 53 | kept | Finding 56 |
| 54 | kept | Finding 57 |
| 55 | kept | Finding 58 |
| 56 | kept | Finding 59 |
| 57 | **RECLASSIFIED** | documented anti-panic adaptation; note only |
| 58 | rewritten | normalizer is DROP (Port Boundary); residual stub-test removal folded into Findings 51/57 |
| 59 | kept | Finding 45 |
| 60 | downgraded | Finding 63 (comment omission only) |
| 61 | kept (bytes drift) | Finding 52 |
| 62 | kept | Finding 53 |
| 63 | kept | Finding 54 |
| 64 | kept | Finding 55 |
| 65 | split | generic-default inversions withdrawn (documented drop, 1f15ebb); LUA1016/LUA1019 halves kept → Finding 56 |
| 66 | kept | Finding 57 |
| 67 | kept | Finding 58 |
| 68 | kept | Finding 59 |
| 69 | kept | Finding 60 |
| 70 | kept | Finding 61 |
| 71 | kept | Finding 62 |
| 72–73 | kept | Findings 63 |

## Fix workflow (from COMMIT.md / PLAN.md / TRANSLATION.md — binding)

- Commits are ONE coherent gate-green step each — never drive-by edits, never a whole subsystem
  in one shot. Complex items land incrementally (stub/reorder first, bodies bottom-up), exactly
  as SCC clusters were ported.
- Never push `main`; `gh pr create` → `gh pr checks --watch` → squash-merge. Gates green on
  every landing: fmt, clippy (-D warnings), check/test `--workspace --all-features`, drift,
  differential byte-exact. Never land red; revert and re-queue on failure.
- Read-before-write (Source Protocol): full C# file, its deps, the full_moon APIs, the ported
  Rust deps. Oracles decide correctness; never edit `references/**` or `corpus/**`.
- Only writable markdown: `loretta-rs/PROGRESS.md`.

## Verdict

Core engine fidelity is genuinely high (options/enums/errors/resources, HexFloat/ObjectDisplay
BMP, minifying utilities, scoping interface layer, folder core, precedence table, lexical-error
matrix; differential byte-identical on all 1870 pairs). The port is **not yet an exact copy**:
66 open findings above, including two reachable panics on ordinary Lua (varargs, overflow
literals), state-corrupting multi-tree scoping/renaming behavior, systematic numeric-literal
diagnostic gaps, minified-output drift, and test-surface gaps wide enough to hide all of it.
Priority: Findings 1–16 (crashes + scoping/script), 17–26 (scanner), then the rest in listed
order.
