// Ported from Loretta.CodeAnalysis.Lua.RenameError (b767b4e): RenameError, IdentifierNameNotSupportedError, VariableConflictError
// C# source: src/Compilers/Lua/Portable/Script/RenameErrors.cs
// NOTE: SyntaxTree and IVariable are from dropped infrastructure.

/// An error found while renaming a variable.
#[derive(Debug, Clone)]
pub enum RenameError {
    /// The provided identifier is not supported in a provided tree.
    IdentifierNameNotSupported,
    /// A conflict with an existing variable.
    VariableConflict { variable_name: String },
}
