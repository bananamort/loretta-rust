using System.Text.Json;
using Microsoft.Build.Locator;
using Microsoft.CodeAnalysis;
using Microsoft.CodeAnalysis.CSharp;
using Microsoft.CodeAnalysis.CSharp.Syntax;
using Microsoft.CodeAnalysis.MSBuild;

// Stage 1 — Extract a Typed Semantic Graph (TSTG) from the C# codebase.
// One row per ISymbol (type, method, property, field, enum, etc.), not per file.
// The graph, not the file tree, is the unit of work.
// This tool fails if any MemberDeclarationSyntax has no corresponding graph node.

var solutionPath = args.Length > 0 && !args[0].EndsWith(".json", StringComparison.OrdinalIgnoreCase)
    ? args[0] : "../../../../references/Loretta/Loretta.sln";
var rawOutput = args.Length > 1 ? args[1]
    : args.Length == 1 && args[0].EndsWith(".json", StringComparison.OrdinalIgnoreCase) ? args[0]
    : null;

if (!MSBuildLocator.IsRegistered)
{
    var instance = MSBuildLocator.QueryVisualStudioInstances().FirstOrDefault()
        ?? throw new InvalidOperationException("No MSBuild instance found. Install .NET SDK 8.");
    MSBuildLocator.RegisterInstance(instance);
}

var droppedDirFragments = new[] { "/Parser/", "/Syntax/", "/Generated/", "/obj/", "/InternalSyntax/" };
var droppedFiles = new[] { "/Portable/LuaExtensions.cs" }; // AGENTS.md: Portable/LuaExtensions.cs is syntax helpers over dropped nodes — DROP; Experimental/LuaExtensions.cs is PORT
var allNodes = new List<NodeRecord>();
var symbolRecords = new List<(ISymbol Symbol, NodeRecord Record)>();
var perDocumentCounts = new List<(string doc, int memberDecls, int nodes)>(); 

// Prefer direct file enumeration for reliability (MSBuildWorkspace often fails on .shproj).
// Find repo root by walking up from AppContext.BaseDirectory until we find references/Loretta.
string? FindRepoRoot(string start)
{
    var dir = new DirectoryInfo(start);
    while (dir != null)
    {
        if (Directory.Exists(Path.Combine(dir.FullName, "references/Loretta/src/Compilers/Lua/Portable")))
            return dir.FullName;
        dir = dir.Parent;
    }
    return null;
}
var repoRoot = FindRepoRoot(AppContext.BaseDirectory)
    ?? FindRepoRoot(Directory.GetCurrentDirectory())
    ?? FindRepoRoot(Path.GetFullPath(solutionPath) is var p ? Path.GetDirectoryName(p) ?? Directory.GetCurrentDirectory() : Directory.GetCurrentDirectory())
    ?? Path.GetFullPath(Path.Combine(AppContext.BaseDirectory, "../../../../.."));
if (!Directory.Exists(Path.Combine(repoRoot, "references/Loretta/src/Compilers/Lua/Portable")))
{
    Console.Error.WriteLine($"Could not find repo root from {AppContext.BaseDirectory}. Tried {repoRoot}");
    return 5;
}
var outputPath = rawOutput != null ? Path.GetFullPath(rawOutput) : Path.Combine(repoRoot, "loretta-rs/nodes.json");
var outputDir = Path.GetDirectoryName(outputPath);
if (outputDir != null) Directory.CreateDirectory(outputDir);
var edgesPath = Path.Combine(outputDir ?? repoRoot, "edges.json");
var topoPath = Path.Combine(outputDir ?? repoRoot, "topo.json");
var luaPortable = Path.Combine(repoRoot, "references/Loretta/src/Compilers/Lua/Portable");
var luaExperimental = Path.Combine(repoRoot, "references/Loretta/src/Compilers/Lua/Experimental");
var luaCommandLine = Path.Combine(repoRoot, "references/Loretta/src/Compilers/Lua/CommandLine");

