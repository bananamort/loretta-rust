// Differential operations: options, diagnostics, lex, parse.
// Mirrors tools/differential/Program.cs RunOperation. scope/rename/constantfold/
// minify land as their ported subsystems land (scoping/script/experimental).

use crate::json::Json;
use crate::{collect_tokens, kind_name, token_full_text, token_text};
use loretta::luasyntaxoptions::LuaSyntaxOptions;

type Diagnostic = (String, String, String);

pub fn preset_options(preset: &str) -> LuaSyntaxOptions {
    match preset {
        "Lua51" => LuaSyntaxOptions::LUA51,
        "Lua52" => LuaSyntaxOptions::LUA52,
        "Lua53" => LuaSyntaxOptions::LUA53,
        "Lua54" => LuaSyntaxOptions::LUA54,
        "LuaJIT20" => LuaSyntaxOptions::LUAJIT20,
        "LuaJIT21" => LuaSyntaxOptions::LUAJIT21,
        "GMod" => LuaSyntaxOptions::GMOD,
        "Luau" => LuaSyntaxOptions::LUAU,
        "FiveM" => LuaSyntaxOptions::FIVEM,
        "All" => LuaSyntaxOptions::ALL,
        "AllWithIntegers" => LuaSyntaxOptions::ALL_WITH_INTEGERS,
        _ => LuaSyntaxOptions::ALL,
    }
}

pub fn compute_diagnostics(code: &str) -> Result<(Vec<Diagnostic>, bool), String> {
    match full_moon::parse(code) {
        Ok(_) => Ok((Vec::new(), false)),
        Err(errors) => Err(format!(
            "parser diagnostics mapping not ported yet; full_moon errors: {errors:?}"
        )),
    }
}

pub fn options(preset: &str) -> Result<Json, String> {
    use loretta::luaparseoptions::LuaParseOptions;

    let opts = preset_options(preset);
    let parse_opts = LuaParseOptions::new(opts.clone());
    let features: Vec<Json> = parse_opts
        .features
        .iter()
        .map(|(k, v)| Json::String(format!("{k}={v}")))
        .collect();
    let with_features: Vec<Json> = parse_opts
        .with_features(vec![("foo".to_string(), "bar".to_string())])
        .features
        .iter()
        .map(|(k, v)| Json::String(format!("{k}={v}")))
        .collect();
    Ok(Json::Object(vec![
        ("preset".into(), Json::String(opts.to_string())),
        (
            "language".into(),
            Json::String(parse_opts.language().to_string()),
        ),
        (
            "documentationMode".into(),
            Json::String(parse_opts.documentation_mode().to_string()),
        ),
        ("features".into(), Json::Array(features)),
        ("withFeatures".into(), Json::Array(with_features)),
    ]))
}

pub fn diagnostics(code: &str) -> Result<Json, String> {
    let (diagnostics, has_errors) = compute_diagnostics(code)?;
    let diags: Vec<Json> = diagnostics
        .iter()
        .map(|d| {
            Json::Object(vec![
                ("id".into(), Json::String(d.0.clone())),
                ("severity".into(), Json::String(d.1.clone())),
                ("message".into(), Json::String(d.2.clone())),
            ])
        })
        .collect();
    Ok(Json::Object(vec![
        ("diagnostics".into(), Json::Array(diags)),
        ("hasErrors".into(), Json::Bool(has_errors)),
    ]))
}

pub fn lex(code: &str) -> Result<Json, String> {
    let ast = full_moon::parse(code).map_err(|errors| format!("parse failed: {errors:?}"))?;
    let tokens = collect_tokens(&ast);
    let token_jsons: Vec<Json> = tokens
        .iter()
        .map(|t| {
            Json::Object(vec![
                ("kind".into(), Json::String(kind_name(t.token()))),
                ("text".into(), Json::String(token_text(t))),
                ("fullText".into(), Json::String(token_full_text(t))),
                ("isMissing".into(), Json::Bool(false)),
            ])
        })
        .collect();
    let round_trip = ast.to_string() == code;
    Ok(Json::Object(vec![
        ("tokens".into(), Json::Array(token_jsons)),
        ("count".into(), Json::Number(tokens.len() as i64)),
        ("roundTrip".into(), Json::Bool(round_trip)),
    ]))
}

pub fn parse(code: &str) -> Result<Json, String> {
    let ast = full_moon::parse(code).map_err(|errors| format!("parse failed: {errors:?}"))?;
    let (_, has_errors) = compute_diagnostics(code)?;
    Ok(Json::Object(vec![
        ("treeText".into(), Json::String(ast.to_string())),
        (
            "rootKind".into(),
            Json::String("CompilationUnit".to_string()),
        ),
        ("hasErrors".into(), Json::Bool(has_errors)),
    ]))
}

