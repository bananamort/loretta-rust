// Ported from Loretta.CodeAnalysis.Lua.RenameError (b767b4e): RenameError, IdentifierNameNotSupportedError, VariableConflictError
// C# source: src/Compilers/Lua/Portable/Script/RenameErrors.cs

/// An error found while renaming a variable.
#[derive(Debug, Clone)]
pub enum RenameError {
    /// The provided identifier is not supported in a provided tree.
    /// Contains the name of the tree that doesn't support the identifier.
    IdentifierNameNotSupported { tree_name: String },
    /// A conflict with an existing variable.
    /// Contains the name of the variable being conflicted with.
    VariableConflict { variable_name: String },
}
