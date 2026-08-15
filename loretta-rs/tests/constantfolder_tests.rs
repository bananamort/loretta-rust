// Ported from Compilers/Lua/Test/Portable/Experimental/ConstantFolderTests.cs (b767b4e):
// ConstantFolder_FoldsOperationsCorrectly,
// ConstantFolder_DoesNotFoldOtherOperations,
// ConstantFolder_FoldsOperationsCorrectlyWithStringExtractionEnabled
//
// The C# tests parse a bare expression, fold it, and assert the folded node
// is a literal carrying the expected value. The port folds `return <expr>`
// as a chunk (full_moon has no bare-expression entry point) and compares the
// folded expression text against the canonical literal text of the expected
// value (ObjectDisplay formatting), which is value-equivalent.

use loretta::experimental::constantfolder::ConstantFolder;
use loretta::experimental::constantfoldingoptions::ConstantFoldingOptions;
use loretta::symbol_display::objectdisplay::ObjectDisplay;
use loretta::symbol_display::objectdisplayoptions::ObjectDisplayOptions;

/// The C# `object? expected` value.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Expected {
    L(i64),
    D(f64),
    S(&'static str),
    B(bool),
    Nil,
}

/// The canonical text of the expected value (ObjectDisplay formatting).
/// Numeric values are compared as text because the C# literal formatting is
/// the deterministic fold output; strings, booleans and nil are compared as
/// values (operands returned by `and`/`or` keep their original literal text).
fn assert_folded_value(folded: &str, expected: Expected) {
    let t = folded.trim_end();
    match expected {
        Expected::L(v) => assert_eq!(
            t,
            ObjectDisplay::format_literal_i64(v, ObjectDisplayOptions::NONE),
            "long text {folded:?}"
        ),
        Expected::D(v) => assert_eq!(
            t,
            ObjectDisplay::format_literal_f64(v, ObjectDisplayOptions::NONE),
            "double text {folded:?}"
        ),
        Expected::S(s) => {
            let value = if t.starts_with('\'') && t.ends_with('\'') {
                &t[1..t.len() - 1]
            } else if t.starts_with('"') && t.ends_with('"') {
                &t[1..t.len() - 1]
            } else {
                panic!("unexpected string text {folded:?}")
            };
            assert_eq!(value, s, "string value {folded:?}");
        }
        Expected::B(v) => {
            let value = match t {
                "true" => true,
                "false" => false,
                _ => panic!("unexpected bool text {folded:?}"),
            };
            assert_eq!(value, v, "bool value {folded:?}");
        }
        Expected::Nil => assert_eq!(t, "nil", "nil text {folded:?}"),
    }
}

/// Folds `return <source>` and returns the folded expression's text.
fn fold_expression(source: &str, options: ConstantFoldingOptions) -> String {
    let code = format!("return {source}\n");
    let result = full_moon::parse_fallible(&code, full_moon::LuaVersion::new());
    let ast = result.into_ast();
    let folded = ConstantFolder::new(options).fold_ast(ast);
    let text = folded.to_string();
    let body = text.strip_prefix("return ").unwrap_or(&text);
    body.strip_suffix('\n').unwrap_or(body).to_string()
}

