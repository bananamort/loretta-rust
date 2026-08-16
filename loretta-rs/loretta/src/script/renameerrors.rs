// Ported from Loretta.CodeAnalysis.Lua.RenameErrors (b767b4e): RenameError, IdentifierNameNotSupportedError, VariableConflictError
// C# source: src/Compilers/Lua/Portable/Script/RenameErrors.cs

use crate::scoping::ivariable::{IVariable, SharedVariable};

/// An error found while renaming a variable (C# `record RenameError`).
///
/// The C# record hierarchy (RenameError base + the two derived records)
/// maps to this enum; the dropped SyntaxTree argument of
/// IdentifierNameNotSupportedError maps to the tree's source text.
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

impl std::fmt::Debug for RenameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenameError::IdentifierNameNotSupported {
                tree_without_support,
            } => f
                .debug_struct("IdentifierNameNotSupportedError")
                .field("tree_without_support", tree_without_support)
                .finish(),
            RenameError::VariableConflict {
                variable_being_conflicted_with,
            } => f
                .debug_struct("VariableConflictError")
                .field(
                    "variable_being_conflicted_with",
                    &variable_being_conflicted_with.borrow().name().to_string(),
                )
                .finish(),
        }
    }
}

impl Clone for RenameError {
    fn clone(&self) -> Self {
        match self {
            RenameError::IdentifierNameNotSupported {
                tree_without_support,
            } => RenameError::IdentifierNameNotSupported {
                tree_without_support: tree_without_support.clone(),
            },
            RenameError::VariableConflict {
                variable_being_conflicted_with,
            } => RenameError::VariableConflict {
                variable_being_conflicted_with: variable_being_conflicted_with.clone(),
            },
        }
    }
}
