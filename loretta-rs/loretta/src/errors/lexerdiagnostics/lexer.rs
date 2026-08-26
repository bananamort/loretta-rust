// The C# Lexer.cs diagnostic rules — the dropped lexer's trivia (comments, long
// strings, shebang), bad-token, and token-dispatch rules (Lexer.cs; the C# lexer is
// DROP per the Port Boundary — full_moon is the lexer; only the LUA diagnostic rules
// are re-implemented over the source text, see mod.rs).

use super::*;
use crate::errors::errorcode::ErrorCode;

impl<'a> Scanner<'a> {
    /// The C# TryScanComment + the trivia dispatch (Lexer.cs:751-762): the
    /// `--` comment — either a long comment or a single-line comment.
    pub(crate) fn scan_comment(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        // Skip the leading '--'.
        self.pos += 2;
        let is_long = self.try_scan_long_string();
        if is_long {
            if !self.long_string_terminated {
                self.error_at(
                    self.byte_of_char(start),
                    self.byte_pos() - self.byte_of_char(start),
                    ErrorCode::ErrUnfinishedLongComment,
                    Vec::new(),
                );
            }
        } else {
            // Single-line comment: scan to the end of the line.
            while !self.at_end() {
                let c = self.peek().expect("not at end");
                if is_newline(c) {
                    break;
                }
                self.pos += 1;
            }
        }
        self.only_shebangs_and_newlines = false;
    }

    /// The C# TryScanCComment / ScanMultiLineCComment (Lexer.cs:817-899).
    pub(crate) fn scan_c_comment(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        self.pos += 2; // '/*'
        let mut terminated = false;
        while !self.at_end() {
            if self.peek() == Some('*') && self.peek_at(1) == Some('/') {
                self.pos += 2;
                terminated = true;
                break;
            }
            self.pos += 1;
        }
        if !terminated {
            self.error_at(
                self.byte_of_char(start),
                self.byte_pos() - self.byte_of_char(start),
                ErrorCode::ErrUnfinishedLongComment,
                Vec::new(),
            );
        }
        self.only_shebangs_and_newlines = false;
    }

    /// The C# '[' case (Lexer.cs:302-318): a long string token.
    fn scan_long_string_token(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        if self.try_scan_long_string() && !self.long_string_terminated {
            self.error_current(ErrorCode::ErrUnfinishedString);
        }
        // A token ends the trivia run — the next run re-arms the shebang
        // guard (the C# per-run init, Lexer.cs:729; Finding 25).
        self.only_shebangs_and_newlines = true;
    }

    /// The C# TryScanLongString (Lexer.cs:911-985). Returns whether a long
    /// string was scanned; the `long_string_terminated` field carries the
    /// termination. The Lua51 nesting rule emits its diagnostic during the
    /// scan with the current lexeme extent.
    pub(crate) fn try_scan_long_string(&mut self) -> bool {
        if !matches!(self.peek_at(1), Some('=') | Some('[')) {
            return false;
        }
        self.pos += 1; // the '['
        let initial_equals_count = self.consume_equals();
        if self.peek() != Some('[') {
            return false;
        }
        self.pos += 1;
        // Skips the leading new line if there is one.
        self.skip_leading_newline();
        let mut terminated = false;
        loop {
            match self.peek() {
                None => break,
                Some('[') if !self.options.accept_nesting_of_long_strings => {
                    self.pos += 1;
                    if self.peek() == Some('[') {
                        self.pos += 1;
                        self.error_current(ErrorCode::ErrLua51NestingInLongString);
                    }
                }
                Some(']') => {
                    self.pos += 1;
                    let equals_count = self.consume_equals();
                    if equals_count != initial_equals_count {
                        continue;
                    }
                    if self.peek() != Some(']') {
                        continue;
                    }
                    self.pos += 1;
                    terminated = true;
                    break;
                }
                Some(_) => {
                    self.pos += 1;
                }
            }
        }
        self.long_string_terminated = terminated;
        true
    }

    /// The C# ConsumeCharSequence('=').
    fn consume_equals(&mut self) -> usize {
        let mut count = 0;
        while self.peek() == Some('=') {
            self.pos += 1;
            count += 1;
        }
        count
    }

    /// The C# `_ = ScanEndOfLine()` after the long-string opener.
    fn skip_leading_newline(&mut self) {
        if matches!(self.peek(), Some('\n') | Some('\r')) {
            self.pos += 1;
            let c = self.chars.get(self.pos).copied();
            if matches!(c, Some('\n') | Some('\r')) && c != self.chars.get(self.pos - 1).copied() {
                self.pos += 1;
            }
        }
    }

    /// The C# shebang trivia scan (Lexer.cs:782-793).
    fn scan_shebang(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        while !self.at_end() {
            let c = self.peek().expect("not at end");
            if is_newline(c) {
                break;
            }
            self.pos += 1;
        }
        if !self.options.accept_shebang {
            self.error_at(
                self.byte_of_char(start),
                self.byte_pos() - self.byte_of_char(start),
                ErrorCode::ErrShebangNotSupportedInLuaVersion,
                Vec::new(),
            );
        }
        // The C# keeps the guard through the shebang (Lexer.cs:782-793
        // never touches it) — it only stays true when it was already true
        // (the dispatch gate), so no write is needed (Finding 25).
    }

