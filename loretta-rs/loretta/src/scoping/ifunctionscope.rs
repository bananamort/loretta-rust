// Ported from Loretta.CodeAnalysis.Lua.IFunctionScope (b767b4e): IFunctionScope, IFunctionScopeInternal, FunctionScope
// C# source: src/Compilers/Lua/Portable/Scoping/IFunctionScope.cs

use crate::scoping::ivariable::SharedVariable;
use crate::scoping::Scope;

/// A function's scope (C# IFunctionScope).
pub trait IFunctionScope: crate::scoping::iscope::IScope {
    /// The parameters.
    fn parameters(&self) -> Vec<SharedVariable>;

    /// The variables captured by this scope (declared elsewhere but used
    /// here).
    fn captured_variables(&self) -> Vec<SharedVariable>;
}

/// C# FunctionScope data (IFunctionScope.cs:29-44): the parameter and
/// captured-variable lists.
#[derive(Default)]
pub struct FunctionScopeData {
    /// C# FunctionScope._parameters (IFunctionScope.cs:27).
    pub parameters: Vec<SharedVariable>,
    /// C# FunctionScope._capturedVariables (IFunctionScope.cs:28).
    pub captured_variables: Vec<SharedVariable>,
}

impl IFunctionScope for Scope {
    /// C# FunctionScope.Parameters (IFunctionScope.cs:33-35).
    fn parameters(&self) -> Vec<SharedVariable> {
        self.function_data()
            .expect("parameters requires a function scope")
            .parameters
            .clone()
    }

    /// C# FunctionScope.CapturedVariables (IFunctionScope.cs:37-39).
    fn captured_variables(&self) -> Vec<SharedVariable> {
        self.function_data()
            .expect("captured_variables requires a function scope")
            .captured_variables
            .clone()
    }
}

impl Scope {
    /// C# FunctionScope.AddParameter (IFunctionScope.cs:43-49): creates the
    /// parameter variable and appends it to the parameter list.
    pub fn add_parameter_in(
        scope: &std::rc::Rc<std::cell::RefCell<Scope>>,
        name: &str,
        declaration: Option<crate::scoping::node::Node>,
    ) -> SharedVariable {
        let parameter = Self::create_variable_in(
            scope,
            crate::scoping::variablekind::VariableKind::Parameter,
            name,
            declaration,
        );
        scope
            .borrow_mut()
            .function_data_mut()
            .expect("add_parameter requires a function scope")
            .parameters
            .push(parameter.clone());
        parameter
    }
}
