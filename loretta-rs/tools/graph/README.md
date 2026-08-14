# Graph Extractor

Parses the C# codebase into a typed semantic graph. One node per distinct `ISymbol`, not per file.

- Input: `references/Loretta/src/Compilers/Lua/Portable` etc. (auto-found via repo root)
- Output:
  - `loretta-rs/nodes.json` — `772` distinct `NodeRecord` (`833` `MemberDeclarationSyntax`, `53` files, deduped 9 partials)
  - `loretta-rs/edges.json` — `1220` edges (`declares`/`inherits`/`implements`/`type-uses`/`contains-nested`)
  - `loretta-rs/topo.json` — bottom-up order (`430` leaves sorted, `342` in SCC withheld as clusters per `docs/PLAN.md`)

## Build

```sh
dotnet restore --packages .packages
dotnet build --no-restore
```

## Run

```sh
dotnet run --no-build
# writes loretta-rs/nodes.json (459K, 772 nodes), edges.json, topo.json
# or explicit:
dotnet run --no-build -- /path/to/nodes.json
```

## Gates (fail if missed)

- Every `MemberDeclarationSyntax` in included dirs must map to a distinct `ISymbol`. `Generated/`, `Parser/`, `Syntax/`, `InternalSyntax/`, `obj/` are excluded per `docs/AGENTS.md` Port Boundary.
- `nodes.Count > 0`; per-file `memberDecls>0` with 0 new symbols only allowed for deduped partials.
- Edges deduped; topo via Kahn (dependencies first), remaining SCC appended file/line.

See `docs/PLAN.md` Stage 1-2.
