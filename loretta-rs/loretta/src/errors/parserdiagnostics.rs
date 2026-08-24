// Ported from Loretta.CodeAnalysis.Lua parser diagnostics (b767b4e): the
// parser-level diagnostics pass — the C# tree.GetDiagnostics() statements the
// lexer pass does not cover. Starts with the version-gated statement rules
// the differential corpus exercises.
// C# source: src/Compilers/Lua/Portable/Parser/LanguageParser.cs
// (ERR_NonFunctionCallBeingUsedAsStatement — Finding 46 corrected the
// citation from the nonexistent Syntax/LuaParser.cs)

use crate::continuetype::ContinueType;
use crate::errors::errorcode::ErrorCode;
use crate::errors::lexerdiagnostics::LexerDiagnostic;
use crate::luasyntaxoptions::LuaSyntaxOptions;
use full_moon::ast::{BinOp, Expression, LastStmt};
use full_moon::visitors::Visitor;

/// The parser-level diagnostics for the AST under the given options.
///
/// The diagnostics share the lexer pass's shape (code, span, arguments,
/// severity flag) — the oracle renders both uniformly. Entries are ordered
/// by source position, matching the C# tree.GetDiagnostics() ordering.
pub fn parser_diagnostics(
    ast: &full_moon::ast::Ast,
    options: &LuaSyntaxOptions,
    source: &str,
) -> Vec<LexerDiagnostic> {
    let mut collector = ContinueCollector {
        continue_is_identifier: options.continue_type == ContinueType::None,
        accept_bitwise_operators: options.accept_bitwise_operators,
        source,
        diagnostics: Vec::new(),
    };
    collector.visit_ast(ast);
    collector.diagnostics.sort_by_key(|d| d.start);
    collector.diagnostics
}

struct ContinueCollector<'a> {
    /// Under ContinueType::None the C# parser treats `continue` as an
    /// identifier expression statement (the only non-call expression
    /// statement the grammar can reach), which reports
    /// ERR_NonFunctionCallBeingUsedAsStatement per occurrence.
    continue_is_identifier: bool,
    /// C# LanguageParser.cs:908-912: the single '&'/'|' BINARY OPERATORS
    /// report the bitwise gating on the operator token (the lexer has no
    /// rule for them; '>>' gets no error at all — the parser combines two
    /// '>' tokens silently, LanguageParser.cs:840-845; Finding 22).
    accept_bitwise_operators: bool,
    /// The source text — the continue error spans the whole expression
    /// statement including the semicolon (the full_moon LastStmt covers
    /// only the keyword token — Finding 54).
    source: &'a str,
    diagnostics: Vec<LexerDiagnostic>,
}

impl Visitor for ContinueCollector<'_> {
    fn visit_expression(&mut self, expression: &Expression) {
        if let Expression::BinaryOperator { binop, .. } = expression {
            if !self.accept_bitwise_operators {
                match binop {
                    BinOp::Ampersand(token) | BinOp::Pipe(token) => {
                        let start = token.start_position().bytes();
                        let width = token.end_position().bytes() - start;
                        self.diagnostics.push(LexerDiagnostic {
                            code: ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion,
                            start,
                            width,
                            arguments: Vec::new(),
                            is_warning: false,
                        });
                    }
                    _ => {}
                }
            }
        }
        // No manual descent: the full_moon Visit impls walk the children
        // themselves (ast/visitors.rs — the visitor methods are pure
        // hooks).
    }

    fn visit_last_stmt(&mut self, last_stmt: &LastStmt) {
        if let LastStmt::Continue(token) = last_stmt {
            if self.continue_is_identifier {
                // The C# error is attached to the whole expression
                // statement — the keyword AND the semicolon
                // (LanguageParser.cs:215-220); the full_moon LastStmt
                // covers only the keyword token, so the trailing
                // semicolon is included from the source (Finding 54).
                let start = token.start_position().bytes();
                let mut end = token.end_position().bytes();
                if self.source.as_bytes().get(end) == Some(&b';') {
                    end += 1;
                }
                let width = end - start;
                self.diagnostics.push(LexerDiagnostic {
                    code: ErrorCode::ErrNonFunctionCallBeingUsedAsStatement,
                    start,
                    width,
                    arguments: Vec::new(),
                    is_warning: false,
                });
            }
        }
    }
}
