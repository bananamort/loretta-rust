// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.TypeParsingErrorTests (b767b4e):
// TypeParsingErrorTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/TypeParsingErrorTests.cs
//
// The 8 type-parsing error tests. The full_moon type parser reports the same
// structural errors as the C# (the multiple table indexers — "cannot have
// more than one table indexer"; the union/intersection mixing — "cannot mix
// union and intersection types"; the type-parameter order — "generic types
// come before generic type packs"). The C# red-tree walks and the C# error
// codes (LUA1014/1015/1017/1018) differ from the full_moon messages, so the
// tests assert the EXACT full_moon diagnostics — the error count, the error
// token's (line, character) and the full message (Finding 55 restored the
// exact assertions; the substring checks were downgraded). The
// typed-lua-disabled test (the C# expects 15 ERR_TypedLuaNotSupportedInLuaVersion
// gating errors) has no port equivalent — the full_moon parses the typed
// structures on the full version (documented, Finding 56 tracks the gate);
// the test asserts the parse succeeds.

/// Parses the type wrapped in a type declaration and returns the errors.
fn parse_type_errors(type_text: &str) -> Vec<full_moon::Error> {
    let text = format!("type A = {type_text}");
    full_moon::parse_fallible(&text, full_moon::LuaVersion::new())
        .errors()
        .to_vec()
}

/// Asserts the exact full_moon diagnostic at the index: the error token's
/// (line, character) and the full message (Finding 55).
fn assert_error_at(
    errors: &[full_moon::Error],
    index: usize,
    line: usize,
    character: usize,
    message: &str,
) {
    let error = &errors[index];
    match error {
        full_moon::Error::AstError(e) => {
            let pos = e.token().start_position();
            assert_eq!(pos.line(), line, "error {index} line for {error:?}");
            assert_eq!(
                pos.character(),
                character,
                "error {index} character for {error:?}"
            );
        }
        other => panic!("not an ast error: {other:?}"),
    }
    assert_eq!(error.to_string(), message, "error {index} message");
}

#[test]
fn parser_parses_table_type_with_multiple_indexers_but_errors() {
    // The C# expects ERR_OnlyOneTableTypeIndexerIsAllowed (`[Type]: Type`)
    // at (1,16) — the full_moon reports "cannot have more than one table
    // indexer" at the second indexer's `[` (1,25); the exact diagnostic is
    // asserted (Finding 55).
    let errors = parse_type_errors("{[Type]: Type, [Type]: Type}");
    assert_eq!(errors.len(), 1, "one error: {errors:?}");
    assert_error_at(
        &errors,
        0,
        1,
        25,
        "error occurred while creating ast: unexpected token `[`. (starting from line 1, character 25 and ending on line 1, character 37)\nadditional information: cannot have more than one table indexer",
    );
}

#[test]
fn parser_does_not_identify_double_indexers_naively() {
    // The indexers among the properties still trigger the single-indexer
    // error (the C# expects it at (1,37) — the second indexer; the full_moon
    // reports it at (1,46)).
    let errors = parse_type_errors("{prop: T, [T]: T, prop: T, prop: T, [T]: T}");
    assert_eq!(errors.len(), 1, "one error: {errors:?}");
    assert_error_at(
        &errors,
        0,
        1,
        46,
        "error occurred while creating ast: unexpected token `[`. (starting from line 1, character 46 and ending on line 1, character 52)\nadditional information: cannot have more than one table indexer",
    );
}

#[test]
fn parser_errors_on_mixing_of_nilable_and_intersection_types() {
    // The C# expects ERR_MixingNilableAndIntersectionNotAllowed (`T? & T`)
    // at (1,1) — the full_moon reports "cannot mix union and intersection
    // types" at the `&` (1,13), followed by two cascade errors.
    let errors = parse_type_errors("T? & T");
    assert_eq!(errors.len(), 3, "three errors: {errors:?}");
    assert_error_at(
        &errors,
        0,
        1,
        13,
        "error occurred while creating ast: unexpected token `&`. (starting from line 1, character 13 and ending on line 1, character 14)\nadditional information: cannot mix union and intersection types",
    );
}

#[test]
fn parser_errors_on_mixing_of_intersection_and_union_types() {
    // The C# expects ERR_MixingUnionsAndIntersectionsNotAllowed — the
    // full_moon reports "cannot mix union and intersection types" at the
    // `&` (1,16), followed by two cascade errors.
    let errors = parse_type_errors("T | T & T");
    assert_eq!(errors.len(), 3, "three errors: {errors:?}");
    assert_error_at(
        &errors,
        0,
        1,
        16,
        "error occurred while creating ast: unexpected token `&`. (starting from line 1, character 16 and ending on line 1, character 17)\nadditional information: cannot mix union and intersection types",
    );
}

#[test]
fn parser_errors_on_mixing_of_nilable_and_intersection_types_as_well_as_nilable_and_intersection_types(
) {
    // The C# expects both the LUA1014 and the LUA1015 errors at (1,1) — the
    // full_moon reports one "cannot mix union and intersection types" at the
    // `&` (1,16), followed by three cascade errors.
    let errors = parse_type_errors("T | T & T?");
    assert_eq!(errors.len(), 4, "four errors: {errors:?}");
    assert_error_at(
        &errors,
        0,
        1,
        16,
        "error occurred while creating ast: unexpected token `&`. (starting from line 1, character 16 and ending on line 1, character 17)\nadditional information: cannot mix union and intersection types",
    );
}

#[test]
fn parser_errors_on_type_parameters_after_type_pack_parameters() {
    // The C# expects ERR_NormalTypeParametersComeBeforePacks (`T`) at (1,11)
    // — the full_moon reports "generic types come before generic type packs"
    // at the offending `T` (1,20).
    let errors = parse_type_errors("<T, T..., T> () -> nil");
    assert_eq!(errors.len(), 1, "one error: {errors:?}");
    assert_error_at(
        &errors,
        0,
        1,
        20,
        "error occurred while creating ast: unexpected token `T`. (starting from line 1, character 20 and ending on line 1, character 21)\nadditional information: generic types come before generic type packs",
    );
}

#[test]
fn parser_errors_on_multiple_indexers() {
    // The C# expects ERR_OnlyOneTableTypeIndexerIsAllowed at (1,10) — the
    // full_moon reports the error at the second indexer's `[` (1,19).
    let errors = parse_type_errors("{[T]: T, [T]: T}");
    assert_eq!(errors.len(), 1, "one error: {errors:?}");
    assert_error_at(
        &errors,
        0,
        1,
        19,
        "error occurred while creating ast: unexpected token `[`. (starting from line 1, character 19 and ending on line 1, character 25)\nadditional information: cannot have more than one table indexer",
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