/// GotoLabel oracle: builds a GotoLabel from a fixed Lua 5.2 sample
/// ("::top::" + two gotos), adds the jumps, and dumps name/labelText/jumps —
/// mirroring the C# reference's GotoLabelOp.
pub fn gotolabel() -> Result<Json, String> {
    use full_moon::ast::lua52::{Goto, Label};
    use full_moon::ast::Stmt;
    use loretta::scoping::igotolabel::{GotoLabel, IGotoLabelInternal};

    const SAMPLE: &str = "::top::\ngoto top\ngoto top\n";
    let ast = full_moon::parse_fallible(SAMPLE, full_moon::LuaVersion::lua52())
        .into_result()
        .map_err(|errors| format!("parse failed: {errors:?}"))?;
    let mut label: Option<Label> = None;
    let mut gotos: Vec<Goto> = Vec::new();
    for stmt in ast.nodes().stmts() {
        match stmt {
            Stmt::Label(l) => {
                if label.is_none() {
                    label = Some(l.clone());
                }
            }
            Stmt::Goto(g) => gotos.push(g.clone()),
            _ => {}
        }
    }
    let label = label.expect("sample contains a label");
    let name = label.name().token().to_string();
    let mut goto_label = GotoLabel::new(name.clone(), Some(label.clone()));
    for g in &gotos {
        goto_label.add_jump(g.clone());
    }
    Ok(Json::Object(vec![
        ("name".into(), Json::String(name)),
        ("labelText".into(), Json::String(label.to_string())),
        (
            "jumps".into(),
            Json::Array(gotos.iter().map(|g| Json::String(g.to_string())).collect()),
        ),
    ]))
}

/// MessageProvider oracle: the category of every ErrorCode in declaration
/// order, mirroring the C# reference's MessageProviderOp.
pub fn messageprovider() -> Result<Json, String> {
    use loretta::errors::errorcode::ErrorCode;
    use loretta::errors::messageprovider::MessageProvider;

    let mut codes: Vec<ErrorCode> = vec![
        ErrorCode::Void,
        ErrorCode::Unknown,
        ErrorCode::ErrInvalidStringEscape,
        ErrorCode::ErrUnfinishedString,
        ErrorCode::ErrInvalidNumber,
        ErrorCode::ErrNumericLiteralTooLarge,
        ErrorCode::ErrUnfinishedLongComment,
        ErrorCode::ErrShebangNotSupportedInLuaVersion,
        ErrorCode::ErrBinaryNumericLiteralNotSupportedInVersion,
        ErrorCode::ErrOctalNumericLiteralNotSupportedInVersion,
        ErrorCode::ErrHexFloatLiteralNotSupportedInVersion,
        ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion,
        ErrorCode::ErrCCommentsNotSupportedInVersion,
        ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion,
        ErrorCode::ErrBadCharacter,
        ErrorCode::ErrUnexpectedToken,
        ErrorCode::ErrHexStringEscapesNotSupportedInVersion,
        ErrorCode::ErrAmbiguousFunctionCallOrNewStatement,
        ErrorCode::ErrNonFunctionCallBeingUsedAsStatement,
        ErrorCode::ErrCannotBeAssignedTo,
        ErrorCode::ErrDoubleOverflow,
        ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion,
        ErrorCode::WrnLineBreakMayAffectErrorReporting,
        ErrorCode::ErrWhitespaceEscapeNotSupportedInVersion,
        ErrorCode::ErrUnicodeEscapeMissingOpenBrace,
        ErrorCode::ErrUnicodeEscapeMissingCloseBrace,
        ErrorCode::ErrEscapeTooLarge,
        ErrorCode::ErrHexDigitExpected,
        ErrorCode::ErrUnicodeEscapesNotSupportedLuaInVersion,
        ErrorCode::ErrNumberSuffixNotSupportedInVersion,
        ErrorCode::ErrLuajitSuffixInFloat,
        ErrorCode::ErrLua51NestingInLongString,
        ErrorCode::ErrInterpolatedStringMustStartWithBacktickCharacter,
        ErrorCode::ErrUnclosedExpressionHole,
        ErrorCode::ErrDoubleBraceInInterpolation,
        ErrorCode::ErrInterpolatedStringsNotSupportedInVersion,
        ErrorCode::ErrIdentifierExpectedKw,
        ErrorCode::ErrIdentifierExpected,
        ErrorCode::ErrSemicolonExpected,
        ErrorCode::ErrCloseParenExpected,
        ErrorCode::ErrLbraceExpected,
        ErrorCode::ErrRbraceExpected,
        ErrorCode::ErrSyntaxError,
        ErrorCode::ErrInsufficientStack,
        ErrorCode::ErrIfExpressionsNotSupportedInLuaVersion,
        ErrorCode::ErrIfExpressionConditionExpected,
        ErrorCode::ErrExpressionExpected,
        ErrorCode::ErrInvalidExpressionPart,
        ErrorCode::ErrInvalidStatement,
        ErrorCode::ErrCompoundAssignmentNotSupportedInLuaVersion,
        ErrorCode::ErrMixingNilableAndIntersectionNotAllowed,
        ErrorCode::ErrMixingUnionsAndIntersectionsNotAllowed,
        ErrorCode::ErrTypedLuaNotSupportedInLuaVersion,
        ErrorCode::ErrOnlyOneTableTypeIndexerIsAllowed,
        ErrorCode::ErrNormalTypeParametersComeBeforePacks,
        ErrorCode::ErrGotoNotSupportedInLuaVersion,
        ErrorCode::ErrBadDocumentationMode,
    ];
    // .NET Enum.GetValues returns the values in increasing order of the
    // underlying value interpreted as unsigned (observed: 1..2000, -2, -1).
    codes.sort_by_key(|code| *code as u32);
    let results: Vec<Json> = codes
        .iter()
        .map(|code| {
            let mut fields = vec![
                ("code".into(), Json::Number(*code as i64)),
                (
                    "severity".into(),
                    Json::Number(MessageProvider::get_severity(*code) as i64),
                ),
                (
                    "category".into(),
                    Json::String(MessageProvider::get_category(*code)),
                ),
                (
                    "description".into(),
                    Json::String(MessageProvider::get_description(*code)),
                ),
            ];
            if let Some(message) = MessageProvider::load_message(*code) {
                fields.push(("message".into(), Json::String(message)));
            }
            fields.push((
                "messageFormat".into(),
                Json::String(MessageProvider::get_message_format(*code)),
            ));
            Json::Object(fields)
        })
        .collect();
    Ok(Json::Object(vec![("results".into(), Json::Array(results))]))
}

