// Ported from Loretta.CodeAnalysis.Lua parser diagnostics (b767b4e): the
// parser-level diagnostics pass — the C# tree.GetDiagnostics() statements the
// lexer pass does not cover. Starts with the version-gated statement rules
// the differential corpus exercises.
// C# source: src/Compilers/Lua/Portable/Syntax/LuaParser.cs
// (ERR_NonFunctionCallBeingUsedAsStatement)

use crate::continuetype::ContinueType;
use crate::errors::errorcode::ErrorCode;
use crate::errors::lexerdiagnostics::LexerDiagnostic;
use crate::luasyntaxoptions::LuaSyntaxOptions;
use full_moon::ast::LastStmt;
use full_moon::visitors::Visitor;

/// The parser-level diagnostics for the AST under the given options.
///
/// The diagnostics share the lexer pass's shape (code, span, arguments,
/// severity flag) — the oracle renders both uniformly. Entries are ordered
/// by source position, matching the C# tree.GetDiagnostics() ordering.
pub fn parser_diagnostics(
    ast: &full_moon::ast::Ast,
    options: &LuaSyntaxOptions,
) -> Vec<LexerDiagnostic> {
    let mut collector = ContinueCollector {
        continue_is_identifier: options.continue_type == ContinueType::None,
        diagnostics: Vec::new(),
    };
    collector.visit_ast(ast);
    collector.diagnostics.sort_by_key(|d| d.start);
    collector.diagnostics
}

struct ContinueCollector {
    /// Under ContinueType::None the C# parser treats `continue` as an
    /// identifier expression statement (the only non-call expression
    /// statement the grammar can reach), which reports
    /// ERR_NonFunctionCallBeingUsedAsStatement per occurrence.
    continue_is_identifier: bool,
    diagnostics: Vec<LexerDiagnostic>,
}

impl Visitor for ContinueCollector {
    fn visit_last_stmt(&mut self, last_stmt: &LastStmt) {
        if let LastStmt::Continue(token) = last_stmt {
            if self.continue_is_identifier {
                let start = token.start_position().bytes();
                let width = token.end_position().bytes() - start;
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
