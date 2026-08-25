// Ported from Loretta.CodeAnalysis.Lua.Experimental.LuaExtensions (b767b4e): LuaExtensions
// C# source: src/Compilers/Lua/Experimental/LuaExtensions.cs

use full_moon::ast::Ast;
use full_moon::visitors::VisitorMut;

use crate::experimental::minifying::islotallocator::ISlotAllocator;
use crate::experimental::minifying::namingstrategies::NamingStrategies;
use crate::experimental::minifying::namingstrategy::NamingStrategy;
use crate::experimental::minifying::renamingrewriter::RenamingRewriter;
use crate::experimental::minifying::sortedslotallocator::SortedSlotAllocator;
use crate::experimental::minifying::triviarewriter::TriviaRewriter;
use crate::script::script::Script;

/// The extension methods for syntax trees (experimental).
pub struct LuaExtensions;

/// C# LuaExtensions.ConstantFold (LuaExtensions.cs:14-15):
/// `new ConstantFolder(options).Visit(node)` — runs constant folding on the
/// tree rooted by the provided node. The syntax options mirror the C#
/// tree's LuaParseOptions (the lexer computes the token values with them —
/// the escape echo/skip is preset-dependent, Finding 36).
pub fn constant_fold(
    ast: full_moon::ast::Ast,
    options: crate::experimental::constantfoldingoptions::ConstantFoldingOptions,
    syntax_options: crate::luasyntaxoptions::LuaSyntaxOptions,
) -> full_moon::ast::Ast {
    crate::experimental::constantfolder::ConstantFolder::new(options, syntax_options).fold(ast)
}

/// C# LuaExtensions.Minify(SyntaxTree) (LuaExtensions.cs:18-19): minifies
/// with the alphabetical naming strategy (the dropped SyntaxTree maps to the
/// tree text; the result is the minified text).
pub fn minify(tree: &str) -> String {
    minify_with_strategy(tree, Box::new(NamingStrategies::alphabetical))
}

/// C# LuaExtensions.Minify(SyntaxTree, NamingStrategy)
/// (LuaExtensions.cs:22-23): minifies with the sorted slot allocator.
pub fn minify_with_strategy(tree: &str, naming_strategy: NamingStrategy) -> String {
    minify_with(tree, naming_strategy, Box::new(SortedSlotAllocator::new()))
}

/// C# LuaExtensions.Minify(SyntaxTree, NamingStrategy, ISlotAllocator)
/// (LuaExtensions.cs:42-51): the renaming rewriter over the tree root,
/// followed by the trivia rewriter.
pub fn minify_with(
    tree: &str,
    naming_strategy: NamingStrategy,
    slot_allocator: Box<dyn ISlotAllocator>,
) -> String {
    // The C# Minify runs the rewriters over the error-recovery tree root —
    // the C# tree never fails to parse (Candidate E). full_moon's
    // AstResult carries the reconstructed AST alongside the errors, so the
    // recovered tree is minified like the C# visits its error nodes.
    let full_ast = full_moon::parse_fallible(tree, full_moon::LuaVersion::new()).into_ast();
    let script = Script::new(vec![tree.to_string()]);
    let mut renaming_rewriter = RenamingRewriter::new(script, naming_strategy, slot_allocator);
    let renamed = renaming_rewriter.rewrite(&full_ast);
    let mut trivia_rewriter = TriviaRewriter::INSTANCE;
    let minified = trivia_rewriter.visit_ast(renamed);
    minified.to_string()
}

/// Parses the code to an AST (the dropped SyntaxTree maps to the text).
pub fn parse_ast(tree: &str) -> Option<Ast> {
    full_moon::parse_fallible(tree, full_moon::LuaVersion::new())
        .into_result()
        .ok()
}
