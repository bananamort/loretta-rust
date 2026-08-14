# Git and Commits

Git history is the coordination log between agents. It must be complete enough that a fresh agent can reconstruct the project from `git log` and the current graph state.

## Workflow

- Never push directly to `main`. Every change goes through a pull request.
- Start a session with `git pull --rebase` and `git log --oneline -20`.
- Claim one graph node at a time (topo-sorted, bottom-up). Do not batch.
- After every landing, run the gates. If any gate fails, revert and re-queue at end of topo order. No `blocked` status — every node stays `pending`/`claimed`/`done` until it lands.

## Commits

One item per commit. No drive-by edits.

- `port: <C# type> -> <rust path>` — one ported item
- `harness: <what>` — differential harness, corpus
- `graph: <what>` — TSTG/graph updates
- `spec: <what>` — edits to `AGENTS.md`, `PLAN.md`, `COMMIT.md`, or `TRANSLATION.md` (use when spec is wrong; not for parking nodes)

Commit and push via PR before claiming the next item. Do not leave a dirty tree at session end.

## Pull Requests

- `gh pr create` to open. `gh pr checks <n> --watch` to wait. Do not merge until all checks pass.
- Squash-merge: `gh pr merge <n> --squash --delete-branch`.
- If the graph and git history disagree, history wins.

## CI

Runs on every PR. Must be green before merge:

- `cargo check --workspace --all-features`
- `cargo clippy --workspace --all-features` (no new warnings, `#[allow]` forbidden)
- `cargo test --workspace --all-features` (includes ported tests)
- `cargo fmt --all --check`
- Drift and differential checks (see `AGENTS.md`)

Uses `--locked`. Keep `Cargo.lock` in sync (`cargo metadata --locked`).

## Version Bumps

May skip CI: commit straight to `main` with `[skip ci]` in the message (e.g. `release: bump workspace to X [skip ci]`). This is the one exception to the no-direct-push rule. Publishing to crates.io is irreversible — confirm before publishing.

## Sandbox

All work stays inside the repo directory. See `AGENTS.md` Port Boundary counts for the locked decisions.

- Never write outside the repo. No global installs. Scratch space is `loretta-rs/.scratch/` (gitignored).
- Rust: run all cargo commands from `loretta-rs/`; `target/` stays in the workspace.
- C# harness: `NUGET_PACKAGES=loretta-rs/tools/differential/.packages dotnet restore --packages loretta-rs/tools/differential/.packages` then `dotnet build --no-restore`.
- Tool caches (`.scratch/`, `.packages/`, `.tools/`) must be gitignored the moment they are created.
- Set overrides inline per command, never by editing shell profiles or global git config.
