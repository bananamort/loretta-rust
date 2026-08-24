// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Experimental.ConstantFolderTests (b767b4e): ConstantFolderTests
// C# source: src/Compilers/Lua/Test/Portable/Experimental/ConstantFolderTests.cs
//
// The C# ParseAndValidateExpressionAsync parses a bare expression. full_moon
// has no expression entry point (the differential parse op confirms `1 + 1`
// errors as "unexpected token, this needs to be a statement"), so the port
// parses the expression wrapped in a statement context (`local _ = <expr>`)
// and extracts the assignment's first expression — the C# round-trip and
// no-diagnostics assertions hold on the extracted expression text.

use full_moon::ast::{Ast, Block, Expression, Stmt};
use full_moon::tokenizer::{Symbol, TokenType};

use loretta::experimental::constantfoldingoptions::ConstantFoldingOptions;
use loretta::experimental::luaextensions::constant_fold;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

/// The C# expected literal values (long/double/bool/string/null).
#[derive(Debug, Clone, PartialEq)]
enum LuaValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Nil,
}

impl LuaValue {
    fn num_eq(&self, other: &LuaValue) -> bool {
        match (self, other) {
            (LuaValue::Integer(a), LuaValue::Integer(b)) => a == b,
            (LuaValue::Float(a), LuaValue::Float(b)) => a == b,
            (LuaValue::Integer(a), LuaValue::Float(b)) => *a as f64 == *b,
            (LuaValue::Float(a), LuaValue::Integer(b)) => *a == *b as f64,
            _ => false,
        }
    }
}

/// Asserts the folded value equals the expected (C# HasValue) with the
/// integer-float value equality the C# object comparison provides.
fn assert_value(actual: &LuaValue, expected: &LuaValue) {
    match (actual, expected) {
        (LuaValue::Integer(_) | LuaValue::Float(_), LuaValue::Integer(_) | LuaValue::Float(_)) => {
            assert!(
                actual.num_eq(expected),
                "expected {expected:?} got {actual:?}"
            )
        }
        _ => assert_eq!(actual, expected, "expected {expected:?} got {actual:?}"),
    }
}

/// The folded literal's value (C# token.HasValue).
fn folded_value(expr: &Expression) -> Option<LuaValue> {
    match expr {
        Expression::Symbol(t) if t.is_symbol(Symbol::Nil) => Some(LuaValue::Nil),
        Expression::Symbol(t) if t.is_symbol(Symbol::True) => Some(LuaValue::Bool(true)),
        Expression::Symbol(t) if t.is_symbol(Symbol::False) => Some(LuaValue::Bool(false)),
        Expression::Number(t) => {
            let text = t.token().to_string();
            if is_double_text(&text) {
                text.parse::<f64>().ok().map(LuaValue::Float)
            } else {
                text.parse::<i64>().ok().map(LuaValue::Integer)
            }
        }
        Expression::String(t) => {
            let TokenType::StringLiteral { literal, .. } = t.token().token_type() else {
                return None;
            };
            Some(LuaValue::String(literal.as_str().to_string()))
        }
        _ => None,
    }
}

fn is_double_text(text: &str) -> bool {
    text.contains('.')
        || text.contains('e')
        || text.contains('E')
        || text.contains('p')
        || text.contains('P')
}

/// Parses the expression in the statement wrapper and returns the local
/// assignment block for extraction.
fn parse_wrapped(source: &str, syntax_options: &LuaSyntaxOptions) -> (Ast, String) {
    let wrapper = format!("local _ = {source}");
    let options = loretta::luaparseoptions::LuaParseOptions::new(syntax_options.clone());
    let ast = full_moon::parse_fallible(
        &wrapper,
        loretta_tests::luatestbase::options_to_version(&options),
    )
    .into_result()
    .unwrap_or_else(|errors| panic!("the wrapped expression must parse ({source:?}): {errors:?}"));
    (ast, wrapper)
}

