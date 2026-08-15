// Ported from Compilers/Lua/Test/Portable/SymbolDisplay/ObjectDisplayTests.cs (b767b4e)
// (the null-argument test is vacuous in Rust: &str has no null value)

use loretta::symbol_display::objectdisplay::ObjectDisplay;
use loretta::symbol_display::objectdisplayoptions::ObjectDisplayOptions;
use loretta::utilities::hexfloat::HexFloat;

#[test]
fn objectdisplay_format_literal_string_only_adds_quotes_when_asked_to() {
    let no_quotes = ObjectDisplay::format_literal_str("hello", ObjectDisplayOptions::NONE);
    let with_quotes = ObjectDisplay::format_literal_str("hello", ObjectDisplayOptions::USE_QUOTES);
    assert!(
        !(no_quotes.starts_with('"') || no_quotes.ends_with('"')),
        "no quotes output has quotes: {no_quotes:?}"
    );
    assert!(
        with_quotes.starts_with('"') && with_quotes.ends_with('"'),
        "with quotes output has no quotes: {with_quotes:?}"
    );
}

#[test]
fn objectdisplay_format_literal_string_only_escapes_non_printable_characters_when_asked_to() {
    let input = "\0\t\r\n";
    let unescaped = ObjectDisplay::format_literal_str(input, ObjectDisplayOptions::NONE);
    let escaped = ObjectDisplay::format_literal_str(
        input,
        ObjectDisplayOptions::ESCAPE_NON_PRINTABLE_CHARACTERS,
    );
    assert_eq!(unescaped, input);
    assert_eq!(escaped, "\\0\\t\\r\\n");
}

#[test]
fn objectdisplay_format_literal_string_only_escapes_with_utf8_when_asked_to() {
    let input = "\u{FEFF}";
    let unescaped = ObjectDisplay::format_literal_str(
        input,
        ObjectDisplayOptions::ESCAPE_NON_PRINTABLE_CHARACTERS,
    );
    let escaped = ObjectDisplay::format_literal_str(
        input,
        ObjectDisplayOptions::ESCAPE_NON_PRINTABLE_CHARACTERS
            | ObjectDisplayOptions::ESCAPE_WITH_UTF8,
    );
    assert_eq!(unescaped, "\\u{FEFF}");
    assert_eq!(escaped, "\\xEF\\xBB\\xBF");
}

#[test]
fn objectdisplay_format_literal_string_outputs_long_string_when_quotes_requested_new_line_present_and_escaping_not_requested(
) {
    let cases: &[(&str, &str)] = &[
        ("a\na\na", "[[a\na\na]]"),
        ("[[a\na\na]]", "[=[[[a\na\na]]]=]"),
    ];
    for (input, expected) in cases {
        let output = ObjectDisplay::format_literal_str(input, ObjectDisplayOptions::USE_QUOTES);
        assert_eq!(output, *expected, "verbatim({input:?})");
    }
}

#[test]
fn objectdisplay_format_literal_string_does_not_escape_space() {
    let input = "hello there";
    let output = ObjectDisplay::format_literal_str(
        input,
        ObjectDisplayOptions::USE_QUOTES | ObjectDisplayOptions::ESCAPE_NON_PRINTABLE_CHARACTERS,
    );
    assert_eq!(output, "\"hello there\"");
}

#[test]
fn objectdisplay_format_literal_bool_returns_the_correct_values() {
    let cases: &[(bool, &str)] = &[(true, "true"), (false, "false")];
    for (input, expected) in cases {
        assert_eq!(ObjectDisplay::format_literal_bool(*input), *expected);
    }
}

#[test]
fn objectdisplay_format_literal_double_outputs_hexadecimal_floats_when_asked_to() {
    let input = 255.255;
    let decimal = ObjectDisplay::format_literal_f64(input, ObjectDisplayOptions::NONE);
    let hexadecimal =
        ObjectDisplay::format_literal_f64(input, ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS);
    assert_eq!(decimal, "255.255");
    assert_eq!(hexadecimal, HexFloat::double_to_hex_string(input));
}

#[test]
fn objectdisplay_format_literal_long_outputs_hexadecimal_integers_when_asked_to() {
    let input = 65535i64;
    let decimal = ObjectDisplay::format_literal_i64(input, ObjectDisplayOptions::NONE);
    let hexadecimal =
        ObjectDisplay::format_literal_i64(input, ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS);
    assert_eq!(decimal, "65535");
    assert_eq!(hexadecimal, "0xFFFF");
}

#[test]
fn objectdisplay_format_literal_ulong_outputs_hexadecimal_integers_when_asked_to() {
    let input = 65535u64;
    let decimal = ObjectDisplay::format_literal_u64(input, ObjectDisplayOptions::NONE);
    let hexadecimal =
        ObjectDisplay::format_literal_u64(input, ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS);
    assert_eq!(decimal, "65535ULL");
    assert_eq!(hexadecimal, "0xFFFFULL");
}

#[test]
fn objectdisplay_format_literal_complex_outputs_hexadecimal_numbers_when_asked_to() {
    // C# new Complex(0, 255.255) — only the imaginary part is formatted.
    let imaginary = 255.255;
    let decimal = ObjectDisplay::format_literal_complex(imaginary, ObjectDisplayOptions::NONE);
    let hexadecimal = ObjectDisplay::format_literal_complex(
        imaginary,
        ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS,
    );
    assert_eq!(decimal, "255.255i");
    assert_eq!(
        hexadecimal,
        format!("{}i", HexFloat::double_to_hex_string(imaginary))
    );
}
