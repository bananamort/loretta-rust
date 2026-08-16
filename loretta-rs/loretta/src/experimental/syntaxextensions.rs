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

/// Extension methods for syntax nodes (experimental).
pub struct SyntaxExtensions;

/// C# LuaExtensions.ConstantFold (LuaExtensions.cs:14-15):
/// `new ConstantFolder(options).Visit(node)` — runs constant folding on the
/// tree rooted by the provided node.
pub fn constant_fold(
    ast: full_moon::ast::Ast,
    options: crate::experimental::constantfoldingoptions::ConstantFoldingOptions,
) -> full_moon::ast::Ast {
    crate::experimental::constantfolder::ConstantFolder::new(options).fold(ast)
}