var includedDirs = new[] { luaPortable, luaExperimental, luaCommandLine }.Where(Directory.Exists).ToList();
if (includedDirs.Count == 0)
{
    Console.Error.WriteLine($"No included dirs found under {repoRoot}. Tried: {string.Join(", ", new[] { luaPortable, luaExperimental, luaCommandLine })}");
    return 2;
}

var allCsFiles = includedDirs.SelectMany(d => Directory.GetFiles(d, "*.cs", SearchOption.AllDirectories))
    .Where(f => !droppedDirFragments.Any(frag => f.Replace('\\', '/').Contains(frag, StringComparison.Ordinal))
                && !droppedFiles.Any(df => f.Replace('\\', '/').EndsWith(df, StringComparison.Ordinal)))
    .ToList();

Console.WriteLine($"Found {allCsFiles.Count} C# files in included dirs (after dropping {string.Join(", ", droppedDirFragments)}):");
foreach (var d in includedDirs) Console.WriteLine($"  - {d}");

var syntaxTrees = new List<CSharpSyntaxTree>();
var filePathByTree = new Dictionary<SyntaxTree, string>();
foreach (var file in allCsFiles)
{
    var text = await File.ReadAllTextAsync(file);
    var tree = (CSharpSyntaxTree)CSharpSyntaxTree.ParseText(text, path: file);
    syntaxTrees.Add(tree);
    filePathByTree[tree] = file;
}

// Minimal references to allow SemanticModel to resolve ISymbol (object, System, Tsu, etc.).
var refs = new List<MetadataReference> { MetadataReference.CreateFromFile(typeof(object).Assembly.Location) };
var tsuDll = Path.Combine(repoRoot, "references/Loretta/src/Compilers/Lua/Portable/bin/Debug/net8.0/Tsu.dll");
if (File.Exists(tsuDll))
    refs.Add(MetadataReference.CreateFromFile(tsuDll));

var compilation = CSharpCompilation.Create("Loretta.TSTG",
    syntaxTrees: syntaxTrees,
    references: refs,
    options: new CSharpCompilationOptions(OutputKind.DynamicallyLinkedLibrary, nullableContextOptions: NullableContextOptions.Enable));

Console.WriteLine($"\nCompilation: {compilation.SyntaxTrees.Count()} trees, {refs.Count} refs");

var seenSymbols = new HashSet<ISymbol>(SymbolEqualityComparer.Default);
foreach (var tree in syntaxTrees)
{
    var filePath = filePathByTree[tree];
    var root = await tree.GetRootAsync();
    var semanticModel = compilation.GetSemanticModel(tree);

        // Every MemberDeclarationSyntax must become a node — or the extractor is incomplete.
        var memberDecls = root.DescendantNodes().OfType<MemberDeclarationSyntax>().ToList();
        var beforeCount = allNodes.Count;

        foreach (var decl in memberDecls)
        {
            // Skip namespace declarations themselves — we want the members inside.
            if (decl is NamespaceDeclarationSyntax or FileScopedNamespaceDeclarationSyntax)
                continue;

            var symbols = GetDeclaredSymbols(decl, semanticModel);
            foreach (var symbol in symbols)
            {
                if (symbol == null) continue;
                if (symbol.IsImplicitlyDeclared) continue;
                if (!seenSymbols.Add(symbol)) continue; // dedup partials — same ISymbol from multiple decls

                var line = decl.GetLocation().GetLineSpan().StartLinePosition.Line + 1;
                // Unique Id: display string + kind + line + file (handles overloads + partials)
                var display = symbol.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat);
                var id = $"{display}:{symbol.Kind}@{Path.GetFileName(filePath)}:{line}";

                var rec = new NodeRecord(
                    Id: id,
                    Document: filePath,
                    SymbolName: display,
                    SymbolKind: symbol.Kind.ToString(),
                    TypeKind: (symbol as INamedTypeSymbol)?.TypeKind.ToString(),
                    ContainingType: symbol.ContainingType?.ToDisplayString(SymbolDisplayFormat.CSharpErrorMessageFormat),
                    FilePath: filePath,
                    Line: line
                );
                allNodes.Add(rec);
                symbolRecords.Add((symbol, rec));
            }
        }

        var added = allNodes.Count - beforeCount;
        perDocumentCounts.Add((filePath, memberDecls.Count, added));
        // Note: partial types deduped, so a file with only a partial decl may add 0 nodes legitimately
        if (added == 0 && memberDecls.Count > 0)
        {
            // Only warn if none of the decls were seen before (i.e., truly 0 new symbols)
            var anyUnseen = false;
            foreach (var decl in memberDecls)
            {
                if (decl is NamespaceDeclarationSyntax or FileScopedNamespaceDeclarationSyntax) continue;
                foreach (var s in GetDeclaredSymbols(decl, semanticModel))
                {
                    if (s != null && !s.IsImplicitlyDeclared && !seenSymbols.Contains(s))
                    {
                        anyUnseen = true;
                        break;
                    }
                }
            }
            if (anyUnseen)
                Console.Error.WriteLine($"  WARN: {filePath}: {memberDecls.Count} MemberDeclarationSyntax but 0 nodes — check GetDeclaredSymbols");
        }
    }