/// Returns the folded first expression of the wrapped local assignment.
fn fold_expression(source: &str, extract_numbers: bool) -> Expression {
    let syntax_options = LuaSyntaxOptions::ALL_WITH_INTEGERS;
    let (ast, _wrapper) = parse_wrapped(source, &syntax_options);
    let options = ConstantFoldingOptions {
        extract_numbers_from_strings: extract_numbers,
    };
    let folded = constant_fold(ast, options);
    let block = folded.nodes();
    let stmt = block
        .stmts()
        .next()
        .expect("the wrapper must contain the local assignment");
    first_local_expression(stmt).expect("the wrapper statement must be a local assignment")
}

fn first_local_expression(stmt: &Stmt) -> Option<Expression> {
    let Stmt::LocalAssignment(la) = stmt else {
        return None;
    };
    la.expressions().iter().next().cloned()
}

/// The folded expression text and the wrapper text (for the no-fold
/// assertion).
fn folded_wrapper_text(source: &str, extract_numbers: bool) -> (String, String) {
    let syntax_options = LuaSyntaxOptions::ALL_WITH_INTEGERS;
    let (ast, wrapper) = parse_wrapped(source, &syntax_options);
    let options = ConstantFoldingOptions {
        extract_numbers_from_strings: extract_numbers,
    };
    let folded = constant_fold(ast, options);
    let expr = {
        let block: &Block = folded.nodes();
        let stmt = block
            .stmts()
            .next()
            .expect("the wrapper must contain the local assignment");
        first_local_expression(stmt).expect("the wrapper statement must be a local assignment")
    };
    (expr.to_string(), wrapper)
}

