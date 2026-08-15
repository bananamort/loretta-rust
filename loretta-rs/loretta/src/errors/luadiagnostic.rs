// Ported from Loretta.CodeAnalysis.Lua.LuaDiagnostic (b767b4e): LuaDiagnostic
// C# source: src/Compilers/Lua/Portable/Errors/LuaDiagnostic.cs
// NOTE: DiagnosticWithInfo, DiagnosticInfo, Location are from dropped Core infrastructure.
// Simplified to standalone struct with error code and message.

use crate::errors::errorcode::ErrorCode;

/// A Lua-specific diagnostic.
pub struct LuaDiagnostic {
    /// The error code for this diagnostic.
    pub code: ErrorCode,
    /// The diagnostic message.
    pub message: String,
    /// Whether this diagnostic is suppressed.
    pub is_suppressed: bool,
}

impl LuaDiagnostic {
    /// Creates a new LuaDiagnostic.
    pub fn new(code: ErrorCode, message: String, is_suppressed: bool) -> Self {
        Self {
            code,
            message,
            is_suppressed,
        }
    }

    /// Returns a copy of this diagnostic with the given suppression status.
    pub fn with_is_suppressed(&self, is_suppressed: bool) -> Self {
        if self.is_suppressed != is_suppressed {
            Self::new(self.code, self.message.clone(), is_suppressed)
        } else {
            Self {
                code: self.code,
                message: self.message.clone(),
                is_suppressed: self.is_suppressed,
            }
        }
    }
}

impl std::fmt::Display for LuaDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lua{:04}: {}", self.code as i32, self.message)
    }
}
