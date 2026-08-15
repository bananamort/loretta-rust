// Ported from Loretta.CodeAnalysis.Lua.Operations.UnaryOperatorKind (b767b4e): UnaryOperatorKind
// C# source: src/Compilers/Lua/Portable/Operations/UnaryOperatorKind.cs

/// Kind of unary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum UnaryOperatorKind {
    /// Represents unknown or error operator kind.
    None = 0,
    /// Represents the Lua `#` operator.
    Length,
    /// Represents the Lua `not` operator.
    Not,
    /// Represents the Lua `-` operator.
    Negation,
    /// Represents the Lua `~` operator.
    BitwiseNot,
}