#[test]
fn constant_folder_folds_operations_correctly() {
    let cases: &[(&str, LuaValue)] = &[
        // Unary operators — negation.
        ("-1", LuaValue::Integer(-1)),
        ("-1.0", LuaValue::Float(-1.0)),
        ("-1.5", LuaValue::Float(-1.5)),
        // Logical not.
        ("not nil", LuaValue::Bool(true)),
        ("not true", LuaValue::Bool(false)),
        ("not false", LuaValue::Bool(true)),
        ("not 1", LuaValue::Bool(false)),
        ("not 'a'", LuaValue::Bool(false)),
        ("not function()end", LuaValue::Bool(false)),
        // Bitwise not.
        ("~1.0", LuaValue::Float(-2.0)),
        ("~1", LuaValue::Integer(-2)),
        // Length.
        ("#''", LuaValue::Float(0.0)),
        ("#'a'", LuaValue::Float(1.0)),
        ("#'ab'", LuaValue::Float(2.0)),
        ("#'abc'", LuaValue::Float(3.0)),
        // Addition.
        ("1 + 1", LuaValue::Integer(2)),
        ("1.5 + 1.5", LuaValue::Float(3.0)),
        ("1.5 + 1", LuaValue::Float(2.5)),
        ("9223372036854775807 + 1", LuaValue::Integer(i64::MIN)),
        // Overflowing and unusual integer literals (Finding 2): the C#
        // TryParse family folds overflow to 0 (Lexer.Numbers.cs), hex is a
        // two's-complement bit pattern, and binary values >= 2^63 fold to 0.
        // Every case pinned against the C# oracle (AllWithIntegers).
        ("9223372036854775808 + 1", LuaValue::Integer(1)),
        ("18446744073709551615 + 1", LuaValue::Integer(1)),
        ("0xffffffffffffffff + 1", LuaValue::Integer(0)),
        ("0x8000000000000000 + 1", LuaValue::Integer(i64::MIN + 1)),
        ("0x10000000000000000 + 1", LuaValue::Integer(1)),
        ("0b101 + 1", LuaValue::Integer(6)),
        (
            "0b1000000000000000000000000000000000000000000000000000000000000000 + 1",
            LuaValue::Integer(1),
        ),
        (
            "0b111111111111111111111111111111111111111111111111111111111111111 + 1",
            LuaValue::Integer(i64::MIN),
        ),
        ("1_000 + 1", LuaValue::Integer(1001)),
        ("0x1_0 + 1", LuaValue::Integer(17)),
        ("0b1_0 + 1", LuaValue::Integer(3)),
        ("-9223372036854775808 + 1", LuaValue::Integer(1)),
        // Hex digits containing 'e'/'E' (Finding 19): 0xE5 and 0x1e5 are
        // integers — only '.'/'p'/'P' make a hex literal a double — so
        // they fold with exact i64 arithmetic (visible beyond 2^53).
        ("0xE5 + 1", LuaValue::Integer(230)),
        ("0x1e5 + 1", LuaValue::Integer(486)),
        (
            "0xE0000000000000 == 0xE0000000000001",
            LuaValue::Bool(false),
        ),
        ("0xE0000000000000 == 0xE0000000000000", LuaValue::Bool(true)),
        // Subtraction.
        ("1 - 1", LuaValue::Integer(0)),
        ("1.5 - 1.5", LuaValue::Float(0.0)),
        ("1.5 - 1", LuaValue::Float(0.5)),
        (
            "-9223372036854775807 - 5",
            LuaValue::Integer(i64::MAX.wrapping_neg().wrapping_sub(5)),
        ),
        // Multiplication.
        ("1.5 * 2.5", LuaValue::Float(3.75)),
        ("1 * 2", LuaValue::Integer(2)),
        ("1.5 * 2", LuaValue::Float(3.0)),
        ("9223372036854775807 * 2", LuaValue::Integer(-2)),
        ("9223372036854775807 * -20", LuaValue::Integer(20)),
        // Division.
        ("1.5 / 1.5", LuaValue::Float(1.0)),
        ("5 / 2", LuaValue::Float(2.5)),
        ("5.0 / 2", LuaValue::Float(2.5)),
        ("2 / 5", LuaValue::Float(0.4)),
        (
            "9223372036854775807 / -1",
            LuaValue::Float(-9223372036854775808.0),
        ),
        // Modulo.
        ("5 % 2", LuaValue::Integer(1)),
        ("5 % 2.5", LuaValue::Float(0.0)),
        ("5.5 % 1", LuaValue::Float(0.5)),
        // Exponentiation.
        ("2 ^ 2", LuaValue::Float(4.0)),
        ("4 ^ 0.5", LuaValue::Float(2.0)),
        // Concatenation.
        ("'a' .. 'b'", LuaValue::String("ab".into())),
        ("'a' .. true", LuaValue::String("atrue".into())),
        ("'a' .. false", LuaValue::String("afalse".into())),
        // Equality.
        ("'a' == 'a'", LuaValue::Bool(true)),
        ("'a' == 'b'", LuaValue::Bool(false)),
        ("1 == 1", LuaValue::Bool(true)),
        ("1 == 2", LuaValue::Bool(false)),
        ("1.0 == 1", LuaValue::Bool(true)),
        ("1.1 == 1", LuaValue::Bool(false)),
        ("nil == nil", LuaValue::Bool(true)),
        ("true == true", LuaValue::Bool(true)),
        ("true == false", LuaValue::Bool(false)),
        ("false == false", LuaValue::Bool(true)),
        ("'a' == false", LuaValue::Bool(false)),
        // Inequality.
        ("'a' ~= 'a'", LuaValue::Bool(false)),
        ("'a' ~= 'b'", LuaValue::Bool(true)),
        ("1 ~= 1", LuaValue::Bool(false)),
        ("1 ~= 2", LuaValue::Bool(true)),
        ("1.0 ~= 1", LuaValue::Bool(false)),
        ("1.1 ~= 1", LuaValue::Bool(true)),
        ("nil ~= nil", LuaValue::Bool(false)),
        ("1 ~= nil", LuaValue::Bool(true)),
        ("true ~= true", LuaValue::Bool(false)),
        ("true ~= false", LuaValue::Bool(true)),
        ("false ~= false", LuaValue::Bool(false)),
        // Less than.
        ("1 < 2", LuaValue::Bool(true)),
        ("1 < 1", LuaValue::Bool(false)),
        ("2 < 1", LuaValue::Bool(false)),
        ("1 < 1.5", LuaValue::Bool(true)),
        ("1.5 < 1", LuaValue::Bool(false)),
        ("1.5 < 1.5", LuaValue::Bool(false)),
        ("'a' < 'b'", LuaValue::Bool(true)),
        ("'a' < 'a'", LuaValue::Bool(false)),
        ("'b' < 'a'", LuaValue::Bool(false)),
        // Less than or equals.
        ("1 <= 1", LuaValue::Bool(true)),
        ("1 <= 2", LuaValue::Bool(true)),
        ("2 <= 1", LuaValue::Bool(false)),
        ("1.5 <= 1.5", LuaValue::Bool(true)),
        ("1.5 <= 2", LuaValue::Bool(true)),
        ("2 <= 1.5", LuaValue::Bool(false)),
        ("'a' <= 'a'", LuaValue::Bool(true)),
        ("'a' <= 'b'", LuaValue::Bool(true)),
        ("'b' <= 'a'", LuaValue::Bool(false)),
        // Greater than.
        ("2 > 1", LuaValue::Bool(true)),
        ("1 > 1", LuaValue::Bool(false)),
        ("1 > 2", LuaValue::Bool(false)),
        ("1.5 > 1", LuaValue::Bool(true)),
        ("1 > 1.5", LuaValue::Bool(false)),
        ("1.5 > 1.5", LuaValue::Bool(false)),
        ("'b' > 'a'", LuaValue::Bool(true)),
        ("'a' > 'a'", LuaValue::Bool(false)),
        ("'a' > 'b'", LuaValue::Bool(false)),
        // Greater than or equal.
        ("1 >= 1", LuaValue::Bool(true)),
        ("2 >= 1", LuaValue::Bool(true)),
        ("1 >= 2", LuaValue::Bool(false)),
        ("1.5 >= 1.5", LuaValue::Bool(true)),
        ("2 >= 1.5", LuaValue::Bool(true)),
        ("1.5 >= 2", LuaValue::Bool(false)),
        ("'a' >= 'a'", LuaValue::Bool(true)),
        ("'b' >= 'a'", LuaValue::Bool(true)),
        ("'a' >= 'b'", LuaValue::Bool(false)),
        // Logical and.
        ("nil and 2", LuaValue::Nil),
        ("true and 2", LuaValue::Integer(2)),
        ("false and 2", LuaValue::Bool(false)),
        ("1 and 2", LuaValue::Integer(2)),
        ("'a' and 2", LuaValue::Integer(2)),
        ("function()end and 2", LuaValue::Integer(2)),
        // Logical or.
        ("nil or 2", LuaValue::Integer(2)),
        ("true or 2", LuaValue::Bool(true)),
        ("false or 2", LuaValue::Integer(2)),
        ("1 or 2", LuaValue::Integer(1)),
        ("'a' or 2", LuaValue::String("a".into())),
        ("2 or function()end", LuaValue::Integer(2)),
        // Bitwise or.
        ("1 | 1", LuaValue::Integer(1)),
        ("1 | 1.0", LuaValue::Integer(1)),
        ("1.0 | 1", LuaValue::Integer(1)),
        ("1.0 | 1.0", LuaValue::Float(1.0)),
        ("1 | 2", LuaValue::Integer(3)),
        // Bitwise and.
        ("1 & 1", LuaValue::Integer(1)),
        ("1 & 1.0", LuaValue::Integer(1)),
        ("1.0 & 1", LuaValue::Integer(1)),
        ("1.0 & 1.0", LuaValue::Float(1.0)),
        ("1 & 2", LuaValue::Integer(0)),
        // Right shift.
        ("511 >> 3", LuaValue::Integer(63)),
        ("511 >> 3.0", LuaValue::Integer(63)),
        ("511.0 >> 3", LuaValue::Integer(63)),
        ("511.0 >> 3.0", LuaValue::Float(63.0)),
        // Left shift.
        ("511 << 3", LuaValue::Integer(4088)),
        ("511 << 3.0", LuaValue::Integer(4088)),
        ("511.0 << 3", LuaValue::Integer(4088)),
        ("511.0 << 3.0", LuaValue::Float(4088.0)),
        // Bitwise xor.
        ("42 ~ 21", LuaValue::Integer(63)),
        ("42 ~ 21.0", LuaValue::Integer(63)),
        ("42.0 ~ 21", LuaValue::Integer(63)),
        ("42.0 ~ 21.0", LuaValue::Float(63.0)),
        ("42 ~ 42", LuaValue::Integer(0)),
        ("42 ~ 42.0", LuaValue::Integer(0)),
        ("42.0 ~ 42", LuaValue::Integer(0)),
        ("42.0 ~ 42.0", LuaValue::Float(0.0)),
    ];
    for (source, expected) in cases {
        let folded = fold_expression(source, false);
        let actual = folded_value(&folded)
            .unwrap_or_else(|| panic!("the fold of {source:?} must produce a literal: {folded:?}"));
        assert_value(&actual, expected);
    }
}