// Gates: ensure we did not miss anything.
var memberDeclTotal = perDocumentCounts.Sum(x => x.memberDecls);
var nodeTotal = allNodes.Count;

Console.WriteLine($"\n=== Gates ===");
Console.WriteLine($"MemberDeclarationSyntax total (included dirs, excluding dropped): {memberDeclTotal}");
Console.WriteLine($"Graph nodes distinct (deduped partials): {nodeTotal}");
Console.WriteLine($"Files scanned: {allCsFiles.Count}");

// Gate 1: every file that had MemberDeclarationSyntax but produced 0 new nodes is noted.
// Partials legitimately produce 0 for the second file (same ISymbol), so only warn, not fail.
var emptyDocs = perDocumentCounts.Where(x => x.memberDecls > 0 && x.nodes == 0).ToList();
if (emptyDocs.Count > 0)
{
    Console.WriteLine($"NOTE: {emptyDocs.Count} files had MemberDeclarationSyntax but added 0 new distinct nodes (likely partials):");
    foreach (var d in emptyDocs) Console.WriteLine($"  - {d.doc} ({d.memberDecls} decls)");
}

// Gate 2: total nodes must be > 0.
if (nodeTotal == 0)
{
    Console.Error.WriteLine("FAIL: graph has 0 nodes — extractor missed everything.");
    return 4;
}

// Build edges: declares / inherits / implements / type-uses / contains-nested / calls / overrides
// Map named types to node Id for quick lookup (for type-uses/inherits)
var typeIdBySymbol = new Dictionary<ISymbol, string>(SymbolEqualityComparer.Default);
// General map for all symbols (for calls/overrides)
var symbolIdBySymbol = new Dictionary<ISymbol, string>(SymbolEqualityComparer.Default);
foreach (var (sym, rec) in symbolRecords)
{
    symbolIdBySymbol[sym.OriginalDefinition] = rec.Id;
    if (!symbolIdBySymbol.ContainsKey(sym))
        symbolIdBySymbol[sym] = rec.Id;
    if (sym is INamedTypeSymbol nt)
    {
        typeIdBySymbol[nt.OriginalDefinition] = rec.Id;
        if (!typeIdBySymbol.ContainsKey(nt))
            typeIdBySymbol[nt] = rec.Id;
    }
}

var edges = new List<EdgeRecord>();
var edgeSet = new HashSet<(string From, string To, string Kind)>();

void AddEdge(string from, string to, string kind)
{
    if (from == to) return;
    var key = (from, to, kind);
    if (edgeSet.Add(key))
        edges.Add(new EdgeRecord(from, to, kind));
}

