// Ported from Loretta.CodeAnalysis.Lua.IFunctionScope (b767b4e): IFunctionScope, IFunctionScopeInternal, FunctionScope
// C# source: src/Compilers/Lua/Portable/Scoping/IFunctionScope.cs

use crate::scoping::iscope::{IScope, Scope};
use crate::scoping::ivariable::VariableRef;

/// A function's scope.
pub trait IFunctionScope: IScope {
    /// The parameters.
    fn parameters(&self) -> &[VariableRef];

    /// Contains the variables that are captured by this scope.
    /// Variables captured by the scope are variables that weren't declared
    /// on the scope but are used in it.
    fn captured_variables(&self) -> &[VariableRef];
}

/// The C# `FunctionScope : Scope` class — flattened into [`Scope`] (see
/// `Scope::new_function` for the C# `FunctionScope` ctor, `Scope::parameters`
/// /`Scope::captured_variables` for the properties and
/// `Scope::add_parameter` for `AddParameter`). The C# `IFunctionScopeInternal`
/// interface is documented drop (internal plumbing).
impl IFunctionScope for Scope {
    fn parameters(&self) -> &[VariableRef] {
        Scope::parameters(self)
    }

    fn captured_variables(&self) -> &[VariableRef] {
        Scope::captured_variables(self)
    }
}
