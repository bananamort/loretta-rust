// The C# Lexer.Identifiers.cs diagnostic rules — the LuaJIT identifier gating (the
// C# lexer is DROP per the Port Boundary — only the LUA diagnostic rules are
// re-implemented, see mod.rs).

use super::*;
use crate::errors::errorcode::ErrorCode;

impl<'a> Scanner<'a> {
    /// The C# ScanIdentifier (Lexer.Identifiers.cs:29-173) — the diagnostics
    /// only (the LuaJIT identifier rules).
    pub(crate) fn scan_identifier(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        let mut has_unicode = false;
        loop {
            match self.peek() {
                Some(c) if c.is_ascii_alphabetic() || c == '_' || is_decimal(c) => {
                    self.pos += 1;
                }
                Some(c) if c >= '\u{7F}' => {
                    has_unicode = true;
                    self.pos += 1;
                }
                _ => break,
            }
        }
        if has_unicode && !self.options.use_lua_jit_identifier_rules {
            self.error_current(ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion);
        }
        // A token ends the trivia run — the next run re-arms the shebang
        // guard (the C# per-run init, Lexer.cs:729; Finding 25).
        self.only_shebangs_and_newlines = true;
    }
}
