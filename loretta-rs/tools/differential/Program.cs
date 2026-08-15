using System.Collections.Immutable;
using System.Text;
using System.Text.Json;
using System.Text.Json.Serialization;
using Loretta.CodeAnalysis;
using Loretta.CodeAnalysis.Lua;
using Loretta.CodeAnalysis.Lua.Experimental;
using Loretta.CodeAnalysis.Lua.Experimental.Minifying;
using Loretta.CodeAnalysis.Lua.SymbolDisplay;
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
        var ops = code.Length > 500_000 ? new[] { "diagnostics", "parse" } : new[] { "options", "diagnostics", "lex", "parse", "scope", "constantfold", "minify", "charutils", "objectdisplay", "messageprovider", "gotolabel" };
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
        "options" => new
        {
            preset = syntaxOpts.ToString(),
            language = parseOpts.Language,
            documentationMode = parseOpts.DocumentationMode.ToString(),
            features = parseOpts.Features.Select(f => f.Key + "=" + f.Value).ToArray(),
            withFeatures = parseOpts.WithFeatures(new[] { new KeyValuePair<string, string>("foo", "bar") }).Features.Select(f => f.Key + "=" + f.Value).ToArray()
        },
        "diagnostics" => await DiagnosticsOp(parseOpts, code),
        "lex" => await LexOp(parseOpts, code),
        "parse" => await ParseOp(parseOpts, code),
        "scope" => await ScopeOp(parseOpts, code, label),
        "rename" => await RenameOp(parseOpts, code),
        "constantfold" => await ConstantFoldOp(parseOpts, code),
        "minify" => await MinifyOp(parseOpts, code),
        "charutils" => CharUtilsOp(code),
        "stringutils" => new { note = "covered via lex/parse" },
        "hexfloat" => new { note = "covered via parse" },
        "objectdisplay" => ObjectDisplayOp(),
        "operator" => new { note = "covered via parse/constantfold" },
        "messageprovider" => MessageProviderOp(),
        "gotolabel" => await GotoLabelOp(),
        _ => new { error = $"unknown operation {operation}" }
    };
}

// CharUtils (internal in Loretta): invoked via reflection so the oracle uses
// the original C# implementation directly.
static object CharUtilsOp(string code)
{
    var charUtilsType = typeof(SyntaxFactory).Assembly.GetType("Loretta.CodeAnalysis.Lua.Utilities.CharUtils")!;
    var isBinary = charUtilsType.GetMethod("IsBinary", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static)!;
    var isDecimal = charUtilsType.GetMethod("IsDecimal", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static)!;
    var isOctal = charUtilsType.GetMethod("IsOctal", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static)!;
    var isWhitespace = charUtilsType.GetMethod("IsWhitespace", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static)!;
    return new { results = code.EnumerateRunes().Select(r => new { ch = r.ToString(), isBinary = r.Value <= char.MaxValue && (bool) isBinary.Invoke(null, new object[] { (char) r.Value })!, isDecimal = r.Value <= char.MaxValue && (bool) isDecimal.Invoke(null, new object[] { (char) r.Value })!, isOctal = r.Value <= char.MaxValue && (bool) isOctal.Invoke(null, new object[] { (char) r.Value })!, isWhitespace = r.Value <= char.MaxValue && (bool) isWhitespace.Invoke(null, new object[] { (char) r.Value })! }).ToArray() };
}

// ObjectDisplay.FormatLiteral(double, ObjectDisplayOptions, CultureInfo?) oracle:
// a fixed sample set of doubles, formatted in decimal ("R", invariant) and
// hexadecimal modes, straight from the reference Loretta.
static object ObjectDisplayOp()
{
    double[] values =
    {
        0.0, -0.0, 1.0, -1.0, 0.1, 255.255, 100.0, 1e5, 1e6, 1e7, 1e15, 1e16, 1e17, 1e18, 1e20, 1e-1, 1e-2, 1e-3, 1e-4, 1e-5, 1e-6,
        3.141592653589793, 123456789012345678.0, 1.2345678901234567e16, 9999999999999999.0, 1.5e-300, 1e300, 5e-324, 0.5, 2.0, 6.25, -123.456,
        double.NaN, double.PositiveInfinity, double.NegativeInfinity, 2.2250738585072014e-308, 1.7976931348623157e308
    };
    return new
    {
        results = values.Select(v => new
        {
            decimalLiteral = ObjectDisplay.FormatLiteral(v, ObjectDisplayOptions.None),
            hexadecimalLiteral = ObjectDisplay.FormatLiteral(v, ObjectDisplayOptions.UseHexadecimalNumbers)
        }).ToArray()
    };
}