#[test]
fn constant_folder_folds_operations_correctly() {
    let cases: &[(&str, Expected)] = &[
        // Unary operators
        //     Negation
        ("-1", Expected::L(-1)),
        ("-1.0", Expected::D(-1.0)),
        ("-1.5", Expected::D(-1.5)),
        //     Logical not
        ("not nil", Expected::B(true)),
        ("not true", Expected::B(false)),
        ("not false", Expected::B(true)),
        ("not 1", Expected::B(false)),
        ("not 'a'", Expected::B(false)),
        ("not function()end", Expected::B(false)),
        //     Bitwise not
        ("~1.0", Expected::D(!1i64 as f64)),
        ("~1", Expected::L(!1)),
        //     Length
        ("#''", Expected::D(0.0)),
        ("#'a'", Expected::D(1.0)),
        ("#'ab'", Expected::D(2.0)),
        ("#'abc'", Expected::D(3.0)),
        // Binary operators
        //     Addition
        ("1 + 1", Expected::L(2)),
        ("1.5 + 1.5", Expected::D(3.0)),
        ("1.5 + 1", Expected::D(2.5)),
        //         Overflow (can't test for doubles as infinity doesn't get folded)
        (
            "9223372036854775807 + 1",
            Expected::L(9223372036854775807i64.wrapping_add(1)),
        ),
        //     Subtraction
        ("1 - 1", Expected::L(0)),
        ("1.5 - 1.5", Expected::D(0.0)),
        ("1.5 - 1", Expected::D(0.5)),
        //         Underflow
        (
            "-9223372036854775807 - 5",
            Expected::L((-9223372036854775807i64).wrapping_sub(5)),
        ),
        //     Multiplication
        ("1.5 * 2.5", Expected::D(1.5 * 2.5)),
        ("1 * 2", Expected::L(2)),
        ("1.5 * 2", Expected::D(3.0)),
        //         Overflow
        ("9223372036854775807 * 2", Expected::L(-2)),
        ("9223372036854775807 * -20", Expected::L(20)),
        //     Division
        ("1.5 / 1.5", Expected::D(1.0)),
        ("5 / 2", Expected::D(2.5)),
        ("5.0 / 2", Expected::D(2.5)),
        ("2 / 5", Expected::D(0.4)),
        //         Overflow in integer division
        (
            "9223372036854775807 / -1",
            Expected::D(9223372036854775807f64 / -1.0),
        ),
        //     Modulo
        ("5 % 2", Expected::L(1)),
        ("5 % 2.5", Expected::D(0.0)),
        ("5.5 % 1", Expected::D(0.5)),
        //     Exponentiation
        ("2 ^ 2", Expected::D(4.0)),
        ("4 ^ 0.5", Expected::D(2.0)),
        //     Concatenation
        ("'a' .. 'b'", Expected::S("ab")),
        ("'a' .. true", Expected::S("atrue")),
        ("'a' .. false", Expected::S("afalse")),
        //     Equality
        ("'a' == 'a'", Expected::B(true)),
        ("'a' == 'b'", Expected::B(false)),
        ("1 == 1", Expected::B(true)),
        ("1 == 2", Expected::B(false)),
        ("1.0 == 1", Expected::B(true)),
        ("1.1 == 1", Expected::B(false)),
        ("nil == nil", Expected::B(true)),
        ("true == true", Expected::B(true)),
        ("true == false", Expected::B(false)),
        ("false == false", Expected::B(true)),
        ("'a' == false", Expected::B(false)),
        //     Inequality
        ("'a' ~= 'a'", Expected::B(false)),
        ("'a' ~= 'b'", Expected::B(true)),
        ("1 ~= 1", Expected::B(false)),
        ("1 ~= 2", Expected::B(true)),
        ("1.0 ~= 1", Expected::B(false)),
        ("1.1 ~= 1", Expected::B(true)),
        ("nil ~= nil", Expected::B(false)),
        ("1 ~= nil", Expected::B(true)),
        ("true ~= true", Expected::B(false)),
        ("true ~= false", Expected::B(true)),
        ("false ~= false", Expected::B(false)),
        //     Less than
        ("1 < 2", Expected::B(true)),
        ("1 < 1", Expected::B(false)),
        ("2 < 1", Expected::B(false)),
        ("1 < 1.5", Expected::B(true)),
        ("1.5 < 1", Expected::B(false)),
        ("1.5 < 1.5", Expected::B(false)),
        ("'a' < 'b'", Expected::B(true)),
        ("'a' < 'a'", Expected::B(false)),
        ("'b' < 'a'", Expected::B(false)),
        //     Less than or equals
        ("1 <= 1", Expected::B(true)),
        ("1 <= 2", Expected::B(true)),
        ("2 <= 1", Expected::B(false)),
        ("1.5 <= 1.5", Expected::B(true)),
        ("1.5 <= 2", Expected::B(true)),
        ("2 <= 1.5", Expected::B(false)),
        ("'a' <= 'a'", Expected::B(true)),
        ("'a' <= 'b'", Expected::B(true)),
        ("'b' <= 'a'", Expected::B(false)),
        //     Greater than
        ("2 > 1", Expected::B(true)),
        ("1 > 1", Expected::B(false)),
        ("1 > 2", Expected::B(false)),
        ("1.5 > 1", Expected::B(true)),
        ("1 > 1.5", Expected::B(false)),
        ("1.5 > 1.5", Expected::B(false)),
        ("'b' > 'a'", Expected::B(true)),
        ("'a' > 'a'", Expected::B(false)),
        ("'a' > 'b'", Expected::B(false)),
        //     Greater than or equal
        ("1 >= 1", Expected::B(true)),
        ("2 >= 1", Expected::B(true)),
        ("1 >= 2", Expected::B(false)),
        ("1.5 >= 1.5", Expected::B(true)),
        ("2 >= 1.5", Expected::B(true)),
        ("1.5 >= 2", Expected::B(false)),
        ("'a' >= 'a'", Expected::B(true)),
        ("'b' >= 'a'", Expected::B(true)),
        ("'a' >= 'b'", Expected::B(false)),
        //     Logical and
        ("nil and 2", Expected::Nil),
        ("true and 2", Expected::L(2)),
        ("false and 2", Expected::B(false)),
        ("1 and 2", Expected::L(2)),
        ("'a' and 2", Expected::L(2)),
        ("function()end and 2", Expected::L(2)),
        //     Logical or
        ("nil or 2", Expected::L(2)),
        ("true or 2", Expected::B(true)),
        ("false or 2", Expected::L(2)),
        ("1 or 2", Expected::L(1)),
        ("'a' or 2", Expected::S("a")),
        ("2 or function()end", Expected::L(2)),
        //     Bitwise or
        ("1 | 1", Expected::L(1)),
        ("1 | 1.0", Expected::L(1)),
        ("1.0 | 1", Expected::L(1)),
        ("1.0 | 1.0", Expected::D(1.0)),
        ("1 | 2", Expected::L(3)),
        //     Bitwise and
        ("1 & 1", Expected::L(1)),
        ("1 & 1.0", Expected::L(1)),
        ("1.0 & 1", Expected::L(1)),
        ("1.0 & 1.0", Expected::D(1.0)),
        ("1 & 2", Expected::L(0)),
        //     Right shift
        ("511 >> 3", Expected::L(511i64 >> 3)),
        ("511 >> 3.0", Expected::L(511i64 >> 3)),
        ("511.0 >> 3", Expected::L(511i64 >> 3)),
        ("511.0 >> 3.0", Expected::D((511i64 >> 3) as f64)),
        //     Left shift
        ("511 << 3", Expected::L(511i64 << 3)),
        ("511 << 3.0", Expected::L(511i64 << 3)),
        ("511.0 << 3", Expected::L(511i64 << 3)),
        ("511.0 << 3.0", Expected::D((511i64 << 3) as f64)),
        //     Bitwise xor
        ("42 ~ 21", Expected::L(42i64 ^ 21)),
        ("42 ~ 21.0", Expected::L(42i64 ^ 21)),
        ("42.0 ~ 21", Expected::L(42i64 ^ 21)),
        ("42.0 ~ 21.0", Expected::D((42i64 ^ 21) as f64)),
        ("42 ~ 42", Expected::L(0)),
        ("42 ~ 42.0", Expected::L(0)),
        ("42.0 ~ 42", Expected::L(0)),
        ("42.0 ~ 42.0", Expected::D(0.0)),
    ];
    for (source, expected) in cases {
        let folded = fold_expression(source, ConstantFoldingOptions::DEFAULT);
        assert_folded_value(&folded, *expected);
    }
}

