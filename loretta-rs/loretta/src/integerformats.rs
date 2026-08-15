// Ported from Loretta.CodeAnalysis.Lua.IntegerFormats (b767b4e): IntegerFormats
// C# source: src/Compilers/Lua/Portable/IntegerFormats.cs

/// The format integers should be stored as.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegerFormats {
    /// No integer support at all and numbers are parsed as f64s without overflow behavior.
    NotSupported = 0,
    /// Integers are stored as a f64.
    Double = 1,
    /// Integers are stored as an i64.
    Int64 = 2,
}
