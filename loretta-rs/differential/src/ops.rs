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
    let opts = preset_options(preset);
    Ok(Json::Object(vec![(
        "preset".into(),
        Json::String(opts.to_string()),
    )]))
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
