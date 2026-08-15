// Ported from Loretta.CodeAnalysis.Lua.VariableKind (b767b4e): VariableKind
// C# source: src/Compilers/Lua/Portable/Scoping/VariableKind.cs

/// The kind of variables available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VariableKind {
    /// A local variable.
    Local,
    /// A global variable.
    Global,
    /// A function parameter.
    Parameter,
    /// A loop iteration variable.
    Iteration,
}
