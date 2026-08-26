// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFolder.NumberParsing (b767b4e)
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.NumberParsing.cs

use crate::experimental::constantfolder::{parse_decimal_double, NumValue};
use crate::utilities::hexfloat::HexFloat;
use crate::utilities::stringutils::StringUtils;

/// C# TryParseNumberInString (ConstantFolder.NumberParsing.cs:20-66).
pub(crate) fn try_parse_number_in_string(value: &str) -> Option<NumValue> {
    let value = StringUtils::trim(value);
    // s_decIntegerRegex: ^[+\-]?\d+$ with long.TryParse(AllowLeadingSign)
    if is_dec_integer(value) {
        if let Ok(i64) = value.parse::<i64>() {
            return Some(NumValue::Long(i64));
        }
    }
    // s_hexIntegerRegex: ^[+\-]?0[xX][\da-fA-F]+$ with
    // long.TryParse(AllowLeadingSign | AllowHexSpecifier) — on .NET 8+ this
    // style combination throws ArgumentException at call time (pinned by the
    // constantfold-hex corpus case), so any hex-integer string panics with
    // the exact framework message.
    if is_hex_integer(value) {
        panic!(
            "With the AllowHexSpecifier or AllowBinarySpecifier bit set in the enum bit field, \
             the only other valid bits that can be combined into the enum value must be \
             AllowLeadingWhite and AllowTrailingWhite. (Parameter 'style')"
        );
    }
    // s_decFloatRegex with RealParser.TryParseDouble (invariant round-trip).
    // The value is the DECIMAL run even for "0x..." strings — the C#
    // RealParser takes the leading decimal run ("0x1.8p10" -> 0.0, the
    // decFloat-first ordering, Finding 31). The C# returns FALSE on
    // Overflow (RealParser.cs:30-36) — the extraction fails and the
    // expression stays untouched (Finding 30); the literal path keeps
    // the inf (the C# lexer's out value on overflow is the Infinity
    // bits, Lexer.Numbers.cs:274-278).
    if is_dec_float(value) {
        if let Some(f64) = parse_decimal_double(value).filter(|v| !v.is_infinite()) {
            return Some(NumValue::Double(f64));
        }
    }
    // s_hexFloatRegex with HexFloat.DoubleFromHexString (try/catch -> None).
    if is_hex_float(value) {
        if let Ok(f64) = HexFloat::double_from_hex_string(value) {
            return Some(NumValue::Double(f64));
        }
    }
    None
}

/// s_decIntegerRegex: ^[+\-]?\d+$
fn is_dec_integer(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        idx = 1;
    }
    if idx == bytes.len() {
        return false;
    }
    bytes[idx..].iter().all(|b| b.is_ascii_digit())
}

/// s_hexIntegerRegex: ^[+\-]?0[xX][\da-fA-F]+$
fn is_hex_integer(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        idx = 1;
    }
    if bytes.len() - idx < 3 {
        return false;
    }
    if bytes[idx] != b'0' || (bytes[idx + 1] != b'x' && bytes[idx + 1] != b'X') {
        return false;
    }
    bytes[idx + 2..].iter().all(|b| b.is_ascii_hexdigit())
}

/// s_decFloatRegex: [+\-]?(\.\d+|\d+(\.\d+)?)([eE][+\-]?\d+)? — the C#
/// regex is UNANCHORED (NumberParsing.cs:16-18): the string only needs
/// to CONTAIN a match, so the leading garbage is skipped ("v1.5"
/// contains "1.5"); the trailing garbage is ignored by the RealParser
/// (Finding 28).
fn is_dec_float(value: &str) -> bool {
    let bytes = value.as_bytes();
    for start in 0..bytes.len() {
        let mut i = start;
        if bytes[i] == b'+' || bytes[i] == b'-' {
            i += 1;
        }
        if i >= bytes.len() || !(bytes[i] == b'.' || bytes[i].is_ascii_digit()) {
            continue;
        }
        if dec_float_match(&bytes[i..]) {
            return true;
        }
    }
    false
}

