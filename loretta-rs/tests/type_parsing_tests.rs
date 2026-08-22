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
    // The C# `<...>` type-argument-list forms (the two WithTypeArgumentList
    // cases) parse cleanly through the full_moon Generic instantiation
    // (GenericPack / VariadicPack arguments) — covered by the individual
    // parser_parses_*_with_type_argument_list tests below.
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

/// C# Parser_ParsesSimpleTypeName_WithTypeArgumentList (TypeParsingTests.cs:58): 'Type<Type, Type..., ...Type, Type.Member>' parses as the simple type name with a type argument list.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsessimpletypename_withtypeargumentlist() {
    let parsed_text = "type A = Type<Type, Type..., ...Type, Type.Member>";
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

/// C# Parser_ParsesCompositeTypeName (TypeParsingTests.cs:71): 'Type.Member' parses as the composite dotted type name.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsescompositetypename() {
    let parsed_text = "type A = Type.Member";
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

/// C# Parser_ParsesCompositeTypeName_WithTypeArgumentList (TypeParsingTests.cs:88): 'Type.Member<Type, Type..., ...Type, Type.Member>' parses as the composite type name with a type argument list.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsescompositetypename_withtypeargumentlist() {
    let parsed_text = "type A = Type.Member<Type, Type..., ...Type, Type.Member>";
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

/// C# Parser_ParsesTypeofType_WithStrings (TypeParsingTests.cs:106): "typeof('hi')" parses as the typeof type over a string literal.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsestypeoftype_withstrings() {
    let parsed_text = "type A = typeof('hi')";
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

/// C# Parser_ParsesTypeofType_WithNumbers (TypeParsingTests.cs:126): 'typeof(1)' parses as the typeof type over a number literal.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsestypeoftype_withnumbers() {
    let parsed_text = "type A = typeof(1)";
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

/// C# Parser_ParsesTypeofType_WithTables (TypeParsingTests.cs:145): 'typeof({ 1 })' parses as the typeof type over a table constructor.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsestypeoftype_withtables() {
    let parsed_text = "type A = typeof({ 1 })";
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

/// C# Parser_ParsesTypeofType_WithComplexExpression (TypeParsingTests.cs:173): "typeof(tbl[1].member:method { 'hi' })" parses as the typeof type over a complex call expression.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsestypeoftype_withcomplexexpression() {
    let parsed_text = "type A = typeof(tbl[1].member:method { 'hi' })";
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

/// C# Parser_ParsesArrayType_WithSimpleTypeNameElement (TypeParsingTests.cs:227): '{Type}' parses as the array type over a simple type name.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesarraytype_withsimpletypenameelement() {
    let parsed_text = "type A = {Type}";
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

/// C# Parser_ParsesArrayType_WithCompositeTypeNameElement (TypeParsingTests.cs:246): '{Type.Member}' parses as the array type over a composite type name.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesarraytype_withcompositetypenameelement() {
    let parsed_text = "type A = {Type.Member}";
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

/// C# Parser_ParsesArrayType_WithArrayTypeElement (TypeParsingTests.cs:270): '{{Type}}' parses as the nested array type.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesarraytype_witharraytypeelement() {
    let parsed_text = "type A = {{Type}}";
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

/// C# Parser_ParsesTableType_WithIndexer (TypeParsingTests.cs:296): '{[Type]: Type}' parses as the table type with an indexer.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsestabletype_withindexer() {
    let parsed_text = "type A = {[Type]: Type}";
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

/// C# Parser_ParsesTypeTable_WithProperty (TypeParsingTests.cs:325): '{prop1: Type1, prop2: Type2, prop3: Type3}' parses as the table type with properties.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsestypetable_withproperty() {
    let parsed_text = "type A = {prop1: Type1, prop2: Type2, prop3: Type3}";
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

/// C# Parser_ParsesFunctionTypes_WithoutTypeParameters_AndNoTrailingVariadicPack_AndTypeReturn (TypeParsingTests.cs:367): '(T) -> T' parses as the function type without type parameters returning a type.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesfunctiontypes_withouttypeparameters_andnotrailingvariadicpack_andtypereturn() {
    let parsed_text = "type A = (T) -> T";
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

