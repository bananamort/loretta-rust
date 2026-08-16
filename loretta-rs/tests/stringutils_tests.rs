// Ported from Compilers/Lua/Test/Portable/Utilities/StringUtilsTests.cs (b767b4e): StringUtilsTests
// Full C# re-read 2026-08-16 (32 lines, row 753): 2 #[test] case tables cover
// both [Test]s and all 12 [Arguments] cases (the string and span overloads
// merge into one trim(&str) per the port).

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
