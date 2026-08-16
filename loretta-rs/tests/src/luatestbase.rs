// Ported from Loretta.CodeAnalysis.Lua.Test.Utilities.LuaTestBase (b767b4e): LuaTestBase
// C# source: src/Compilers/Lua/Test/Utilities/LuaTestBase.cs

use full_moon::ast::Ast;

use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

/// C# LuaTestBase (LuaTestBase.cs:14-225): the test base with the syntax-tree
/// factories, the round-trip checks, the binding helpers, and the validate
/// helpers. The dropped infra maps as follows:
///   - SyntaxTree/SyntaxNode -> full_moon Ast/Expression/Statement/Type
///     (the parse factories parse a chunk and return the AST).
///   - SourceText/Encoding -> &str (UTF-8; the C# UTF-16 encoding parameter
///     maps to the Rust text bytes).
///   - CheckSerializable/SerializeTo/DeserializeFrom -> dropped (pooling and
///     serialization infra, AGENTS.md Locked Decision 1); the round-trip the
///     tests rely on is the TEXT round-trip, preserved in the helpers below.
///   - GetDiagnostics().Verify() -> the full_moon parse error list (the C#
///     parse diagnostics map to the dropped parser's error reporting; the
///     port asserts the parse produced no errors).
pub struct LuaTestBase;

/// The options -> full_moon LuaVersion mapping used by the parse factories
/// (the CLI's preset mapping is the reference; the test default options map
/// to the all-features default version).
pub fn options_to_version(_options: &LuaParseOptions) -> full_moon::LuaVersion {
    full_moon::LuaVersion::new()
}

impl LuaTestBase {
    /// C# ParseAsync (LuaTestBase.cs:20-33): parses the text with the
    /// provided options (default = LuaParseOptions.Default).
    pub fn parse_async(text: &str, options: Option<&LuaParseOptions>) -> Ast {
        let default = LuaParseOptions::default_options();
        let options = options.unwrap_or(&default);
        full_moon::parse_fallible(text, options_to_version(options))
            .into_result()
            .expect("parse failed")
    }

    /// C# ParseAsync(IEnumerable<string>) / ParseAsync(options, params
    /// string[]) (LuaTestBase.cs:83-95): the array overloads.
    pub fn parse_async_many(sources: &[&str], options: Option<&LuaParseOptions>) -> Vec<Ast> {
        sources
            .iter()
            .map(|s| Self::parse_async(s, options))
            .collect()
    }

    /// C# ParseExpressionAsync (LuaTestBase.cs:35-42): the dropped
    /// ExpressionSyntax maps to the parsed chunk's AST (the mechanical
    /// adaptation — the tests round-trip the text either way).
    pub fn parse_expression_async(text: &str, options: Option<&LuaParseOptions>) -> Ast {
        Self::parse_async(text, options)
    }

    /// C# ParseStatementAsync (LuaTestBase.cs:44-51).
    pub fn parse_statement_async(text: &str, options: Option<&LuaParseOptions>) -> Ast {
        Self::parse_async(text, options)
    }

    /// C# ParseTypeAsync (LuaTestBase.cs:53-61): defaults to the Luau
    /// syntax options in the C#; the all-features default version is the
    /// port's equivalent.
    pub fn parse_type_async(text: &str, options: Option<&LuaParseOptions>) -> Ast {
        let options = match options {
            Some(o) => o,
            None => &LuaParseOptions::new(LuaSyntaxOptions::LUAU),
        };
        full_moon::parse_fallible(text, options_to_version(options))
            .into_result()
            .expect("parse failed")
    }

    /// C# ParseWithRoundTripCheckAsync (LuaTestBase.cs:97-105): validates
    /// that the text round-trips through the parse.
    pub fn parse_with_round_trip_check_async(text: &str, options: Option<&LuaParseOptions>) -> Ast {
        let ast = Self::parse_async(text, options);
        assert_eq!(ast.to_string(), text, "the text must round-trip");
        ast
    }

    /// C# ParseExpressionWithRoundTripCheckAsync (LuaTestBase.cs:107-116).
    pub fn parse_expression_with_round_trip_check_async(
        text: &str,
        options: Option<&LuaParseOptions>,
    ) -> Ast {
        Self::parse_with_round_trip_check_async(text, options)
    }

    /// C# ParseStatementWithRoundTripCheckAsync (LuaTestBase.cs:118-127).
    pub fn parse_statement_with_round_trip_check_async(
        text: &str,
        options: Option<&LuaParseOptions>,
    ) -> Ast {
        Self::parse_with_round_trip_check_async(text, options)
    }

    /// C# ParseTypeWithRoundTripCheckAsync (LuaTestBase.cs:129-137).
    pub fn parse_type_with_round_trip_check_async(
        text: &str,
        options: Option<&LuaParseOptions>,
    ) -> Ast {
        let ast = Self::parse_type_async(text, options);
        assert_eq!(ast.to_string(), text, "the text must round-trip");
        ast
    }

    /// C# GetSyntaxNodeList(SyntaxTree) (LuaTestBase.cs:141-143): the node +
    /// its descendants in order — the port's AST descent collects each
    /// node's full text.
    pub fn get_syntax_node_list(ast: &Ast) -> Vec<String> {
        let mut list = Vec::new();
        collect_nodes(ast.nodes(), &mut list);
        list
    }

    /// C# GetSyntaxNodeForBinding (LuaTestBase.cs:150): the first node whose
    /// text matches the `--[[bind]]` markers (BindingStart/BindingEnd,
    /// LuaTestBase.cs:152-153).
    pub fn get_syntax_node_for_binding(nodes: &[String]) -> Option<String> {
        Self::get_syntax_node_of_type_for_binding(nodes)
    }