#[test]
fn constant_folder_does_not_fold_other_operations() {
    let cases: &[&str] = &[
        // Unary operator
        "-a",
        "-{}",
        "-'1'",
        //     Logical not
        "not func()",
        //     Bitwise not
        "~a",
        "~1.5",
        "~'1'",
        //     Length
        "#{}",
        "#{nil}",
        // Binary operator
        //     Addition
        "nil + true",
        "function()end + true",
        "'1' + '1'",
        //         Infinity
        "1.7976931348623157E+308 + 1.7976931348623157E+308",
        //     Subtraction
        "nil - true",
        "function()end - true",
        "'1' - '1'",
        //     Multiplication
        "nil * 2",
        "function()end * 2",
        "'1' * '1'",
        //         Infinity
        "1.7976931348623157E+308 * 2",
        //     Division
        "2 / a",
        "1.7976931348623157E+308 / true",
        "'1' / '1'",
        //     Modulo
        "true % 2",
        "2 % f()",
        "'1' % '1'",
        //     Exponentiation
        "1.7976931348623157E+308 ^ 2",
        //     Concatenation
        "1 .. 2",
        //     Equality
        "{} == {}",
        "function()end == function()end",
        "a == a",
        //     Inequality
        "{} ~= {}",
        "function()end ~= function()end",
        "a ~= a",
        //     Less than
        "true < true",
        "true < false",
        "function()end < function()end",
        //     Less than or equals
        "true <= true",
        "a <= a",
        "function()end <= function()end",
        //     Greater than
        "true > true",
        "true > false",
        "function()end > function()end",
        //     Greater than or equals
        "true >= true",
        "true >= false",
        "function()end >= function()end",
        //     Logical and
        "func() and 1",
        "a and 1",
        "{} and 2",
        //     Logical or
        "func() or 1",
        "a or 1",
        "{} or 2",
        //     Bitwise or
        "1.5 | 1",
        "1 | 1.5",
        "1.1 | 1.1",
        "a | a",
        "function()end | function()end",
        "'1' | '1'",
        //     Bitwise and
        "1.5 & 1",
        "1 & 1.5",
        "1.1 & 1.1",
        "a & a",
        "function()end & function()end",
        "'1' & '1'",
        //     Right shift
        "1.5 >> 1",
        "1 >> 1.5",
        "1.5 >> 1.5",
        "a >> a",
        "function()end >> function()end",
        "'1' >> '1'",
        //     Left shift
        "1.5 << 1",
        "1 << 1.5",
        "1.5 << 1.5",
        "a << a",
        "function()end << function()end",
        "'1' << '1'",
        //     Bitwise xor
        "1.5 ~ 1.5",
        "1.1 ~ 1.1",
        "'1' ~ '1'",
    ];
    for source in cases {
        let code = format!("return {source}\n");
        let folded = fold_expression(source, ConstantFoldingOptions::DEFAULT);
        assert_eq!(folded, *source, "no-fold({source:?})");
        let _ = code;
    }
}