    /// The bad-character token (the C# ScanToken default) — the lexer emits
    /// ERR_BadCharacter and the parser emits ERR_InvalidStatement on the bad
    /// token (the port emits both, as the reference tests expect).
    fn scan_other(&mut self) {
        let start = self.pos;
        let start_byte = self.byte_of_char(start);
        let count = self.bad_token_count;
        self.bad_token_count += 1;
        if count > 200 {
            // C# Lexer.cs:700-713: past 200 bad tokens the current token
            // absorbs the rest of the input — one BadCharacter with the
            // remainder as the argument, one InvalidStatement over the
            // remainder.
            let text = self.source[start_byte..].to_string();
            let width = self.source.len() - start_byte;
            self.pos = self.chars.len();
            self.error_at(start_byte, width, ErrorCode::ErrBadCharacter, vec![text]);
            self.error_at(
                start_byte,
                width,
                ErrorCode::ErrInvalidStatement,
                Vec::new(),
            );
        } else {
            let c = self.peek().expect("a char");
            self.pos += 1;
            self.error_at(
                start_byte,
                1,
                ErrorCode::ErrBadCharacter,
                vec![c.to_string()],
            );
            self.error_at(start_byte, 1, ErrorCode::ErrInvalidStatement, Vec::new());
        }
        // A token ends the trivia run — the next run re-arms the shebang
        // guard (the C# per-run init, Lexer.cs:729; Finding 25).
        self.only_shebangs_and_newlines = true;
    }
}

/// Scans the source and produces the lexer diagnostics for the options.
pub fn lexer_diagnostics(source: &str, options: &LuaSyntaxOptions) -> Vec<LexerDiagnostic> {
    let mut s = Scanner::new(source, options);
    while !s.at_end() {
        let c = s.peek().expect("not at end");
        if c == ' ' || c == '\t' {
            // C# Lexer.cs:735-739: the space/tab fast path keeps the
            // shebang guard.
            s.pos += 1;
            continue;
        }
        if matches!(c, '\u{0B}' | '\u{0C}') {
            // C# Lexer.cs:743-749: '\v' and '\f' clear the shebang guard
            // (Finding 25).
            s.pos += 1;
            s.only_shebangs_and_newlines = false;
            continue;
        }
        if is_newline(c) {
            // C# Lexer.cs:776-780: the newline trivia KEEPS the guard —
            // it starts true at each trivia run (after every token,
            // Finding 25); the port's old re-arm resurrected it after
            // comments.
            s.scan_end_of_line();
            continue;
        }
        match c {
            '-' if s.peek_at(1) == Some('-') => s.scan_comment(),
            '/' if s.peek_at(1) == Some('*') && s.options.accept_c_comment_syntax => {
                s.scan_c_comment()
            }
            '[' if matches!(s.peek_at(1), Some('=') | Some('[')) => s.scan_long_string_token(),
            '#' if s.only_shebangs_and_newlines && s.peek_at(1) == Some('!') => s.scan_shebang(),
            '"' | '\'' => s.scan_short_string(),
            '`' => s.scan_backtick_string(),
            '0'..='9' => s.scan_number(),
            c if c.is_ascii_alphabetic() || c == '_' || c >= '\u{7F}' => s.scan_identifier(),
            // The valid symbol/operator characters (the C# ScanToken cases) —
            // no diagnostics for the covered tests. The single '&' and '|'
            // have NO lexer rule — the C# parser reports the binary-operator
            // gating (LanguageParser.cs:908-912, ported in
            // parserdiagnostics.rs); the lexer reports only '<<'
            // (Lexer.cs:501-507 — Finding 22). Every TOKEN re-arms the
            // shebang guard for the next trivia run (the C# per-run init,
            // Lexer.cs:729 — Finding 25).
            '&' | '|' | '#' | '+' | '-' | '*' | '/' | '^' | '=' | '~' | '%' | ',' | '.' | ':'
            | ';' | '?' | '!' | '(' | ')' | '[' | ']' | '{' | '}' => {
                s.pos += 1;
                s.only_shebangs_and_newlines = true;
            }
            '<' => {
                s.pos += 1;
                if s.peek() == Some('<') {
                    // C# Lexer.cs:501-507: the '<<' token carries the
                    // bitwise gating (the C# AddError at the lexeme extent
                    // — both characters).
                    s.pos += 1;
                    if !s.options.accept_bitwise_operators {
                        s.error_at(
                            s.byte_pos() - 2,
                            2,
                            ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion,
                            Vec::new(),
                        );
                    }
                }
                s.only_shebangs_and_newlines = true;
            }
            '>' => {
                s.pos += 1;
                s.only_shebangs_and_newlines = true;
            }
            _ => s.scan_other(),
        }
    }
    s.diagnostics
}
