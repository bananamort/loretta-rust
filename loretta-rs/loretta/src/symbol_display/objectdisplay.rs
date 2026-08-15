// Ported from Loretta.CodeAnalysis.Lua.SymbolDisplay.ObjectDisplay (b767b4e): ObjectDisplay
// C# source: src/Compilers/Lua/Portable/SymbolDisplay/ObjectDisplay.cs
// NOTE: ObjectDisplayOptions, PooledStringBuilder, CharUtils are from dropped/simplified infrastructure.

/// Displays an object in the Lua style.
pub struct ObjectDisplay;

impl ObjectDisplay {
    /// The nil literal in Lua.
    pub const NIL_LITERAL: &'static str = "nil";

    /// Returns a string representation of a boolean.
    pub fn format_literal_bool(value: bool) -> &'static str {
        if value {
            "true"
        } else {
            "false"
        }
    }

    /// Returns a Lua number literal with the given value.
    pub fn format_literal_double(value: f64, use_hex: bool) -> String {
        if use_hex {
            format!("0x{:X}", value.to_bits())
        } else {
            format!("{value}")
        }
    }

    /// Returns a Lua number literal with the given value.
    pub fn format_literal_i64(value: i64, use_hex: bool) -> String {
        if use_hex {
            format!("0x{value:X}")
        } else {
            format!("{value}")
        }
    }

    /// Returns a Lua number literal with the given value and the ULL suffix.
    pub fn format_literal_u64(value: u64, use_hex: bool) -> String {
        if use_hex {
            format!("0x{value:X}ULL")
        } else {
            format!("{value}ULL")
        }
    }

    /// Returns true if the character needs escaping in a Lua string literal.
    pub fn needs_escaping(ch: char) -> bool {
        match ch {
            // ASCII characters that never need escaping
            '0'..='9'
            | 'a'..='z'
            | 'A'..='Z'
            | '_'
            | ' '
            | '`'
            | '!'
            | '@'
            | '#'
            | '%'
            | '&'
            | '*'
            | '-'
            | '='
            | '+'
            | '('
            | ')'
            | '['
            | ']'
            | '{'
            | '}'
            | '|'
            | '/'
            | ';'
            | ':'
            | '<'
            | '>'
            | ','
            | '.'
            | '?' => false,
            _ => {
                // Check if it's a printable character
                !ch.is_ascii_graphic() && !ch.is_ascii_whitespace() && ch != '\0'
            }
        }
    }

    /// Tries to replace a character with its escape sequence.
    pub fn try_replace_char(c: char, utf8_encode: bool) -> Option<String> {
        match c {
            '\\' => Some("\\\\".to_string()),
            '\0' => Some("\\0".to_string()),
            '\x07' => Some("\\a".to_string()),
            '\x08' => Some("\\b".to_string()),
            '\x0C' => Some("\\f".to_string()),
            '\n' => Some("\\n".to_string()),
            '\r' => Some("\\r".to_string()),
            '\t' => Some("\\t".to_string()),
            '\x0B' => Some("\\v".to_string()),
            _ => {
                if Self::needs_escaping(c) {
                    if utf8_encode {
                        Some(Self::encode_char_to_utf8(c))
                    } else {
                        Some(format!("\\u{{{}}}", c as u32))
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Encodes a character to UTF-8 escape sequence.
    fn encode_char_to_utf8(c: char) -> String {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        let mut result = String::from("\\x");
        for byte in encoded.bytes() {
            result.push_str(&format!("{byte:02X}"));
        }
        result
    }
}
