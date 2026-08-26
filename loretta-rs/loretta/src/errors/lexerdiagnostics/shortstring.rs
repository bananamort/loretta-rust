// The C# Lexer.ShortString.cs diagnostic rules — string literals, escape sequences,
// unicode escapes, and the backtick (interpolated) string hole scanner (the C# lexer
// is DROP per the Port Boundary — only the LUA diagnostic rules are re-implemented,
// see mod.rs).

use super::*;
use crate::backtickstringtype::BacktickStringType;
use crate::errors::errorcode::ErrorCode;

impl<'a> Scanner<'a> {
    /// The C# ScanStringLiteral (Lexer.ShortString.cs:8-52) — the diagnostics
    /// only (the escape rules + the unfinished-string error).
    pub(crate) fn scan_short_string(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        let quote = self.peek().expect("a quote");
        self.pos += 1; // the quote
        loop {
            match self.peek() {
                None => {
                    self.error_current(ErrorCode::ErrUnfinishedString);
                    break;
                }
                Some(c) if is_newline(c) => {
                    self.error_current(ErrorCode::ErrUnfinishedString);
                    break;
                }
                Some(c) if c == quote => {
                    self.pos += 1;
                    break;
                }
                Some('\\') => self.scan_escape_sequence(),
                Some(_) => {
                    self.pos += 1;
                }
            }
        }
        // A token ends the trivia run — the next run re-arms the shebang
        // guard (the C# per-run init, Lexer.cs:729; Finding 25).
        self.only_shebangs_and_newlines = true;
    }