#[test]
fn constant_folder_folds_operations_correctly_with_string_extraction_enabled() {
    let options = ConstantFoldingOptions {
        extract_numbers_from_strings: true,
    };
    let cases: &[(&str, Expected)] = &[
        // Unary operators
        //     Negation
        ("-'1'", Expected::L(-1)),
        ("-'1.0'", Expected::D(-1.0)),
        ("-'1.5'", Expected::D(-1.5)),
        //     Bitwise not
        ("~'1.0'", Expected::D(!1i64 as f64)),
        ("~'1'", Expected::D(!1i64 as f64)),
        // Binary operators
        //     Addition
        ("'1' + 1", Expected::L(2)),
        ("1.5 + '1.5'", Expected::D(3.0)),
        ("'1.5' + 1", Expected::D(2.5)),
        //         Overflow
        (
            "'9223372036854775807' + 1",
            Expected::L(9223372036854775807i64.wrapping_add(1)),
        ),
        //     Subtraction
        ("'1' - 1", Expected::L(0)),
        ("1.5 - '1.5'", Expected::D(0.0)),
        ("'1.5' - 1", Expected::D(0.5)),
        //         Underflow
        (
            "'-9223372036854775808' - 2",
            Expected::L((-9223372036854775808i64).wrapping_sub(2)),
        ),
        //     Multiplication
        ("'1.5' * 2.5", Expected::D(1.5 * 2.5)),
        ("1 * '2'", Expected::L(2)),
        ("'1.5' * 2", Expected::D(3.0)),
        //         Overflow
        ("'9223372036854775807' * 2", Expected::L(-2)),
        ("'9223372036854775807' * -20", Expected::L(20)),
        //     Division
        ("'1.5' / 1.5", Expected::D(1.0)),
        ("'5' / 2", Expected::D(2.5)),
        ("5.0 / '2'", Expected::D(2.5)),
        ("'2' / 5", Expected::D(0.4)),
        //         Overflow in integer division
        (
            "'9223372036854775807' / -1",
            Expected::D(9223372036854775807f64 / -1.0),
        ),
        //     Modulo
        ("'5' % 2", Expected::L(1)),
        ("5 % '2.5'", Expected::D(0.0)),
        ("'5.5' % 1", Expected::D(0.5)),
        //     Exponentiation
        ("'2' ^ 2", Expected::D(4.0)),
        ("4 ^ '0.5'", Expected::D(2.0)),
        //     Bitwise or
        ("'1' | 1", Expected::L(1)),
        ("1 | '1.0'", Expected::L(1)),
        ("'1.0' | 1", Expected::L(1)),
        ("1.0 | '1.0'", Expected::D(1.0)),
        ("'1' | 2", Expected::L(3)),
        //     Bitwise and
        ("'1' & 1", Expected::L(1)),
        ("1 & '1.0'", Expected::L(1)),
        ("'1.0' & 1", Expected::L(1)),
        ("1.0 & '1.0'", Expected::D(1.0)),
        ("'1' & 2", Expected::L(0)),
        //     Right shift
        ("'511' >> 3", Expected::L(511i64 >> 3)),
        ("511 >> '3.0'", Expected::L(511i64 >> 3)),
        ("'511.0' >> 3", Expected::L(511i64 >> 3)),
        ("511.0 >> '3.0'", Expected::D((511i64 >> 3) as f64)),
        //     Left shift
        ("'511' << 3", Expected::L(511i64 << 3)),
        ("511 << '3.0'", Expected::L(511i64 << 3)),
        ("'511.0' << 3", Expected::L(511i64 << 3)),
        ("511.0 << '3.0'", Expected::D((511i64 << 3) as f64)),
        //     Bitwise xor
        ("'42' ~ 21", Expected::L(42i64 ^ 21)),
        ("42 ~ '21.0'", Expected::L(42i64 ^ 21)),
        ("'42.0' ~ 21", Expected::L(42i64 ^ 21)),
        ("42.0 ~ '21.0'", Expected::D((42i64 ^ 21) as f64)),
        ("'42' ~ 42", Expected::L(0)),
        ("42 ~ '42.0'", Expected::L(0)),
        ("'42.0' ~ 42", Expected::L(0)),
        ("42.0 ~ '42.0'", Expected::D(0.0)),
    ];
    for (source, expected) in cases {
        let folded = fold_expression(source, options.clone());
        assert_folded_value(&folded, *expected);
    }
}
