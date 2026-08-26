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

// The scanner methods are split across this directory by the C# lexer partial files
// (lexer.rs <- Lexer.cs, shortstring.rs <- Lexer.ShortString.cs, numbers.rs <-
// Lexer.Numbers.cs, identifiers.rs <- Lexer.Identifiers.cs — distinctive names, no
// dotted files); the module path stays crate::errors::lexerdiagnostics.

use crate::errors::errorcode::ErrorCode;
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
    /// The cumulative bad-token count (the C# _badTokenCount, Lexer.cs:
    /// 44) — past 200 the current bad token absorbs the rest of the
    /// input (Finding 26).
    bad_token_count: usize,
    /// Whether the last long-string scan was terminated (the C# out
    /// isTerminated).
    long_string_terminated: bool,
}

// The long-string termination flag lives on the scanner struct (declared
// with the fields above).

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

/// C# CharUtils.IsWhitespace (CharUtils.cs:155-159) — [ \t\n\v\f\r] —
/// the `\z` escape's skip set (Lexer.ShortString.cs:141). The dispatch
/// inlines the C# trivia handling instead (Lexer.cs:735-749: the
/// space/tab fast path keeps the shebang guard, '\v'/'\f' clear it) —
/// Findings 23 + 25.
fn is_whitespace(c: char) -> bool {
    c == ' ' || ('\t'..='\r').contains(&c)
}

/// The C# CharUtils.DecimalValue — the ASCII digit value.
fn decimal_value(c: char) -> u32 {
    c as u32 - '0' as u32
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
            bad_token_count: 0,
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
}

mod identifiers;
mod lexer;
mod numbers;
mod shortstring;

pub use lexer::lexer_diagnostics;
