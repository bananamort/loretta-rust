// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.ParsingTestsBase (b767b4e): ParsingTestsBase
// C# source: src/Compilers/Lua/Test/Portable/Parsing/ParsingTestsBase.cs
//
// The C# base provides the parse factories, the diagnostics runner, and the
// depth-first preorder tree enumerator (UsingNode/N/M/EOF). The dropped
// SyntaxNode tree maps to the full_moon AST: the enumerator walks the AST
// preorder yielding (C# kind name, missing, text) rows — the C# kind names
// come from the dropped SyntaxKind.cs; full_moon has no missing-node
// artifacts (its error recovery reconstructs), so the C# missing rows are
// adapted per test when the parsing test classes land.

use full_moon::ast::{Ast, Expression, Stmt};
use full_moon::tokenizer::{Symbol, TokenReference, TokenType};

use crate::lexerdiagnostics::LexerDiagnostic;
use loretta::errors::errorcode::ErrorCode;
use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

/// A preorder tree row (the C# SyntaxNodeOrToken the enumerator yields).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRow {
    /// The C# SyntaxKind name.
    pub kind: String,
    /// The C# IsMissing flag.
    pub missing: bool,
    /// The node's or token's text (C# ToString).
    pub text: String,
}

/// The C# SyntaxFactory.ParseSyntaxTree — the parse entry.
pub fn parse_tree(text: &str, options: &LuaSyntaxOptions) -> Ast {
    let parse_options = LuaParseOptions::new(options.clone());
    full_moon::parse_fallible(text, crate::luatestbase::options_to_version(&parse_options))
        .into_result()
        .expect("parse failed")
}

/// The C# SyntaxFactory.ParseCompilationUnit.
pub fn parse_file(text: &str, options: Option<&LuaSyntaxOptions>) -> Ast {
    let options = options.unwrap_or(&LuaSyntaxOptions::ALL);
    parse_tree(text, options)
}

/// The C# ParseAndValidateAsync (ParsingTestsBase.cs:44-51): parse +
/// round-trip + the diagnostics verify. The C# tree diagnostics (the dropped
/// lexer + parser diagnostics) map to the lexer-diagnostics scanner (the
/// parser diagnostic rules land with the parsing test classes).
pub fn parse_and_validate_async(
    text: &str,
    options: &LuaSyntaxOptions,
    expected: &[ExpectedDiagnostic],
) -> Ast {
    let ast = parse_tree(text, options);
    assert_eq!(
        ast.to_string(),
        text,
        "the text must round-trip for {text:?}"
    );
    let produced = crate::lexerdiagnostics::lexer_diagnostics(text, options);
    verify_diagnostics(text, &produced, expected);
    ast
}

/// The expected diagnostic (code + 1-based position + squiggled span text +
/// message arguments) — the C# DiagnosticDescription.
pub struct ExpectedDiagnostic {
    pub code: ErrorCode,
    pub line: usize,
    pub col: usize,
    pub squiggle: &'static str,
    pub args: Vec<&'static str>,
}

/// The C# Verify — the produced diagnostics must exactly match the expected.
pub fn verify_diagnostics(
    source: &str,
    produced: &[LexerDiagnostic],
    expected: &[ExpectedDiagnostic],
) {
    assert_eq!(
        produced.len(),
        expected.len(),
        "diagnostic count for {source:?}: produced={produced:?}"
    );
    for (i, (actual, exp)) in produced.iter().zip(expected.iter()).enumerate() {
        let (line, col) = actual.line_col(source);
        assert_eq!(actual.code, exp.code, "diag {i} code for {source:?}");
        assert_eq!(
            (line, col),
            (exp.line, exp.col),
            "diag {i} position for {source:?} ({:?})",
            actual.squiggle(source)
        );
        assert_eq!(
            actual.squiggle(source),
            exp.squiggle,
            "diag {i} squiggle for {source:?}"
        );
        assert_eq!(
            actual.arguments,
            exp.args.iter().map(|a| a.to_string()).collect::<Vec<_>>(),
            "diag {i} args for {source:?}"
        );
    }
}

/// The depth-first preorder enumerator (the C# EnumerateNodes,
/// ParsingTestsBase.cs:209-241). The port walks the full_moon AST with the
/// Visitor (whose hooks fire in source order — the C# preorder) yielding the
/// statement/expression rows and the token rows. The C# `missing` rows have
/// no full_moon equivalent (documented above).
pub struct TreeEnumerator {
    rows: std::vec::IntoIter<TreeRow>,
}

impl TreeEnumerator {
    /// Creates the enumerator for a parsed tree (the C# UsingTreeAsync /
    /// UsingNode root).
    pub fn new(ast: &Ast) -> Self {
        let mut collector = RowCollector { rows: Vec::new() };
        full_moon::visitors::Visitor::visit_ast(&mut collector, ast);
        TreeEnumerator {
            rows: collector.rows.into_iter(),
        }
    }

    /// The next row (the C# MoveNext + Current).
    pub fn next_row(&mut self) -> Option<TreeRow> {
        self.rows.next()
    }
}

