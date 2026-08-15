// Ported from Loretta.CodeAnalysis.Lua.LuaDiagnosticFormatter (b767b4e): LuaDiagnosticFormatter
// C# source: src/Compilers/Lua/Portable/Errors/LuaDiagnosticFormatter.cs
// NOTE: DiagnosticFormatter is from dropped Core infrastructure.

/// The Lua diagnostic formatter.
pub struct LuaDiagnosticFormatter;

impl LuaDiagnosticFormatter {
    /// The diagnostic formatter instance.
    pub const INSTANCE: Self = Self;
}
