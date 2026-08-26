// The C# Lexer.Numbers.cs diagnostic rules — the hex/binary/octal/decimal number
// scanners and their overflow/suffix diagnostics (the C# lexer is DROP per the Port
// Boundary — only the LUA diagnostic rules are re-implemented, see mod.rs).

use super::*;
use crate::errors::errorcode::ErrorCode;
use crate::integerformats::IntegerFormats;

impl<'a> Scanner<'a> {
    /// The number scanning (Lexer.Numbers.cs) — the diagnostics only.
    pub(crate) fn scan_number(&mut self) {
        let start = self.pos;
        self.lexeme_start = start;
        if self.peek() == Some('0') {
            // C# Lexer.cs:546-598: the underscores between the '0' and the
            // prefix letter belong to the prefixed literal — the dispatch
            // scans past them and reports the underscore gating here (the
            // in-parser checks report it again — the C# emits twice for
            // the prefixed literals, Finding 24).
            let mut i = 1;
            while self.peek_at(i) == Some('_') {
                i += 1;
            }
            let has_prefix_underscores = i != 1;
            match self.peek_at(i) {
                Some('x') | Some('X') => {
                    self.pos += i + 1;
                    if !self.options.accept_underscore_in_number_literals && has_prefix_underscores
                    {
                        self.error_current(
                            ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                        );
                    }
                    self.scan_hex_number();
                }
                Some('b') | Some('B') => {
                    self.pos += i + 1;
                    if !self.options.accept_underscore_in_number_literals && has_prefix_underscores
                    {
                        self.error_current(
                            ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                        );
                    }
                    self.scan_binary_number();
                }
                Some('o') | Some('O') => {
                    self.pos += i + 1;
                    if !self.options.accept_underscore_in_number_literals && has_prefix_underscores
                    {
                        self.error_current(
                            ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
                        );
                    }
                    self.scan_octal_number();
                }
                _ => self.scan_decimal_number(),
            }
        } else {
            self.scan_decimal_number();
        }
        // A token ends the trivia run — the next run re-arms the shebang
        // guard (the C# per-run init, Lexer.cs:729; Finding 25).
        self.only_shebangs_and_newlines = true;
        let _ = start;
    }

    fn scan_hex_number(&mut self) {
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
                if is_decimal(c) || c == '_' {
                    self.pos += 1;
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
        // C# Lexer.Numbers.cs:359-360: the hex underscore check runs on the
        // FULL token text (info.Text.IndexOf('_')) — the prefix underscores
        // (0_xF) included — so the dispatch-time and in-parser errors both
        // land for those literals (Finding 24).
        if !self.options.accept_underscore_in_number_literals
            && self.source[self.byte_of_char(self.lexeme_start)..number_end].contains('_')
        {
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
