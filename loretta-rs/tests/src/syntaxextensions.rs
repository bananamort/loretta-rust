// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.SyntaxExtensions (b767b4e): SyntaxExtensions
// C# source: src/Compilers/Lua/Test/Portable/Parsing/SyntaxExtensions.cs

use full_moon::ast::AstResult;
use full_moon::tokenizer::{Token, TokenReference};
use full_moon::Error;

/// C# SyntaxExtensions (Parsing/SyntaxExtensions.cs:6-90): the internal test
/// helper over dropped green-node infra. The port's mapping:
///   - SyntaxNode / SyntaxToken / SyntaxTrivia -> the full_moon
///     TokenReference / Ast (the dropped InternalSyntax green nodes, Locked
///     Decision 1).
///   - SyntaxTriviaList -> the full_moon leading/trailing trivia token
///     sequences (TokenReference::leading_trivia / trailing_trivia,
///     tokenizer/structs.rs:746-753).
///   - DiagnosticInfo / SyntaxDiagnosticInfoList -> the full_moon parse Error
///     list (AstResult::errors, ast/parser_structs.rs:283). full_moon errors
///     carry no severity, so the C# error/warning split (the private
///     ErrorsOrWarnings collectors, lines 70-89) maps as: errors = the parse
///     errors, warnings = empty (the version-gating warnings land with the
///     audit finding B), errors_and_warnings = the parse errors.
pub struct SyntaxExtensions;

impl SyntaxExtensions {
    /// C# GetLeadingTrivia(SyntaxNode) (line 10-11) and
    /// GetLeadingTrivia(SyntaxToken) (line 29): the first token's / token's
    /// leading trivia.
    pub fn get_leading_trivia(token: &TokenReference) -> Vec<Token> {
        token.leading_trivia().cloned().collect()
    }

    /// C# GetTrailingTrivia(SyntaxNode) (line 13-14) and
    /// GetTrailingTrivia(SyntaxToken) (line 31): the last token's / token's
    /// trailing trivia.
    pub fn get_trailing_trivia(token: &TokenReference) -> Vec<Token> {
        token.trailing_trivia().cloned().collect()
    }

    /// C# Errors (lines 16-17, 33-34, 46-47, 59-60): the node's error
    /// diagnostics. The full_moon parse errors are the port's only parse
    /// diagnostics.
    pub fn errors(result: &AstResult) -> Vec<Error> {
        result.errors().to_vec()
    }

    /// C# Warnings (lines 19-20, 36-37, 49-50, 62-63): the node's warning
    /// diagnostics. full_moon errors carry no severity; the C# warning split
    /// (e.g. the version-gating hexfloat/char diagnostics) is the audit-finding
    /// B work in loretta/src/errors — empty here until that lands.
    pub fn warnings(_result: &AstResult) -> Vec<Error> {
        Vec::new()
    }

    /// C# ErrorsAndWarnings (lines 22-23, 39-40, 52-53, 65-66): all node
    /// diagnostics.
    pub fn errors_and_warnings(result: &AstResult) -> Vec<Error> {
        result.errors().to_vec()
    }
}
