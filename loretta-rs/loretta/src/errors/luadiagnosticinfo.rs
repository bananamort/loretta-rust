// Ported from Loretta.CodeAnalysis.Lua.LuaDiagnosticInfo (b767b4e): LuaDiagnosticInfo
// C# source: src/Compilers/Lua/Portable/Errors/LuaDiagnosticInfo.cs

use crate::errors::errorcode::ErrorCode;

/// Diagnostic information for Lua errors.
pub struct LuaDiagnosticInfo {
    /// The error code.
    pub code: ErrorCode,
    /// The formatted arguments.
    pub arguments: Vec<String>,
    /// Whether this is a warning being treated as an error.
    pub is_warning_as_error: bool,
}

impl LuaDiagnosticInfo {
    /// Creates a new LuaDiagnosticInfo with no arguments.
    pub fn new(code: ErrorCode) -> Self {
        Self::with_arguments(code, Vec::new())
    }

    /// Creates a new LuaDiagnosticInfo with arguments.
    pub fn with_arguments(code: ErrorCode, arguments: Vec<String>) -> Self {
        Self {
            code,
            arguments,
            is_warning_as_error: false,
        }
    }

    /// Creates a new LuaDiagnosticInfo with warning-as-error flag.
    pub fn with_warning_as_error(
        is_warning_as_error: bool,
        code: ErrorCode,
        arguments: Vec<String>,
    ) -> Self {
        Self {
            code,
            arguments,
            is_warning_as_error,
        }
    }
}