    /// C# GetSyntaxNodeOfTypeForBinding<TNode> (LuaTestBase.cs:155-183).
    pub fn get_syntax_node_of_type_for_binding(nodes: &[String]) -> Option<String> {
        for text in nodes {
            let expr_full_text = text.trim();
            if expr_full_text.starts_with("--[[bind]]") {
                if expr_full_text.contains("--[[/bind]]") {
                    if expr_full_text.ends_with("--[[/bind]]") {
                        return Some(text.clone());
                    }
                    continue;
                }
                return Some(text.clone());
            }
            if expr_full_text.ends_with("--[[/bind]]") {
                if expr_full_text.contains("--[[bind]]") {
                    if expr_full_text.starts_with("--[[bind]]") {
                        return Some(text.clone());
                    }
                } else {
                    return Some(text.clone());
                }
            }
        }
        None
    }

    /// C# ParseAndValidateAsync (LuaTestBase.cs:170-178): parse + round-trip
    /// + diagnostics verify (the port asserts the parse produced no errors).
    pub fn parse_and_validate_async(text: &str, options: Option<&LuaSyntaxOptions>) -> Ast {
        let parse_options = LuaParseOptions::new(options.cloned().unwrap_or(LuaSyntaxOptions::ALL));
        let ast = Self::parse_with_round_trip_check_async(text, Some(&parse_options));
        assert!(
            full_moon::parse_fallible(text, options_to_version(&parse_options))
                .errors()
                .is_empty(),
            "the parse must produce no diagnostics"
        );
        ast
    }

    /// C# ParseAndValidateExpressionAsync (LuaTestBase.cs:180-188).
    pub fn parse_and_validate_expression_async(
        text: &str,
        options: Option<&LuaSyntaxOptions>,
    ) -> Ast {
        Self::parse_and_validate_async(text, options)
    }

    /// C# ParseAndValidateTypeAsync (LuaTestBase.cs:190-197): defaults to the
    /// Luau syntax options in the C#.
    pub fn parse_and_validate_type_async(text: &str, options: Option<&LuaSyntaxOptions>) -> Ast {
        let parse_options =
            LuaParseOptions::new(options.cloned().unwrap_or(LuaSyntaxOptions::LUAU));
        let ast = Self::parse_type_with_round_trip_check_async(text, Some(&parse_options));
        assert!(
            full_moon::parse_fallible(text, options_to_version(&parse_options))
                .errors()
                .is_empty(),
            "the parse must produce no diagnostics"
        );
        ast
    }
}

/// Collects a block's statements and their descendant expressions' texts
/// (the C# GetSyntaxNodeList descent).
fn collect_nodes(block: &full_moon::ast::Block, list: &mut Vec<String>) {
    for stmt in block.stmts() {
        list.push(stmt.to_string());
        collect_stmt_exprs(stmt, list);
    }
    if let Some(last) = block.last_stmt() {
        list.push(last.to_string());
    }
}

fn collect_stmt_exprs(stmt: &full_moon::ast::Stmt, list: &mut Vec<String>) {
    match stmt {
        full_moon::ast::Stmt::Assignment(a) => {
            for var in a.variables().iter() {
                list.push(var.to_string());
            }
            for expr in a.expressions().iter() {
                collect_expr(expr, list);
            }
        }
        full_moon::ast::Stmt::LocalAssignment(la) => {
            for expr in la.expressions().iter() {
                collect_expr(expr, list);
            }
        }
        full_moon::ast::Stmt::Do(d) => collect_nodes(d.block(), list),
        full_moon::ast::Stmt::While(w) => {
            collect_expr(w.condition(), list);
            collect_nodes(w.block(), list);
        }
        full_moon::ast::Stmt::Repeat(r) => {
            collect_nodes(r.block(), list);
            collect_expr(r.until(), list);
        }
        full_moon::ast::Stmt::If(if_stmt) => {
            collect_expr(if_stmt.condition(), list);
            collect_nodes(if_stmt.block(), list);
            if let Some(else_ifs) = if_stmt.else_if() {
                for else_if in else_ifs {
                    collect_expr(else_if.condition(), list);
                    collect_nodes(else_if.block(), list);
                }
            }
            if let Some(else_block) = if_stmt.else_block() {
                collect_nodes(else_block, list);
            }
        }
        full_moon::ast::Stmt::NumericFor(nf) => {
            collect_expr(nf.start(), list);
            collect_expr(nf.end(), list);
            if let Some(step) = nf.step() {
                collect_expr(step, list);
            }
            collect_nodes(nf.block(), list);
        }
        full_moon::ast::Stmt::GenericFor(gf) => {
            for expr in gf.expressions().iter() {
                collect_expr(expr, list);
            }
            collect_nodes(gf.block(), list);
        }
        full_moon::ast::Stmt::FunctionDeclaration(fd) => {
            collect_nodes(fd.body().block(), list);
        }
        full_moon::ast::Stmt::LocalFunction(lf) => {
            collect_nodes(lf.body().block(), list);
        }
        full_moon::ast::Stmt::FunctionCall(call) => {
            list.push(call.to_string());
        }
        _ => {}
    }
}

fn collect_expr(expr: &full_moon::ast::Expression, list: &mut Vec<String>) {
    list.push(expr.to_string());
    match expr {
        full_moon::ast::Expression::BinaryOperator { lhs, rhs, .. } => {
            collect_expr(lhs, list);
            collect_expr(rhs, list);
        }
        full_moon::ast::Expression::UnaryOperator { expression, .. } => {
            collect_expr(expression, list);
        }
        full_moon::ast::Expression::Parentheses { expression, .. } => {
            collect_expr(expression, list);
        }
        full_moon::ast::Expression::FunctionCall(call) => {
            list.push(call.to_string());
        }
        _ => {}
    }
}
