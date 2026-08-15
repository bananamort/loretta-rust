// Ported from Loretta.CodeAnalysis.Lua.LuaDiagnostic (b767b4e): LuaDiagnostic
// C# source: src/Compilers/Lua/Portable/Errors/LuaDiagnostic.cs

use crate::errors::errorcode::ErrorCode;
use crate::errors::luadiagnosticformatter::LuaDiagnosticFormatter;

/// Severity level for diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// C# InternalDiagnosticSeverity.Void (cast of InternalErrorCode.Void).
    Void = -2,
    /// C# InternalDiagnosticSeverity.Unknown (cast of InternalErrorCode.Unknown).
    Unknown = -1,
    Hidden = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

/// A Lua-specific diagnostic.
pub struct LuaDiagnostic {
    /// The error code for this diagnostic.
    pub code: ErrorCode,
    /// The diagnostic message.
    pub message: String,
    /// The severity of the diagnostic.
    pub severity: DiagnosticSeverity,
    /// Whether this diagnostic is suppressed.
    pub is_suppressed: bool,
}

impl LuaDiagnostic {
    /// Creates a new LuaDiagnostic.
    pub fn new(
        code: ErrorCode,
        message: String,
        severity: DiagnosticSeverity,
        is_suppressed: bool,
    ) -> Self {
        Self {
            code,
            message,
            severity,
            is_suppressed,
        }
    }

    /// Returns a copy of this diagnostic with the given severity.
    pub fn with_severity(&self, severity: DiagnosticSeverity) -> Self {
        if self.severity != severity {
            Self::new(
                self.code,
                self.message.clone(),
                severity,
                self.is_suppressed,
            )
        } else {
            self.clone()
        }
    }

    /// Returns a copy of this diagnostic with the given suppression status.
    pub fn with_is_suppressed(&self, is_suppressed: bool) -> Self {
        if self.is_suppressed != is_suppressed {
            Self::new(
                self.code,
                self.message.clone(),
                self.severity,
                is_suppressed,
            )
        } else {
            self.clone()
        }
    }
}

impl Clone for LuaDiagnostic {
    fn clone(&self) -> Self {
        Self {
            code: self.code,
            message: self.message.clone(),
            severity: self.severity,
            is_suppressed: self.is_suppressed,
        }
    }
}

impl std::fmt::Display for LuaDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        LuaDiagnosticFormatter::INSTANCE.format(self, f)
    }
}
