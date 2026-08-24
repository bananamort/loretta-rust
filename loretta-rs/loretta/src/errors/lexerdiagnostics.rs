// Ported from Loretta.CodeAnalysis.Lua.Syntax.InternalSyntax.LuaLexer (b767b4e) — the dropped
// lexer's DIAGNOSTIC rules, reimplemented over the source text (the C# lexer lives in
// Portable/Parser/, which is DROP per the Port Boundary; the port's lexer is full_moon, which
// does not emit the LUA codes). This module mirrors the C# Lexer.cs / Lexer.ShortString.cs /
// Lexer.Numbers.cs / Lexer.Identifiers.cs error rules char-for-char so the LexicalErrorTests
// (row 773+) can verify the exact codes, spans, squiggles, and arguments, and so the
// differential's diagnostics op can reproduce the C# reference's per-preset version-gating
// diagnostics (audit finding B — the 50 pending cases).
//
// Span semantics (AbstractLexer.cs:52-80): `AddError(code)` captures the CURRENT lexeme
// extent (lexeme start .. current position); explicit-position errors are token-relative in
// the C# and become absolute byte offsets here.

use crate::backtickstringtype::BacktickStringType;
use crate::errors::errorcode::ErrorCode;
use crate::integerformats::IntegerFormats;
use crate::luasyntaxoptions::LuaSyntaxOptions;

/// A produced lexer diagnostic (the C# SyntaxDiagnosticInfo).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexerDiagnostic {
    pub code: ErrorCode,
    /// The absolute byte offset of the span start.
    pub start: usize,
    /// The byte width of the span.
    pub width: usize,
    /// The message arguments (the C# params object[] args).
    pub arguments: Vec<String>,
    /// Whether the code is a warning (the C# WRN_* codes).
    pub is_warning: bool,
}

impl LexerDiagnostic {
    /// The squiggled text (the source over the span) — the C#
    /// DiagnosticDescription compares it.
    pub fn squiggle<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.start + self.width]
    }

    /// The 1-based (line, column) start position (the C# WithLocation).
    /// Columns count chars (the C# UTF-16 units; the test sources are ASCII
    /// ahead of every expected span); `\r`, `\r\n` and `\n` each count as a
    /// single line break (the C# SourceText lines).
    pub fn line_col(&self, source: &str) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        let mut prev_cr = false;
        for (i, ch) in source.char_indices() {
            if i >= self.start {
                break;
            }
            if ch == '\r' {
                line += 1;
                col = 1;
                prev_cr = true;
            } else if ch == '\n' {
                if !prev_cr {
                    line += 1;
                    col = 1;
                }
                prev_cr = false;
            } else {
                col += 1;
                prev_cr = false;
            }
        }
        (line, col)
    }
}

