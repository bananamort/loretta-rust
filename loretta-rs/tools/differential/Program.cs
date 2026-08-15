using System.Collections.Immutable;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Loretta.CodeAnalysis;
using Loretta.CodeAnalysis.Lua;
using Loretta.CodeAnalysis.Lua.Experimental;
using Loretta.CodeAnalysis.Lua.Experimental.Minifying;
using Loretta.CodeAnalysis.Lua.Syntax;
using Loretta.CodeAnalysis.Text;

var cmdArgs = Environment.GetCommandLineArgs().Skip(1).ToArray();
if (cmdArgs.Length < 2)
{
    Console.Error.WriteLine("Usage: Differential <operation> <preset> <code|file> [--out <dir>]");
    return 2;
}
var operation = cmdArgs[0];
var presetArg = cmdArgs[1];
var inputArg = cmdArgs.Length > 2 ? cmdArgs[2] : "";
string? outDir = null;
for (int i = 3; i < cmdArgs.Length; i++) if (cmdArgs[i] == "--out" && i + 1 < cmdArgs.Length) outDir = cmdArgs[i + 1];

string code = File.Exists(inputArg) ? await File.ReadAllTextAsync(inputArg) : inputArg;
if (inputArg == "--stdin") code = await Console.In.ReadToEndAsync();

if (operation == "all" && File.Exists(inputArg) && outDir != null)
{
    var presets = new[] { "Lua51", "Lua52", "Lua53", "Lua54", "LuaJIT20", "LuaJIT21", "GMod", "Luau", "FiveM", "All", "AllWithIntegers" };
    foreach (var preset in presets)
    {
        var opts = GetPreset(preset);
        var parseOpts = new LuaParseOptions(opts);
        var dir = Path.Combine(outDir, preset, Path.GetFileNameWithoutExtension(inputArg));
        Directory.CreateDirectory(dir);
        var ops = code.Length > 500_000 ? new[] { "diagnostics", "parse" } : new[] { "diagnostics", "lex", "parse", "scope", "constantfold", "minify" };
        foreach (var op in ops)
        {
            try
            {
                var obj = await RunOperation(op, parseOpts, code, inputArg);
                var json = JsonSerializer.Serialize(obj, new JsonSerializerOptions { WriteIndented = true, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull, ReferenceHandler = ReferenceHandler.IgnoreCycles });
                await File.WriteAllTextAsync(Path.Combine(dir, op + ".json"), json);
            }
            catch (Exception ex)
            {
                var json = JsonSerializer.Serialize(new { error = ex.Message, op, preset }, new JsonSerializerOptions { WriteIndented = true });
                await File.WriteAllTextAsync(Path.Combine(dir, op + ".json"), json);
            }
        }
    }
    Console.WriteLine($"Wrote expected for {inputArg} to {outDir}");
    return 0;
}

var singleOpts = GetPreset(presetArg);
var singleParseOpts = new LuaParseOptions(singleOpts);
object result = operation == "all" ? await RunAll(singleParseOpts, code, inputArg) : await RunOperation(operation, singleParseOpts, code, inputArg);
var outJson = JsonSerializer.Serialize(result, new JsonSerializerOptions { WriteIndented = true, DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull });
if (outDir != null) { Directory.CreateDirectory(outDir); var name = File.Exists(inputArg) ? Path.GetFileNameWithoutExtension(inputArg) + "." + operation + ".json" : operation + ".json"; await File.WriteAllTextAsync(Path.Combine(outDir, name), outJson); Console.WriteLine($"Wrote {Path.Combine(outDir, name)}"); } else Console.WriteLine(outJson);
return 0;

static LuaSyntaxOptions GetPreset(string name) => name switch
{
    "Lua51" => LuaSyntaxOptions.Lua51, "Lua52" => LuaSyntaxOptions.Lua52, "Lua53" => LuaSyntaxOptions.Lua53, "Lua54" => LuaSyntaxOptions.Lua54,
    "LuaJIT20" => LuaSyntaxOptions.LuaJIT20, "LuaJIT21" => LuaSyntaxOptions.LuaJIT21, "GMod" => LuaSyntaxOptions.GMod, "Luau" => LuaSyntaxOptions.Luau,
    "FiveM" => LuaSyntaxOptions.FiveM, "All" => LuaSyntaxOptions.All, "AllWithIntegers" => LuaSyntaxOptions.AllWithIntegers, _ => LuaSyntaxOptions.All
};

static async Task<object> RunAll(LuaParseOptions opts, string code, string label)
{
    var dict = new Dictionary<string, object?>();
    foreach (var op in new[] { "options", "diagnostics", "lex", "parse", "scope", "constantfold", "minify" })
        dict[op] = await RunOperation(op, opts, code, label);
    return dict;
}

