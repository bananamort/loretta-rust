// Ported from Loretta.CodeAnalysis.Lua parser diagnostics (b767b4e): the
// parser-level diagnostics pass — the C# tree.GetDiagnostics() statements the
// lexer pass does not cover. Ported: the version-gated statement rules the
// differential corpus exercises (typed-Lua gates, goto/label gating with the
// C# reachability — see below), the single '&'/'|' bitwise gating, the
// compound-assignment LUA1013 gate for the plain operators, and the
// LUA0018 identifier-statement pair for a goto under a goto-disabled preset.
// NOT ported (C# LanguageParser.cs / LanguageParser.Types.cs): the general
// recovery diagnostics LUA1012 (:198), LUA1010/1011 (:973-976), LUA1001
// (EatToken, :333), LUA1014/1015/1017/1018 (Types.cs :116-123/:203-208/
// :463-468), LUA0019 (:731/740/775), LUA0015 (:1378), and the general
// LUA0018 non-call-expression-statement rule (:215-220) outside the continue
// and disabled-goto cases. The op gates this pass on full_moon::parse
// succeeding (differential/src/ops.rs:45-47), so parse-failed sources lose
// these parser diagnostics entirely while the C# recovers a tree and keeps
// going. C# source: src/Compilers/Lua/Portable/Parser/LanguageParser.cs
// (ERR_NonFunctionCallBeingUsedAsStatement — Finding 46 corrected the
// citation from the nonexistent Syntax/LuaParser.cs)

use crate::continuetype::ContinueType;
use crate::errors::errorcode::ErrorCode;
use crate::errors::lexerdiagnostics::LexerDiagnostic;
use crate::luasyntaxoptions::LuaSyntaxOptions;
use full_moon::ast::{
    BinOp, CompoundOp, Expression, FunctionBody, LastStmt, LocalAssignment, Stmt,
};
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
        accept_if_expressions: options.accept_if_expressions,
        continue_is_identifier: options.continue_type == ContinueType::None,
        accept_bitwise_operators: options.accept_bitwise_operators,
        accept_compound_assignment: options.accept_compound_assignment,
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
    /// C# ParseIfExpression (LanguageParser.cs:1329-1330): the whole
    /// if-expression reports ERR_IfExpressionsNotSupportedInLuaVersion
    /// when the option is off.
    accept_if_expressions: bool,
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
    /// C# ParseCompoundAssignment (LanguageParser.cs:787-788): the whole
    /// compound statement reports
    /// ERR_CompoundAssignmentNotSupportedInLuaVersion when the option is
    /// off — for the plain operators the C# lexer produces
    /// unconditionally (probed 'x += 1' @Lua51 -> [LUA1013]).
    accept_compound_assignment: bool,
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
    /// Pushes a diagnostic over the byte range [start, end). Node-level
    /// (the C# parser attaches these to nodes — the harness's tree pass
    /// only, never doubled).
    fn push(&mut self, code: ErrorCode, start: usize, end: usize) {
        self.diagnostics.push(LexerDiagnostic {
            code,
            start,
            width: end - start,
            arguments: Vec::new(),
            is_warning: false,
            node_level: true,
            sort_site: start,
        });
    }

    /// The C# statement nodes include TryMatchSemicolon()'s token — the
    /// semicolon INCLUDING its leading trivia (whitespace and comments) is
    /// part of the node's full span (probed on the packaged runtime:
    /// '::label:: --c\n;' -> LUA1019 [0..15), 'goto x --c\n;' -> the second
    /// LUA0018 [5..12), 'continue --c\n;' -> LUA0018 [0..14)). Returns the
    /// span end: past the trivia and the `;` when a semicolon follows the
    /// statement's last token, or the token end itself (the last token's
    /// trailing trivia is excluded from the C# full span). C-style comments
    /// cannot appear here: every preset where these arms fire has
    /// accept_c_comment_syntax = false (Lua51/LuaJIT/Luau).
    fn skip_trivia_to_optional_semicolon(&self, cursor: usize) -> usize {
        let bytes = self.source.as_bytes();
        let mut i = cursor;
        loop {
            while matches!(
                bytes.get(i),
                Some(b' ')
                    | Some(b'\t')
                    | Some(b'\r')
                    | Some(b'\n')
                    | Some(b'\x0B')
                    | Some(b'\x0C')
            ) {
                i += 1;
            }
            if bytes.get(i) == Some(&b'-') && bytes.get(i + 1) == Some(&b'-') {
                i += 2;
                if bytes.get(i) == Some(&b'[') {
                    // The long-comment form '--[[' '='* ... ']' '='* ']'
                    // (the C# TryScanLongString, Lexer.cs:911-985).
                    let mut eq = 0;
                    let mut j = i + 1;
                    while bytes.get(j) == Some(&b'=') {
                        eq += 1;
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'[') {
                        let mut k = j + 1;
                        loop {
                            match bytes.get(k) {
                                None => break,
                                Some(b']') => {
                                    let mut m = k + 1;
                                    let mut eq2 = 0;
                                    while eq2 < eq && bytes.get(m) == Some(&b'=') {
                                        eq2 += 1;
                                        m += 1;
                                    }
                                    if eq2 == eq && bytes.get(m) == Some(&b']') {
                                        i = m + 1;
                                        break;
                                    }
                                    k += 1;
                                }
                                _ => k += 1,
                            }
                        }
                        continue;
                    }
                }
                // A single-line comment runs to the end of the line.
                while !matches!(bytes.get(i), None | Some(b'\n') | Some(b'\r')) {
                    i += 1;
                }
                continue;
            }
            break;
        }
        if bytes.get(i) == Some(&b';') {
            i + 1
        } else {
            cursor
        }
    }
}

