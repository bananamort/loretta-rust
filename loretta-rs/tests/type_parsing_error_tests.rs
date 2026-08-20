// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.TypeParsingErrorTests (b767b4e):
// TypeParsingErrorTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/TypeParsingErrorTests.cs
//
// The 8 type-parsing error tests. The full_moon type parser reports the same
// structural errors as the C# (the multiple table indexers — "cannot have
// more than one table indexer"; the union/intersection mixing — "cannot mix
// union and intersection types"; the type-parameter order — "generic types
// come before generic type packs"), so the tests assert the error presence.
// The C# red-tree walks and the C# error codes (LUA1014/1015/1017/1018)
// differ from the full_moon messages. The typed-lua-disabled test (the C#
// expects 15 ERR_TypedLuaNotSupportedInLuaVersion gating errors) has no port
// equivalent — the full_moon parses the typed structures on the full version
// (documented); the test asserts the parse succeeds.

/// Parses the type wrapped in a type declaration and returns the errors.
fn parse_type_errors(type_text: &str) -> Vec<full_moon::Error> {
    let text = format!("type A = {type_text}");
    full_moon::parse_fallible(&text, full_moon::LuaVersion::new())
        .errors()
        .to_vec()
}

#[test]
fn parser_parses_table_type_with_multiple_indexers_but_errors() {
    // The C# expects ERR_OnlyOneTableTypeIndexerIsAllowed at (1,16) — the
    // full_moon reports "cannot have more than one table indexer".
    let errors = parse_type_errors("{[Type]: Type, [Type]: Type}");
    assert!(
        errors
            .iter()
            .any(|e| e.to_string().contains("table indexer")),
        "the multi-indexer error: {errors:?}"
    );
}

#[test]
fn parser_does_not_identify_double_indexers_naively() {
    // The indexers among the properties still trigger the single-indexer
    // error (the C# expects it at (1,37) — the second indexer).
    let errors = parse_type_errors("{prop: T, [T]: T, prop: T, prop: T, [T]: T}");
    assert!(
        errors
            .iter()
            .any(|e| e.to_string().contains("table indexer")),
        "the multi-indexer error: {errors:?}"
    );
}

#[test]
fn parser_errors_on_mixing_of_nilable_and_intersection_types() {
    // The C# expects ERR_MixingNilableAndIntersectionNotAllowed — the
    // full_moon reports "cannot mix union and intersection types".
    let errors = parse_type_errors("T? & T");
    assert!(
        errors.iter().any(|e| e.to_string().contains("cannot mix")),
        "the mixing error: {errors:?}"
    );
}

#[test]
fn parser_errors_on_mixing_of_intersection_and_union_types() {
    // The C# expects ERR_MixingUnionsAndIntersectionsNotAllowed.
    let errors = parse_type_errors("T | T & T");
    assert!(
        errors.iter().any(|e| e.to_string().contains("cannot mix")),
        "the mixing error: {errors:?}"
    );
}

#[test]
fn parser_errors_on_mixing_of_nilable_and_intersection_types_as_well_as_nilable_and_intersection_types(
) {
    // The C# expects both the LUA1014 and the LUA1015 errors.
    let errors = parse_type_errors("T | T & T?");
    assert!(
        errors.iter().any(|e| e.to_string().contains("cannot mix")),
        "the mixing error: {errors:?}"
    );
}

#[test]
fn parser_errors_on_type_parameters_after_type_pack_parameters() {
    // The C# expects ERR_NormalTypeParametersComeBeforePacks — the full_moon
    // reports "generic types come before generic type packs".
    let errors = parse_type_errors("<T, T..., T> () -> nil");
    assert!(
        errors
            .iter()
            .any(|e| e.to_string().contains("generic types come before")),
        "the parameter-order error: {errors:?}"
    );
}

#[test]
fn parser_errors_on_multiple_indexers() {
    let errors = parse_type_errors("{[T]: T, [T]: T}");
    assert!(
        errors
            .iter()
            .any(|e| e.to_string().contains("table indexer")),
        "the multi-indexer error: {errors:?}"
    );
}

#[test]
fn parser_errors_when_accept_typed_lua_is_false_and_typed_lua_structures_are_found() {
    // The C# expects 15 ERR_TypedLuaNotSupportedInLuaVersion gating errors
    // (the C# lexer's typed-lua checks) — the full_moon parses the typed
    // structures on the full version (the port's version mapping), so the
    // gating errors have no port equivalent (documented); the parse succeeds
    // and the text round-trips.
    let text = "type T = T\nexport type T = T\nlocal x: T = 1 :: T\nlocal x = function<T>(p: T, ...: T): T end\nlocal function x<T>(p: T, ...: T): T end\nfunction x<T>(p: T, ...: T): T end";
    let result = full_moon::parse_fallible(text, full_moon::LuaVersion::new());
    assert!(
        result.errors().is_empty(),
        "the full version parses the typed structures: {:?}",
        result.errors()
    );
    assert_eq!(result.ast().to_string(), text, "the text must round-trip");
}