#[test]
fn constant_folder_does_not_fold_other_operations() {
    let cases: &[&str] = &[
        // Unary operators.
        "-a",
        "-{}",
        "-'1'",
        "not func()",
        "~a",
        "~1.5",
        "~'1'",
        "#{}",
        "#{nil}",
        // Binary operators.
        "nil + true",
        "function()end + true",
        "'1' + '1'",
        "1.7976931348623157E+308 + 1.7976931348623157E+308",
        "nil - true",
        "function()end - true",
        "'1' - '1'",
        "nil * 2",
        "function()end * 2",
        "'1' * '1'",
        "1.7976931348623157E+308 * 2",
        "2 / a",
        "1.7976931348623157E+308 / true",
        "'1' / '1'",
        "true % 2",
        "2 % f()",
        "'1' % '1'",
        // `%` by zero (Finding 3): the C# double path yields NaN and does
        // not fold (ConstantFolder.cs:100-102) — the port matches instead
        // of panicking on the Long path.
        "5 % 0",
        "5.0 % 0",
        "5 % 0.0",
        "5.0 % 0.0",
        "-5 % 0",
        "1.7976931348623157E+308 ^ 2",
        "1 .. 2",
        "{} == {}",
        "function()end == function()end",
        "a == a",
        "{} ~= {}",
        "function()end ~= function()end",
        "a ~= a",
        "true < true",
        "true < false",
        "function()end < function()end",
        "true <= true",
        "a <= a",
        "function()end <= function()end",
        "true > true",
        "true > false",
        "function()end > function()end",
        "true >= true",
        "true >= false",
        "function()end >= function()end",
        "func() and 1",
        "a and 1",
        "{} and 2",
        "func() or 1",
        "a or 1",
        "{} or 2",
        "1.5 | 1",
        "1 | 1.5",
        "1.1 | 1.1",
        "a | a",
        "function()end | function()end",
        "'1' | '1'",
        "1.5 & 1",
        "1 & 1.5",
        "1.1 & 1.1",
        "a & a",
        "function()end & function()end",
        "'1' & '1'",
        "1.5 >> 1",
        "1 >> 1.5",
        "1.5 >> 1.5",
        "a >> a",
        "function()end >> function()end",
        "'1' >> '1'",
        "1.5 << 1",
        "1 << 1.5",
        "1.5 << 1.5",
        "a << a",
        "function()end << function()end",
        "'1' << '1'",
        "1.5 ~ 1.5",
        "1.1 ~ 1.1",
        "'1' ~ '1'",
    ];
    for source in cases {
        let (folded, _wrapper) = folded_wrapper_text(source, false);
        assert_eq!(
            folded, *source,
            "the fold of {source:?} must not change the expression"
        );
    }
}