IEnumerable<INamedTypeSymbol> GetReferencedNamedTypes(ITypeSymbol type)
{
    switch (type)
    {
        case INamedTypeSymbol named:
            yield return named;
            foreach (var arg in named.TypeArguments)
            {
                foreach (var inner in GetReferencedNamedTypes(arg))
                    yield return inner;
            }
            if (named.OriginalDefinition is INamedTypeSymbol orig && orig.TypeArguments.Length > 0)
            {
                // already handled via TypeArguments
            }
            break;
        case IArrayTypeSymbol arr:
            foreach (var inner in GetReferencedNamedTypes(arr.ElementType))
                yield return inner;
            break;
        case IPointerTypeSymbol ptr:
            foreach (var inner in GetReferencedNamedTypes(ptr.PointedAtType))
                yield return inner;
            break;
        default:
            break;
    }
}

foreach (var (sym, rec) in symbolRecords)
{
    var fromId = rec.Id;

    // Edge: member -> containing type, or nested type -> outer type
    if (sym.ContainingType != null)
    {
        var container = sym.ContainingType.OriginalDefinition;
        if (typeIdBySymbol.TryGetValue(container, out var containerId))
        {
            var kind = sym.Kind == SymbolKind.NamedType ? "contains-nested" : "declares";
            AddEdge(fromId, containerId, kind);
        }
        else if (typeIdBySymbol.TryGetValue(sym.ContainingType, out var containerId2))
        {
            var kind = sym.Kind == SymbolKind.NamedType ? "contains-nested" : "declares";
            AddEdge(fromId, containerId2, kind);
        }
    }

    if (sym is INamedTypeSymbol nt)
    {
        // inherits
        if (nt.BaseType != null)
        {
            var baseSym = nt.BaseType.OriginalDefinition;
            if (typeIdBySymbol.TryGetValue(baseSym, out var baseId))
                AddEdge(fromId, baseId, "inherits");
            else if (typeIdBySymbol.TryGetValue(nt.BaseType, out var baseId2))
                AddEdge(fromId, baseId2, "inherits");
        }
        // implements
        foreach (var iface in nt.Interfaces)
        {
            var ifaceSym = iface.OriginalDefinition;
            if (typeIdBySymbol.TryGetValue(ifaceSym, out var ifaceId))
                AddEdge(fromId, ifaceId, "implements");
            else if (typeIdBySymbol.TryGetValue(iface, out var ifaceId2))
                AddEdge(fromId, ifaceId2, "implements");
        }
        // type-uses via members of this type (for ordering types that contain fields of other ported types)
        foreach (var member in nt.GetMembers())
        {
            ITypeSymbol? memberType = member switch
            {
                IFieldSymbol f => f.Type,
                IPropertySymbol prop => prop.Type,
                IEventSymbol e => e.Type,
                _ => null
            };
            if (memberType != null)
            {
                foreach (var refNt in GetReferencedNamedTypes(memberType))
                {
                    var key = refNt.OriginalDefinition;
                    if (typeIdBySymbol.TryGetValue(key, out var refId) && refId != fromId)
                        AddEdge(fromId, refId, "type-uses");
                    else if (typeIdBySymbol.TryGetValue(refNt, out var refId2) && refId2 != fromId)
                        AddEdge(fromId, refId2, "type-uses");
                }
            }
            if (member is IMethodSymbol ms)
            {
                foreach (var refNt in GetReferencedNamedTypes(ms.ReturnType))
                {
                    var key = refNt.OriginalDefinition;
                    if (typeIdBySymbol.TryGetValue(key, out var refId) && refId != fromId)
                        AddEdge(fromId, refId, "type-uses");
                    else if (typeIdBySymbol.TryGetValue(refNt, out var refId2) && refId2 != fromId)
                        AddEdge(fromId, refId2, "type-uses");
                }
                foreach (var param in ms.Parameters)
                {
                    foreach (var refNt in GetReferencedNamedTypes(param.Type))
                    {
                        var key = refNt.OriginalDefinition;
                        if (typeIdBySymbol.TryGetValue(key, out var refId) && refId != fromId)
                            AddEdge(fromId, refId, "type-uses");
                        else if (typeIdBySymbol.TryGetValue(refNt, out var refId2) && refId2 != fromId)
                            AddEdge(fromId, refId2, "type-uses");
                    }
                }
            }
        }
    }
    else
    {
        // member type-uses
        ITypeSymbol? t = sym switch
        {
            IFieldSymbol f => f.Type,
            IPropertySymbol prop2 => prop2.Type,
            IMethodSymbol m => m.ReturnType,
            IEventSymbol e => e.Type,
            _ => null
        };
        if (t != null)
        {
            foreach (var refNt in GetReferencedNamedTypes(t))
            {
                var key = refNt.OriginalDefinition;
                if (typeIdBySymbol.TryGetValue(key, out var refId) && refId != fromId)
                    AddEdge(fromId, refId, "type-uses");
                else if (typeIdBySymbol.TryGetValue(refNt, out var refId2) && refId2 != fromId)
                    AddEdge(fromId, refId2, "type-uses");
            }
        }
        if (sym is IMethodSymbol ms2)
        {
            foreach (var param in ms2.Parameters)
            {
                foreach (var refNt in GetReferencedNamedTypes(param.Type))
                {
                    var key = refNt.OriginalDefinition;
                    if (typeIdBySymbol.TryGetValue(key, out var refId) && refId != fromId)
                        AddEdge(fromId, refId, "type-uses");
                    else if (typeIdBySymbol.TryGetValue(refNt, out var refId2) && refId2 != fromId)
                        AddEdge(fromId, refId2, "type-uses");
                }
            }
        }
        // overrides (method/property/event overrides base)
        ISymbol? overridden = null;
        if (sym is IMethodSymbol mOver) overridden = mOver.OverriddenMethod;
        else if (sym is IPropertySymbol pOver) overridden = pOver.OverriddenProperty;
        else if (sym is IEventSymbol eOver) overridden = eOver.OverriddenEvent;
        if (overridden != null)
        {
            var key = overridden.OriginalDefinition;
            if (symbolIdBySymbol.TryGetValue(key, out var overId) && overId != fromId)
                AddEdge(fromId, overId, "overrides");
            else if (symbolIdBySymbol.TryGetValue(overridden, out var overId2) && overId2 != fromId)
                AddEdge(fromId, overId2, "overrides");
        }
        // calls (method body invocations) — for ordering callees before callers
        if (sym is IMethodSymbol)
        {
            var syntaxRef = sym.DeclaringSyntaxReferences.FirstOrDefault();
            if (syntaxRef != null)
            {
                var syntax = syntaxRef.GetSyntax();
                var tree = syntax.SyntaxTree;
                var model = compilation.GetSemanticModel(tree);
                foreach (var invoc in syntax.DescendantNodes().OfType<InvocationExpressionSyntax>())
                {
                    var called = model.GetSymbolInfo(invoc).Symbol;
                    if (called != null)
                    {
                        var key = called.OriginalDefinition;
                        if (symbolIdBySymbol.TryGetValue(key, out var calleeId) && calleeId != fromId)
                            AddEdge(fromId, calleeId, "calls");
                        else if (symbolIdBySymbol.TryGetValue(called, out var calleeId2) && calleeId2 != fromId)
                            AddEdge(fromId, calleeId2, "calls");
                    }
                }
            }
        }
    }
}

