// Ported from Loretta.CodeAnalysis.Lua.Utilities.CharUtils (b767b4e): CharUtils
// C# source: src/Compilers/Lua/Portable/Utilities/CharUtils.cs

/// A general character utility class.
pub struct CharUtils;

impl CharUtils {
    /// Checks whether the provided value is in the range [start, end].
    #[inline]
    pub fn is_in_range(start: char, value: char, end: char) -> bool {
        (value as u32).wrapping_sub(start as u32) <= (end as u32).wrapping_sub(start as u32)
    }

    /// Converts the provided ASCII character into lower-case ASCII.
    #[inline]
    pub fn ascii_lower_case(ch: char) -> char {
        (ch as u8 | 0b100000) as char
    }

    /// Checks if the next greatest value's index indicates the character is in
    /// the middle of a range.
    #[inline]
    fn inner_is_in_ranges_index_check(idx: usize) -> bool {
        // If the next greatest value's index is odd, then the character is in
        // the middle of a range. Since the length is always even, we don't need
        // to worry about the element not being in the array since it'll return 0
        // or an even number which will not pass the odd check.
        idx % 2 == 1
    }

    /// Checks if the provided character is in the middle of any of the ranges
    /// in the provided sorted and flattened range list.
    pub fn is_in_ranges(ranges: &[char], ch: char) -> bool {
        if ranges.len() == 2 {
            Self::is_in_range(ranges[0], ch, ranges[1])
        } else {
            match ranges.binary_search(&ch) {
                Ok(_) => true,
                Err(idx) => Self::inner_is_in_ranges_index_check(idx),
            }
        }
    }

    /// Creates a flagset from a list of unicode categories (as u8 discriminants).
    pub fn create_category_flag_set(categories: &[u8]) -> u32 {
        let mut flag_set: u32 = 0;
        for &cat in categories {
            flag_set |= 1u32 << cat;
        }
        flag_set
    }

    /// Checks if the provided category (as u8 discriminant) is in the flagset.
    #[inline]
    pub fn is_category_in_set(flag_set: u32, category: u8) -> bool {
        ((1u32 << category) & flag_set) != 0
    }

    /// Checks whether the provided character is a binary character (0 or 1).
    #[inline]
    pub fn is_binary(ch: char) -> bool {
        Self::is_in_range('0', ch, '1')
    }

    /// Checks whether the provided character is an octal character (between 0 and 7).
    #[inline]
    pub fn is_octal(ch: char) -> bool {
        Self::is_in_range('0', ch, '7')
    }

    /// Checks whether the provided character is a decimal character (between 0 and 9).
    #[inline]
    pub fn is_decimal(ch: char) -> bool {
        Self::is_in_range('0', ch, '9')
    }

    /// Gets the decimal value of the provided character.
    #[inline]
    pub fn decimal_value(ch: char) -> i64 {
        (ch as i64) - ('0' as i64)
    }

    /// Checks whether the provided character is a hexadecimal character.
    #[inline]
    pub fn is_hexadecimal(ch: char) -> bool {
        Self::is_decimal(ch) || Self::is_in_range('a', Self::ascii_lower_case(ch), 'f')
    }

    /// Checks whether the provided character is an alpha character (a-z, A-Z).
    #[inline]
    pub fn is_alpha(ch: char) -> bool {
        Self::is_in_range('a', Self::ascii_lower_case(ch), 'z')
    }

    /// Checks whether the provided character is an alphanumeric character (a-z, A-Z, 0-9).
    #[inline]
    pub fn is_alpha_numeric(ch: char) -> bool {
        Self::is_decimal(ch) || Self::is_alpha(ch)
    }

    /// Checks whether the provided character is whitespace.
    #[inline]
    pub fn is_whitespace(ch: char) -> bool {
        ch == ' ' || Self::is_in_range('\t', ch, '\r')
    }

    /// Checks whether the provided character is a newline character.
    #[inline]
    pub fn is_new_line(ch: char) -> bool {
        ch == '\n' || ch == '\r'
    }

    /// Checks whether the provided character is a valid first identifier character.
    #[inline]
    pub fn is_valid_first_identifier_char(ch: char) -> bool {
        ch == '_' || Self::is_alpha(ch) || ch as u32 >= 0x7F
    }

    /// Checks whether the provided character is a valid trailing identifier character.
    #[inline]
    pub fn is_valid_trailing_identifier_char(ch: char) -> bool {
        Self::is_valid_first_identifier_char(ch) || Self::is_decimal(ch)
    }

    /// Encodes the provided character into a hexadecimal escape sequence representing its UTF-8 bytes.
    pub fn encode_char_to_utf8(ch: char) -> String {
        let n = ch as u32;
        if n < 0x7F {
            format!("\\x{n:02X}")
        } else if n < 0x7FF {
            // 00000yyy yyxxxxxx -> [ 110yyyyy 10xxxxxx ]
            let byte01 = (0b11000000 | ((n >> 6) & 0b11111)) as u8;
            let byte02 = (0b10000000 | (n & 0b111111)) as u8;
            format!("\\x{byte01:02X}\\x{byte02:02X}")
        } else {
            // zzzzyyyy yyxxxxxx -> [ 1110zzzz 10yyyyyy 10xxxxxx ]
            let byte01 = (0b11100000 | ((n >> 12) & 0b1111)) as u8;
            let byte02 = (0b10000000 | ((n >> 6) & 0b111111)) as u8;
            let byte03 = (0b10000000 | (n & 0b111111)) as u8;
            format!("\\x{byte01:02X}\\x{byte02:02X}\\x{byte03:02X}")
        }
    }
}
