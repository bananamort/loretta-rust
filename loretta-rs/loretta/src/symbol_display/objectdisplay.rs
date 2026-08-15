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
            Some(PrimitiveValue::Complex(imaginary)) => {
                Some(Self::format_literal_complex(imaginary, options))
            }
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
                // C# NeedsEscaping flagset over .NET UnicodeCategory values
                // (ClosePunctuation=21, ConnectorPunctuation=18, CurrencySymbol=26,
                // DashPunctuation=19, DecimalDigitNumber=8, FinalQuotePunctuation=23,
                // InitialQuotePunctuation=22, LetterNumber=9, LowercaseLetter=1,
                // MathSymbol=25, OpenPunctuation=20, OtherLetter=4, OtherNumber=10,
                // OtherPunctuation=24, TitlecaseLetter=2, UppercaseLetter=0).
                const CATEGORY_FLAG_SET: u32 = (1u32 << 21)
                    | (1u32 << 18)
                    | (1u32 << 26)
                    | (1u32 << 19)
                    | (1u32 << 8)
                    | (1u32 << 23)
                    | (1u32 << 22)
                    | (1u32 << 9)
                    | (1u32 << 1)
                    | (1u32 << 25)
                    | (1u32 << 20)
                    | (1u32 << 4)
                    | (1u32 << 10)
                    | (1u32 << 24)
                    | (1u32 << 2)
                    | (1u32 << 0);
                let category = crate::symbol_display::unicode_categories::category_of(ch as u32);
                !CharUtils::is_category_in_set(CATEGORY_FLAG_SET, category)
            }
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

        // C# seeds 61 '='s — the fast path can handle up to 60, so the slow
        // path starts one above it (ObjectDisplay.cs:346).
        let mut equals = "=".repeat(61);
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
            Self::format_double_r(value)
        }
    }

    /// .NET `double.ToString("R", CultureInfo.InvariantCulture)`: shortest
    /// round-trippable digits laid out like "G" — scientific when the decimal
    /// exponent is <= -5 or >= 17, otherwise decimal (verified against the
    /// objectdisplay oracle across all 11 presets).
    pub fn format_double_r(value: f64) -> String {
        if value.is_nan() {
            return "NaN".to_string();
        }
        if value == f64::INFINITY {
            return "Infinity".to_string();
        }
        if value == f64::NEG_INFINITY {
            return "-Infinity".to_string();
        }
        if value == 0.0 && value.is_sign_negative() {
            return "-0".to_string();
        }
        // Rust's shortest scientific formatting produces the same shortest
        // round-trippable digits as .NET "R" ("d[.ddd]e±X").
        let sci = format!("{:e}", value);
        let (negative, digits, exp) = Self::parse_shortest_sci(&sci);
        let n = digits.len() as i32;
        if exp <= -5 || exp >= 17 {
            let mut out = String::new();
            if negative {
                out.push('-');
            }
            out.push(char::from(b'0' + digits[0]));
            if n > 1 {
                out.push('.');
                for &d in &digits[1..] {
                    out.push(char::from(b'0' + d));
                }
            }
            out.push('E');
            if exp < 0 {
                out.push('-');
            } else {
                out.push('+');
            }
            out.push_str(&format!("{:02}", exp.abs()));
            out
        } else if exp >= 0 {
            let mut out = String::new();
            if negative {
                out.push('-');
            }
            let point = (exp + 1) as usize;
            if point >= digits.len() {
                for &d in &digits {
                    out.push(char::from(b'0' + d));
                }
                for _ in 0..(point - digits.len()) {
                    out.push('0');
                }
            } else {
                for (i, &d) in digits.iter().enumerate() {
                    if i == point {
                        out.push('.');
                    }
                    out.push(char::from(b'0' + d));
                }
            }
            out
        } else {
            let mut out = String::new();
            if negative {
                out.push('-');
            }
            out.push_str("0.");
            for _ in 0..(-exp - 1) {
                out.push('0');
            }
            for &d in &digits {
                out.push(char::from(b'0' + d));
            }
            out
        }
    }

    /// Parses Rust's shortest scientific form "d[.ddd]e±X" into
    /// (negative, significant digits, decimal exponent).
    fn parse_shortest_sci(s: &str) -> (bool, Vec<u8>, i32) {
        let (negative, rest) = match s.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, s),
        };
        let (mantissa, exp) = rest.split_once('e').expect("scientific form");
        let digits: Vec<u8> = mantissa
            .chars()
            .filter(|c| *c != '.')
            .map(|c| c as u8 - b'0')
            .collect();
        (negative, digits, exp.parse().expect("exponent"))
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

    /// Returns a Lua number literal with the given imaginary part and the `i` suffix.
    /// C# `FormatLiteral(Complex value, ...)` = `FormatLiteral(value.Imaginary, ...) + "i"`
    /// — only the imaginary part participates, so the port carries it directly.
    pub fn format_literal_complex(imaginary: f64, options: ObjectDisplayOptions) -> String {
        format!("{}i", Self::format_literal_f64(imaginary, options))
    }
}

/// Represents a primitive value for FormatPrimitive.
#[derive(Clone)]
pub enum PrimitiveValue {
    String(String),
    Bool(bool),
    Double(f64),
    Long(i64),
    Ulong(u64),
    /// C# System.Numerics.Complex — only the imaginary part is formatted.
    Complex(f64),
}
