// Ported from Compilers/Lua/Test/Portable/Utilities/StringUtilsTests.cs (b767b4e):
// StringUtils_Trim_WorksCorrectly, StringUtils_TrimSpan_WorksCorrectly
// (the string and span overloads are merged into one trim(&str) per the port)

use loretta::utilities::stringutils::StringUtils;

#[test]
fn stringutils_trim_works_correctly() {
    let cases: &[(&str, &str)] = &[
        ("a", "a"),
        (" a", "a"),
        ("\ta\t", "a"),
        (" a ", "a"),
        ("a ", "a"),
        ("\u{b}\t\r\n a\u{b}\r\n\t ", "a"),
    ];
    for (input, expected) in cases {
        assert_eq!(StringUtils::trim(input), *expected, "trim({input:?})");
    }
}

#[test]
fn stringutils_trim_span_works_correctly() {
    // Merged ReadOnlySpan<char> overload: identical semantics to trim(string).
    let cases: &[(&str, &str)] = &[
        ("a", "a"),
        (" a", "a"),
        ("\ta\t", "a"),
        (" a ", "a"),
        ("a ", "a"),
        ("\u{b}\t\r\n a\u{b}\r\n\t ", "a"),
    ];
    for (input, expected) in cases {
        assert_eq!(StringUtils::trim(input), *expected, "trim_span({input:?})");
    }
}
