// Ported from Loretta.CodeAnalysis.Lua.SymbolDisplay.ObjectDisplay (b767b4e): ObjectDisplay
// C# source: src/Compilers/Lua/Portable/SymbolDisplay/ObjectDisplay.cs

use crate::symbol_display::objectdisplayoptions::ObjectDisplayOptions;
use crate::utilities::charutils::CharUtils;

/// Displays an object in the Lua style.
pub struct ObjectDisplay;

impl ObjectDisplay {
    /// The nil literal in Lua.
    pub const NIL_LITERAL: &'static str = "nil";

    /// Returns a string representation of an object of primitive type.
    /// Handles bool, string, double, long, ulong, and null.
    pub fn format_primitive(
        obj: Option<PrimitiveValue>,
        options: ObjectDisplayOptions,
    ) -> Option<String> {
        match obj {
            None => Some(Self::NIL_LITERAL.to_string()),
            Some(PrimitiveValue::String(s)) => Some(Self::format_literal_str(&s, options)),
            Some(PrimitiveValue::Bool(b)) => Some(Self::format_literal_bool(b)),
            Some(PrimitiveValue::Double(d)) => Some(Self::format_literal_f64(d, options)),
            Some(PrimitiveValue::Long(l)) => Some(Self::format_literal_i64(l, options)),
            Some(PrimitiveValue::Ulong(u)) => Some(Self::format_literal_u64(u, options)),
        }
    }