#[test]
fn constant_folder_folds_operations_correctly_with_string_extraction_enabled() {
    let cases: &[(&str, LuaValue)] = &[
        // Unary operators — negation over strings.
        ("-'1'", LuaValue::Integer(-1)),
        ("-'1.0'", LuaValue::Float(-1.0)),
        ("-'1.5'", LuaValue::Float(-1.5)),
        // Bitwise not over strings.
        ("~'1.0'", LuaValue::Float(-2.0)),
        ("~'1'", LuaValue::Float(-2.0)),
        // Addition over strings.
        ("'1' + 1", LuaValue::Integer(2)),
        ("1.5 + '1.5'", LuaValue::Float(3.0)),
        ("'1.5' + 1", LuaValue::Float(2.5)),
        ("'9223372036854775807' + 1", LuaValue::Integer(i64::MIN)),
        // Subtraction over strings.
        ("'1' - 1", LuaValue::Integer(0)),
        ("1.5 - '1.5'", LuaValue::Float(0.0)),
        ("'1.5' - 1", LuaValue::Float(0.5)),
        (
            "'-9223372036854775808' - 2",
            LuaValue::Integer(i64::MIN.wrapping_sub(2)),
        ),
        // Multiplication over strings.
        ("'1.5' * 2.5", LuaValue::Float(3.75)),
        ("1 * '2'", LuaValue::Integer(2)),
        ("'1.5' * 2", LuaValue::Float(3.0)),
        ("'9223372036854775807' * 2", LuaValue::Integer(-2)),
        ("'9223372036854775807' * -20", LuaValue::Integer(20)),
        // Division over strings.
        ("'1.5' / 1.5", LuaValue::Float(1.0)),
        ("'5' / 2", LuaValue::Float(2.5)),
        ("5.0 / '2'", LuaValue::Float(2.5)),
        ("'2' / 5", LuaValue::Float(0.4)),
        (
            "'9223372036854775807' / -1",
            LuaValue::Float(-9223372036854775808.0),
        ),
        // Modulo over strings.
        ("'5' % 2", LuaValue::Integer(1)),
        ("5 % '2.5'", LuaValue::Float(0.0)),
        ("'5.5' % 1", LuaValue::Float(0.5)),
        // Exponentiation over strings.
        ("'2' ^ 2", LuaValue::Float(4.0)),
        ("4 ^ '0.5'", LuaValue::Float(2.0)),
        // Bitwise or over strings.
        ("'1' | 1", LuaValue::Integer(1)),
        ("1 | '1.0'", LuaValue::Integer(1)),
        ("'1.0' | 1", LuaValue::Integer(1)),
        ("1.0 | '1.0'", LuaValue::Float(1.0)),
        ("'1' | 2", LuaValue::Integer(3)),
        // Bitwise and over strings.
        ("'1' & 1", LuaValue::Integer(1)),
        ("1 & '1.0'", LuaValue::Integer(1)),
        ("'1.0' & 1", LuaValue::Integer(1)),
        ("1.0 & '1.0'", LuaValue::Float(1.0)),
        ("'1' & 2", LuaValue::Integer(0)),
        // Shifts over strings.
        ("'511' >> 3", LuaValue::Integer(63)),
        ("511 >> '3.0'", LuaValue::Integer(63)),
        ("'511.0' >> 3", LuaValue::Integer(63)),
        ("511.0 >> '3.0'", LuaValue::Float(63.0)),
        ("'511' << 3", LuaValue::Integer(4088)),
        ("511 << '3.0'", LuaValue::Integer(4088)),
        ("'511.0' << 3", LuaValue::Integer(4088)),
        ("511.0 << '3.0'", LuaValue::Float(4088.0)),
        // Bitwise xor over strings.
        ("'42' ~ 21", LuaValue::Integer(63)),
        ("42 ~ '21.0'", LuaValue::Integer(63)),
        ("'42.0' ~ 21", LuaValue::Integer(63)),
        ("42.0 ~ '21.0'", LuaValue::Float(63.0)),
        ("'42' ~ 42", LuaValue::Integer(0)),
        ("42 ~ '42.0'", LuaValue::Integer(0)),
        ("'42.0' ~ 42", LuaValue::Integer(0)),
        ("42.0 ~ '42.0'", LuaValue::Float(0.0)),
    ];
    for (source, expected) in cases {
        let folded = fold_expression(source, true);
        let actual = folded_value(&folded)
            .unwrap_or_else(|| panic!("the fold of {source:?} must produce a literal: {folded:?}"));
        assert_value(&actual, expected);
    }
}

