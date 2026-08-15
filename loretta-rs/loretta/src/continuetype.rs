// Ported from Loretta.CodeAnalysis.Lua.ContinueType (b767b4e): ContinueType
// C# source: src/Compilers/Lua/Portable/ContinueType.cs

/// The type of continue the lua flavor being parsed has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContinueType {
    /// No continue.
    None,

    /// Continue is a keyword.
    Keyword,

    /// Continue is a contextual keyword (is only a keyword when used as a statement).
    ContextualKeyword,
}