    /// The C# ScanEscapeSequence (Lexer.ShortString.cs:104-299) — the
    /// diagnostics only.
    fn scan_escape_sequence(&mut self) {
        let escape_start = self.pos;
        self.pos += 1; // the '\'
        let Some(ch) = self.peek() else {
            return;
        };
        self.pos += 1;
        match ch {
            'a' | 'b' | 'f' | 'n' | 'r' | 't' | 'v' | '\\' | '\'' | '"' => {}
            '\n' | '\r' => {
                // The escaped newline may swallow its pair.
                if let Some(next) = self.peek() {
                    if is_newline(next) && next != ch {
                        self.pos += 1;
                    }
                }
            }
            'z' => {
                // The C# skips the whitespace, then reports the width.
                while let Some(c) = self.peek() {
                    if is_whitespace(c) {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
                if !self.options.accept_invalid_escapes && !self.options.accept_whitespace_escape {
                    self.error_at(
                        self.byte_of_char(escape_start),
                        self.byte_pos() - self.byte_of_char(escape_start),
                        ErrorCode::ErrWhitespaceEscapeNotSupportedInVersion,
                        Vec::new(),
                    );
                }
            }
            '0'..='9' => {
                // The C# ParseDecimalInteger — up to three digits; > 255 is
                // an invalid escape.
                let mut num = decimal_value(ch);
                let mut read = 1;
                while read < 3 {
                    match self.peek() {
                        Some(c) if is_decimal(c) => {
                            self.pos += 1;
                            num = num * 10 + decimal_value(c);
                            read += 1;
                        }
                        _ => break,
                    }
                }
                if num > 255 {
                    self.error_at(
                        self.byte_of_char(escape_start),
                        self.byte_pos() - self.byte_of_char(escape_start),
                        ErrorCode::ErrInvalidStringEscape,
                        Vec::new(),
                    );
                }
            }
            'x' => {
                // The C# \x (ShortString.cs:166-182): under
                // AcceptInvalidEscapes && !AcceptHexEscapesInStrings the
                // C# jumps to the default case (the silent echo) BEFORE
                // the hex-digit parsing, so the missing-digit
                // ErrInvalidStringEscape never fires (Finding 43).
                let hex_silent = self.options.accept_invalid_escapes
                    && !self.options.accept_hex_escapes_in_strings;
                // The C# ParseHexadecimalEscapeInteger — up to two digits.
                let mut read = 0;
                while read < 2 {
                    match self.peek() {
                        Some(c) if is_hexadecimal(c) => {
                            self.pos += 1;
                            read += 1;
                        }
                        _ => break,
                    }
                }
                if read < 1 && !hex_silent {
                    self.error_at(
                        self.byte_of_char(escape_start),
                        self.byte_pos() - self.byte_of_char(escape_start),
                        ErrorCode::ErrInvalidStringEscape,
                        Vec::new(),
                    );
                }
                if !self.options.accept_invalid_escapes
                    && !self.options.accept_hex_escapes_in_strings
                {
                    self.error_at(
                        self.byte_of_char(escape_start),
                        self.byte_pos() - self.byte_of_char(escape_start),
                        ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
                        Vec::new(),
                    );
                }
            }
            'u' => self.scan_unicode_escape(escape_start),
            _ => {
                if !self.options.accept_invalid_escapes {
                    self.error_at(
                        self.byte_of_char(escape_start),
                        self.byte_pos() - self.byte_of_char(escape_start),
                        ErrorCode::ErrInvalidStringEscape,
                        Vec::new(),
                    );
                }
            }
        }
    }

    /// The C# ParseUnicodeEscape (Lexer.ShortString.cs:265-298).
    fn scan_unicode_escape(&mut self, escape_start: usize) {
        let missing_opening_brace = self.peek() != Some('{');
        if !missing_opening_brace {
            self.pos += 1;
        }
        // Up to sixteen hex digits.
        let mut read = 0;
        let mut codepoint: u64 = 0;
        while read < 16 {
            match self.peek() {
                Some(c) if is_hexadecimal(c) => {
                    self.pos += 1;
                    codepoint = (codepoint << 4)
                        | (if is_decimal(c) {
                            decimal_value(c) as u64
                        } else {
                            (c.to_ascii_lowercase() as u64) - ('a' as u64) + 10
                        });
                    read += 1;
                }
                _ => break,
            }
        }
        // The C# ParseHexadecimalNumber reports the missing-digit error at
        // its own failure point (before the closing-brace handling).
        if read < 1 {
            self.error_at(
                self.byte_of_char(escape_start),
                self.byte_pos() - self.byte_of_char(escape_start),
                ErrorCode::ErrHexDigitExpected,
                Vec::new(),
            );
        }
        let missing_closing_brace = self.peek() != Some('}');
        if !missing_closing_brace {
            self.pos += 1;
        }
        if missing_opening_brace {
            self.error_at(
                self.byte_of_char(escape_start),
                self.byte_pos() - self.byte_of_char(escape_start),
                ErrorCode::ErrUnicodeEscapeMissingOpenBrace,
                Vec::new(),
            );
        }
        if missing_closing_brace {
            self.error_at(
                self.byte_of_char(escape_start),
                self.byte_pos() - self.byte_of_char(escape_start),
                ErrorCode::ErrUnicodeEscapeMissingCloseBrace,
                Vec::new(),
            );
        }
        if codepoint > 0x10FFFF {
            self.error_at(
                self.byte_of_char(escape_start),
                self.byte_pos() - self.byte_of_char(escape_start),
                ErrorCode::ErrEscapeTooLarge,
                vec!["10FFFF".to_string()],
            );
        }
        if !self.options.accept_invalid_escapes && !self.options.accept_unicode_escape {
            self.error_at(
                self.byte_of_char(escape_start),
                self.byte_pos() - self.byte_of_char(escape_start),
                ErrorCode::ErrUnicodeEscapesNotSupportedLuaInVersion,
                Vec::new(),
            );
        }
    }

    /// The C# ScanInterpolatedStringLiteral diagnostics — the unfinished
    /// error (span = the last character) and the version gating.
    pub(crate) fn scan_backtick_string(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        self.pos += 1; // the '`'
                       // The C# sub-scanner's first-error slot — one PER
                       // ScanInterpolatedStringLiteral invocation (each nested recursion
                       // has its own InterpolatedStringScanner, Lexer.ShortString.cs:312).
        let mut sub_error: Option<(usize, usize, ErrorCode, Vec<String>)> = None;
        let mut unfinished = false;
        loop {
            match self.peek() {
                None => {
                    unfinished = true;
                    break;
                }
                Some(c) if is_newline(c) => {
                    unfinished = true;
                    break;
                }
                Some('`') => {
                    self.pos += 1;
                    break;
                }
                Some('{') => {
                    // The C# HandleOpenBraceInContent +
                    // ScanInterpolatedStringLiteralHoleBalancedText
                    // (Lexer.ShortString.cs:437-580): '{{' reports
                    // LUA0035; an unclosed hole reports LUA0034; a
                    // mismatched closer reports ERR_SyntaxError with the
                    // expected-char argument. The TrySetError semantics
                    // keep the FIRST error only.
                    self.scan_backtick_hole(&mut sub_error);
                }
                Some('\\') => {
                    // The C# InterpolatedStringScanner contents loop runs
                    // ScanEscapeSequence per '\' (Lexer.ShortString.cs:
                    // 424-427) — "handle escapes but not care about their
                    // issues" — and ScanEscapeSequence's AddError calls
                    // still land on the token, so the escape diagnostics
                    // fire inside backtick strings too (AUDIT.md Finding
                    // 1(a); probed @Lua54 'local x = `a\qb`': C#
                    // [LUA0036, LUA0001, LUA0001]).
                    self.scan_escape_sequence();
                }
                Some(_) => {
                    self.pos += 1;
                }
            }
        }
        // The C# ScanInterpolatedStringLiteralEnd (Lexer.ShortString.cs:
        // 376-403): the missing-close-quote error goes through TrySetError —
        // first-wins against any hole or nested-scan error recorded earlier
        // on this level's slot.
        if unfinished {
            Self::try_set_error(
                &mut sub_error,
                self.byte_pos().saturating_sub(1),
                1,
                ErrorCode::ErrUnfinishedString,
                Vec::new(),
            );
        }
        if let Some((error_start, error_width, error_code, error_args)) = sub_error.take() {
            // The C# AddError(error) (Lexer.ShortString.cs:70): the sub-
            // scanner's first error, emitted BEFORE the gate (:70 vs
            // :71-72).
            self.error_at(error_start, error_width, error_code, error_args);
        }
        // The C# gate (Lexer.ShortString.cs:71-72) fires UNCONDITIONALLY
        // after the scan for non-InterpolatedStringLiteral presets — the
        // finished AND unfinished paths alike (AUDIT.md Finding 1(d)). The
        // two paths diverge in WHERE the port emits it:
        // - Unfinished: the token survives into statement position, so this
        //   scanner mirrors the LEXER copy (the harness's token pass doubles
        //   it like the C# tree+token passes).
        // - Finished: in an expression context the C# parser supersedes the
        //   token copy with its node-level error
        //   (LanguageParser.InterpolatedString.cs:59-60) and the reference
        //   reports LUA0036 once — so the finished-path emission lives in
        //   parserdiagnostics.rs (the single-report pass) instead.
        if unfinished && self.options.backtick_string_type == BacktickStringType::None {
            self.error_at(
                self.byte_of_char(start),
                self.byte_pos() - self.byte_of_char(start),
                ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
                Vec::new(),
            );
        }
        // A token ends the trivia run — the next run re-arms the shebang
        // guard (the C# per-run init, Lexer.cs:729; Finding 25).
        self.only_shebangs_and_newlines = true;
    }

    /// The C# InterpolatedStringScanner's hole scanning
    /// (Lexer.ShortString.cs:437-580): HandleOpenBraceInContent +
    /// ScanInterpolatedStringLiteralHoleBalancedText +
    /// ScanInterpolatedStringLiteralHoleBracketed. Diagnostics obey the
    /// TrySetError first-error-wins rule on this scan's `sub_error` slot.
    fn scan_backtick_hole(
        &mut self,
        sub_error: &mut Option<(usize, usize, ErrorCode, Vec<String>)>,
    ) {
        let open_brace_position = self.pos;
        self.pos += 1; // the '{'
        if self.peek() == Some('{') {
            self.pos += 1;
            // MakeError(openBracePosition - 1, width: 2).
            let start = self.byte_of_char(open_brace_position.saturating_sub(1));
            Self::try_set_error(
                sub_error,
                start,
                2,
                ErrorCode::ErrDoubleBraceInInterpolation,
                Vec::new(),
            );
            return;
        }
        // ScanInterpolatedStringLiteralHoleBalancedText(endingChar: '}').
        self.scan_hole_balanced_text('}', sub_error);
        let close_brace_position = self.pos;
        if self.peek() == Some('}') {
            self.pos += 1;
        } else {
            // MakeError(openBracePosition - 1, width: 2).
            let start = self.byte_of_char(open_brace_position.saturating_sub(1));
            Self::try_set_error(
                sub_error,
                start,
                2,
                ErrorCode::ErrUnclosedExpressionHole,
                Vec::new(),
            );
        }
        let _ = close_brace_position;
    }

    /// The C# ScanInterpolatedStringLiteralHoleBalancedText
    /// (Lexer.ShortString.cs:470-558). Newlines are always allowed inside a
    /// hole. Nested backtick strings recurse through
    /// ScanInterpolatedStringLiteral (whose gate fires per the preset);
    /// quoted strings run ScanStringLiteral (the escape diagnostics fire).
    fn scan_hole_balanced_text(
        &mut self,
        ending_char: char,
        sub_error: &mut Option<(usize, usize, ErrorCode, Vec<String>)>,
    ) {
        loop {
            match self.peek() {
                None => return,
                Some(c) if is_newline(c) => {
                    // IsAtEnd(allowNewline: true) inside a hole: only a real
                    // EOF stops the scan.
                    if self.pos + 1 >= self.chars.len() {
                        return;
                    }
                    self.pos += 1;
                    continue;
                }
                Some(c) => {
                    if c == '`' {
                        // The C# recurses into ScanInterpolatedStringLiteral
                        // for a nested interpolated string — which owns its
                        // own first-error slot.
                        self.scan_backtick_string();
                        continue;
                    }
                    if c == ending_char || c == '}' || c == ')' || c == ']' {
                        if c == ending_char {
                            return;
                        }
                        // MakeError(Position, width: 1, ERR_SyntaxError,
                        // endingChar) — then consume it.
                        let start = self.byte_pos();
                        Self::try_set_error(
                            sub_error,
                            start,
                            1,
                            ErrorCode::ErrSyntaxError,
                            vec![ending_char.to_string()],
                        );
                        self.pos += 1;
                        continue;
                    }
                    match c {
                        '"' | '\'' => {
                            // RecoveringFromRunawayLexing: after an error the
                            // next quote ends the string scan (Lexer.
                            // ShortString.cs:506-514); otherwise the nested
                            // string's escape diagnostics fire.
                            if sub_error.is_some() {
                                return;
                            }
                            self.scan_short_string();
                            continue;
                        }
                        '/' if self.options.accept_c_comment_syntax
                            && self.peek_at(1) == Some('*') =>
                        {
                            self.scan_c_comment();
                            continue;
                        }
                        '/' => {
                            self.pos += 1;
                            continue;
                        }
                        '-' => {
                            // TryScanComment: '--' comments; anything else
                            // consumes one char.
                            if self.peek_at(1) == Some('-') {
                                self.scan_comment();
                            } else {
                                self.pos += 1;
                            }
                            continue;
                        }
                        '{' => {
                            self.scan_hole_bracketed('{', '}', sub_error);
                            continue;
                        }
                        '(' => {
                            self.scan_hole_bracketed('(', ')', sub_error);
                            continue;
                        }
                        '[' => {
                            if matches!(self.peek_at(1), Some('=') | Some('[')) {
                                let was_terminated = self.try_scan_long_string();
                                let _ = was_terminated;
                            } else {
                                self.scan_hole_bracketed('[', ']', sub_error);
                            }
                            continue;
                        }
                        _ => {
                            self.pos += 1;
                            continue;
                        }
                    }
                }
            }
        }
    }

    /// The C# ScanInterpolatedStringLiteralHoleBracketed
    /// (Lexer.ShortString.cs:573-580).
    fn scan_hole_bracketed(
        &mut self,
        start_char: char,
        end_char: char,
        sub_error: &mut Option<(usize, usize, ErrorCode, Vec<String>)>,
    ) {
        debug_assert_eq!(self.peek(), Some(start_char));
        self.pos += 1;
        self.scan_hole_balanced_text(end_char, sub_error);
        if self.peek() == Some(end_char) {
            self.pos += 1;
        }
    }

    /// The C# InterpolatedStringScanner.TrySetError (Lexer.ShortString.cs:
    /// 321-323): only the FIRST error is recorded.
    fn try_set_error(
        sub_error: &mut Option<(usize, usize, ErrorCode, Vec<String>)>,
        start: usize,
        width: usize,
        code: ErrorCode,
        args: Vec<String>,
    ) {
        if sub_error.is_none() {
            *sub_error = Some((start, width, code, args));
        }
    }
}
