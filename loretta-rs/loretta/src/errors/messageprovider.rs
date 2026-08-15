// Ported from Loretta.CodeAnalysis.Lua.MessageProvider (b767b4e): MessageProvider
// C# source: src/Compilers/Lua/Portable/Errors/MessageProvider.cs
// NOTE: CommonMessageProvider, IObjectWritable, ObjectWriter are from dropped Core infrastructure.

use crate::errors::errorcode::ErrorCode;
use crate::errors::luadiagnostic::LuaDiagnostic;

/// Provides messages for Lua diagnostics.
pub struct MessageProvider;

impl MessageProvider {
    /// The singleton instance.
    pub const INSTANCE: Self = Self;

    /// The code prefix for Lua diagnostics.
    pub const CODE_PREFIX: &'static str = "LUA";

    /// Creates a diagnostic from an error code and arguments.
    pub fn create_diagnostic(code: ErrorCode, message: String) -> LuaDiagnostic {
        LuaDiagnostic::new(code, message, false)
    }

    /// Gets the message prefix for a diagnostic.
    pub fn get_message_prefix(id: &str, is_error: bool) -> String {
        if is_error {
            format!("error {id}")
        } else {
            format!("warning {id}")
        }
    }
}
