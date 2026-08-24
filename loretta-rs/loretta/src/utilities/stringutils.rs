// Ported from Loretta.CodeAnalysis.Lua.Utilities.StringUtils (b767b4e): StringUtils
// C# source: src/Compilers/Lua/Portable/Utilities/StringUtils.cs

use crate::utilities::charutils::CharUtils;

/// A class with utilities for strings.
pub struct StringUtils;

impl StringUtils {
    /// Returns whether the provided string is a valid identifier.
    pub fn is_identifier(value: &str) -> bool {
        // C# string.IsNullOrWhiteSpace (StringUtils.cs:38-39): a
        // whitespace-only name — including the >= U+007F whitespace such
        // as U+00A0 — must fail; the >= 0x7F first-char rule alone
        // passed it in Rust (Finding 50).
        if value.trim().is_empty() {
            return false;
        }

        let bytes = value.as_bytes();
        if !CharUtils::is_valid_first_identifier_char(bytes[0] as char) {
            return false;
        }

        for &b in &bytes[1..] {
            if !CharUtils::is_valid_trailing_identifier_char(b as char) {
                return false;
            }
        }

        true
    }

    /// Trims whitespace from both ends of the string.
    /// C# CharUtils.IsWhitespace is only ' ' and '\t'-'\r', so str::trim() is not equivalent.
    pub fn trim(value: &str) -> &str {
        let bytes = value.as_bytes();
        let mut start = 0;
        while start < bytes.len() && CharUtils::is_whitespace(bytes[start] as char) {
            start += 1;
        }
        let mut end = bytes.len();
        while end > start && CharUtils::is_whitespace(bytes[end - 1] as char) {
            end -= 1;
        }
        &value[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_identifier_rejects_whitespace_names() {
        // Finding 50: the C# IsIdentifier starts with the
        // IsNullOrWhiteSpace guard (StringUtils.cs:38-39) — whitespace
        // names (including the >= U+007F whitespace such as U+00A0 and
        // U+2003) fail, while the port's >= 0x7F first-char rule passed
        // them.
        assert!(!StringUtils::is_identifier(""));
        assert!(!StringUtils::is_identifier(" \t"));
        assert!(!StringUtils::is_identifier("\u{00A0}"));
        assert!(!StringUtils::is_identifier("\u{2003}"));
        assert!(StringUtils::is_identifier("a"));
        assert!(StringUtils::is_identifier("_abc1"));
        assert!(StringUtils::is_identifier("\u{00E9}"));
    }
}