    /// Returns a string representation of a boolean.
    pub fn format_literal_bool(value: bool) -> String {
        if value {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }

    /// Returns true if the character should be replaced and returns the replacement text.
    fn try_replace_char(c: char, utf8_encode: bool) -> Option<String> {
        let replace_with = match c {
            '\\' => Some("\\\\"),
            '\0' => Some("\\0"),
            '\x07' => Some("\\a"),
            '\x08' => Some("\\b"),
            '\x0C' => Some("\\f"),
            '\n' => Some("\\n"),
            '\r' => Some("\\r"),
            '\t' => Some("\\t"),
            '\x0B' => Some("\\v"),
            _ => None,
        };

        if let Some(r) = replace_with {
            return Some(r.to_string());
        }

        if Self::needs_escaping(c) {
            let replacement = if utf8_encode {
                CharUtils::encode_char_to_utf8(c)
            } else {
                format!("\\u{{{:04X}}}", c as u32)
            };
            return Some(replacement);
        }

        None
    }

    fn needs_escaping(ch: char) -> bool {
        match ch {
            // ASCII characters that never need escaping.
            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | 'a' | 'b' | 'c' | 'd'
            | 'e' | 'f' | 'g' | 'h' | 'i' | 'j' | 'k' | 'l' | 'm' | 'n' | 'o' | 'p' | 'q' | 'r'
            | 's' | 't' | 'u' | 'v' | 'w' | 'x' | 'y' | 'z' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F'
            | 'G' | 'H' | 'I' | 'J' | 'K' | 'L' | 'M' | 'N' | 'O' | 'P' | 'Q' | 'R' | 'S' | 'T'
            | 'U' | 'V' | 'W' | 'X' | 'Y' | 'Z' | '_' | ' ' | '`' | '!' | '@' | '#' | '%' | '&'
            | '*' | '-' | '=' | '+' | '\u{00A7}' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
            | '/' | ';' | ':' | '<' | '>' | ',' | '.' | '?' => false,
            _ => {
                // Unicode category flagset check — verbatim from C# ObjectDisplay.NeedsEscaping.
                // ClosePunctuation=0, ConnectorPunctuation=1, CurrencySymbol=2,
                // DashPunctuation=3, DecimalDigitNumber=4, FinalQuotePunctuation=5,
                // InitialQuotePunctuation=6, LetterNumber=7, LowercaseLetter=8,
                // MathSymbol=9, OpenPunctuation=10, OtherLetter=11, OtherNumber=12,
                // OtherPunctuation=13, TitlecaseLetter=14, UppercaseLetter=15
                const CATEGORY_FLAG_SET: u32 = (1u32 << 0)   // ClosePunctuation
                    | (1u32 << 1)   // ConnectorPunctuation
                    | (1u32 << 2)   // CurrencySymbol
                    | (1u32 << 3)   // DashPunctuation
                    | (1u32 << 4)   // DecimalDigitNumber
                    | (1u32 << 5)   // FinalQuotePunctuation
                    | (1u32 << 6)   // InitialQuotePunctuation
                    | (1u32 << 7)   // LetterNumber
                    | (1u32 << 8)   // LowercaseLetter
                    | (1u32 << 9)   // MathSymbol
                    | (1u32 << 10)  // OpenPunctuation
                    | (1u32 << 11)  // OtherLetter
                    | (1u32 << 12)  // OtherNumber
                    | (1u32 << 13)  // OtherPunctuation
                    | (1u32 << 14)  // TitlecaseLetter
                    | (1u32 << 15); // UppercaseLetter
                let category = Self::get_unicode_category(ch);
                !CharUtils::is_category_in_set(CATEGORY_FLAG_SET, category)
            }
        }
    }

    /// Maps a char to its general Unicode category (simplified from System.Globalization).
    fn get_unicode_category(ch: char) -> u8 {
        if ch.is_ascii_digit() {
            4 // DecimalDigitNumber
        } else if ch.is_ascii_lowercase() {
            8 // LowercaseLetter
        } else if ch.is_ascii_uppercase() {
            15 // UppercaseLetter
        } else if ch.is_ascii_punctuation() || ch.is_ascii_whitespace() {
            13 // OtherPunctuation
        } else {
            11 // OtherLetter (fallback for non-ASCII)
        }
    }

    /// Returns a Lua string literal with the given value.
    pub fn format_literal_str(value: &str, options: ObjectDisplayOptions) -> String {
        const SHORT_STRING_QUOTE: char = '"';

        let use_quotes = options.includes_option(ObjectDisplayOptions::USE_QUOTES);
        let escape_non_printable =
            options.includes_option(ObjectDisplayOptions::ESCAPE_NON_PRINTABLE_CHARACTERS);
        let utf8_escape = options.includes_option(ObjectDisplayOptions::ESCAPE_WITH_UTF8);

        let is_verbatim = use_quotes && !escape_non_printable && Self::contains_new_line(value);

        let mut result = String::new();
        let end_delimiter;

        if use_quotes {
            if is_verbatim {
                let (start, end) = Self::slow_get_verbatim_equals(value);
                result.push_str(&start);
                end_delimiter = end;
            } else {
                end_delimiter = SHORT_STRING_QUOTE.to_string();
                result.push(SHORT_STRING_QUOTE);
            }
        } else {
            end_delimiter = String::new();
        }

        for ch in value.chars() {
            if escape_non_printable {
                if let Some(replace_with) = Self::try_replace_char(ch, utf8_escape) {
                    result.push_str(&replace_with);
                } else if use_quotes && !is_verbatim && ch == SHORT_STRING_QUOTE {
                    result.push('\\');
                    result.push(SHORT_STRING_QUOTE);
                } else {
                    result.push(ch);
                }
            } else if use_quotes && !is_verbatim && ch == SHORT_STRING_QUOTE {
                result.push('\\');
                result.push(SHORT_STRING_QUOTE);
            } else {
                result.push(ch);
            }
        }

        if use_quotes {
            result.push_str(&end_delimiter);
        }

        result
    }

    fn contains_new_line(s: &str) -> bool {
        s.contains('\r') || s.contains('\n')
    }

    /// Tries to find the shortest verbatim string delimiter.
    /// Returns (start_delimiter, end_delimiter).
    fn try_fast_get_verbatim_equals(value: &str) -> Option<(String, String)> {
        let buffer_size = 62;
        let mut start_buffer = vec!['\0'; buffer_size];
        let mut end_buffer = vec!['\0'; buffer_size];
        let mut idx = 1;

        start_buffer[0] = '[';
        start_buffer[1] = '[';
        end_buffer[0] = ']';
        end_buffer[1] = ']';

        loop {
            let start_slice: String = start_buffer[..(idx + 1)].iter().collect();
            let end_slice: String = end_buffer[..(idx + 1)].iter().collect();

            if !value.contains(&start_slice) && !value.contains(&end_slice) {
                return Some((start_slice, end_slice));
            }

            if idx >= buffer_size - 1 {
                return None;
            }

            start_buffer[idx] = '=';
            start_buffer[idx + 1] = '[';
            end_buffer[idx] = '=';
            end_buffer[idx + 1] = ']';
            idx += 1;
        }
    }

    /// Finds verbatim string delimiters (slow path for long strings).
    fn slow_get_verbatim_equals(value: &str) -> (String, String) {
        if let Some((start, end)) = Self::try_fast_get_verbatim_equals(value) {
            return (start, end);
        }

        let mut equals = "=".repeat(62);
        loop {
            let start_delimiter = format!("[{equals}[");
            let end_delimiter = format!("]{equals}]");
            if !value.contains(&start_delimiter) && !value.contains(&end_delimiter) {
                return (start_delimiter, end_delimiter);
            }
            equals.push('=');
        }
    }

    /// Returns a Lua number literal with the given value (f64).
    pub fn format_literal_f64(value: f64, options: ObjectDisplayOptions) -> String {
        if options.includes_option(ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS) {
            crate::utilities::hexfloat::HexFloat::double_to_hex_string(value)
        } else {
            format!("{value}")
        }
    }

    /// Returns a Lua number literal with the given value (i64).
    pub fn format_literal_i64(value: i64, options: ObjectDisplayOptions) -> String {
        if options.includes_option(ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS) {
            format!("0x{value:X}")
        } else {
            format!("{value}")
        }
    }

    /// Returns a Lua number literal with the given value and the ULL suffix (u64).
    pub fn format_literal_u64(value: u64, options: ObjectDisplayOptions) -> String {
        if options.includes_option(ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS) {
            format!("0x{value:X}ULL")
        } else {
            format!("{value}ULL")
        }
    }
}

/// Represents a primitive value for FormatPrimitive.
pub enum PrimitiveValue {
    String(String),
    Bool(bool),
    Double(f64),
    Long(i64),
    Ulong(u64),
}
