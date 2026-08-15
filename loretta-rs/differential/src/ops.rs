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

/// Folds constants in the provided code with the default (no string number
/// extraction) and all (with extraction) options, mirroring the C# reference
/// output: original text plus the folded text and whether it changed.
pub fn constantfold(code: &str) -> Result<Json, String> {
    use loretta::experimental::constantfolder::ConstantFolder;
    use loretta::experimental::constantfoldingoptions::ConstantFoldingOptions;

    let original = code.to_string();
    let fold = |options| {
        let result = full_moon::parse_fallible(code, full_moon::LuaVersion::new());
        let ast = result.into_ast();
        let folded = ConstantFolder::new(options).fold_ast(ast);
        let text = folded.to_string();
        let same = text == original;
        (text, same)
    };
    let (without_text, without_same) = fold(ConstantFoldingOptions::DEFAULT);
    let (with_text, with_same) = fold(ConstantFoldingOptions::ALL);
    Ok(Json::Object(vec![
        ("original".into(), Json::String(original)),
        (
            "withoutExtraction".into(),
            Json::Object(vec![
                ("foldedText".into(), Json::String(without_text)),
                ("same".into(), Json::Bool(without_same)),
            ]),
        ),
        (
            "withExtraction".into(),
            Json::Object(vec![
                ("foldedText".into(), Json::String(with_text)),
                ("same".into(), Json::Bool(with_same)),
            ]),
        ),
    ]))
}

/// The C# scope-kind names.
fn scope_kind_name(kind: loretta::scoping::scopekind::ScopeKind) -> &'static str {
    match kind {
        loretta::scoping::scopekind::ScopeKind::Global => "Global",
        loretta::scoping::scopekind::ScopeKind::File => "File",
        loretta::scoping::scopekind::ScopeKind::Function => "Function",
        loretta::scoping::scopekind::ScopeKind::Block => "Block",
    }
}

/// The C# variable-kind names.
fn variable_kind_name(kind: loretta::scoping::variablekind::VariableKind) -> &'static str {
    match kind {
        loretta::scoping::variablekind::VariableKind::Local => "Local",
        loretta::scoping::variablekind::VariableKind::Global => "Global",
        loretta::scoping::variablekind::VariableKind::Parameter => "Parameter",
        loretta::scoping::variablekind::VariableKind::Iteration => "Iteration",
    }
}

/// Serializes a scope: kind, nodeKind, declaredVariables and containedScopes.
fn serialize_scope(scope: &loretta::scoping::iscope::Scope) -> Json {
    use loretta::scoping::iscope::IScope;
    use loretta::scoping::ivariable::IVariable;
    let mut fields = Vec::new();
    fields.push(("kind".into(), Json::String(scope_kind_name(scope.kind()).into())));
    if let Some(node_kind) = scope.node_kind() {
        fields.push(("nodeKind".into(), Json::String(node_kind.into())));
    }
    let declared: Vec<Json> = scope
        .declared_variables()
        .iter()
        .map(|v| {
            let v = v.borrow();
            Json::Object(vec![
                ("name".into(), Json::String(v.name().to_string())),
                (
                    "kind".into(),
                    Json::String(variable_kind_name(v.kind()).into()),
                ),
            ])
        })
        .collect();
    fields.push(("declaredVariables".into(), Json::Array(declared)));
    let contained: Vec<Json> = scope
        .contained_scopes()
        .iter()
        .map(|s| serialize_scope(&s.borrow()))
        .collect();
    fields.push(("containedScopes".into(), Json::Array(contained)));
    Json::Object(fields)
}

/// Builds the scope tree for the provided code and serializes it, mirroring
/// the C# reference output (label, rootScope tree, scopeCount).
pub fn scope(code: &str, label: &str) -> Result<Json, String> {
    use loretta::scoping::scopeandvariablemanager::ScopeAndVariableManager;

    let result = full_moon::parse_fallible(code, full_moon::LuaVersion::new());
    let ast = result.into_ast();
    let manager = ScopeAndVariableManager::new(vec![ast]);
    let state = manager.get_lazy_state();
    let root = state.root_scope.clone();
    let root_json = serialize_scope(&root.borrow());
    let scope_count = count_scopes(&root);
    Ok(Json::Object(vec![
        ("label".into(), Json::String(label.into())),
        ("rootScope".into(), root_json),
        ("scopeCount".into(), Json::Number(scope_count as i64)),
    ]))
}

fn count_scopes(scope: &loretta::scoping::iscope::ScopeRef) -> usize {
    use loretta::scoping::iscope::IScope;
    1 + scope
        .borrow()
        .contained_scopes()
        .iter()
        .map(count_scopes)
        .sum::<usize>()
}
