// Ported from Loretta.CodeAnalysis.Lua.ErrorFacts (b767b4e): ErrorFacts
// C# source: src/Compilers/Lua/Portable/Errors/ErrorFacts.cs

use crate::errors::errorcode::ErrorCode;
use crate::errors::luadiagnostic::DiagnosticSeverity;
use crate::errors::messageprovider::MessageProvider;

/// Utility methods for error codes.
pub struct ErrorFacts;

impl ErrorFacts {
    /// Gets the diagnostic ID for the given error code.
    pub fn get_id(code: ErrorCode) -> String {
        format!("LUA{:04}", code as i32)
    }

    /// Returns true if the error code is a warning.
    pub fn is_warning(code: ErrorCode) -> bool {
        matches!(code, ErrorCode::WrnLineBreakMayAffectErrorReporting)
    }

    /// Returns true if the error code is fatal.
    pub fn is_fatal(_code: ErrorCode) -> bool {
        false
    }

    /// Returns true if the error code is info.
    pub fn is_info(_code: ErrorCode) -> bool {
        false
    }

    /// The C# generated IsHidden (ErrorFacts.g.cs:34-41) — the switch has
    /// no cases, so every code (Void and Unknown included) is false
    /// (Finding 45; the port's Void/Unknown match was a fabrication).
    pub fn is_hidden(_code: ErrorCode) -> bool {
        false
    }

    /// Gets the severity for the given error code.
    pub fn get_severity(code: ErrorCode) -> DiagnosticSeverity {
        match code {
            ErrorCode::Void => DiagnosticSeverity::Void,
            ErrorCode::Unknown => DiagnosticSeverity::Unknown,
            _ if Self::is_warning(code) => DiagnosticSeverity::Warning,
            _ if Self::is_info(code) => DiagnosticSeverity::Info,
            _ if Self::is_hidden(code) => DiagnosticSeverity::Hidden,
            _ => DiagnosticSeverity::Error,
        }
    }

    /// Gets the message for the given error code.
    pub fn get_message(code: ErrorCode) -> String {
        MessageProvider::get_message_format(code)
    }

    /// Gets the message format for the given error code.
    pub fn get_message_format(code: ErrorCode) -> String {
        MessageProvider::get_message_format(code)
    }

    /// Gets the title for the given error code.
    pub fn get_title(code: ErrorCode) -> String {
        MessageProvider::get_title(code)
    }

    /// Gets the description for the given error code.
    pub fn get_description(code: ErrorCode) -> String {
        MessageProvider::get_description(code)
    }

    /// Gets the category for the given error code.
    /// C# fallback is `Diagnostic.CompilerDiagnosticCategory = "Compiler"`
    /// (Core Diagnostic.cs) since the categories map is always empty.
    pub fn get_category(_code: ErrorCode) -> String {
        "Compiler".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_hidden_is_false_for_every_code() {
        // Finding 45: the C# generated IsHidden has no cases — all codes
        // (Void and Unknown included) are false (ErrorFacts.g.cs:34-41).
        assert!(!ErrorFacts::is_hidden(ErrorCode::Void));
        assert!(!ErrorFacts::is_hidden(ErrorCode::Unknown));
        assert!(!ErrorFacts::is_hidden(ErrorCode::ErrInvalidStringEscape));
        assert!(!ErrorFacts::is_hidden(
            ErrorCode::WrnLineBreakMayAffectErrorReporting
        ));
    }
}