impl Visitor for ContinueCollector<'_> {
    fn visit_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::IfExpression(if_expr) if !self.accept_if_expressions => {
                // The C# ParseIfExpression gate (LanguageParser.cs:1329-1330):
                // the whole if-expression carries
                // ERR_IfExpressionsNotSupportedInLuaVersion when the option
                // is off (the C# parses the expression under every preset —
                // the dispatch at LanguageParser.cs:937 — and gates after).
                let start = if_expr
                    .start_position()
                    .expect("the if expression start")
                    .bytes();
                let end = if_expr
                    .end_position()
                    .expect("the if expression end")
                    .bytes();
                self.push(
                    ErrorCode::ErrIfExpressionsNotSupportedInLuaVersion,
                    start,
                    end,
                );
            }
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
                // The C# NEVER reaches ParseGotoStatement's LUA1019 gate
                // (LanguageParser.cs:608-609): when AcceptGoto is false the
                // keyword itself is demoted to an identifier
                // (Lexer.Identifiers.cs:9-14 via
                // SyntaxFacts.HasKeywordBeenDisabled, SyntaxFacts.cs:48-54),
                // so `goto x;` parses as TWO bare identifier expression
                // statements and the C# observably emits LUA0018 on each
                // (probed @Lua51/@Luau: [LUA0018, LUA0018]; the gate needs
                // !AcceptGoto while the keyword needs AcceptGoto —
                // unreachable). The port replicates the observable pair over
                // the goto token and the label-name token.
                let start = goto
                    .goto_token()
                    .start_position()
                    .expect("the goto start")
                    .bytes();
                let end = goto
                    .goto_token()
                    .end_position()
                    .expect("the goto end")
                    .bytes();
                self.push(
                    ErrorCode::ErrNonFunctionCallBeingUsedAsStatement,
                    start,
                    end,
                );
                let label_start = goto
                    .label_name()
                    .start_position()
                    .expect("the label name start")
                    .bytes();
                let label_end = goto
                    .label_name()
                    .end_position()
                    .expect("the label name end")
                    .bytes();
                // The C# second statement node includes TryMatchSemicolon()'s
                // token — the span covers the identifier through the `;`
                // (with its leading trivia): probed 'goto x;' -> [5..7),
                // 'goto x ;' -> [5..8), 'goto x --c\n;' -> [5..12).
                let end = self.skip_trivia_to_optional_semicolon(label_end);
                self.push(
                    ErrorCode::ErrNonFunctionCallBeingUsedAsStatement,
                    label_start,
                    end,
                );
            }
            // The C# label LUA1019 is reachable only when the lexer produces
            // ColonColonToken — AcceptGoto || AcceptTypedLua
            // (Lexer.cs:272-283) — while the gate needs !AcceptGoto, i.e.
            // !AcceptGoto && AcceptTypedLua (Luau). Under presets with
            // neither (Lua51), the C# never forms a label statement and its
            // output is the general parser-recovery diagnostics instead.
            Stmt::Label(label) if !self.accept_goto && self.accept_typed_lua => {
                // The C# error covers the whole GotoLabelStatement node
                // (LanguageParser.cs:631-648): `::label::` plus the optional
                // TryMatchSemicolon() token INCLUDING its leading trivia
                // (comments and whitespace), excluding the semicolon's
                // trailing trivia (probed on the packaged runtime:
                // '::label::;' -> [0..10), '::label:: ;' -> [0..11),
                // '::label:: --c\n;' -> [0..15),
                // '::label:: --[=[c]=]\n;' -> [0..21)).
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
                let end = self.skip_trivia_to_optional_semicolon(end);
                self.push(ErrorCode::ErrGotoNotSupportedInLuaVersion, start, end);
            }
            Stmt::CompoundAssignment(ca) if !self.accept_compound_assignment => {
                // The C# ParseCompoundAssignment gate (LanguageParser.cs:
                // 787-788): the whole compound statement carries LUA1013
                // when the option is off. Only the plain operators the C#
                // lexer produces unconditionally fire the clean gate —
                // probed 'x += 1' @Lua51 -> [LUA1013] over [0..6),
                // 'x += 1 ;' -> [0..8) (the node includes the semicolon).
                // '//=' (DoubleSlashEqual) never does: its token requires
                // AcceptFloorDivision AND AcceptCompoundAssignment in the
                // C# lexer, so the C# emits the recovery family instead
                // (probed @Lua51/@Lua53/FiveM); the cfxlua-only compound
                // ops cannot appear under the default version.
                let is_plain = matches!(
                    ca.compound_operator(),
                    CompoundOp::PlusEqual(_)
                        | CompoundOp::MinusEqual(_)
                        | CompoundOp::StarEqual(_)
                        | CompoundOp::SlashEqual(_)
                        | CompoundOp::CaretEqual(_)
                        | CompoundOp::PercentEqual(_)
                        | CompoundOp::TwoDotsEqual(_)
                );
                if is_plain {
                    let start = ca
                        .start_position()
                        .expect("the compound statement start")
                        .bytes();
                    let end = ca
                        .end_position()
                        .expect("the compound statement end")
                        .bytes();
                    let end = self.skip_trivia_to_optional_semicolon(end);
                    self.push(
                        ErrorCode::ErrCompoundAssignmentNotSupportedInLuaVersion,
                        start,
                        end,
                    );
                }
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
                // semicolon is included from the source (Finding 54;
                // the semicolon's leading trivia too: probed
                // 'continue;' -> [0..9), 'continue ;' -> [0..10),
                // 'continue --c\n;' -> [0..14)).
                let start = token.start_position().expect("the continue start").bytes();
                let end = token.end_position().expect("the continue end").bytes();
                let end = self.skip_trivia_to_optional_semicolon(end);
                self.push(
                    ErrorCode::ErrNonFunctionCallBeingUsedAsStatement,
                    start,
                    end,
                );
            }
        }
    }
}
