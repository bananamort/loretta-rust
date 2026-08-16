// Ported from Loretta.CodeAnalysis.Lua.Experimental.SyntaxExtensions (b767b4e): SyntaxExtensions
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.cs (line 572)
//
// INTENTIONALLY DROPPED per Port Boundary (documented in PROGRESS.md rows 99/294):
// the C# FoldConstants extension operates on SyntaxNode via LuaSyntaxRewriter — both
// dropped infrastructure (AGENTS.md Locked Decision 1). The C# is [Obsolete("Use
// ConstantFold instead.")] and its functional equivalent is the ported
// ConstantFolder (loretta/src/experimental/constantfolder.rs), exposed as
// LuaExtensions::constant_fold. The C# original, kept verbatim for the audit trail:
//
//   [Obsolete("Use ConstantFold instead.")]
//   [Browsable(false), EditorBrowsable(EditorBrowsableState.Never)]
//   public static SyntaxNode FoldConstants(this SyntaxNode node, ConstantFoldingOptions options) =>
//       new ConstantFolder(options).Visit(node);

/// Extension methods for syntax nodes (experimental).
///
/// The C# surface (SyntaxExtensions.FoldConstants) is intentionally dropped —
/// see the header note. The type exists so the graph node has a landing spot
/// and the module wiring stays intact.
pub struct SyntaxExtensions;
