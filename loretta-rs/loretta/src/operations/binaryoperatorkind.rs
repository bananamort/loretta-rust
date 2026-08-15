// Ported from Loretta.CodeAnalysis.Lua.Operations.BinaryOperatorKind (b767b4e): BinaryOperatorKind
// C# source: src/Compilers/Lua/Portable/Operations/BinaryOperatorKind.cs

/// Kind of binary operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum BinaryOperatorKind {
    /// Represents unknown or error operator kind.
    None = 0,
    /// Represents the Lua `+` operator.
    Addition,
    /// Represents the Lua `-` operator.
    Subtraction,
    /// Represents the Lua `*` operator.
    Multiplication,
    /// Represents the Lua `/` operator.
    Division,
    /// Represents the Lua `%` operator.
    Modulus,
    /// Represents the Lua `^` operator.
    Exponentiation,
    /// Represents the Lua `..` operator.
    StringConcatenation,
    /// Represents the Lua `&` operator.
    BitwiseAnd,
    /// Represents the Lua `|` operator.
    BitwiseOr,
    /// Represents the Lua `~` operator.
    ExclusiveOr,
    /// Represents the Lua `<<` operator.
    LeftShift,
    /// Represents the Lua `>>` operator.
    RightShift,
    /// Represents the Lua `==` operator.
    Equals,
    /// Represents the Lua `!=` operator.
    NotEquals,
    /// Represents the Lua `>` operator.
    GreaterThan,
    /// Represents the Lua `>=` operator.
    GreaterThanOrEqual,
    /// Represents the Lua `<` operator.
    LessThan,
    /// Represents the Lua `<=` operator.
    LessThanOrEqual,
    /// Represents the Lua `and` operator.
    And,
    /// Represents the Lua `or` operator.
    Or,
}