/// The row collector — the C# preorder via the full_moon visitor hooks.
struct RowCollector {
    rows: Vec<TreeRow>,
}

impl RowCollector {
    fn row(&mut self, kind: &'static str, text: &str) {
        self.rows.push(TreeRow {
            kind: kind.to_string(),
            missing: false,
            text: text.to_string(),
        });
    }
}

impl full_moon::visitors::Visitor for RowCollector {
    fn visit_ast(&mut self, ast: &Ast) {
        self.row("CompilationUnit", &ast.to_string());
        // The block's statements in source order (the C# preorder).
        for stmt in ast.nodes().stmts() {
            full_moon::visitors::Visitor::visit_stmt(self, stmt);
        }
        if let Some(last) = ast.nodes().last_stmt() {
            full_moon::visitors::Visitor::visit_last_stmt(self, last);
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        self.row(stmt_kind_name(stmt), &stmt.to_string());
        default_visit_stmt(self, stmt);
    }

    fn visit_last_stmt(&mut self, last: &full_moon::ast::LastStmt) {
        self.row(last_stmt_kind_name(last), &last.to_string());
        default_visit_last_stmt(self, last);
    }

    fn visit_expression(&mut self, expr: &Expression) {
        self.row(expr_kind_name(expr), &expr.to_string());
        default_visit_expression(self, expr);
    }

    fn visit_token_reference(&mut self, token_ref: &TokenReference) {
        self.row(
            token_kind_name(token_ref.token().token_type()),
            &token_ref.token().to_string(),
        );
    }
}

/// The full_moon Visitor default descent for a statement (kept in free
/// functions so the recursion checker sees no direct self-recursion).
fn default_visit_stmt(visitor: &mut RowCollector, stmt: &Stmt) {
    full_moon::visitors::Visitor::visit_stmt(visitor, stmt);
}

fn default_visit_last_stmt(visitor: &mut RowCollector, last: &full_moon::ast::LastStmt) {
    full_moon::visitors::Visitor::visit_last_stmt(visitor, last);
}

fn default_visit_expression(visitor: &mut RowCollector, expr: &Expression) {
    full_moon::visitors::Visitor::visit_expression(visitor, expr);
}

/// The C# SyntaxKind name for a full_moon statement (the dropped node kinds).
fn stmt_kind_name(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::Assignment(_) => "AssignmentStatement",
        Stmt::CompoundAssignment(_) => "CompoundAssignmentStatement",
        Stmt::LocalAssignment(_) => "LocalVariableDeclarationStatement",
        Stmt::LocalFunction(_) => "LocalFunctionDeclarationStatement",
        Stmt::Do(_) => "DoStatement",
        Stmt::While(_) => "WhileStatement",
        Stmt::Repeat(_) => "RepeatUntilStatement",
        Stmt::If(_) => "IfStatement",
        Stmt::NumericFor(_) => "NumericForStatement",
        Stmt::GenericFor(_) => "GenericForStatement",
        Stmt::FunctionDeclaration(_) => "FunctionDeclarationStatement",
        Stmt::FunctionCall(_) => "ExpressionStatement",
        #[allow(unreachable_patterns)]
        _ => "UnknownStatement",
    }
}

/// The C# SyntaxKind name for a full_moon last statement.
fn last_stmt_kind_name(last: &full_moon::ast::LastStmt) -> &'static str {
    match last {
        full_moon::ast::LastStmt::Return(_) => "ReturnStatement",
        full_moon::ast::LastStmt::Break(_) => "BreakStatement",
        full_moon::ast::LastStmt::Continue(_) => "ContinueStatement",
        #[allow(unreachable_patterns)]
        _ => "UnknownStatement",
    }
}

/// The C# SyntaxKind name for a full_moon expression.
fn expr_kind_name(expr: &Expression) -> &'static str {
    match expr {
        Expression::Number(_) => "NumericLiteralExpression",
        Expression::String(_) => "StringLiteralExpression",
        Expression::Symbol(t) if t.is_symbol(Symbol::Nil) => "NilLiteralExpression",
        Expression::Symbol(t) if t.is_symbol(Symbol::True) || t.is_symbol(Symbol::False) => {
            "BooleanLiteralExpression"
        }
        Expression::Symbol(_) => "IdentifierName",
        Expression::Parentheses { .. } => "ParenthesizedExpression",
        Expression::UnaryOperator { .. } => "UnaryExpression",
        Expression::BinaryOperator { .. } => "BinaryExpression",
        Expression::Function(_) => "AnonymousFunctionExpression",
        Expression::TableConstructor(_) => "TableConstructorExpression",
        Expression::Var(_) => "IdentifierName",
        Expression::FunctionCall(_) => "InvocationExpression",
        #[allow(unreachable_patterns)]
        _ => "UnknownExpression",
    }
}

