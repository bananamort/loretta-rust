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
use full_moon::ast::{BinOp, Expression, FunctionBody, LastStmt, LocalAssignment, Stmt};
use full_moon::node::Node;
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
        accept_goto: options.accept_goto,
        accept_typed_lua: options.accept_typed_lua,
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
    /// C# ParseGotoStatement / ParseGotoLabelStatement
    /// (LanguageParser.cs:608-609, 644-645): the goto statement and the
    /// label report ERR_GotoNotSupportedInLuaVersion on the whole node
    /// when the option is off (Finding 56).
    accept_goto: bool,
    /// C# ParseTypeDeclaration / ParseTypeFunctionDeclaration /
    /// TryParseTypeCast (LanguageParser.cs:280-285, 317-322, 953-954)
    /// plus the type bindings on the locals, function parameters, type
    /// parameters and return types: the typed structures report
    /// ERR_TypedLuaNotSupportedInLuaVersion on their whole node when the
    /// option is off (Finding 56).
    accept_typed_lua: bool,
    /// The source text — the continue error spans the whole expression
    /// statement including the semicolon (the full_moon LastStmt covers
    /// only the keyword token — Finding 54).
    source: &'a str,
    diagnostics: Vec<LexerDiagnostic>,
}

impl<'a> ContinueCollector<'a> {
    /// Pushes a diagnostic over the byte range [start, end).
    fn push(&mut self, code: ErrorCode, start: usize, end: usize) {
        self.diagnostics.push(LexerDiagnostic {
            code,
            start,
            width: end - start,
            arguments: Vec::new(),
            is_warning: false,
        });
    }
}

impl Visitor for ContinueCollector<'_> {
    fn visit_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::BinaryOperator { binop, .. } => {
                if !self.accept_bitwise_operators {
                    match binop {
                        BinOp::Ampersand(token) | BinOp::Pipe(token) => {
                            let start = token.start_position().expect("the operator start").bytes();
                            let end = token.end_position().expect("the operator end").bytes();
                            self.push(
                                ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion,
                                start,
                                end,
                            );
                        }
                        _ => {}
                    }
                }
            }
            Expression::TypeAssertion {
                expression,
                type_assertion,
                ..
            } if !self.accept_typed_lua => {
                // The C# cast gate (LanguageParser.cs:953-954) covers the
                // whole cast — the asserted expression through the cast
                // type ("x :: table").
                let start = expression
                    .as_ref()
                    .start_position()
                    .expect("the cast start")
                    .bytes();
                let end = type_assertion.end_position().expect("the cast end").bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
            _ => {}
        }
        // No manual descent: the full_moon Visit impls walk the children
        // themselves (ast/visitors.rs — the visitor methods are pure
        // hooks).
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Goto(goto) if !self.accept_goto => {
                // The C# error covers the whole `goto label`
                // (LanguageParser.cs:608-609).
                let start = goto
                    .goto_token()
                    .start_position()
                    .expect("the goto start")
                    .bytes();
                let end = goto
                    .label_name()
                    .end_position()
                    .expect("the goto end")
                    .bytes();
                self.push(ErrorCode::ErrGotoNotSupportedInLuaVersion, start, end);
            }
            Stmt::Label(label) if !self.accept_goto => {
                // The C# error covers the whole `::label::`
                // (LanguageParser.cs:644-645).
                let start = label
                    .left_colons()
                    .start_position()
                    .expect("the label start")
                    .bytes();
                let end = label
                    .right_colons()
                    .end_position()
                    .expect("the label end")
                    .bytes();
                self.push(ErrorCode::ErrGotoNotSupportedInLuaVersion, start, end);
            }
            Stmt::TypeDeclaration(decl) if !self.accept_typed_lua => {
                // The whole `type T = T` (LanguageParser.cs:280-285).
                let start = decl
                    .type_token()
                    .start_position()
                    .expect("the type declaration start")
                    .bytes();
                let end = decl
                    .end_position()
                    .expect("the type declaration end")
                    .bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
            Stmt::ExportedTypeDeclaration(decl) if !self.accept_typed_lua => {
                // The whole `export type T = T`.
                let start = decl
                    .start_position()
                    .expect("the exported declaration start")
                    .bytes();
                let end = decl
                    .end_position()
                    .expect("the exported declaration end")
                    .bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
            Stmt::TypeFunction(func) if !self.accept_typed_lua => {
                // The whole `type function ...` (LanguageParser.cs:317-322).
                let start = func
                    .start_position()
                    .expect("the type function start")
                    .bytes();
                let end = func.end_position().expect("the type function end").bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
            Stmt::ExportedTypeFunction(func) if !self.accept_typed_lua => {
                // The whole `export type function ...`.
                let start = func
                    .start_position()
                    .expect("the type function start")
                    .bytes();
                let end = func.end_position().expect("the type function end").bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
            _ => {}
        }
        // No manual descent: the full_moon Visit impls walk the children
        // themselves.
    }

    fn visit_local_assignment(&mut self, local_assignment: &LocalAssignment) {
        if !self.accept_typed_lua {
            for specifier in local_assignment.type_specifiers().flatten() {
                // The `: T` binding on a local name.
                let start = specifier
                    .start_position()
                    .expect("the specifier start")
                    .bytes();
                let end = specifier.end_position().expect("the specifier end").bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
        }
        // No manual descent.
    }

    fn visit_function_body(&mut self, function_body: &FunctionBody) {
        if !self.accept_typed_lua {
            if let Some(generics) = function_body.generics() {
                // The `<T>` type parameters — the end is the closing `>`
                // of the arrows (the GenericDeclaration node's own end
                // excludes it).
                let start = generics
                    .start_position()
                    .expect("the generics start")
                    .bytes();
                let (_, close) = generics.arrows().tokens();
                let end = close.end_position().expect("the generics end").bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
            for specifier in function_body.type_specifiers().flatten() {
                // The per-parameter `: T` bindings.
                let start = specifier
                    .start_position()
                    .expect("the parameter specifier start")
                    .bytes();
                let end = specifier
                    .end_position()
                    .expect("the parameter specifier end")
                    .bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
            if let Some(return_type) = function_body.return_type() {
                // The `: T` return binding.
                let start = return_type
                    .start_position()
                    .expect("the return type start")
                    .bytes();
                let end = return_type
                    .end_position()
                    .expect("the return type end")
                    .bytes();
                self.push(ErrorCode::ErrTypedLuaNotSupportedInLuaVersion, start, end);
            }
        }
        // No manual descent.
    }

    fn visit_last_stmt(&mut self, last_stmt: &LastStmt) {
        if let LastStmt::Continue(token) = last_stmt {
            if self.continue_is_identifier {
                // The C# error is attached to the whole expression
                // statement — the keyword AND the semicolon
                // (LanguageParser.cs:215-220); the full_moon LastStmt
                // covers only the keyword token, so the trailing
                // semicolon is included from the source (Finding 54).
                let start = token.start_position().expect("the continue start").bytes();
                let mut end = token.end_position().expect("the continue end").bytes();
                if self.source.as_bytes().get(end) == Some(&b';') {
                    end += 1;
                }
                self.push(
                    ErrorCode::ErrNonFunctionCallBeingUsedAsStatement,
                    start,
                    end,
                );
            }
        }
    }
}
