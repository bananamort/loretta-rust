// Ported from Loretta.CodeAnalysis.Lua.Experimental.LuaExtensions (b767b4e): LuaExtensions
// C# source: src/Compilers/Lua/Experimental/LuaExtensions.cs

/// The extension methods for syntax trees (experimental).
pub struct LuaExtensions;

/// C# LuaExtensions.ConstantFold (LuaExtensions.cs:14-15):
/// `new ConstantFolder(options).Visit(node)` — runs constant folding on the
/// tree rooted by the provided node.
pub fn constant_fold(
    ast: full_moon::ast::Ast,
    options: crate::experimental::constantfoldingoptions::ConstantFoldingOptions,
) -> full_moon::ast::Ast {
    crate::experimental::constantfolder::ConstantFolder::new(options).fold(ast)
}
