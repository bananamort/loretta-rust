// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.TypeParsingTests (b767b4e):
// TypeParsingTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/TypeParsingTests.cs
//
// The 54 tests parse the typed-Lua structures with the Luau options and walk
// the C# red-tree shapes (the SimpleTypeName / CompositeTypeName / TypeofType
// / ArrayType / TableType / FunctionType node shapes). The dropped red-tree
// shapes have no full_moon equivalents; the port parses each structure with
// the version mapping and asserts the clean parse + the round-trip. The
// type-only forms (the C# UsingTypeAsync) parse through the wrapped type
// declaration.

use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::luatestbase::options_to_version;

/// The parsed type and statement forms of the C# tests: (text, whether it is
/// a type-only form that needs the type-declaration wrapper).
const CASES: &[(&str, bool)] = &[
    // The C# `Type{...}` type-argument-brace syntax (cases 1 and 3) has no
    // full_moon equivalent (the full_moon parses the `{...}` as a table
    // type) — the two cases are dropped (documented).
    ("Type", true),
    ("Type.Member", true),
    ("typeof('hi')", true),
    ("typeof(1)", true),
    ("typeof({ 1 })", true),
    ("typeof(tbl[1].member:method { 'hi' })", true),
    ("{Type}", true),
    ("{Type.Member}", true),
    ("{{Type}}", true),
    ("{[Type]: Type}", true),
    ("{prop1: Type1, prop2: Type2, prop3: Type3}", true),
    ("(T) -> T", true),
    ("(T, ...T) -> (T, ...T)", true),
    ("(p1: T, ...T) -> (T, ...T)", true),
    ("(p1: T, T) -> (T, ...T)", true),
    ("(T, p2: T) -> (T, ...T)", true),
    ("(p1: T, p2: T) -> (T, ...T)", true),
    // The C# generic-list forms with the type-parameter defaults and the
    // pack defaults (`<T, T = T, T... = ...T, T... = T...>`, cases 17-20)
    // have no full_moon equivalent (its generic-list syntax lacks the
    // defaults) — the four cases are dropped (documented).
    ("'value'", true),
    ("true", true),
    ("false", true),
    ("nil", true),
    ("(T)", true),
    ("{T}?", true),
    ("T & T", true),
    ("T | T", true),
    ("local Var: T = true", false),
    ("for i:T = 1, 5 do end", false),
    ("for i:T in iter() do end", false),
    ("for i: T, v in iter() do end", false),
    ("function a(b:T, c:A) end", false),
    ("function a(b, c:A) end", false),
    ("function a(b:T, ...:A) end", false),
    ("local a = function(b:T, c:T) end", false),
    ("function a() : T end", false),
    ("local a = function() : T end", false),
    ("type a = T", false),
    ("export type a = T", false),
    ("local a = b :: T", false),
    ("local a = b :: T + b :: T", false),
    ("local a = -b :: T", false),
    ("local a = b ^ b :: T", false),
    ("function a(): () end", false),
    ("type T = T<>", false),
    ("type function myTypeFunc() return types.number end", false),
    (
        "export type function myTypeFunc() return types.number end",
        false,
    ),
    ("type function serialize(arg) return arg end", false),
    (
        "type function myTypeFunc(): T return types.number end",
        false,
    ),
    ("type function serialize(arg: T) return arg end", false),
];

#[test]
fn parser_parses_typed_lua_structures() {
    for (i, (text, is_type_only)) in CASES.iter().enumerate() {
        let parsed_text = if *is_type_only {
            format!("type A = {text}")
        } else {
            text.to_string()
        };
        let result = full_moon::parse_fallible(
            &parsed_text,
            options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
        );
        assert!(
            result.errors().is_empty(),
            "case {i} ({text:?}): no parse errors: {:?}",
            result.errors()
        );
        assert_eq!(
            result.ast().to_string(),
            parsed_text,
            "case {i} ({text:?}): the text must round-trip"
        );
    }
}

/// C# Parser_ParsesSimpleTypeName (TypeParsingTests.cs:46): 'Type' parses as the simple type name.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsessimpletypename() {
    let parsed_text = "type A = Type";
    let result = full_moon::parse_fallible(
        parsed_text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    );
    assert!(
        result.errors().is_empty(),
        "no parse errors: {:?}",
        result.errors()
    );
    assert_eq!(
        result.ast().to_string(),
        parsed_text,
        "the text must round-trip"
    );
}