/// The decFloat pattern body (without the sign — the caller consumes it):
/// (\.\d+ | \d+(\.\d+)?) ([eE][+\-]?\d+)? — a match anywhere in the slice
/// (the trailing garbage after the matched number is not part of it).
fn dec_float_match(rest: &[u8]) -> bool {
    if rest.is_empty() {
        return false;
    }
    // (\.\d+ | \d+(\.\d+)?)
    let mut i = 0;
    if rest[0] == b'.' {
        i = 1;
        if i == rest.len() || !rest[i].is_ascii_digit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        if !rest[0].is_ascii_digit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_digit() {
            i += 1;
        }
        if i < rest.len() && rest[i] == b'.' {
            i += 1;
            if i < rest.len() && rest[i].is_ascii_digit() {
                while i < rest.len() && rest[i].is_ascii_digit() {
                    i += 1;
                }
            }
        }
    }
    // ([eE][+\-]?\d+)? — the optional group: when the exponent digits
    // are missing the group fails and the match is the number alone (the
    // regex backtracks).
    if i < rest.len() && (rest[i] == b'e' || rest[i] == b'E') {
        i += 1;
        if i < rest.len() && (rest[i] == b'+' || rest[i] == b'-') {
            i += 1;
        }
        if i < rest.len() && rest[i].is_ascii_digit() {
            while i < rest.len() && rest[i].is_ascii_digit() {
                i += 1;
            }
        }
    }
    true
}

/// s_hexFloatRegex:
/// [+\-]?0x(\.[\da-fA-F]+|[\da-fA-F]+(\.[\da-fA-F]+)?)([pP][+\-]?\d+)?
fn is_hex_float(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        idx = 1;
    }
    let rest = &bytes[idx..];
    if rest.len() < 2 || rest[0] != b'0' || (rest[1] != b'x' && rest[1] != b'X') {
        return false;
    }
    let mut i = 2;
    let mut digits = 0;
    if i < rest.len() && rest[i] == b'.' {
        i += 1;
        if i == rest.len() || !rest[i].is_ascii_hexdigit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_hexdigit() {
            i += 1;
            digits += 1;
        }
    } else {
        if i == rest.len() || !rest[i].is_ascii_hexdigit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_hexdigit() {
            i += 1;
            digits += 1;
        }
        if i < rest.len() && rest[i] == b'.' {
            i += 1;
            while i < rest.len() && rest[i].is_ascii_hexdigit() {
                i += 1;
                digits += 1;
            }
        }
    }
    if digits == 0 {
        return false;
    }
    // ([pP][+\-]?\d+)?
    if i < rest.len() && (rest[i] == b'p' || rest[i] == b'P') {
        i += 1;
        if i < rest.len() && (rest[i] == b'+' || rest[i] == b'-') {
            i += 1;
        }
        if i == rest.len() || !rest[i].is_ascii_digit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_digit() {
            i += 1;
        }
    }
    i == rest.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_number_forms() {
        assert_eq!(try_parse_number_in_string("10"), Some(NumValue::Long(10)));
        assert_eq!(try_parse_number_in_string("+10"), Some(NumValue::Long(10)));
        assert_eq!(try_parse_number_in_string("-10"), Some(NumValue::Long(-10)));
        assert_eq!(
            try_parse_number_in_string("1.5"),
            Some(NumValue::Double(1.5))
        );
        assert_eq!(
            try_parse_number_in_string(".5"),
            Some(NumValue::Double(0.5))
        );
        assert_eq!(
            try_parse_number_in_string("1e2"),
            Some(NumValue::Double(100.0))
        );
        assert_eq!(
            try_parse_number_in_string("1E-2"),
            Some(NumValue::Double(0.01))
        );
        assert_eq!(
            try_parse_number_in_string("0x1.8p10"),
            // Finding 28/31: the decFloat comes first and the C# RealParser
            // takes the leading decimal run — the "0" — so the extraction
            // yields 0.0 (the C# oracle: print("0x1.8p10" + 1) -> print(1)).
            Some(NumValue::Double(0.0))
        );
        assert_eq!(try_parse_number_in_string("abc"), None);
        // Any hex-integer string panics with the pinned .NET ArgumentException
        // (the AllowLeadingSign | AllowHexSpecifier style is invalid on
        // .NET 8+; see the constantfold-hex corpus case).
        for hex in ["0x10", "0Xff", "-0x10"] {
            let result = std::panic::catch_unwind(|| {
                let _ = try_parse_number_in_string(hex);
            });
            assert!(result.is_err(), "hex string {hex:?} must panic");
        }
    }
}
