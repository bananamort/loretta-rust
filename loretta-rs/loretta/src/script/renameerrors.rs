// Ported from Loretta.CodeAnalysis.Lua.RenameErrors (b767b4e): RenameError, IdentifierNameNotSupportedError, VariableConflictError
// C# source: src/Compilers/Lua/Portable/Script/RenameErrors.cs

use crate::scoping::ivariable::SharedVariable;

/// An error found while renaming a variable (C# `record RenameError`).
///
/// The C# record hierarchy (RenameError base + the two derived records)
/// maps to this enum; the dropped SyntaxTree argument of
/// IdentifierNameNotSupportedError maps to the tree's source text.
#[derive(Debug, Clone, PartialEq)]
pub enum RenameError {
    /// An error that represents the provided identifier not being supported
    /// in a provided tree (C# IdentifierNameNotSupportedError).
    IdentifierNameNotSupported {
        /// The tree the identifier name is not supported on (C#
        /// SyntaxTree — the dropped infra maps to the tree's text).
        tree_without_support: String,
    },
    /// Represents a conflict with an existing variable (C#
    /// VariableConflictError).
    VariableConflict {
        /// The variable that is conflicted with.
        variable_being_conflicted_with: SharedVariable,
    },
}
