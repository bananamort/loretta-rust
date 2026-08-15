// Ported from Loretta.CodeAnalysis.Lua.Experimental.LuaExtensions (b767b4e): LuaExtensions
// C# source: src/Compilers/Lua/Experimental/LuaExtensions.cs

use crate::experimental::constantfolder::ConstantFolder;
use crate::experimental::constantfoldingoptions::ConstantFoldingOptions;
use crate::experimental::minifying::islotallocator::ISlotAllocator;
use crate::experimental::minifying::namingstrategy::NamingStrategy;
use crate::experimental::minifying::sortedslotallocator::SortedSlotAllocator;
use full_moon::ast::Ast;

/// The extension methods for Lua syntax trees.
pub struct LuaExtensions;

impl LuaExtensions {
    /// Runs constant folding on the tree rooted by the provided node.
    /// C# `new ConstantFolder(options).Visit(node)` — the LuaSyntaxRewriter
    /// traversal is replaced by `ConstantFolder::fold_ast` (full_moon visitor).
    pub fn constant_fold(ast: Ast, options: ConstantFoldingOptions) -> Ast {
        ConstantFolder::new(options).fold_ast(ast)
    }

    /// C# `Minify(SyntaxTree)` — defaults to the alphabetical naming strategy
    /// and the sorted slot allocator.
    pub fn minify_default(ast: Ast) -> Ast {
        // C# `Minifying.NamingStrategies.Alphabetical` — the unavailable-name
        // check is pending IScope (scoping SCC); the strategies run against an
        // empty unavailable set until then.
        let mut allocator = SortedSlotAllocator::new();
        Self::minify_with_allocator(ast, Box::new(alphabetical_placeholder), &mut allocator)
    }

    /// C# `Minify(SyntaxTree, NamingStrategy)` — defaults to the sorted slot
    /// allocator.
    pub fn minify_with_strategy(ast: Ast, naming_strategy: NamingStrategy) -> Ast {
        let mut allocator = SortedSlotAllocator::new();
        Self::minify_with_allocator(ast, naming_strategy, &mut allocator)
    }

    /// C# `Minify(SyntaxTree, NamingStrategy, ISlotAllocator)` — the
    /// RenamingRewriter (PROGRESS rows 540+) and the Script scope analysis
    /// (SCC cluster) are pending; the tree is returned unchanged until they
    /// land.
    pub fn minify_with_allocator(
        ast: Ast,
        naming_strategy: NamingStrategy,
        slot_allocator: &mut dyn ISlotAllocator,
    ) -> Ast {
        let _ = (naming_strategy, slot_allocator);
        ast
    }
}

/// The alphabetical strategy placeholder used by `minify_default` until the
/// NamingStrategies port's unavailable-name check lands with IScope.
fn alphabetical_placeholder(slot: i32) -> String {
    crate::experimental::minifying::namingstrategies::NamingStrategies::alphabetical(slot)
}
