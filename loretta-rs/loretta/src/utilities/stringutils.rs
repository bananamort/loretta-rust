// Ported from Loretta.CodeAnalysis.Lua.Utilities.StringUtils (b767b4e): StringUtils
// C# source: src/Compilers/Lua/Portable/Utilities/StringUtils.cs

use crate::utilities::charutils::CharUtils;

/// A class with utilities for strings.
pub struct StringUtils;

impl StringUtils {
    /// Returns whether the provided string is a valid identifier.
    pub fn is_identifier(value: &str) -> bool {
        if value.is_empty() {
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
    pub fn trim(value: &str) -> &str {
        value.trim()
    }
}
