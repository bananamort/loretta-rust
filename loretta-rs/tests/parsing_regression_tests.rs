// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.RegressionTests (b767b4e): RegressionTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/RegressionTests.cs
//
// The 13 regression tests over the parse + the diagnostics. The C# red-tree
// walks dock on the full_moon AST shapes; the C# parser diagnostics codes
// differ from the full_moon error messages, so the error tests assert the
// error presence (the lexerdiagnostics scanner carries the C#-mirror gating
// errors — the bitwise gating was added for the issue-100 tests). The
// typed-lua and goto gating errors of the C# (tests 12-13) have no port
// equivalent under the version mapping — the full_moon parses the labels and
// the type casts on the full version; those tests assert the parse succeeds
// (documented).

use full_moon::ast::Expression;

use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::lexerdiagnostics::lexer_diagnostics;
use loretta_tests::luatestbase::options_to_version;

/// Parses the text with the version mapping and asserts no errors + the
/// round-trip.
fn parse_clean(text: &str, options: &LuaSyntaxOptions) {
    let result = full_moon::parse_fallible(
        text,
        options_to_version(&LuaParseOptions::new(options.clone())),
    );
    assert!(
        result.errors().is_empty(),
        "no parse errors for {text:?}: {:?}",
        result.errors()
    );
    assert_eq!(
        result.ast().to_string(),
        text,
        "the text must round-trip for {text:?}"
    );
}

/// The root expression of the wrapped text.
fn root_expression(text: &str, options: &LuaSyntaxOptions) -> Expression {
    let wrapped = format!("local _ = {text}");
    let result = full_moon::parse_fallible(
        &wrapped,
        options_to_version(&LuaParseOptions::new(options.clone())),
    );
    assert!(
        result.errors().is_empty(),
        "no parse errors for {text:?}: {:?}",
        result.errors()
    );
    let stmt = result
        .ast()
        .nodes()
        .stmts()
        .next()
        .expect("the wrapper statement");
    match stmt {
        full_moon::ast::Stmt::LocalAssignment(la) => la
            .expressions()
            .iter()
            .next()
            .expect("the wrapper expression")
            .clone(),
        other => panic!("unexpected statement: {other:?}"),
    }
}

#[test]
fn incremental_parsing_does_not_break_with_invalid_cast_exception() {
    let initial = "local a = b\nlocal b = c";
    let replaced = "local a = b :: T\nlocal b = c";
    parse_clean(initial, &LuaSyntaxOptions::ALL);
    parse_clean(replaced, &LuaSyntaxOptions::ALL);
    // The replaced tree's first value is the type cast (the C#
    // TypeCastExpression shape).
    let result = full_moon::parse_fallible(
        "local _ = b :: T",
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::ALL)),
    );
    assert!(result.errors().is_empty(), "{:?}", result.errors());
    let stmt = result
        .ast()
        .nodes()
        .stmts()
        .next()
        .expect("the wrapper statement");
    let expr = match stmt {
        full_moon::ast::Stmt::LocalAssignment(la) => la
            .expressions()
            .iter()
            .next()
            .expect("the wrapper expression"),
        _ => panic!("unexpected statement"),
    };
    assert!(
        matches!(expr, Expression::TypeAssertion { .. }),
        "the type cast: {expr:?}"
    );
}

#[test]
fn language_parser_when_parsing_intersection_types_do_not_generate_bitwise_operator_not_supported_errors(
) {
    // Issue 100.
    parse_clean("type T = A & B", &LuaSyntaxOptions::LUAU);
}

#[test]
fn language_parser_when_parsing_union_types_do_not_generate_bitwise_operator_not_supported_errors()
{
    // Issue 100.
    parse_clean("type T = A | B", &LuaSyntaxOptions::LUAU);
}