/// C# Parser_ParsesFunctionTypes_WithoutTypeParameters_AndTrailingVariadicPack_AndTypePackReturn (TypeParsingTests.cs:392): '(T, ...T) -> (T, ...T)' parses as the function type with a trailing variadic pack returning a type pack.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesfunctiontypes_withouttypeparameters_andtrailingvariadicpack_andtypepackreturn() {
    let parsed_text = "type A = (T, ...T) -> (T, ...T)";
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

/// C# Parser_ParsesFunctionTypes_WithoutTypeParameters_AndTrailingVariadicPack_AndTypePackReturn_AndParameterNameOnFirstParameter (TypeParsingTests.cs:445): '(p1: T, ...T) -> (T, ...T)' parses as the function type with a named first parameter and a trailing variadic pack.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesfunctiontypes_withouttypeparameters_andtrailingvariadicpack_andtypepackreturn_andparameternameonfirstparameter(
) {
    let parsed_text = "type A = (p1: T, ...T) -> (T, ...T)";
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

/// C# Parser_ParsesFunctionTypes_WithoutTypeParameters_AndTypePackReturn_AndParameterNameOnFirstParameter (TypeParsingTests.cs:500): '(p1: T, T) -> (T, ...T)' parses as the function type with a named first parameter and a type pack return.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesfunctiontypes_withouttypeparameters_andtypepackreturn_andparameternameonfirstparameter(
) {
    let parsed_text = "type A = (p1: T, T) -> (T, ...T)";
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

/// C# Parser_ParsesFunctionTypes_WithoutTypeParameters_AndTypePackReturn_AndParameterNameOnSecondParameter (TypeParsingTests.cs:551): '(T, p2: T) -> (T, ...T)' parses as the function type with a named second parameter and a type pack return.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesfunctiontypes_withouttypeparameters_andtypepackreturn_andparameternameonsecondparameter(
) {
    let parsed_text = "type A = (T, p2: T) -> (T, ...T)";
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

/// C# Parser_ParsesFunctionTypes_WithoutTypeParameters_AndTypePackReturn_AndParameterNameOnBothParameters (TypeParsingTests.cs:602): '(p1: T, p2: T) -> (T, ...T)' parses as the function type with both parameters named and a type pack return.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesfunctiontypes_withouttypeparameters_andtypepackreturn_andparameternameonbothparameters(
) {
    let parsed_text = "type A = (p1: T, p2: T) -> (T, ...T)";
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

/// C# Parser_ParsesFunctionType_WithTypeParameters_AndTrailingVariadicPack_AndTypeReturn (TypeParsingTests.cs:655): '<T, T = T, T... = ...T, T... = T...> (T, ...T) -> T' parses as the function type with type parameters and a trailing variadic pack returning a type.
/// The C# type-parameter list supports defaults (`T = T`) and pack defaults
/// (`T... = ...T`); full_moon's generic-list syntax lacks the defaults, so
/// the C# shape is dropped (documented in the file header). The port pins
/// the drop: the unsupported syntax must be reported.
#[test]
fn parser_parsesfunctiontype_withtypeparameters_andtrailingvariadicpack_andtypereturn() {
    // The C# type-parameter list supports defaults (`T = T`) and pack
    // defaults (`T... = ...T`); full_moon's generic-list syntax lacks the
    // defaults, so the C# shape is dropped (documented in the file header).
    // The port pins the drop: the unsupported syntax must be reported.
    let parsed_text = "type A = <T, T = T, T... = ...T, T... = T...> (T, ...T) -> T";
    let result = full_moon::parse_fallible(
        parsed_text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    );
    assert!(
        !result.errors().is_empty(),
        "the C# type-parameter defaults are unsupported: expected parse errors"
    );
}

/// C# Parser_ParsesFunctionType_WithTypeParameters_AndTrailingVariadicPack_AndTypeReturn_AndParameterNameOnFirstParameter (TypeParsingTests.cs:754): '<T, T = T, T... = ...T, T... = T...> (p1: T, ...T) -> T' parses as the function type with type parameters, a named first parameter and a trailing variadic pack.
/// The C# type-parameter list supports defaults (`T = T`) and pack defaults
/// (`T... = ...T`); full_moon's generic-list syntax lacks the defaults, so
/// the C# shape is dropped (documented in the file header). The port pins
/// the drop: the unsupported syntax must be reported.
#[test]
fn parser_parsesfunctiontype_withtypeparameters_andtrailingvariadicpack_andtypereturn_andparameternameonfirstparameter(
) {
    // The C# type-parameter list supports defaults (`T = T`) and pack
    // defaults (`T... = ...T`); full_moon's generic-list syntax lacks the
    // defaults, so the C# shape is dropped (documented in the file header).
    // The port pins the drop: the unsupported syntax must be reported.
    let parsed_text = "type A = <T, T = T, T... = ...T, T... = T...> (p1: T, ...T) -> T";
    let result = full_moon::parse_fallible(
        parsed_text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    );
    assert!(
        !result.errors().is_empty(),
        "the C# type-parameter defaults are unsupported: expected parse errors"
    );
}

/// C# Parser_ParsesFunctionType_WithTypeParameters_AndTypeReturn_AndParameterNameOnSecondParameter (TypeParsingTests.cs:854): '<T, T = T, T... = ...T, T... = T...> (T, p2: T) -> T' parses as the function type with type parameters and a named second parameter.
/// The C# type-parameter list supports defaults (`T = T`) and pack defaults
/// (`T... = ...T`); full_moon's generic-list syntax lacks the defaults, so
/// the C# shape is dropped (documented in the file header). The port pins
/// the drop: the unsupported syntax must be reported.
#[test]
fn parser_parsesfunctiontype_withtypeparameters_andtypereturn_andparameternameonsecondparameter() {
    // The C# type-parameter list supports defaults (`T = T`) and pack
    // defaults (`T... = ...T`); full_moon's generic-list syntax lacks the
    // defaults, so the C# shape is dropped (documented in the file header).
    // The port pins the drop: the unsupported syntax must be reported.
    let parsed_text = "type A = <T, T = T, T... = ...T, T... = T...> (T, p2: T) -> T";
    let result = full_moon::parse_fallible(
        parsed_text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    );
    assert!(
        !result.errors().is_empty(),
        "the C# type-parameter defaults are unsupported: expected parse errors"
    );
}

/// C# Parser_ParsesFunctionType_WithTypeParameters_AndTypeReturn_AndParameterNameOnBothParameters (TypeParsingTests.cs:949): '<T, T = T, T... = ...T, T... = T...> (p1: T, p2: T) -> T' parses as the function type with type parameters and both parameters named.
/// The C# type-parameter list supports defaults (`T = T`) and pack defaults
/// (`T... = ...T`); full_moon's generic-list syntax lacks the defaults, so
/// the C# shape is dropped (documented in the file header). The port pins
/// the drop: the unsupported syntax must be reported.
#[test]
fn parser_parsesfunctiontype_withtypeparameters_andtypereturn_andparameternameonbothparameters() {
    // The C# type-parameter list supports defaults (`T = T`) and pack
    // defaults (`T... = ...T`); full_moon's generic-list syntax lacks the
    // defaults, so the C# shape is dropped (documented in the file header).
    // The port pins the drop: the unsupported syntax must be reported.
    let parsed_text = "type A = <T, T = T, T... = ...T, T... = T...> (p1: T, p2: T) -> T";
    let result = full_moon::parse_fallible(
        parsed_text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::LUAU)),
    );
    assert!(
        !result.errors().is_empty(),
        "the C# type-parameter defaults are unsupported: expected parse errors"
    );
}

/// C# Parser_ParsesStringType (TypeParsingTests.cs:1046): "'value'" parses as the singleton string type.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesstringtype() {
    let parsed_text = "type A = 'value'";
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

/// C# Parser_ParsesTrueType (TypeParsingTests.cs:1058): 'true' parses as the singleton true type.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsestruetype() {
    let parsed_text = "type A = true";
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

/// C# Parser_ParsesFalseType (TypeParsingTests.cs:1070): 'false' parses as the singleton false type.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesfalsetype() {
    let parsed_text = "type A = false";
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

/// C# Parser_ParsesNilType (TypeParsingTests.cs:1082): 'nil' parses as the nil type.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesniltype() {
    let parsed_text = "type A = nil";
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

/// C# Parser_ParsesParenthesizedTypes (TypeParsingTests.cs:1094): '(T)' parses as the parenthesized type.
/// The C# red-tree shapes have no full_moon equivalent; the port asserts
/// the clean parse + the round-trip.
#[test]
fn parser_parsesparenthesizedtypes() {
    let parsed_text = "type A = (T)";
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