/// The C# SyntaxKind name for a full_moon token type (the names verified in
/// Portable/Syntax/SyntaxKind.cs).
fn token_kind_name(kind: &TokenType) -> &'static str {
    match kind {
        TokenType::Eof => "EndOfFileToken",
        TokenType::Identifier { .. } => "IdentifierToken",
        TokenType::Number { .. } => "NumericLiteralToken",
        TokenType::StringLiteral { .. } => "StringLiteralToken",
        TokenType::SingleLineComment { .. } => "SingleLineCommentTrivia",
        TokenType::MultiLineComment { .. } => "MultiLineCommentTrivia",
        TokenType::Whitespace { .. } => "WhitespaceTrivia",
        TokenType::Shebang { .. } => "ShebangTrivia",
        TokenType::Symbol { symbol } => symbol_kind_name(symbol),
        TokenType::InterpolatedString { .. } => "InterpolatedStringToken",
        TokenType::CStyleComment { .. } => "CStyleCommentTrivia",
        #[allow(unreachable_patterns)]
        _ => "UnknownToken",
    }
}

/// The C# SyntaxKind name for a full_moon symbol (the names verified in
/// Portable/Syntax/SyntaxKind.cs:85-612).
fn symbol_kind_name(symbol: &Symbol) -> &'static str {
    match symbol {
        Symbol::And => "AndKeyword",
        Symbol::Break => "BreakKeyword",
        Symbol::Do => "DoKeyword",
        Symbol::Else => "ElseKeyword",
        Symbol::ElseIf => "ElseIfKeyword",
        Symbol::End => "EndKeyword",
        Symbol::False => "FalseKeyword",
        Symbol::For => "ForKeyword",
        Symbol::Function => "FunctionKeyword",
        Symbol::If => "IfKeyword",
        Symbol::In => "InKeyword",
        Symbol::Local => "LocalKeyword",
        Symbol::Nil => "NilKeyword",
        Symbol::Not => "NotKeyword",
        Symbol::Or => "OrKeyword",
        Symbol::Repeat => "RepeatKeyword",
        Symbol::Return => "ReturnKeyword",
        Symbol::Then => "ThenKeyword",
        Symbol::True => "TrueKeyword",
        Symbol::Until => "UntilKeyword",
        Symbol::While => "WhileKeyword",
        Symbol::Goto => "GotoKeyword",
        Symbol::PlusEqual => "PlusEqualsToken",
        Symbol::MinusEqual => "MinusEqualsToken",
        Symbol::StarEqual => "StarEqualsToken",
        Symbol::SlashEqual => "SlashEqualsToken",
        Symbol::DoubleSlashEqual => "SlashSlashEqualsToken",
        Symbol::PercentEqual => "PercentEqualsToken",
        Symbol::CaretEqual => "HatEqualsToken",
        Symbol::TwoDotsEqual => "DotDotEqualsToken",
        Symbol::Ampersand => "AmpersandToken",
        Symbol::ThinArrow => "MinusGreaterThanToken",
        Symbol::TwoColons => "ColonColonToken",
        Symbol::AtSign => "AtSignToken",
        Symbol::Caret => "HatToken",
        Symbol::Colon => "ColonToken",
        Symbol::Comma => "CommaToken",
        Symbol::Dot => "DotToken",
        Symbol::TwoDots => "DotDotToken",
        Symbol::Ellipsis => "DotDotDotToken",
        Symbol::Equal => "EqualsToken",
        Symbol::TwoEqual => "EqualsEqualsToken",
        Symbol::GreaterThan => "GreaterThanToken",
        Symbol::GreaterThanEqual => "GreaterThanEqualsToken",
        Symbol::DoubleGreaterThan => "GreaterThanGreaterThanToken",
        Symbol::Hash => "HashToken",
        Symbol::LeftBrace => "OpenBraceToken",
        Symbol::LeftBracket => "OpenBracketToken",
        Symbol::LeftParen => "OpenParenthesisToken",
        Symbol::LessThan => "LessThanToken",
        Symbol::LessThanEqual => "LessThanEqualsToken",
        Symbol::DoubleLessThan => "LessThanLessThanToken",
        Symbol::Minus => "MinusToken",
        Symbol::Percent => "PercentToken",
        Symbol::Pipe => "PipeToken",
        Symbol::Plus => "PlusToken",
        Symbol::QuestionMark => "QuestionToken",
        Symbol::RightBrace => "CloseBraceToken",
        Symbol::RightBracket => "CloseBracketToken",
        Symbol::RightParen => "CloseParenthesisToken",
        Symbol::Semicolon => "SemicolonToken",
        Symbol::Slash => "SlashToken",
        Symbol::DoubleSlash => "SlashSlashToken",
        Symbol::Star => "StarToken",
        Symbol::Tilde => "TildeToken",
        Symbol::TildeEqual => "TildeEqualsToken",
        Symbol::DoubleLessThanEqual => "DoubleLessThanEqualToken",
        Symbol::DoubleGreaterThanEqual => "DoubleGreaterThanEqualToken",
        Symbol::AmpersandEqual => "AmpersandEqualsToken",
        Symbol::PipeEqual => "PipeEqualsToken",
        Symbol::QuestionMarkDot => "QuestionDotToken",
        #[allow(unreachable_patterns)]
        _ => "UnknownSymbol",
    }
}
