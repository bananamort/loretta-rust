// Ported from Loretta.CodeAnalysis.Lua.ScopeKind (b767b4e): ScopeKind
// C# source: src/Compilers/Lua/Portable/Scoping/ScopeKind.cs

/// The type of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeKind {
    /// The global scope.
    Global,
    /// A file's scope.
    File,
    /// A local function's scope.
    Function,
    /// A block's scope.
    Block,
}
