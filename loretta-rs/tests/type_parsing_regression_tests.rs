// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.TypeParsingRegressionTests (b767b4e):
// TypeParsingRegressionTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/TypeParsingRegressionTests.cs
//
// The 7 regression tests parse the typed-Lua structures with the Luau
// options and verify no diagnostics. The port parses each text with the
// version mapping and asserts the parse is clean and the text round-trips.

use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::luatestbase::options_to_version;

fn parse_clean(text: &str) {
    let result = full_moon::parse_fallible(
        text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
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

#[test]
fn language_parser_properly_parses_type_as_a_contextual_keyword() {
    // Issue 125 — `type` is an ordinary identifier outside the type
    // declarations.
    parse_clean("local type\ntype = 2\nprint(type)");
}

#[test]
fn language_parser_properly_parses_export_as_a_contextual_keyword() {
    // Issue 125.
    parse_clean("local export\nexport = 2\nprint(export)");
}

#[test]
fn language_parser_parses_function_types_with_parameter_names_correctly() {
    // Issue 119.
    parse_clean("export type a = (p1: any) -> any");
}

#[test]
fn language_parser_parses_variadic_function_return_types_correctly() {
    // Issue 119.
    parse_clean("function sample(a): ...any\n    print \"hi\"\nend");
}

#[test]
fn language_parser_parses_variadic_function_type_return_types_correctly() {
    // Issue 119 — the type form (the port parses the wrapped type
    // declaration; the C# UsingTypeAsync parses the bare type).
    parse_clean("type A = ((Player, ...any) -> ...any)?");
}

#[test]
fn language_parser_parses_leading_intersection_type_correctly() {
    // Issue 150.
    parse_clean("local f: & number & string = \"hi\"");
}

#[test]
fn language_parser_parses_leading_union_type_correctly() {
    // Issue 150.
    parse_clean("local f: | number | string = \"hi\"");
}