// MessageProvider.GetCategory oracle: the category of every ErrorCode from
// the reference MessageProvider (internal — reflection).
static object MessageProviderOp()
{
    var assembly = typeof(SyntaxFactory).Assembly;
    var errorCodeType = assembly.GetType("Loretta.CodeAnalysis.Lua.ErrorCode")!;
    var messageProviderType = assembly.GetType("Loretta.CodeAnalysis.Lua.MessageProvider")!;
    var instance = messageProviderType.GetField("Instance", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static)!.GetValue(null)!;
    var getCategory = messageProviderType.GetMethod("GetCategory", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance)!;
    var getDescription = messageProviderType.GetMethod("GetDescription", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance)!;
    var loadMessage = messageProviderType.GetMethod("LoadMessage", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance)!;
    var getMessageFormat = messageProviderType.GetMethod("GetMessageFormat", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance)!;
    var getSeverity = messageProviderType.GetMethod("GetSeverity", System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Instance)!;
    return new { results = Enum.GetValues(errorCodeType).Cast<object>().Select(code => new { code = (int) code, severity = (int) getSeverity.Invoke(instance, new object[] { (int) code })!, category = (string) getCategory.Invoke(instance, new object[] { (int) code })!, description = getDescription.Invoke(instance, new object[] { (int) code })!.ToString(), message = (string) loadMessage.Invoke(instance, new object[] { (int) code, null })!, messageFormat = getMessageFormat.Invoke(instance, new object[] { (int) code })!.ToString() }).ToArray() };
}

// GotoLabel (internal) oracle: builds a GotoLabel from a fixed Lua 5.2 sample
// (label + two gotos), adds the jumps, and dumps name/labelText/jumps.
static async Task<object> GotoLabelOp()
{
    const string sample = "::top::\ngoto top\ngoto top\n";
    var text = SourceText.From(sample, Encoding.UTF8);
    var tree = SyntaxFactory.ParseSyntaxTree(text, new LuaParseOptions(LuaSyntaxOptions.Lua52), "");
    var root = await tree.GetRootAsync();
    var label = root.DescendantNodesAndSelf().OfType<GotoLabelStatementSyntax>().First();
    var gotos = root.DescendantNodes().OfType<GotoStatementSyntax>().ToArray();
    var assembly = typeof(SyntaxFactory).Assembly;
    var gotoLabelType = assembly.GetType("Loretta.CodeAnalysis.Lua.GotoLabel")!;
    var ctor = gotoLabelType.GetConstructor(System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public, null, new[] { typeof(string), typeof(GotoLabelStatementSyntax) }, null)!;
    var addJump = gotoLabelType.GetMethod("AddJump", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public)!;
    var labelName = label.Identifier.ValueText;
    var labelObj = ctor.Invoke(new object?[] { labelName, label });
    foreach (var g in gotos) addJump.Invoke(labelObj, new object[] { g });
    var name = (string) gotoLabelType.GetProperty("Name", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public)!.GetValue(labelObj)!;
    var jumps = ((System.Collections.IEnumerable) gotoLabelType.GetProperty("JumpSyntaxes", System.Reflection.BindingFlags.Instance | System.Reflection.BindingFlags.Public)!.GetValue(labelObj)!).Cast<GotoStatementSyntax>();
    return new { name, labelText = label.ToFullString(), jumps = jumps.Select(j => j.ToFullString()).ToArray() };
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
