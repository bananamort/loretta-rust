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

    /// Returns true if the error code is hidden.
    pub fn is_hidden(code: ErrorCode) -> bool {
        matches!(code, ErrorCode::Void | ErrorCode::Unknown)
    }

    /// Gets the severity for the given error code.
    pub fn get_severity(code: ErrorCode) -> DiagnosticSeverity {
        if Self::is_hidden(code) {
            DiagnosticSeverity::Hidden
        } else if Self::is_warning(code) {
            DiagnosticSeverity::Warning
        } else if Self::is_info(code) {
            DiagnosticSeverity::Info
        } else {
            DiagnosticSeverity::Error
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
    pub fn get_category(_code: ErrorCode) -> String {
        "compiler".to_string()
    }
}
