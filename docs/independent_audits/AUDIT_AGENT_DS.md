Independent Line-by-Line Audit Report — Loretta→Rust port
I tool-call-read 90 C# files and 87 Rust files in full this pass (plus the four governing docs, the full_moon AstResult region, and the differential harness ops.rs in full; SyntaxFacts.cs/CommonMessageProvider.cs/Program.cs partially, as they are not in the pair list).
1. Gate outputs (verbatim, run from loretta-rs/)
cargo fmt --all --check                          → (clean, exit 0)
cargo clippy --workspace --all-targets --all-features -- -D warnings
                                                → Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.18s
cargo check --workspace --all-features --locked → Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
cargo test --workspace --all-features --locked → 26 "test result: ok" lines, every "failed" = 0; per-binary passes sum to 234
cargo run -q -p differential -- check corpus/expected --out .scratch/diff-check
   Oracle 2 — differential check (Rust vs C# reference)
     identical: 1870
     pending (version-gating diagnostics not ported): 0
     FAILED: 0
   Oracle 2: PASS (no unexpected drift)
drift check → DRIFT OK: 744 nodes, 1335 edges, topo 744
All values match the expected baselines exactly (234/0; 1870/0/0) — no delta to investigate.
2. Per-file verdict table
Rust file	Verdict	Notes
errors/errorcode.rs	EXACT	48 values, incl. gaps at 2/29; Void=-2, Unknown=-1 (InternalErrorCode verified)
errors/errorfacts.rs	EXACT	Generated partial verified (only WRN warning; fatal/info/hidden all false)
errors/messageprovider.rs	EXACT	55-arm format map covers every resx code; Void/Unknown → None
errors/luadiagnostic.rs	ADAPTED	Location dropped; severity enum values exact
errors/luadiagnosticinfo.rs	EXACT	3 ctors mirrored
errors/luadiagnosticformatter.rs	ADAPTED	Matches DiagnosticFormatter default branch (verified)
errors/syntaxdiagnosticinfo.rs	EXACT	4 ctors + WithOffset; serialization dropped
luaresources.rs	EXACT	66/66 strings byte-for-byte (scripted comparison, 0 mismatches)
errors/lexerdiagnostics.rs	DIVERGENT	Backtick-string rules — see Finding 1; everything else EXACT (spans, order, bad-token 200-limit, <<-only gating)
errors/parserdiagnostics.rs	PARTIAL/DIVERGENT	Only gating rules + continue + bitwise — see Finding 2
scoping/* (8 files)	EXACT/ADAPTED	FindVariable first-match order documented (C# HashSet order is implementation-defined); set_label_syntax documented
script/renameerrors.rs	ADAPTED	record hierarchy → enum; SyntaxTree → tree text
script/script.rs	ADAPTED	Rename flow verified incl. conflict dedup, per-tree gates, Candidate-D panic
script/scriptrenamerewriter.rs	ADAPTED	Position-based replacement; re-walk id-base seeding consistent
script/manager.rs, state.rs, basewalker.rs	ADAPTED	Recovered-AST contribution (Candidate E), id bases, location store
script/scopeandvariablewalker.rs	DIVERGENT	For-loop header ordering — see Finding 3
script/gotolabelwalker.rs, gotowalker.rs	ADAPTED	Unified single-pass placeholder binding (Findings 6/7 documented)
utilities/charutils.rs	EXACT	Binary-search check equivalence verified; 4-byte branch documented (Finding 48)
utilities/hexfloat.rs	EXACT	Full line-by-line; table, masks, rounding, sticky bit, messages
utilities/stringutils.rs	ADAPTED	Trim-degenerate residual verified matches its doc
symbol_display/objectdisplay.rs	ADAPTED	Surrogate-split (Finding 49); "R" reimplementation oracle-pinned; verbatim delimiters verified
symbol_display/objectdisplayoptions.rs	EXACT	Flags
symbol_display/unicode_categories.rs	EXACT	Generated table: 4108 arms, complete 0..0x10FFFF coverage, values ≤29; 107-char oracle probe MATCH; single unreachable gap at _
backtick/continue/integerformats.rs	EXACT	 
luasyntaxoptions.rs	EXACT	All 11 presets field-by-field; assert; 26-field Eq/Hash omitting the two escape fields; Display verified
luaparseoptions.rs	ADAPTED	Defaults, With*, features OrdinalIgnoreCase/last-wins, validate (Finding 64)
operations/*	EXACT	21 + 5 variants
experimental/constantfolder.rs	ADAPTED	All fold rules verified vs C# (wrapping shifts/truncation/classification); IsEquivalentTo quote-sensitivity probe MATCH; SetConstructor no-fold documented
constantfoldingoptions.rs, luaextensions.rs	EXACT/ADAPTED	 
minifying/* (9 files)	EXACT/ADAPTED	RequiresSeparator table verified arm-by-arm vs SyntaxFacts.cs:125-229; naming strategies (incl. the Finding-40 ceiling); slot allocators; RenameTable last-use release
syntaxextensions.rs	ADAPTED	Documented DROP (obsolete FoldConstants)
tests/ (20 files)	ADAPTED	1:1 test mapping, identical inputs/expectations; red-tree walks → AST docking + round-trip (documented); ported diagnostics exact (codes/positions/squiggles/args); parser-level diagnostics assert full_moon's exact messages (Finding 55)
tests/src/* (10 files)	ADAPTED	Helpers verified vs C# (LuaTestBase, LuaTestSource, RandomSpaceInserter incl. shift masking, SyntaxTreeExtensions, ShortToken, LexicalTestData incl. the cfxlua-kind GetText verification, the test bases)
3. New findings (divergences not in the accepted-residual list)
Finding 1 — Backtick-string diagnostics are largely unported. lexerdiagnostics.rs:568-624 + the dispatch at :1213 vs Lexer.ShortString.cs:54-73 + the InterpolatedStringScanner (:306-581) + LanguageParser.cs:198. The port's scan_backtick_string omits: (a) escape diagnostics inside the string (the C# runs ScanEscapeSequence in the contents), (b) the hole diagnostics (LUA0034 ERR_UnclosedExpressionHole, LUA0035 ERR_DoubleBraceInInterpolation, ERR_SyntaxError), (c) the parser's LUA1012 ERR_InvalidStatement, and (d) for unfinished backtick strings under a None backtick preset, the LUA0036 gating error (early return). Probe (both harnesses): Lua51  `abc  → C# [LUA0003, LUA0036, LUA1012]×2 vs rust [LUA0003]×2; every preset and every backtick input diverges. Uncovered by the corpus (no backticks) and by the tests. Note: the newline-terminated unfinished span matches the C# test ("e" at (1,18)) — the pinned behavior, not the span, is the issue.
Finding 2 — If-expression gating and the general parser-error diagnostics are unported. parserdiagnostics.rs has no if-expression arm vs LanguageParser.cs:1329-1330 (ERR_IfExpressionsNotSupportedInLuaVersion LUA1008), and none of LUA1012/1010/1011/1000/1001/1014/1015/1017/1018/LUA0019/LUA0015 are ported; the op additionally gates parser diagnostics on full_moon::parse succeeding. Probe: local x = if true then 1 else 2 end under Lua51 → C# [LUA1008, LUA1012, LUA1012] vs rust [] (all 8 presets diverge). The file's own header says "Starts with the version-gated statement rules the differential corpus exercises" — the corpus has no such inputs; the tests assert full_moon's own diagnostics instead (documented Finding 55).
Finding 3 — For-loop header expressions resolve through the loop's own block scope. scopeandvariablewalker.rs:298-357 creates the block scope and the iteration variables before visiting the header expressions; the C# overrides visit header expressions first in the enclosing scope (ScopeAndVariableWalker.cs:182-203 numeric, :205-227 generic). Consequences: for for i = i, 10 do end the header i resolves (via try_get_variable) to the loop's iteration variable in the port (read location + variable-map entry), versus the outer variable in the C#; and header identifiers' referencing_scopes records are the loop block instead of the enclosing scope. The port's in-code justification cites the generated base visitor's order (Syntax.xml.Internal.g.cs:9991-9995 — verified accurate for the base visitor), but the override is what runs. Not observable in the differential scope op (no resolution/referencing data in the JSON) and not covered by the ported tests.
Minor notes (documented in-code but worth recording): the goto-gating span excludes a trailing ; the C# node span includes (unobservable — the differential outputs no spans); FindVariable's shadowed-name tie-break is declaration-order vs the C# HashSet order (documented Finding 63 note); the generic Sequential throw message uses trim_start_matches vs the C# Remove(0, 6) (differs only if the alphabet contains the prefix char); the keyword-kind test covers 22 words vs the C# 26 (the Luau contextuals cannot dock on Symbol::from_str — the actual consumer namingstrategies::is_keyword covers all 26).
4. Explicitly not verified
- The unicode_categories table's per-codepoint correctness for all 1,114,112 scalars — verified structurally (contiguity, coverage, value range), against the real .NET runtime on a 107-char probe (MATCH), and via its documented generation provenance; not recomputed exhaustively by hand.
- CommonMessageProvider.GetIdForErrorCode — read as a function region (the file is Core/Portable DROP); the LUA%04d form matches the C# prefix + code.ToString("0000").
- The C# Program.cs harness — read in regions (diagnostics op; the minify/scope ops used in probes); not read end-to-end.
- HashSet-order-dependent FindVariable results on shadowed same-name locals — the C# side is implementation-defined; the port's tie-break is documented (Finding 63).
5. Final verdict
Does the port meet AGENTS.md Requirements 1–3 and the Definition of Done? Mostly yes, with three uncovered divergences.
- Requirement 1 (Logic Parity): the ported logic matches the C# across the verified surface — error codes/messages/spans/args, folding rules, scoping/script semantics, utilities, options, minifying — with every deviation either documented (as an ADAPT with an in-code citation) or a known residual verified against its documentation. The caveats are Findings 1–3: the backtick-string diagnostics, the parser-error diagnostics beyond version gating, and the for-loop header resolution are genuinely divergent and are not documented as limitations in the code (Finding 3's comment cites the wrong C# authority).
- Requirement 2 (full-moon integration): honored — no lexer/parser/tree porting; the two full_moon-boundary adaptations found (recovered-AST contribution; type-parameter-default drops) are inside the boundary.
- Requirement 3 (Verbatim translation) + Definition of Done: every file carries the // Ported from header; all gates and both oracles pass (234/0; 1870/0/0; drift 744/1335/744); PROGRESS rows exist. The yes is therefore conditional: the three findings above are real divergences from the C# that the two oracles do not exercise. If "meets" is judged strictly as byte-identical behavior on every input, the answer is no, blocked by Findings 1–3; if judged as the documented port's contract (gates + oracles green, adaptations documented), the answer is yes with the caveat that Findings 1–2 are only partially documented ("starts with") and Finding 3's documentation is inaccurate.