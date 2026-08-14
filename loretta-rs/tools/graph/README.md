# Graph Extractor

Parses the C# codebase into a typed semantic graph. One node per distinct `ISymbol`, not per file.

- Input: `references/Loretta/src/Compilers/Lua/Portable` etc. (auto-found via repo root)
- Output:
  - `loretta-rs/nodes.json` — `744` distinct `NodeRecord` (`803` `MemberDeclarationSyntax`, `52` files, deduped 9 partials, `Portable/LuaExtensions.cs` DROP)
  - `loretta-rs/edges.json` — `1335` edges (`declares`/`calls`/`type-uses`/`inherits`/`implements`/`overrides`/`contains-nested` per `docs/AGENTS.md`)
  - `loretta-rs/topo.json` — bottom-up order (`402` leaves sorted, `342` in SCC withheld as clusters per `docs/PLAN.md`)

## Build

```sh
dotnet restore --packages .packages
dotnet build --no-restore
```

## Run

```sh
dotnet run --no-build
# writes loretta-rs/nodes.json (447K, 744 nodes), edges.json, topo.json
# or explicit:
dotnet run --no-build -- /path/to/nodes.json
```

## Gates (fail if missed)

- Every `MemberDeclarationSyntax` in included dirs must map to a distinct `ISymbol`. `Generated/`, `Parser/`, `Syntax/`, `InternalSyntax/`, `obj/` are excluded per `docs/AGENTS.md` Port Boundary.
- `nodes.Count > 0`; per-file `memberDecls>0` with 0 new symbols only allowed for deduped partials.
- Edges deduped; topo via Kahn (dependencies first), remaining SCC appended file/line.

See `docs/PLAN.md` Stage 1-2.