Console.WriteLine($"Edges total: {edges.Count} (declares/inherits/implements/type-uses/contains-nested/calls/overrides)");
foreach (var g in edges.GroupBy(e => e.Kind).OrderBy(g => g.Key))
    Console.WriteLine($"  {g.Key}: {g.Count()}");

// Topological sort (Kahn) — bottom-up: dependencies first
// Edge From -> To means From depends on To, so To must come before From.
// Build inDegree[From] = number of dependencies, and dependents[To] = list of From.
var inDegree = allNodes.ToDictionary(n => n.Id, n => 0);
var dependents = allNodes.ToDictionary(n => n.Id, n => new List<string>());
foreach (var e in edges)
{
    if (!inDegree.ContainsKey(e.From) || !inDegree.ContainsKey(e.To)) continue;
    inDegree[e.From]++;
    dependents[e.To].Add(e.From);
}

// Kahn: start with nodes that have no dependencies (inDegree 0)
var queue = new Queue<string>(inDegree.Where(kv => kv.Value == 0).OrderBy(kv => kv.Key).Select(kv => kv.Key));
var topo = new List<string>();
var visited = new HashSet<string>();

while (queue.Count > 0)
{
    var id = queue.Dequeue();
    if (!visited.Add(id)) continue;
    topo.Add(id);
    foreach (var dep in dependents[id].OrderBy(x => x))
    {
        inDegree[dep]--;
        if (inDegree[dep] == 0)
            queue.Enqueue(dep);
    }
}

