# Audit — Loretta-RS Port (C# reference @ `b767b4e`)

This is the standing audit charter for the Loretta → Rust port. It defines what "verbatim
port" means here, how audits are conducted, and what is already known-and-accepted. It does
NOT contain a list of past findings: completed audit rounds live in git history
(`git log -- docs/AUDIT.md`), and their fixes live in the merged PRs they landed in.

## What you are auditing

The port must satisfy `docs/AGENTS.md` Requirements 1–3 and its Definition of Done:

1. **Logic Parity** — same logic as the C#, no omissions: same names (snake_cased), same
   control flow, same constants, same error codes/messages/severities/spans/ordering, same
   math including wrapping arithmetic and shift masking.
2. **full-moon Integration** — full_moon is the lexer, parser, and AST. Never port Loretta's
   lexer/parser/syntax-tree model; never hand-roll program-structure parsing over raw tokens.
3. **Verbatim Translation** — mechanically diffable against the C# per `TRANSLATION.md`.

Read all four governing docs (`AGENTS.md`, `PLAN.md`, `TRANSLATION.md`, `COMMIT.md`) IN FULL
before judging anything. Where they are stricter than this file, they win.

## How to audit (mandatory method)

- Read both sides IN FULL with your own read tool before judging any file — not greps, not
  diffs alone, not prior audit text, not another agent's summary. Prior passes (see git log)
  may be right or wrong; nothing transfers without your own eyes on it.
- Open every citation. If a code comment says "C# does X at File.cs:N", read those lines. A
  comment contradicting its cited source is itself a finding.
- Adjudicate ambiguous semantics by EXECUTION (probe in `.scratch/`, existing tests, or the
  differential), never by argument.
- Run the oracles yourself from `loretta-rs/`:
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --workspace --all-features --locked`
  - `cargo run -q -p differential -- check corpus/expected --out .scratch/diff-check`
  Report exact printed numbers. Oracles decide correctness.

## Known-and-accepted residuals (do NOT flag; DO verify each still matches its docs)

These diverge from C# by structural necessity and carry in-code documentation:

- Lone-surrogate escapes (`\u{D800}`–`\u{DFFF}`) decode to U+FFFD — Rust strings cannot hold
  surrogates (constantfolder.rs, `\u` arm).
- Folder exponentiation uses `f64::powf` vs C# `Math.Pow` — last-ulp differences possible;
  corpus-visible cases agree (constantfolder.rs `num_pow`).
- Parse-failed trees contribute no scoping/minify state — full_moon returns no AST on failure
  (manager.rs `add_tree`). C# produces error-recovery trees; matching would require a local
  parser = Port Boundary violation.
- `StringUtils::trim` returns `""` on degenerate input instead of throwing (anti-panic).
- GLua operators (`&& || != !`) and C-style comments (`// /* */`) — DROP (Locked Decision 2).
- Type-parameter-default parse acceptance — intentional drop of a Loretta extension beyond
  the Luau RFC.
- `Portable/LuaExtensions.cs` and `SyntaxNormalizer` are inside DROP'd infrastructure — no
  port counterpart exists or should exist.

If any residual's in-code documentation has drifted from reality, that drift is a finding.

## Report format

- Start with true counts: "I read N C# files and M Rust files in full this pass."
- Gate outputs verbatim (fmt/clippy/test totals/differential identical-pending-failed).
- Findings: file:line on BOTH sides, quote what differs, classify
  (bug / spec-sanctioned / de-minimis-unobservable). No sampling — full reads only.
- Explicitly list anything you could not verify and why. An honest gap beats a false pass.

You are auditing for YOURSELF, not confirming anyone else's conclusions. If your independent
read finds nothing new, say so — but only after actually doing the reading.
