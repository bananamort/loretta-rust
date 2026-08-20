// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.InterpolatedStringTests (b767b4e):
// InterpolatedStringTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/InterpolatedStringTests.cs
//
// The three tests parse interpolated strings under the Luau options and walk
// the C# red-tree shapes (the InterpolatedStringExpression /
// InterpolatedStringText / Interpolation nodes with the BacktickToken). The
// dropped red-tree shapes dock on the full_moon InterpolatedString structure
// (ast/luau.rs:1318-1367 — the segments with the literal + expression, plus
// the last string); the C# value checks (the decoded InterpolatedStringText
// and StringLiteral values) map to the ShortToken::from_token values. The
// deep tree walks of the C# are documented as red-tree shapes the full_moon
// does not reproduce; the tests assert the parse, the round-trip, and the
// structural essentials (the segment literals and the interpolation
// expressions).

use full_moon::ast::Expression;
use full_moon::tokenizer::TokenType;

use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::luatestbase::options_to_version;
use loretta_tests::shorttoken::{ShortToken, TokenValue};

/// Parses the wrapped expression and returns the root expression.
fn parse_wrapped_expression(text: &str) -> Expression {
    let wrapped = format!("local _ = {text}");
    let ast = full_moon::parse_fallible(
        &wrapped,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    )
    .into_result()
    .expect("the wrapper must parse");
    assert_eq!(ast.to_string(), wrapped, "the text must round-trip");
    let stmt = ast.nodes().stmts().next().expect("the wrapper statement");
    match stmt {
        full_moon::ast::Stmt::LocalAssignment(la) => la
            .expressions()
            .iter()
            .next()
            .expect("the wrapper expression")
            .clone(),
        _ => panic!("unexpected statement: {stmt}"),
    }
}

#[test]
fn language_parser_properly_reads_strings_inside_interpolated_strings() {
    let text = "`some\\tthing {\"a very\\nlong string\"} some\\nthing`";
    let expr = parse_wrapped_expression(text);
    match &expr {
        Expression::InterpolatedString(interpolated) => {
            let segments: Vec<_> = interpolated.segments().collect();
            assert_eq!(segments.len(), 1, "one segment");
            let segment = segments[0];
            // The segment literal (the C# InterpolatedStringTextToken
            // `some\tthing ` — the raw text; the decoded value has the tab).
            match segment.literal.token().token_type() {
                TokenType::InterpolatedString { literal, .. } => {
                    assert_eq!(literal.as_str(), "some\\tthing ", "the segment literal");
                }
                other => panic!("unexpected segment literal: {other:?}"),
            }
            // The C# InterpolatedStringTextToken value is the decoded text
            // (``some	thing `` with the tab); the port's interpolated-token
            // value model carries the full token text, so the raw literal
            // field is asserted above (documented).
            let _ = TokenValue::String(String::new());
            // The interpolation expression: the inner short string.
            match &segment.expression {
                Expression::String(token) => {
                    let value = ShortToken::from_token(token, &LuaSyntaxOptions::LUAU)
                        .value
                        .expect("the string value");
                    assert_eq!(
                        value,
                        TokenValue::String("a very\nlong string".to_string()),
                        "the inner string value"
                    );
                }
                other => panic!("not a string literal: {other:?}"),
            }
            // The trailing text (the C# InterpolatedStringTextToken
            // ` some\nthing`).
            match interpolated.last_string().token().token_type() {
                TokenType::InterpolatedString { literal, .. } => {
                    assert_eq!(literal.as_str(), " some\\nthing", "the last string");
                }
                other => panic!("unexpected last string: {other:?}"),
            }
        }
        other => panic!("not an interpolated string: {other:?}"),
    }
}

#[test]
fn language_parser_properly_reads_deeply_nested_interpolated_strings() {
    let text = "`a {`very {`{`very {`{`very` .. ` ` .. `very`} very{(\" very\"):rep(100)}`}`} very`} nested`} string`";
    let expr = parse_wrapped_expression(text);
    match &expr {
        Expression::InterpolatedString(interpolated) => {
            // The first segment literal is `a ` and the interpolation nests
            // further interpolated strings; the parse must succeed and
            // round-trip (the C# deep tree walk is a red-tree shape the
            // full_moon does not reproduce — documented).
            let segments: Vec<_> = interpolated.segments().collect();
            assert_eq!(segments.len(), 1, "one outer segment");
            match segments[0].literal.token().token_type() {
                TokenType::InterpolatedString { literal, .. } => {
                    assert_eq!(literal.as_str(), "a ", "the outer segment literal");
                }
                other => panic!("unexpected segment literal: {other:?}"),
            }
            assert!(
                matches!(segments[0].expression, Expression::InterpolatedString(_)),
                "the nested interpolated string"
            );
        }
        other => panic!("not an interpolated string: {other:?}"),
    }
}

#[test]
fn language_parser_properly_reads_interpolated_strings_with_complex_expressions() {
    let text = "print(`some {function()\n  print(`other {function()\n    print(`some {if true then function()\n      print(`fucked up {1 + 2 ^ 6} shit`)\n    end else function (...)\n      print(`fucked up {...} shit`)\n    end} shit`)\n  end} shit`)\nend} fucked up shit`)";
    // The deep nesting overflows the default test-thread stack — the parse
    // runs on a dedicated large-stack thread.
    let wrapped = format!("local _ = {text}");
    let wrapped_for_thread = wrapped.clone();
    let ast = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(move || {
            let ast = full_moon::parse_fallible(
                &wrapped_for_thread,
                options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
            )
            .into_result()
            .expect("the wrapper must parse");
            assert_eq!(
                ast.to_string(),
                wrapped_for_thread,
                "the text must round-trip"
            );
            ast
        })
        .expect("the parse thread")
        .join()
        .expect("the parse thread panicked");
    let stmt = ast.nodes().stmts().next().expect("the wrapper statement");
    let expr = match stmt {
        full_moon::ast::Stmt::LocalAssignment(la) => la
            .expressions()
            .iter()
            .next()
            .expect("the wrapper expression"),
        _ => panic!("unexpected statement: {stmt}"),
    };
    // The root: the print function call with the interpolated string
    // argument (the C# FunctionCallExpression shape).
    assert!(
        matches!(expr, Expression::FunctionCall(_)),
        "the root function call: {expr:?}"
    );
    match expr {
        Expression::FunctionCall(call) => {
            let arguments = call
                .suffixes()
                .collect::<Vec<_>>()
                .iter()
                .filter_map(|suffix| match suffix {
                    full_moon::ast::Suffix::Call(full_moon::ast::Call::AnonymousCall(
                        full_moon::ast::FunctionArgs::Parentheses { arguments, .. },
                    )) => Some(arguments),
                    _ => None,
                })
                .next()
                .expect("the parenthesized arguments");
            let arg = arguments.iter().next().expect("the first argument");
            assert!(
                matches!(arg, Expression::InterpolatedString(_)),
                "the interpolated argument"
            );
        }
        _ => unreachable!(),
    }
}