// If cycle remains, append remaining nodes in file/line order (deterministic)
var remaining = allNodes.Where(n => !visited.Contains(n.Id)).OrderBy(n => n.FilePath).ThenBy(n => n.Line).Select(n => n.Id).ToList();
if (remaining.Count > 0)
{
    Console.WriteLine($"WARN: cycle detected — {remaining.Count} nodes remain, appending in file/line order");
    Console.WriteLine($"  Example remaining: {string.Join(", ", remaining.Take(5))}");
    topo.AddRange(remaining);
}

Console.WriteLine($"Topo order: {topo.Count} / {nodeTotal} nodes");
Console.WriteLine($"Included dirs: {string.Join(", ", includedDirs)}");

var json = JsonSerializer.Serialize(allNodes, new JsonSerializerOptions { WriteIndented = true });
await File.WriteAllTextAsync(outputPath, json);
Console.WriteLine($"\nWrote {nodeTotal} nodes to {outputPath}");

var edgesJson = JsonSerializer.Serialize(edges, new JsonSerializerOptions { WriteIndented = true });
await File.WriteAllTextAsync(edgesPath, edgesJson);
Console.WriteLine($"Wrote {edges.Count} edges to {edgesPath}");

var topoJson = JsonSerializer.Serialize(topo, new JsonSerializerOptions { WriteIndented = true });
await File.WriteAllTextAsync(topoPath, topoJson);
Console.WriteLine($"Wrote topo order ({topo.Count}) to {topoPath}");
Console.WriteLine("Gates passed.");
return 0;

static IEnumerable<ISymbol?> GetDeclaredSymbols(MemberDeclarationSyntax decl, SemanticModel model)
{
    if (decl is FieldDeclarationSyntax fd)
    {
        foreach (var v in fd.Declaration.Variables)
        {
            yield return model.GetDeclaredSymbol(v);
        }
        yield break;
    }
    if (decl is EventFieldDeclarationSyntax efd)
    {
        foreach (var v in efd.Declaration.Variables)
            yield return model.GetDeclaredSymbol(v);
        yield break;
    }
    var sym = model.GetDeclaredSymbol(decl);
    if (sym != null) yield return sym;
    else
    {
        var id = decl.DescendantTokens().FirstOrDefault(t => t.IsKind(SyntaxKind.IdentifierToken));
        if (id != default)
            yield return model.GetSymbolInfo(id.Parent!).Symbol;
    }
}

record NodeRecord(
    string Id,
    string Document,
    string SymbolName,
    string SymbolKind,
    string? TypeKind,
    string? ContainingType,
    string FilePath,
    int Line
);

record EdgeRecord(
    string From,
    string To,
    string Kind
);
