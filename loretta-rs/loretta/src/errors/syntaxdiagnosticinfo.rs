// Ported from Loretta.CodeAnalysis.Lua.SyntaxDiagnosticInfo (b767b4e): SyntaxDiagnosticInfo
// C# source: src/Compilers/Lua/Portable/Errors/SyntaxDiagnosticInfo.cs

use crate::errors::errorcode::ErrorCode;

/// Diagnostic information for syntax errors with position information.
pub struct SyntaxDiagnosticInfo {
    /// The offset in the source text.
    pub offset: usize,
    /// The width of the error span.
    pub width: usize,
    /// The error code.
    pub code: ErrorCode,
    /// The formatted arguments.
    pub arguments: Vec<String>,
}

impl SyntaxDiagnosticInfo {
    /// Creates a new SyntaxDiagnosticInfo with offset, width, code, and arguments.
    pub fn new(offset: usize, width: usize, code: ErrorCode, arguments: Vec<String>) -> Self {
        Self {
            offset,
            width,
            code,
            arguments,
        }
    }

    /// Creates a new SyntaxDiagnosticInfo with offset, width, and code (no arguments).
    pub fn with_code(offset: usize, width: usize, code: ErrorCode) -> Self {
        Self::new(offset, width, code, Vec::new())
    }

    /// Creates a new SyntaxDiagnosticInfo with code and arguments (offset and width default to 0).
    pub fn with_arguments(code: ErrorCode, arguments: Vec<String>) -> Self {
        Self::new(0, 0, code, arguments)
    }

    /// Creates a new SyntaxDiagnosticInfo with code only (offset, width, and arguments default to 0/empty).
    pub fn code_only(code: ErrorCode) -> Self {
        Self::new(0, 0, code, Vec::new())
    }

    /// Returns a copy of this diagnostic with a new offset.
    pub fn with_offset(&self, offset: usize) -> Self {
        Self {
            offset,
            width: self.width,
            code: self.code,
            arguments: self.arguments.clone(),
        }
    }
}
