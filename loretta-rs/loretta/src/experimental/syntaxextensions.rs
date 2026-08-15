// Ported from Loretta.CodeAnalysis.Lua.Experimental.SyntaxExtensions (b767b4e): SyntaxExtensions
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.cs (line572)
//
// NOTE: The C# SyntaxExtensions.FoldConstants delegates to ConstantFolder, which extends
// LuaSyntaxRewriter (dropped Syntax infrastructure). The full constant folding logic
// requires reimplementing the visitor pattern using full_moon's AST.
//
// C# original:
//   [Obsolete("Use ConstantFold instead.")]
//   public static SyntaxNode FoldConstants(this SyntaxNode node, ConstantFoldingOptions options) =>
//       new ConstantFolder(options).Visit(node);

use crate::experimental::constantfoldingoptions::ConstantFoldingOptions;

/// Extension methods for syntax nodes (experimental).
pub struct SyntaxExtensions;
