// Ported from Loretta.CodeAnalysis.Lua.LuaDiagnosticFormatter (b767b4e): LuaDiagnosticFormatter
// C# source: src/Compilers/Lua/Portable/Errors/LuaDiagnosticFormatter.cs

use crate::errors::luadiagnostic::{DiagnosticSeverity, LuaDiagnostic};

/// The Lua diagnostic formatter.
pub struct LuaDiagnosticFormatter;

impl LuaDiagnosticFormatter {
    /// The diagnostic formatter instance.
    pub const INSTANCE: Self = Self;

    /// Formats a diagnostic into the given formatter.
    pub fn format(
        &self,
        diagnostic: &LuaDiagnostic,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        let prefix = match diagnostic.severity {
            DiagnosticSeverity::Error => "error",
            DiagnosticSeverity::Warning => "warning",
            DiagnosticSeverity::Info => "info",
            DiagnosticSeverity::Hidden => "hidden",
            // C# GetMessagePrefix throws UnexpectedValue for these (DiagnosticFormatter.cs:63).
            DiagnosticSeverity::Void | DiagnosticSeverity::Unknown => {
                unreachable!("unexpected severity")
            }
        };
        let code = diagnostic.code as i32;
        write!(f, "{prefix} LUA{code:04}: {}", diagnostic.message)
    }
}
