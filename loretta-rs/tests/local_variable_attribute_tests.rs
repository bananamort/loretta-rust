// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.LocalVariableAttributeTests (b767b4e):
// LocalVariableAttributeTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/LocalVariableAttributeTests.cs
//
// The 9 tests parse the local-variable attributes (`local a<const>`). The C#
// red-tree walks (the LocalDeclarationName / VariableAttribute node shapes)
// are dropped-red-tree shapes the full_moon does not reproduce — the tests
// assert the parse, the round-trip, the names, the per-name attributes (the
// full_moon LocalAssignment attributes iterator, ast/mod.rs:1901), and the
// values. The two error tests assert the EXACT full_moon diagnostics — the
// count, the error token's (line, character) and the full message (Finding 55
// restored the exact assertions; the C# codes LUA1001/LUA1006 differ from the
// full_moon messages).

use full_moon::ast::{Expression, LocalAssignment, Stmt};
use full_moon::tokenizer::TokenType;

use loretta_tests::randomspaceinserter::RandomSpaceInserter;

/// Parses the statement and returns the local assignment node.
fn parse_local(text: &str) -> LocalAssignment {
    let result = full_moon::parse_fallible(text, full_moon::LuaVersion::new());
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
    let stmt = result
        .ast()
        .nodes()
        .stmts()
        .next()
        .expect("the statement for {text:?}");
    match stmt {
        Stmt::LocalAssignment(la) => la.clone(),
        other => panic!("not a local assignment for {text:?}: {other:?}"),
    }
}

/// Asserts the local names and the per-name attribute presence.
fn assert_names_and_attributes(local: &LocalAssignment, expected: &[(&str, bool)], text: &str) {
    let names: Vec<_> = local.names().iter().collect();
    let attributes: Vec<_> = local.attributes().collect();
    assert_eq!(names.len(), expected.len(), "name count for {text:?}");
    assert_eq!(
        attributes.len(),
        expected.len(),
        "attribute count for {text:?}"
    );
    for (i, (name, has_attribute)) in expected.iter().enumerate() {
        assert_eq!(names[i].token().to_string(), *name, "name {i} for {text:?}");
        assert_eq!(
            attributes[i].is_some(),
            *has_attribute,
            "attribute {i} for {text:?}"
        );
    }
}

/// Asserts the value expressions are the numeric literals.
fn assert_values(local: &LocalAssignment, expected: &[&str], text: &str) {
    let values: Vec<_> = local.expressions().iter().collect();
    assert_eq!(values.len(), expected.len(), "value count for {text:?}");
    for (i, expected) in expected.iter().enumerate() {
        match values[i] {
            Expression::Number(token) => {
                assert_eq!(
                    token.token().to_string(),
                    *expected,
                    "value {i} for {text:?}"
                );
            }
            other => panic!("not a number for {text:?}: {other:?}"),
        }
    }
}

#[test]
fn parser_generates_an_error_diagnostic_when_identifier_is_missing() {
    // The C# expects ERR_IdentifierExpected at (1,10) — the full_moon
    // reports "expected identifier after `<` for attribute" on the `>`
    // at (1,10), followed by one cascade error; the exact count and the
    // first diagnostic are asserted (Finding 55).
    let result = full_moon::parse_fallible("local a <>", full_moon::LuaVersion::new());
    let errors: Vec<_> = result.errors().to_vec();
    assert_eq!(errors.len(), 2, "two errors: {errors:?}");
    match &errors[0] {
        full_moon::Error::AstError(e) => {
            let pos = e.token().start_position();
            assert_eq!((pos.line(), pos.character()), (1, 10));
        }
        other => panic!("not an ast error: {other:?}"),
    }
    assert_eq!(
        errors[0].to_string(),
        "error occurred while creating ast: unexpected token `>`. (starting from line 1, character 9 and ending on line 1, character 11)\nadditional information: expected identifier after `<` for attribute"
    );
}

#[test]
fn parser_generates_an_error_diagnostic_when_closing_token_is_missing() {
    // The C# expects ERR_SyntaxError ("> expected") — the full_moon
    // reports "expected `>` to close attribute" at the EOF (1,14); the
    // exact diagnostic is asserted (Finding 55).
    let result = full_moon::parse_fallible("local a<const", full_moon::LuaVersion::new());
    let errors: Vec<_> = result.errors().to_vec();
    assert_eq!(errors.len(), 1, "one error: {errors:?}");
    match &errors[0] {
        full_moon::Error::AstError(e) => {
            let pos = e.token().start_position();
            assert_eq!((pos.line(), pos.character()), (1, 14));
        }
        other => panic!("not an ast error: {other:?}"),
    }
    assert_eq!(
        errors[0].to_string(),
        "error occurred while creating ast: unexpected token ``. (starting from line 1, character 14 and ending on line 1, character 14)\nadditional information: expected `>` to close attribute"
    );
}

#[test]
fn parser_parses_local_declaration_with_single_variable_and_no_value() {
    let text = "local a<const>";
    let local = parse_local(text);
    assert_names_and_attributes(&local, &[("a", true)], text);
    assert_eq!(local.expressions().len(), 0, "no values for {text:?}");
}

#[test]
fn parser_parses_local_declaration_with_single_variable_and_value() {
    let text = "local a<const> = 1";
    let local = parse_local(text);
    assert_names_and_attributes(&local, &[("a", true)], text);
    assert_values(&local, &["1"], text);
}

#[test]
fn parser_parses_local_declaration_with_multiple_variables_and_no_value() {
    let text = "local a<const>, b<const>";
    let local = parse_local(text);
    assert_names_and_attributes(&local, &[("a", true), ("b", true)], text);
    assert_eq!(local.expressions().len(), 0, "no values for {text:?}");
}

#[test]
fn parser_parses_local_declaration_with_multiple_variables_and_values() {
    let text = "local a<const>, b<const> = 1, 2";
    let local = parse_local(text);
    assert_names_and_attributes(&local, &[("a", true), ("b", true)], text);
    assert_values(&local, &["1", "2"], text);
}

#[test]
fn parser_works_with_spaces_inside_the_attribute() {
    let variants = RandomSpaceInserter::get_token_pairs(&[
        "local a", "<", "const", ">, b", "<", "const", "> = 1, 2",
    ]);
    assert!(
        !variants.is_empty(),
        "the space inserter must produce variants"
    );
    for code in variants {
        let local = parse_local(&code);
        assert_names_and_attributes(&local, &[("a", true), ("b", true)], &code);
        assert_values(&local, &["1", "2"], &code);
    }
}

#[test]
fn parser_allows_mixing_of_attributed_and_unattributed_variables() {
    let text = "local a, b<const>, c, d<const>, e<const>, f, g = 1, 2, 3, 4, 5, 6";
    let local = parse_local(text);
    assert_names_and_attributes(
        &local,
        &[
            ("a", false),
            ("b", true),
            ("c", false),
            ("d", true),
            ("e", true),
            ("f", false),
            ("g", false),
        ],
        text,
    );
    assert_values(&local, &["1", "2", "3", "4", "5", "6"], text);
}

#[test]
fn parser_keeps_the_identifier_kind_for_attributed_names() {
    // The attributed names lex as the plain identifiers (the full_moon has
    // no attribute-carrying token kind — the attributes live on the
    // LocalAssignment).
    let text = "local a<const>";
    let local = parse_local(text);
    let name = local.names().iter().next().expect("the name");
    assert!(
        matches!(name.token().token_type(), TokenType::Identifier { .. }),
        "the identifier kind: {:?}",
        name.token().token_type()
    );
}