static async Task<object> RunOperation(string operation, LuaParseOptions parseOpts, string code, string label)
{
    var syntaxOpts = parseOpts.SyntaxOptions;
    return operation switch
    {
        "options" => new { preset = syntaxOpts.ToString() },
        "diagnostics" => await DiagnosticsOp(parseOpts, code),
        "lex" => await LexOp(parseOpts, code),
        "parse" => await ParseOp(parseOpts, code),
        "scope" => await ScopeOp(parseOpts, code, label),
        "rename" => await RenameOp(parseOpts, code),
        "constantfold" => await ConstantFoldOp(parseOpts, code),
        "minify" => await MinifyOp(parseOpts, code),
        "charutils" => new { note = "covered via lex/parse" },
        "stringutils" => new { note = "covered via lex/parse" },
        "hexfloat" => new { note = "covered via parse" },
        "objectdisplay" => new { note = "covered via parse" },
        "operator" => new { note = "covered via parse/constantfold" },
        _ => new { error = $"unknown operation {operation}" }
    };
}

static async Task<object> DiagnosticsOp(LuaParseOptions opts, string code)
{
    var text = SourceText.From(code, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, opts, "");
    var diags = tree.GetDiagnostics().Concat((await tree.GetRootAsync()).DescendantTokens().SelectMany(t => t.GetDiagnostics())).ToArray();
    return new { diagnostics = diags.Select(d => new { id = d.Descriptor.Id, severity = d.Severity.ToString(), message = d.GetMessage() }).ToArray(), hasErrors = diags.Any(d => d.Severity == DiagnosticSeverity.Error) };
}

static async Task<object> LexOp(LuaParseOptions opts, string code)
{
    var text = SourceText.From(code, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, opts, "");
    var root = await tree.GetRootAsync();
    var tokens = root.DescendantTokens().ToArray();
    return new { tokens = tokens.Select(t => new { kind = t.Kind().ToString(), text = t.Text, fullText = t.ToFullString(), isMissing = t.IsMissing }).ToArray(), count = tokens.Length, roundTrip = root.ToFullString() == code };
}

static async Task<object> ParseOp(LuaParseOptions opts, string code)
{
    var text = SourceText.From(code, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, opts, "");
    var root = await tree.GetRootAsync();
    return new { treeText = tree.ToString(), rootKind = root.Kind().ToString(), hasErrors = tree.GetDiagnostics().Any(d => d.Severity == DiagnosticSeverity.Error) };
}

static async Task<object> ScopeOp(LuaParseOptions opts, string code, string label)
{
    var text = SourceText.From(code, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, opts, "");
    var script = new Script(ImmutableArray.Create(tree));
    var rootScope = script.RootScope;
    object ScopeToJson(IScope s) => new { kind = s.Kind.ToString(), nodeKind = s.Node?.Kind().ToString(), declaredVariables = s.DeclaredVariables.Select(v => new { name = v.Name, kind = v.Kind.ToString() }).ToArray(), containedScopes = s.ContainedScopes.Select(ScopeToJson).ToArray() };
    return new { label, rootScope = rootScope != null ? ScopeToJson(rootScope) : null, scopeCount = rootScope != null ? CountScopes(rootScope) : 0 };
    static int CountScopes(IScope s) => 1 + s.ContainedScopes.Sum(CountScopes);
}

static async Task<object> RenameOp(LuaParseOptions opts, string code)
{
    var text = SourceText.From(code, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, opts, "");
    var script = new Script(ImmutableArray.Create(tree));
    var root = script.RootScope;
    if (root == null) return new { ok = false, error = "no_root" };
    // Find first variable via recursion on ContainedScopes
    IScope? FindFirstWithVar(IScope s) => s.DeclaredVariables.FirstOrDefault() != null ? s : s.ContainedScopes.Select(FindFirstWithVar).FirstOrDefault(x => x != null);
    var scopeWithVar = FindFirstWithVar(root);
    var firstVar = scopeWithVar?.DeclaredVariables.FirstOrDefault();
    if (firstVar == null) return new { ok = true, note = "no_variable_to_rename", original = tree.ToString() };
    var rename = script.RenameVariable(firstVar, "renamedVar");
    try { var ok = ((dynamic)rename).IsOk; if (ok) { var newScript = ((dynamic)rename).Ok; return new { original = tree.ToString(), ok = true, newText = newScript.ToString() }; } else { return new { original = tree.ToString(), ok = false, errors = ((dynamic)rename).Err.ToString() }; } } catch { return new { original = tree.ToString(), ok = false, error = "rename_failed" }; }
}

static async Task<object> ConstantFoldOp(LuaParseOptions opts, string code)
{
    var text = SourceText.From(code, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, opts, "");
    var root = await tree.GetRootAsync();
    var options = new ConstantFoldingOptions(ExtractNumbersFromStrings: false);
    var optionsAll = new ConstantFoldingOptions(ExtractNumbersFromStrings: true);
    object FoldWith(ConstantFoldingOptions o) { var folded = root.ConstantFold(o); return new { foldedText = folded.ToFullString(), same = folded.ToFullString() == root.ToFullString() }; }
    return new { original = root.ToFullString(), withoutExtraction = FoldWith(options), withExtraction = FoldWith(optionsAll) };
}

static async Task<object> MinifyOp(LuaParseOptions opts, string code)
{
    var text = SourceText.From(code, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, opts, "");
    try { var minified = tree.Minify(NamingStrategies.Alphabetical, new SequentialSlotAllocator()); var root = await minified.GetRootAsync(); return new { minified = root.ToFullString() }; }
    catch (Exception ex) { return new { error = ex.Message }; }
}
