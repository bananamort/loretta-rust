// Ported from Loretta.CodeAnalysis.Lua.IFileScope (b767b4e): IFileScope, IFileScopeInternal, FileScope
// C# source: src/Compilers/Lua/Portable/Scoping/IFileScope.cs

use crate::scoping::iscope::{IScope, Scope};
use crate::scoping::ivariable::VariableRef;

/// A file's scope.
pub trait IFileScope: IScope {
    /// The implicit `arg` that's available in all files.
    fn arg_variable(&self) -> VariableRef;

    /// The implicit vararg that's available in all files.
    fn var_arg_parameter(&self) -> VariableRef;
}

/// The C# `FileScope : Scope` class — flattened into [`Scope`] (see
/// `Scope::new_file` for the C# `FileScope` ctor creating the implicit `arg`
/// and `...` parameters, and `Scope::arg_variable`/`Scope::var_arg_parameter`
/// for the properties). The C# `IFileScopeInternal` interface is documented
/// drop (internal plumbing).
impl IFileScope for Scope {
    fn arg_variable(&self) -> VariableRef {
        Scope::arg_variable(self)
    }

    fn var_arg_parameter(&self) -> VariableRef {
        Scope::var_arg_parameter(self)
    }
}
