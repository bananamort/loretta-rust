// Ported from Loretta.CodeAnalysis.Lua.IFileScope (b767b4e): IFileScope, IFileScopeInternal, FileScope
// C# source: src/Compilers/Lua/Portable/Scoping/IFileScope.cs

use crate::scoping::ivariable::SharedVariable;
use crate::scoping::Scope;

/// A file's scope (C# IFileScope).
pub trait IFileScope: crate::scoping::iscope::IScope {
    /// The implicit `arg` that's available in all files.
    fn arg_variable(&self) -> SharedVariable;

    /// The implicit vararg that's available in all files.
    fn vararg_parameter(&self) -> SharedVariable;
}

/// C# FileScope data (IFileScope.cs:34-40): the implicit variables.
pub struct FileScopeData {
    /// C# FileScope.ArgVariable (IFileScope.cs:34).
    pub arg_variable: SharedVariable,
    /// C# FileScope.VarArgParameter (IFileScope.cs:38).
    pub vararg_parameter: SharedVariable,
}

impl IFileScope for Scope {
    /// C# FileScope.ArgVariable (IFileScope.cs:34-36).
    fn arg_variable(&self) -> SharedVariable {
        self.file_data()
            .expect("arg_variable requires a file scope")
            .arg_variable
            .clone()
    }

    /// C# FileScope.VarArgParameter (IFileScope.cs:38-40).
    fn vararg_parameter(&self) -> SharedVariable {
        self.file_data()
            .expect("vararg_parameter requires a file scope")
            .vararg_parameter
            .clone()
    }
}