/// CharUtils oracle: for every char of the input, the ported CharUtils result
/// per member. Mirrors the C# reference's CharUtilsOp (real Loretta via
/// reflection); members land one node at a time.
pub fn charutils(code: &str) -> Result<Json, String> {
    use loretta::utilities::charutils::CharUtils;
    let results: Vec<Json> = code
        .chars()
        .map(|ch| {
            Json::Object(vec![
                ("ch".into(), Json::String(ch.to_string())),
                ("isBinary".into(), Json::Bool(CharUtils::is_binary(ch))),
                ("isDecimal".into(), Json::Bool(CharUtils::is_decimal(ch))),
                ("isOctal".into(), Json::Bool(CharUtils::is_octal(ch))),
                (
                    "isWhitespace".into(),
                    Json::Bool(CharUtils::is_whitespace(ch)),
                ),
            ])
        })
        .collect();
    Ok(Json::Object(vec![("results".into(), Json::Array(results))]))
}

/// ObjectDisplay oracle: a fixed sample set of doubles formatted in decimal
/// and hexadecimal modes. Mirrors the C# reference's ObjectDisplayOp.
pub fn objectdisplay() -> Result<Json, String> {
    use loretta::symbol_display::objectdisplay::ObjectDisplay;
    use loretta::symbol_display::objectdisplayoptions::ObjectDisplayOptions;

    let values: [f64; 37] = [
        0.0,
        -0.0,
        1.0,
        -1.0,
        0.1,
        255.255,
        100.0,
        1e5,
        1e6,
        1e7,
        1e15,
        1e16,
        1e17,
        1e18,
        1e20,
        1e-1,
        1e-2,
        1e-3,
        1e-4,
        1e-5,
        1e-6,
        std::f64::consts::PI, // same double as the reference's 3.141592653589793
        123456789012345678.0,
        1.2345678901234567e16,
        9999999999999999.0,
        1.5e-300,
        1e300,
        5e-324,
        0.5,
        2.0,
        6.25,
        -123.456,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        2.2250738585072014e-308,
        1.7976931348623157e308,
    ];
    let results: Vec<Json> = values
        .iter()
        .map(|v| {
            Json::Object(vec![
                (
                    "decimalLiteral".into(),
                    Json::String(ObjectDisplay::format_literal_f64(
                        *v,
                        ObjectDisplayOptions::NONE,
                    )),
                ),
                (
                    "hexadecimalLiteral".into(),
                    Json::String(ObjectDisplay::format_literal_f64(
                        *v,
                        ObjectDisplayOptions::USE_HEXADECIMAL_NUMBERS,
                    )),
                ),
            ])
        })
        .collect();
    Ok(Json::Object(vec![("results".into(), Json::Array(results))]))
}