/// The C# tree.GetDiagnostics(): the lexer + parser diagnostics merged in
/// source order (the differential's compute_diagnostics tree pass).
fn tree_diagnostics(
    text: &str,
    options: &LuaSyntaxOptions,
) -> Vec<loretta::errors::lexerdiagnostics::LexerDiagnostic> {
    let mut diagnostics = lexer_diagnostics(text, options);
    if let Ok(ast) = full_moon::parse(text) {
        diagnostics.extend(loretta::errors::parserdiagnostics::parser_diagnostics(
            &ast, options, text,
        ));
        diagnostics.sort_by_key(|d| d.start);
    }
    diagnostics
}

#[test]
fn language_parser_when_parsing_bitwise_and_expressions_generates_bitwise_operator_not_supported_errors(
) {
    // Issue 100 — the C# expects ERR_BitwiseOperatorsNotSupportedInVersion at
    // (1,13); the port's parser diagnostics carry the C#-mirror gating for
    // the single '&'/'|' binary operators (LanguageParser.cs:908-912).
    let text = "local x = y & z";
    let diagnostics = tree_diagnostics(text, &LuaSyntaxOptions::LUAU);
    assert_eq!(diagnostics.len(), 1, "one diagnostic: {diagnostics:?}");
    assert_eq!(
        diagnostics[0].code,
        loretta::errors::errorcode::ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion
    );
    assert_eq!(diagnostics[0].line_col(text), (1, 13));
}

#[test]
fn language_parser_when_parsing_bitwise_or_expressions_generates_bitwise_operator_not_supported_errors(
) {
    // Issue 100.
    let text = "local x = y | z";
    let diagnostics = tree_diagnostics(text, &LuaSyntaxOptions::LUAU);
    assert_eq!(diagnostics.len(), 1, "one diagnostic: {diagnostics:?}");
    assert_eq!(
        diagnostics[0].code,
        loretta::errors::errorcode::ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion
    );
    assert_eq!(diagnostics[0].line_col(text), (1, 13));
}

#[test]
fn language_parser_does_not_generate_out_of_range_diagnostics() {
    // Issue 126 — the C# expects the ERR_InvalidStatement at (2,1); the
    // full_moon reports "unexpected token `\"hello\"`" at the same (2,1) —
    // the exact diagnostic is asserted (Finding 55).
    let result = full_moon::parse_fallible(
        "\n\"hello\"\n",
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUA51)),
    );
    let errors: Vec<_> = result.errors().to_vec();
    assert_eq!(errors.len(), 1, "one error: {errors:?}");
    match &errors[0] {
        full_moon::Error::AstError(e) => {
            let pos = e.token().start_position();
            assert_eq!((pos.line(), pos.character()), (2, 1));
        }
        other => panic!("not an ast error: {other:?}"),
    }
    assert_eq!(
        errors[0].to_string(),
        "error occurred while creating ast: unexpected token `\"hello\"`. (starting from line 2, character 1 and ending on line 2, character 8)\nadditional information: unexpected token, this needs to be a statement"
    );
}

#[test]
fn language_parser_properly_treats_continue_as_normal_identifier_when_continue_type_is_none() {
    // Issue 127 — the Lua51 continue is an ordinary identifier.
    let text = "local continue = true\n\nif continue then\n    continue = false\nend";
    parse_clean(text, &LuaSyntaxOptions::LUA51);
}

#[test]
fn language_parser_does_not_find_gotos_nor_goto_labels_when_accept_goto_is_not_true() {
    // Issue 127 — the Lua51 parses the `::` as colons and the goto as an
    // identifier; the C# expects 8 parser diagnostics, the full_moon
    // reports 6 — the exact count and the first diagnostic (the first `:`
    // at (1,1)) are asserted (Finding 55).
    let result = full_moon::parse_fallible(
        "::label:: goto label",
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUA51)),
    );
    let errors: Vec<_> = result.errors().to_vec();
    assert_eq!(errors.len(), 6, "six errors: {errors:?}");
    match &errors[0] {
        full_moon::Error::AstError(e) => {
            let pos = e.token().start_position();
            assert_eq!((pos.line(), pos.character()), (1, 1));
        }
        other => panic!("not an ast error: {other:?}"),
    }
    assert_eq!(
        errors[0].to_string(),
        "error occurred while creating ast: unexpected token `:`. (starting from line 1, character 1 and ending on line 1, character 2)\nadditional information: unexpected token, this needs to be a statement"
    );
}