#[test]
fn signed_strings_extract_like_the_csharp_realparser() {
    // Finding 29: the C# RealParser rejects leading signs ("does not
    // support a leading sign character", RealParser.cs:14-15) — the
    // mantissa is empty, NoDigits returns true with 0.0 (RealParser.cs:
    // 384-388) — while the decInteger path (long.TryParse
    // AllowLeadingSign) accepts them. Every case pinned against the C#
    // oracle on AllWithIntegers.
    let cases: &[(&str, LuaValue)] = &[
        ("'-1.5' + 1", LuaValue::Integer(1)),
        ("'+1.5' + 1", LuaValue::Integer(1)),
        ("'-1.5e2' + 1", LuaValue::Integer(1)),
        ("'-1.5x' + 1", LuaValue::Integer(1)),
        ("'+1' + 1", LuaValue::Integer(2)),
        ("'-1' + 1", LuaValue::Integer(0)),
    ];
    for (source, expected) in cases {
        let folded = fold_expression(source, true);
        let actual = folded_value(&folded)
            .unwrap_or_else(|| panic!("the fold of {source:?} must produce a literal: {folded:?}"));
        assert_value(&actual, expected);
    }
}

#[test]
fn string_number_extraction_is_unanchored() {
    // Finding 28: the C# decFloat regex is UNANCHORED (the string only
    // needs to contain a match, NumberParsing.cs:16-18) and the
    // RealParser takes the leading numeric run — the trailing garbage is
    // ignored, an empty run (leading garbage or a leading sign) is
    // NoDigits → 0.0, and a digit-less exponent overflows for '+'/none
    // (no extraction) but underflows to 0.0 for '-'. Every case pinned
    // against the C# oracle on AllWithIntegers.
    let cases: &[(&str, LuaValue)] = &[
        ("'v1.5' + 1", LuaValue::Integer(1)),
        ("'1.5x' + 1", LuaValue::Float(2.5)),
        ("'1.x' + 1", LuaValue::Integer(2)),
        ("'-1.5' + 1", LuaValue::Integer(1)),
        ("'e5' + 1", LuaValue::Integer(1)),
        ("'x1.5' + 1", LuaValue::Integer(1)),
        ("' 1.5x' + 1", LuaValue::Float(2.5)),
        ("'1.5e2x' + 1", LuaValue::Float(151.0)),
        ("'0.0015' + 1", LuaValue::Float(1.0015)),
        ("'1.5e-' + 1", LuaValue::Integer(1)),
        ("'0x1.8p10' + 1", LuaValue::Integer(1)),
    ];
    for (source, expected) in cases {
        let folded = fold_expression(source, true);
        let actual = folded_value(&folded)
            .unwrap_or_else(|| panic!("the fold of {source:?} must produce a literal: {folded:?}"));
        assert_value(&actual, expected);
    }
    // The digit-less exponent with '+' or no sign overflows — the
    // expression stays untouched.
    for source in ["'1.5e' + 1", "'1.5e+' + 1"] {
        let (folded, _wrapper) = folded_wrapper_text(source, true);
        assert_eq!(
            folded, source,
            "{source:?} must stay unfolded (the C# overflow): {folded:?}"
        );
    }
}

#[test]
fn unary_folds_take_the_operands_trivia() {
    // Finding 27: the C# folded literal takes the OPERAND's trivia
    // (LiteralExpressionWithTriviaFrom(value, operand),
    // ConstantFolder.cs:31-43) — not the whole unary node's. Every case
    // pinned against the C# oracle on AllWithIntegers (the space before
    // the operator stays with the preceding token; the trailing space
    // after the operand is carried).
    let fold = |code: &str| {
        let ast = full_moon::parse(code).expect("parse");
        let folded = constant_fold(
            ast,
            ConstantFoldingOptions {
                extract_numbers_from_strings: false,
            },
        );
        folded.to_string()
    };
    assert_eq!(fold("print(- 1)"), "print(-1)");
    assert_eq!(fold("print(- 1 )"), "print(-1 )");
    assert_eq!(fold("print( - 1)"), "print( -1)");
    assert_eq!(fold("print(- - 1)"), "print(1)");
    assert_eq!(fold("print(not true )"), "print(false )");
    assert_eq!(fold("print(-(1 + 2))"), "print(-3)");
}