/// The scanner state (the C# TextWindow).
struct Scanner<'a> {
    source: &'a str,
    chars: Vec<char>,
    /// The byte offset of each char.
    byte_of: Vec<usize>,
    pos: usize,
    /// The current lexeme's start char index (the C# LexemeStartPosition).
    lexeme_start: usize,
    diagnostics: Vec<LexerDiagnostic>,
    options: &'a LuaSyntaxOptions,
    /// Whether only shebangs and newlines were seen so far (the C#
    /// onlyShebangsAndNewlines in the trivia scanner).
    only_shebangs_and_newlines: bool,
    /// Whether the last long-string scan was terminated (the C# out
    /// isTerminated).
    long_string_terminated: bool,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str, options: &'a LuaSyntaxOptions) -> Self {
        let chars: Vec<char> = source.chars().collect();
        let mut byte_of = Vec::with_capacity(chars.len());
        let mut offset = 0;
        for c in &chars {
            byte_of.push(offset);
            offset += c.len_utf8();
        }
        Scanner {
            source,
            chars,
            byte_of,
            pos: 0,
            lexeme_start: 0,
            diagnostics: Vec::new(),
            options,
            only_shebangs_and_newlines: true,
            long_string_terminated: false,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_at(&self, delta: usize) -> Option<char> {
        self.chars.get(self.pos + delta).copied()
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn byte_pos(&self) -> usize {
        self.byte_of
            .get(self.pos)
            .copied()
            .unwrap_or(self.source.len())
    }

    fn byte_of_char(&self, index: usize) -> usize {
        self.byte_of
            .get(index)
            .copied()
            .unwrap_or(self.source.len())
    }

    /// The current lexeme's byte extent (the C# TextWindow.Width at the
    /// current position).
    fn lexeme_width(&self) -> usize {
        self.byte_pos() - self.byte_of_char(self.lexeme_start)
    }

    /// The C# AddError(position, width, code, args) — absolute byte offsets.
    fn error_at(&mut self, start: usize, width: usize, code: ErrorCode, args: Vec<String>) {
        self.diagnostics.push(LexerDiagnostic {
            code,
            start,
            width,
            arguments: args,
            is_warning: matches!(code, ErrorCode::WrnLineBreakMayAffectErrorReporting),
        });
    }

    /// The C# AddError(code) — the current lexeme extent.
    fn error_current(&mut self, code: ErrorCode) {
        let start = self.byte_of_char(self.lexeme_start);
        let width = self.lexeme_width();
        self.error_at(start, width, code, Vec::new());
    }

    /// The C# ScanEndOfLine — emits the exotic line-break warning for `\n\r`.
    fn scan_end_of_line(&mut self) {
        self.lexeme_start = self.pos;
        match self.peek() {
            Some('\n') => {
                self.pos += 1;
                if self.peek() == Some('\r') {
                    self.pos += 1;
                    self.error_current(ErrorCode::WrnLineBreakMayAffectErrorReporting);
                }
            }
            Some('\r') => {
                self.pos += 1;
                if self.peek() == Some('\n') {
                    self.pos += 1;
                }
            }
            _ => {}
        }
    }

    /// The C# TryScanComment + the trivia dispatch (Lexer.cs:751-762): the
    /// `--` comment — either a long comment or a single-line comment.
    fn scan_comment(&mut self) {
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
    fn scan_c_comment(&mut self) {
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
        self.only_shebangs_and_newlines = false;
    }

    /// The C# TryScanLongString (Lexer.cs:911-985). Returns whether a long
    /// string was scanned; the `long_string_terminated` field carries the
    /// termination. The Lua51 nesting rule emits its diagnostic during the
    /// scan with the current lexeme extent.
    fn try_scan_long_string(&mut self) -> bool {
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
        self.only_shebangs_and_newlines = false;
    }

    /// The C# ScanStringLiteral (Lexer.ShortString.cs:8-52) — the diagnostics
    /// only (the escape rules + the unfinished-string error).
    fn scan_short_string(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        let quote = self.peek().expect("a quote");
        self.pos += 1; // the quote
        loop {
            match self.peek() {
                None => {
                    self.error_current(ErrorCode::ErrUnfinishedString);
                    return;
                }
                Some(c) if is_newline(c) => {
                    self.error_current(ErrorCode::ErrUnfinishedString);
                    return;
                }
                Some(c) if c == quote => {
                    self.pos += 1;
                    return;
                }
                Some('\\') => self.scan_escape_sequence(),
                Some(_) => {
                    self.pos += 1;
                }
            }
        }
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
                if read < 1 {
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
    fn scan_backtick_string(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        self.pos += 1; // the '`'
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
                Some('\\') => {
                    // The escaped character (incl. the escaped newline) does
                    // not end the string (the C# InterpolatedStringScanner).
                    self.pos += 1;
                    if !self.at_end() {
                        self.pos += 1;
                    }
                }
                Some(_) => {
                    self.pos += 1;
                }
            }
        }
        if unfinished {
            // The C#: MakeError(Position - 1, width: 1) — the last character.
            self.error_at(
                self.byte_pos().saturating_sub(1),
                1,
                ErrorCode::ErrUnfinishedString,
                Vec::new(),
            );
            return;
        }
        // The C#: the interpolated gating error fires only for the None
        // backtick type (the HashLiteral strings use the short-string
        // scanner — Lexer.cs:622-625; Lexer.ShortString.cs:71-72).
        if self.options.backtick_string_type == BacktickStringType::None {
            self.error_at(
                self.byte_of_char(start),
                self.byte_pos() - self.byte_of_char(start),
                ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
                Vec::new(),
            );
        }
        self.only_shebangs_and_newlines = false;
    }

    /// The C# number scanning (Lexer.Numbers.cs) — the diagnostics only.
    fn scan_number(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        let c1 = self.peek_at(1);
        match (self.peek().expect("a digit"), c1) {
            ('0', Some('x') | Some('X')) => {
                self.pos += 2;
                self.scan_hex_number();
            }
            ('0', Some('b') | Some('B')) => {
                self.pos += 2;
                self.scan_binary_number();
            }
            ('0', Some('o') | Some('O')) => {
                self.pos += 2;
                self.scan_octal_number();
            }
            _ => self.scan_decimal_number(),
        }
        self.only_shebangs_and_newlines = false;
        let _ = start;
    }

    fn scan_hex_number(&mut self) {
        let mut has_underscores = false;
        let mut digits = 0usize;
        let mut is_hex_float = false;
        let mut num: u64 = 0;
        // Set when a shift would exceed 64 bits (the C# TryParse failure
        // on values wider than u64::MAX, Lexer.Numbers.cs:364-378, 415-428;
        // Finding 20).
        let mut overflowed = false;
        loop {
            match self.peek() {
                Some(c) if is_hexadecimal(c) => {
                    self.pos += 1;
                    if num > u64::MAX >> 4 {
                        overflowed = true;
                    }
                    num = (num << 4)
                        | (if is_decimal(c) {
                            decimal_value(c) as u64
                        } else {
                            (c.to_ascii_lowercase() as u64) - ('a' as u64) + 10
                        });
                    digits += 1;
                }
                Some('_') => {
                    self.pos += 1;
                    has_underscores = true;
                }
                _ => break,
            }
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            is_hex_float = true;
            while let Some(c) = self.peek() {
                if is_hexadecimal(c) {
                    self.pos += 1;
                    digits += 1;
                } else if c == '_' {
                    self.pos += 1;
                    has_underscores = true;
                } else {
                    break;
                }
            }
        }
        if matches!(self.peek(), Some('p') | Some('P')) {
            self.pos += 1;
            is_hex_float = true;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            while let Some(c) = self.peek() {
                if is_decimal(c) {
                    self.pos += 1;
                } else if c == '_' {
                    self.pos += 1;
                    has_underscores = true;
                } else {
                    break;
                }
            }
        }
        let mut is_unsigned_long = false;
        let mut is_signed_long = false;
        let mut is_complex = false;
        // The number's end (before the suffixes — the C# _builder excludes
        // the suffixes from the value parse).
        let number_end = self.byte_pos();
        if self.advance_if_matches("ull") {
            if is_hex_float {
                self.error_current(ErrorCode::ErrLuajitSuffixInFloat);
            } else {
                is_unsigned_long = true;
            }
        } else if self.advance_if_matches("ll") {
            if is_hex_float {
                self.error_current(ErrorCode::ErrLuajitSuffixInFloat);
            } else {
                is_signed_long = true;
            }
        } else if self.advance_if_matches("i") {
            is_complex = true;
        }
        if has_underscores && !self.options.accept_underscore_in_number_literals {
            self.error_current(ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion);
        }
        if (is_unsigned_long || is_signed_long || is_complex)
            && !self.options.accept_lua_jit_number_suffixes
        {
            self.error_current(ErrorCode::ErrNumberSuffixNotSupportedInVersion);
        }
        // The C# branch order (Lexer.Numbers.cs:364-433): ull/ll
        // TryParses, then the complex HexFloat, then the float/double
        // HexFloat, then the plain long.TryParse. No digit-less
        // ErrInvalidNumber for hex — only the C# binary and octal parsers
        // have that rule (Lexer.Numbers.cs:81-85, 156-160); the hex
        // parser has none (Finding 18).
        if is_unsigned_long || is_signed_long {
            // C# ulong.TryParse / long.TryParse (Lexer.Numbers.cs:
            // 364-378): the digit-less builder fails, and values wider
            // than 64 bits fail. The 64-bit bit patterns parse as
            // two's-complement signed longs (0xffffffffffffffff = -1,
            // 0x8000000000000000 = i64::MIN — no error), and the ull
            // suffix parses the full u64 range — so only the overflow
            // past u64::MAX matters (Finding 20).
            if digits == 0 || overflowed {
                self.error_current(ErrorCode::ErrNumericLiteralTooLarge);
            }
        } else if is_complex {
            // C# HexFloat.DoubleFromHexString (Lexer.Numbers.cs:380-394):
            // the complex value is a double — only a real overflow
            // reports DoubleOverflow, never the integer TooLarge
            // (Finding 21). The C# throws FormatException on a digit-less
            // builder — the port's anti-crash silence.
            if digits > 0 {
                let text = &self.source[self.byte_of_char(self.lexeme_start)..number_end];
                if hex_float_overflows(text) {
                    self.error_current(ErrorCode::ErrDoubleOverflow);
                }
            }
        } else if is_hex_float {
            if !self.options.accept_hex_float_literals {
                self.error_current(ErrorCode::ErrHexFloatLiteralNotSupportedInVersion);
            }
            let text = &self.source[self.byte_of_char(self.lexeme_start)..number_end];
            if hex_float_overflows(text) {
                self.error_current(ErrorCode::ErrDoubleOverflow);
            }
        } else if self.options.hex_integer_format != IntegerFormats::NotSupported {
            // C# long.TryParse (Lexer.Numbers.cs:415-433): the digit-less
            // builder and values wider than 64 bits fail. The double-only
            // presets route the integer hex through
            // HexFloat.DoubleFromHexString with no error for valid text
            // (the C# throws FormatException on the digit-less builder —
            // the port's anti-crash silence, like the StringUtils trim
            // adaptation).
            if digits == 0 || overflowed {
                self.error_current(ErrorCode::ErrNumericLiteralTooLarge);
            }
        }
    }

    fn scan_binary_number(&mut self) {
        let mut num: u64 = 0;
        let mut digits = 0usize;
        let mut has_underscores = false;
        let mut has_overflown = false;
        loop {
            match self.peek() {
                Some(c) if is_binary(c) => {
                    self.pos += 1;
                    if (num & 0x8000_0000_0000_0000) != 0 {
                        has_overflown = true;
                    }
                    num = (num << 1) | decimal_value(c) as u64;
                    digits += 1;
                }
                Some('_') => {
                    self.pos += 1;
                    has_underscores = true;
                }
                _ => break,
            }
        }
        let mut is_unsigned_long = false;
        let mut is_signed_long = false;
        let mut is_complex = false;
        if self.advance_if_matches("ull") {
            is_unsigned_long = true;
        } else if self.advance_if_matches("ll") {
            is_signed_long = true;
        } else if self.advance_if_matches("i") {
            is_complex = true;
        }
        if !self.options.accept_binary_numbers {
            self.error_current(ErrorCode::ErrBinaryNumericLiteralNotSupportedInVersion);
        }
        if has_underscores && !self.options.accept_underscore_in_number_literals {
            self.error_current(ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion);
        }
        if (is_unsigned_long || is_signed_long || is_complex)
            && !self.options.accept_lua_jit_number_suffixes
        {
            self.error_current(ErrorCode::ErrNumberSuffixNotSupportedInVersion);
        }
        if digits < 1 {
            self.error_current(ErrorCode::ErrInvalidNumber);
        }
        if has_overflown || (num > i64::MAX as u64 && !is_unsigned_long) {
            self.error_current(ErrorCode::ErrNumericLiteralTooLarge);
        }
    }

    fn scan_octal_number(&mut self) {
        let mut num: u64 = 0;
        let mut digits = 0usize;
        let mut has_underscores = false;
        let mut has_overflown = false;
        loop {
            match self.peek() {
                Some(c) if is_octal(c) => {
                    self.pos += 1;
                    if (num & 0x7000_0000_0000_0000) != 0 {
                        has_overflown = true;
                    }
                    num = (num << 3) | decimal_value(c) as u64;
                    digits += 1;
                }
                Some('_') => {
                    self.pos += 1;
                    has_underscores = true;
                }
                _ => break,
            }
        }
        if !self.options.accept_octal_numbers {
            self.error_current(ErrorCode::ErrOctalNumericLiteralNotSupportedInVersion);
        }
        if has_underscores && !self.options.accept_underscore_in_number_literals {
            self.error_current(ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion);
        }
        if digits < 1 {
            self.error_current(ErrorCode::ErrInvalidNumber);
        }
        if has_overflown {
            self.error_current(ErrorCode::ErrNumericLiteralTooLarge);
        }
    }

    fn scan_decimal_number(&mut self) {
        let mut is_float = false;
        let mut has_underscores = false;
        // The accumulated integer value (the C# _builder digits —
        // underscores skipped) for the TryParse overflow checks
        // (Finding 17); only the integer part is parsed by the C# long
        // paths, so the fraction/exponent loops don't feed it.
        let mut num: u64 = 0;
        let mut overflowed = false;
        loop {
            match self.peek() {
                Some(c) if is_decimal(c) => {
                    self.pos += 1;
                    let digit = decimal_value(c) as u64;
                    if num > (u64::MAX - digit) / 10 {
                        overflowed = true;
                    } else {
                        num = num * 10 + digit;
                    }
                }
                Some('_') => {
                    self.pos += 1;
                    has_underscores = true;
                }
                _ => break,
            }
        }
        if self.peek() == Some('.') {
            self.pos += 1;
            is_float = true;
            loop {
                match self.peek() {
                    Some(c) if is_decimal(c) => {
                        self.pos += 1;
                    }
                    Some('_') => {
                        self.pos += 1;
                        has_underscores = true;
                    }
                    _ => break,
                }
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.pos += 1;
            is_float = true;
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.pos += 1;
            }
            loop {
                match self.peek() {
                    Some(c) if is_decimal(c) => {
                        self.pos += 1;
                    }
                    Some('_') => {
                        self.pos += 1;
                        has_underscores = true;
                    }
                    _ => break,
                }
            }
        }
        let mut is_unsigned_long = false;
        let mut is_signed_long = false;
        let mut is_complex = false;
        // The number's end (before the suffixes — the C# _builder excludes
        // the suffixes from the value parse).
        let number_end = self.byte_pos();
        if self.advance_if_matches("ull") {
            if is_float {
                self.error_current(ErrorCode::ErrLuajitSuffixInFloat);
            } else {
                is_unsigned_long = true;
            }
        } else if self.advance_if_matches("ll") {
            if is_float {
                self.error_current(ErrorCode::ErrLuajitSuffixInFloat);
            } else {
                is_signed_long = true;
            }
        } else if self.advance_if_matches("i") {
            is_complex = true;
        }
        if has_underscores && !self.options.accept_underscore_in_number_literals {
            self.error_current(ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion);
        }
        if (is_unsigned_long || is_signed_long || is_complex)
            && !self.options.accept_lua_jit_number_suffixes
        {
            self.error_current(ErrorCode::ErrNumberSuffixNotSupportedInVersion);
        }
        if is_unsigned_long {
            // C# ulong.TryParse (Lexer.Numbers.cs:250-254).
            if overflowed {
                self.error_current(ErrorCode::ErrNumericLiteralTooLarge);
            }
        } else if is_signed_long {
            // C# long.TryParse (Lexer.Numbers.cs:258-262).
            if overflowed || num > i64::MAX as u64 {
                self.error_current(ErrorCode::ErrNumericLiteralTooLarge);
            }
        } else if is_complex {
            // C# RealParser.TryParseDouble (Lexer.Numbers.cs:266-270) — the
            // integer text as a double.
            let text = &self.source[self.byte_of_char(self.lexeme_start)..number_end];
            if decimal_float_overflows(text) {
                self.error_current(ErrorCode::ErrDoubleOverflow);
            }
        } else if is_float {
            let text = &self.source[self.byte_of_char(self.lexeme_start)..number_end];
            if decimal_float_overflows(text) {
                self.error_current(ErrorCode::ErrDoubleOverflow);
            }
        } else if self.options.decimal_integer_format == IntegerFormats::NotSupported {
            // C# RealParser.TryParseDouble (Lexer.Numbers.cs:272-279) — the
            // integer text as a double.
            let text = &self.source[self.byte_of_char(self.lexeme_start)..number_end];
            if decimal_float_overflows(text) {
                self.error_current(ErrorCode::ErrDoubleOverflow);
            }
        } else {
            // C# long.TryParse (Lexer.Numbers.cs:282-295) — the Double and
            // Int64 formats both parse the integer as a long
            // (ErrNumericLiteralTooLarge on failure).
            if overflowed || num > i64::MAX as u64 {
                self.error_current(ErrorCode::ErrNumericLiteralTooLarge);
            }
        }
    }

    /// The C# TextWindow.AdvanceIfMatches(uppercase: true).
    fn advance_if_matches(&mut self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let ok = chars.iter().enumerate().all(|(i, expected)| {
            self.peek_at(i).map(|c| c.to_ascii_lowercase() == *expected) == Some(true)
        });
        if ok {
            self.pos += chars.len();
        }
        ok
    }

    /// The C# ScanIdentifier (Lexer.Identifiers.cs:29-173) — the diagnostics
    /// only (the LuaJIT identifier rules).
    fn scan_identifier(&mut self) {
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
        self.only_shebangs_and_newlines = false;
    }

    /// The bad-character token (the C# ScanToken default) — the lexer emits
    /// ERR_BadCharacter and the parser emits ERR_InvalidStatement on the bad
    /// token (the port emits both, as the reference tests expect).
    fn scan_other(&mut self) {
        let start = self.pos;
        let c = self.peek().expect("a char");
        self.pos += 1;
        self.error_at(
            self.byte_of_char(start),
            1,
            ErrorCode::ErrBadCharacter,
            vec![c.to_string()],
        );
        self.error_at(
            self.byte_of_char(start),
            1,
            ErrorCode::ErrInvalidStatement,
            Vec::new(),
        );
        self.only_shebangs_and_newlines = false;
    }
}

/// The C# CharUtils helpers.
fn is_decimal(c: char) -> bool {
    c.is_ascii_digit()
}

fn is_hexadecimal(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_binary(c: char) -> bool {
    matches!(c, '0' | '1')
}

fn is_octal(c: char) -> bool {
    matches!(c, '0'..='7')
}

fn is_newline(c: char) -> bool {
    matches!(c, '\n' | '\r')
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\u{0B}' | '\u{0C}')
}

/// The C# CharUtils.DecimalValue — the ASCII digit value.
fn decimal_value(c: char) -> u32 {
    c as u32 - '0' as u32
}

/// The C# HexFloat.DoubleFromHexString overflow (the exponent-driven
/// overflow throws, e.g. 0x1p999999). The C# number builder strips the
/// underscores before parsing (the C# ConsumeHexDigits appends only the
/// non-underscore digits).
fn hex_float_overflows(text: &str) -> bool {
    let clean: String = text.chars().filter(|c| *c != '_').collect();
    match crate::utilities::hexfloat::HexFloat::double_from_hex_string(&clean) {
        Ok(value) => value.is_infinite() || value.is_nan(),
        Err(_) => true,
    }
}

/// The C# RealParser.TryParseDouble failure (e.g. 1e999999). The C# number
/// builder strips the underscores before parsing (the C# ConsumeDecimalDigits
/// appends only non-underscore digits).
fn decimal_float_overflows(text: &str) -> bool {
    let clean: String = text.chars().filter(|c| *c != '_').collect();
    match clean.parse::<f64>() {
        Ok(value) => value.is_infinite() || value.is_nan(),
        Err(_) => true,
    }
}

/// Scans the source and produces the lexer diagnostics for the options.
pub fn lexer_diagnostics(source: &str, options: &LuaSyntaxOptions) -> Vec<LexerDiagnostic> {
    let mut s = Scanner::new(source, options);
    while !s.at_end() {
        let c = s.peek().expect("not at end");
        if is_whitespace(c) {
            s.pos += 1;
            continue;
        }
        if is_newline(c) {
            s.scan_end_of_line();
            s.only_shebangs_and_newlines = true;
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
            // (Lexer.cs:501-507 — Finding 22).
            '&' | '|' | '#' | '+' | '-' | '*' | '/' | '^' | '=' | '~' | '%' | ',' | '.' | ':'
            | ';' | '?' | '!' | '(' | ')' | '[' | ']' | '{' | '}' => {
                s.pos += 1;
                s.only_shebangs_and_newlines = false;
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
                s.only_shebangs_and_newlines = false;
            }
            '>' => {
                s.pos += 1;
                s.only_shebangs_and_newlines = false;
            }
            _ => s.scan_other(),
        }
    }
    s.diagnostics
}

// The long-string termination flag lives on the scanner struct (declared
// with the fields above).
