// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFolder.NumberParsing (b767b4e):
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.NumberParsing.cs
// (plus the number-literal value extraction the C# lexer precomputes into
// token.Value — full_moon hands raw literal text, so the parsing lives here).

use crate::experimental::constantfolder::{get_inner_expression, NumValue};
use crate::utilities::hexfloat::HexFloat;
use crate::utilities::stringutils::StringUtils;
use full_moon::ast;

/// C# number classification: the token's Value is double iff the text has a
/// '.', exponent ('e'/'E') or hex-float ('p'/'P'). In a hex literal the
/// 'e'/'E' characters are DIGITS, not exponents — only '.' and 'p'/'P'
/// (the hex-float markers) make it a double (e.g. 0xE5, 0x1e5 are
/// integers — Finding 19).
pub(crate) fn number_is_double(text: &str) -> bool {
    if text.starts_with("0x") || text.starts_with("0X") {
        text.contains('.') || text.contains('p') || text.contains('P')
    } else {
        text.contains('.')
            || text.contains('e')
            || text.contains('E')
            || text.contains('p')
            || text.contains('P')
    }
}

/// Parses an integer literal (decimal, hex or binary) like the C# lexer's
/// integer paths (Lexer.Numbers.cs): underscores are skipped (the C#
/// Consume*Digits builders), overflow folds to 0 (TryParse's default out
/// value, with ERR_NumericLiteralTooLarge reported by the diagnostics
/// side), and hex digits are a two's-complement bit pattern
/// (long.TryParse AllowHexSpecifier).
pub(crate) fn parse_integer_literal(text: &str) -> i64 {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        // C# long.TryParse(AllowHexSpecifier) (Lexer.Numbers.cs:374-378):
        // 0xffffffffffffffff is -1; values wider than 64 bits fail -> 0.
        u64::from_str_radix(&rest.replace('_', ""), 16)
            .map(|bits| bits as i64)
            .unwrap_or(0)
    } else if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        // C# ParseBinaryNumber (Lexer.Numbers.cs:86-90): values with bit 63
        // set (no ull suffix) fold to 0.
        u64::from_str_radix(&rest.replace('_', ""), 2)
            .ok()
            .filter(|&bits| bits <= i64::MAX as u64)
            .map(|bits| bits as i64)
            .unwrap_or(0)
    } else {
        // C# long.TryParse (Lexer.Numbers.cs:258-262, 282-295): overflow -> 0.
        text.replace('_', "").parse::<i64>().unwrap_or(0)
    }
}

/// Parses a double literal (decimal float or hex float).
pub(crate) fn parse_double_literal(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.starts_with("0x") || text.starts_with("0X") {
        HexFloat::double_from_hex_string(text).ok()
    } else {
        parse_decimal_double(text)
    }
}

/// The C# RealParser.TryParseDouble (RealParser.cs:30-37) for the string
/// extraction (Finding 28): the leading numeric run is the value and the
/// trailing garbage is ignored (the FromSource loops, RealParser.cs:
/// 288-368). An empty run (leading garbage or a leading sign — the C#
/// "does not support a leading sign character") is NoDigits, which the
/// C# returns as true with 0.0 (RealParser.cs:384-388). A digit-less
/// exponent takes the MAX_EXP fallback (RealParser.cs:361-364): the C#
/// overflows for '+'/none (the extraction fails) and underflows to 0.0
/// for '-'. The C# RealParser is decimal — a "0x1.8p10" string yields
/// 0.0 from the leading "0" (the decFloat comes before the hexFloat,
/// Finding 31).
fn parse_decimal_double(value: &str) -> Option<f64> {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    // The C# NoDigits (RealParser.cs:384-388): the mantissa is empty —
    // leading garbage, a leading sign, or a bare dot — and the C#
    // returns true with 0.0 even when an exponent follows ("e5" -> 0.0).
    if i == 0 || (i == 1 && bytes[0] == b'.') {
        return Some(0.0);
    }
    if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        let sign = match bytes.get(i) {
            Some(b'-') => {
                i += 1;
                Some(-1)
            }
            Some(b'+') => {
                i += 1;
                Some(1)
            }
            _ => None,
        };
        let digits_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if digits_start == i {
            return match sign {
                Some(-1) => Some(0.0),
                _ => None,
            };
        }
    }
    let run = &value[..i];
    if run.is_empty() {
        return Some(0.0);
    }
    run.parse::<f64>().ok()
}

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

/// C# GetValue(node) — the literal token's value for a Number node.
pub(crate) fn number_value(node: &ast::Expression) -> NumValue {
    let inner = get_inner_expression(node);
    let ast::Expression::Number(t) = inner else {
        unreachable!("number value requires a number literal");
    };
    let text = t.token().to_string();
    if number_is_double(&text) {
        let parsed = parse_double_literal(&text)
            .unwrap_or_else(|| panic!("invalid number literal {text:?}"));
        NumValue::Double(parsed)
    } else {
        // C# fold-as-0: TryParse failure leaves the default value and the
        // lexer reports ERR_NumericLiteralTooLarge (Lexer.Numbers.cs).
        NumValue::Long(parse_integer_literal(&text))
    }
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
