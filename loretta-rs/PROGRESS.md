# Progress

This file is the **only** place to track port progress. It is created and updated by agents during the port. `docs/` is read-only.

## How to update

- One row per graph node (type, method, property, field, enum, etc.) — not per file or per module.
- Status: `pending` / `claimed` / `done` / `blocked`
- Update this file in the same PR that lands the node.

| Node ID | C# symbol | Rust file | Status | Notes |
|---|---|---|---|---|
| _example_ | `Scoping/IScope.cs:IScope` | `loretta/src/scoping/scope.rs` | pending | — |

## Blocked

List blocked nodes with precise diagnosis here.

| Node ID | Reason | Evidence |
|---|---|---|
| | | |
