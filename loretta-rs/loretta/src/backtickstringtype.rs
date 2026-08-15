// Ported from Loretta.CodeAnalysis.Lua.BacktickStringType (b767b4e): BacktickStringType
// C# source: src/Compilers/Lua/Portable/BacktickStringType.cs

/// Defines what the type of strings using `` ` `` delimiters will be parsed as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BacktickStringType {
    /// Strings with `` ` `` delimiters have no meaning and will generate errors for unsupported interpolations.
    None,

    /// Strings with `` ` `` delimiters will be parsed as FiveM hash string literals.
    HashLiteral,

    /// Strings with `` ` `` delimiters will be parsed as interpolated string literals.
    InterpolatedStringLiteral,
}