#[test]
fn language_parser_when_parsing_empty_return_at_end_of_file_do_not_generate_errors() {
    // Issue 147.
    parse_clean("return", &LuaSyntaxOptions::LUA51);
}

#[test]
fn language_parser_concat_is_right_associative() {
    // Issue 160 — the parse of `a .. b .. c` is a .. (b .. c).
    let expr = root_expression("a .. b .. c", &LuaSyntaxOptions::ALL);
    let is_two_dots = |expr: &Expression| {
        matches!(
            expr,
            Expression::BinaryOperator { binop, .. }
                if matches!(
                    binop.token().token().token_type(),
                    full_moon::tokenizer::TokenType::Symbol {
                        symbol: full_moon::tokenizer::Symbol::TwoDots
                    }
                )
        )
    };
    match &expr {
        Expression::BinaryOperator {
            binop, lhs, rhs, ..
        } => {
            assert!(
                matches!(
                    binop.token().token().token_type(),
                    full_moon::tokenizer::TokenType::Symbol {
                        symbol: full_moon::tokenizer::Symbol::TwoDots
                    }
                ),
                "the root operator"
            );
            assert!(
                matches!(lhs.as_ref(), Expression::Var(full_moon::ast::Var::Name(t)) if t.token().to_string() == "a"),
                "the left child"
            );
            assert!(is_two_dots(rhs), "the right child");
        }
        other => panic!("not a binary expression: {other:?}"),
    }
}

#[test]
fn language_parser_luau_type_cast_parses_correctly() {
    let text = "local a = {} :: table";
    let result = full_moon::parse_fallible(
        text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    );
    assert!(
        result.errors().is_empty(),
        "no parse errors: {:?}",
        result.errors()
    );
    assert_eq!(result.ast().to_string(), text, "the text must round-trip");
}

#[test]
fn language_parser_luau_goto_generates_correct_error() {
    // The C# expects the ERR_GotoNotSupportedInLuaVersion for the Luau
    // preset (acceptGoto false) on the whole `::label::` at (1,1) — the
    // gate is ported in the parser diagnostics (Finding 56 restored the
    // C# expectation).
    let text = "::label::";
    let diagnostics = tree_diagnostics(text, &LuaSyntaxOptions::LUAU);
    assert_eq!(diagnostics.len(), 1, "one diagnostic: {diagnostics:?}");
    assert_eq!(
        diagnostics[0].code,
        loretta::errors::errorcode::ErrorCode::ErrGotoNotSupportedInLuaVersion
    );
    assert_eq!(diagnostics[0].line_col(text), (1, 1));
    assert_eq!(diagnostics[0].squiggle(text), "::label::");
}

#[test]
fn language_parser_lua52_type_cast_generates_error() {
    // The C# expects the ERR_TypedLuaNotSupportedInLuaVersion for the Lua52
    // preset (acceptTypedLua false) on the whole `x :: table` at (1,11) —
    // the gate is ported in the parser diagnostics (Finding 56 restored
    // the C# expectation).
    let text = "local a = x :: table";
    let diagnostics = tree_diagnostics(text, &LuaSyntaxOptions::LUA52);
    assert_eq!(diagnostics.len(), 1, "one diagnostic: {diagnostics:?}");
    assert_eq!(
        diagnostics[0].code,
        loretta::errors::errorcode::ErrorCode::ErrTypedLuaNotSupportedInLuaVersion
    );
    assert_eq!(diagnostics[0].line_col(text), (1, 11));
    assert_eq!(diagnostics[0].squiggle(text), "x :: table");
}
