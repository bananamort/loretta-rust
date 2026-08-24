// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager.ScopeAndVariableWalker (b767b4e)
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.ScopeAndVariableWalker.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use full_moon::ast;
use full_moon::tokenizer::{Symbol, TokenReference};

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::iscope::Scope;
use crate::scoping::ivariable::{IVariableInternal, SharedVariable};
use crate::scoping::node::Node;
use crate::scoping::scopekind::ScopeKind;
use crate::scoping::variablekind::VariableKind;
use crate::script::scopeandvariablemanager::basewalker::BaseWalker;

/// C# ScopeAndVariableWalker (ScopeAndVariableWalker.cs:8-...): builds the
/// scope tree + the node maps over the full_moon AST (the C#
/// LuaSyntaxWalker maps to a manual descent, following the C# override list
/// exactly).
pub struct ScopeAndVariableWalker {
    /// C# _rootScope (ScopeAndVariableWalker.cs:10).
    root_scope: Rc<RefCell<Scope>>,
    /// C# _variables (ScopeAndVariableWalker.cs:11).
    variables: HashMap<Node, SharedVariable>,
    /// C# _scopeStack (ScopeAndVariableWalker.cs:12).
    scope_stack: Vec<Rc<RefCell<Scope>>>,
    /// C# the BaseWalker._scopes.
    base: BaseWalker,
    /// C# the GotoLabelWalker (unified inline — see the Label arm).
    label_walker: crate::script::scopeandvariablemanager::gotolabelwalker::GotoLabelWalker,
    /// C# the GotoWalker (unified inline — see the Goto arm).
    goto_walker: crate::script::scopeandvariablemanager::gotowalker::GotoWalker,
    /// The identifier walk records (the node, the token's start byte, and
    /// the current scope) — used by the rename rewriter's token replacement
    /// and the minifier's rename table.
    pub identifier_positions: Vec<(Node, usize, Rc<RefCell<Scope>>)>,
    /// The location scopes of every node the variables carry (node ->
    /// (start byte, scope)) — the port's FindScope store. The C# FindScope
    /// walks a node's ancestors to the nearest scoped node; the port
    /// precomputes the enclosing scope when the node is created (the
    /// statement nodes the variables carry as declaration/write locations
    /// are not identifier records, so the minifier resolves their scopes
    /// here; the manager merges the store into the state's scopes map so
    /// the Script.FindScope ancestor walk can resolve the statement nodes).
    pub location_scopes: std::collections::HashMap<Node, (usize, Rc<RefCell<Scope>>)>,
}

impl ScopeAndVariableWalker {
    /// C# ScopeAndVariableWalker(Scope, IDictionary, IDictionary)
    /// (ScopeAndVariableWalker.cs:20-29). The shared node-id counter spans
    /// all trees' walks (Finding 5) — node identities stay unique across
    /// the accumulated state.
    pub fn new(
        root_scope: Rc<RefCell<Scope>>,
        variables: HashMap<Node, SharedVariable>,
        scopes: HashMap<Node, Rc<RefCell<Scope>>>,
        next_id: std::rc::Rc<std::cell::Cell<u64>>,
    ) -> Self {
        let mut walker = ScopeAndVariableWalker {
            root_scope: root_scope.clone(),
            variables,
            scope_stack: Vec::new(),
            base: BaseWalker::with_next_id(scopes.clone(), next_id.clone()),
            label_walker:
                crate::script::scopeandvariablemanager::gotolabelwalker::GotoLabelWalker::new(
                    scopes.clone(),
                    HashMap::new(),
                    next_id.clone(),
                ),
            goto_walker: crate::script::scopeandvariablemanager::gotowalker::GotoWalker::new(
                scopes,
                HashMap::new(),
                next_id,
            ),
            identifier_positions: Vec::new(),
            location_scopes: HashMap::new(),
        };
        walker.scope_stack.push(root_scope);
        walker
    }

    /// The current scope (C# Scope property, ScopeAndVariableWalker.cs:31-32).
    fn scope(&self) -> Rc<RefCell<Scope>> {
        self.scope_stack
            .last()
            .expect("the scope stack must not be empty")
            .clone()
    }

    /// C# CreateFileScope (ScopeAndVariableWalker.cs:34-40).
    fn create_file_scope(&mut self, node: Node) -> Rc<RefCell<Scope>> {
        let scope = Scope::new_file_scope(Some(node.clone()), Some(self.scope()));
        self.base.scopes.insert(node, scope.clone());
        self.scope().borrow_mut().add_child_scope(scope.clone());
        self.scope_stack.push(scope.clone());
        scope
    }

    /// C# CreateFunctionScope (ScopeAndVariableWalker.cs:42-48).
    fn create_function_scope(&mut self, node: Node) -> Rc<RefCell<Scope>> {
        let scope = Scope::new_function_scope(Some(node.clone()), Some(self.scope()));
        self.base.scopes.insert(node, scope.clone());
        self.scope().borrow_mut().add_child_scope(scope.clone());
        self.scope_stack.push(scope.clone());
        scope
    }

    /// C# CreateBlockScope (ScopeAndVariableWalker.cs:50-56).
    fn create_block_scope(&mut self, node: Node) -> Rc<RefCell<Scope>> {
        let scope = Scope::new(ScopeKind::Block, Some(node.clone()), Some(self.scope()));
        self.base.scopes.insert(node, scope.clone());
        self.scope().borrow_mut().add_child_scope(scope.clone());
        self.scope_stack.push(scope.clone());
        scope
    }

    /// C# PopScope (ScopeAndVariableWalker.cs:58-63).
    fn pop_scope(&mut self, scope: &Rc<RefCell<Scope>>) {
        let popped = self
            .scope_stack
            .pop()
            .expect("the scope stack must not be empty");
        debug_assert!(
            Rc::ptr_eq(&popped, scope),
            "the popped scope must be the pushed one"
        );
    }

    /// C# GetVariableOrCreateGlobal (ScopeAndVariableWalker.cs:65-69).
    fn get_variable_or_create_global(&mut self, name: &str) -> SharedVariable {
        let variable = match self.scope().borrow().try_get_variable(name) {
            Some(variable) => variable,
            None => Scope::create_variable_in(&self.root_scope, VariableKind::Global, name, None),
        };
        variable
    }

    /// C# CreateParameter(FunctionScope, ParameterSyntax) — the named /
    /// vararg parameters.
    fn create_parameter(
        &mut self,
        scope: &Rc<RefCell<Scope>>,
        parameter: &ast::Parameter,
    ) -> SharedVariable {
        match parameter {
            ast::Parameter::Name(token) => {
                let token_ref = token.clone();
                let name_text = token.token().to_string();
                let node = self.base.make_node("Parameter", name_text.clone());
                let variable = Scope::add_parameter_in(scope, &name_text, Some(node.clone()));
                self.record_identifier(node.clone(), &token_ref);
                self.variables.insert(node, variable.clone());
                variable
            }
            ast::Parameter::Ellipsis(_) => {
                // C# SyntaxKind.VarArgParameter => "..." (…:78): the vararg
                // is a real parameter named "..." — never a panic; there is
                // no name token to record for the rename rewriter.
                let node = self.base.make_node("Parameter", "...".to_string());
                let variable = Scope::add_parameter_in(scope, "...", Some(node.clone()));
                self.variables.insert(node, variable.clone());
                variable
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!("unsupported parameter kind"),
        }
    }

    /// Adds a read location + referencing scope for a referenced variable
    /// (the shared tail of the C# visits).
    fn add_read(&mut self, node: &Node, variable: &SharedVariable) {
        self.variables.insert(node.clone(), variable.clone());
        variable.borrow_mut().add_read_location(node.clone());
        variable.borrow_mut().add_referencing_scope(self.scope());
        Scope::add_referenced_variable_in(&self.scope(), variable);
    }

    /// Records an identifier token position for the rename rewriter (the
    /// current scope is the C# FindScope of the identifier).
    pub fn record_identifier(&mut self, node: Node, token: &TokenReference) {
        let pos = token.start_position().bytes();
        let scope = self.scope();
        self.location_scopes
            .insert(node.clone(), (pos, scope.clone()));
        self.identifier_positions.push((node, pos, scope));
    }

    /// Records the enclosing scope of a statement node (the C# FindScope of
    /// the statement — the scope the statement lives in, or its own block
    /// scope for the loop/function statements).
    fn record_statement_scope(&mut self, node: &Node, pos: usize, scope: &Rc<RefCell<Scope>>) {
        self.location_scopes
            .insert(node.clone(), (pos, scope.clone()));
    }

    /// C# VisitCompilationUnit (ScopeAndVariableWalker.cs:93-103).
    pub fn visit_ast(&mut self, full_ast: &ast::Ast) {
        let text = full_ast.to_string();
        let node = self.base.make_node("CompilationUnit", text);
        let scope = self.create_file_scope(node);
        self.visit_block(full_ast.nodes());
        self.pop_scope(&scope);
    }

    /// Visits every statement in a block (the C# rewriter's default descent).
    pub fn visit_block(&mut self, block: &ast::Block) {
        for stmt in block.stmts() {
            self.visit_stmt(stmt);
        }
        if let Some(last) = block.last_stmt() {
            match last {
                ast::LastStmt::Return(r) => {
                    for expr in r.returns() {
                        self.visit_expr(expr);
                    }
                }
                ast::LastStmt::Break(_) | ast::LastStmt::Continue(_) => {}
                #[allow(unreachable_patterns)]
                _ => {}
            }
        }
    }

    /// Dispatches a statement (the C# walker's visit overrides).
    pub fn visit_stmt(&mut self, stmt: &ast::Stmt) {
        match stmt {
            ast::Stmt::Assignment(assignment) => {
                // C# VisitAssignmentStatement: values first to avoid circular
                // references (ScopeAndVariableWalker.cs:139-166).
                for expr in assignment.expressions().iter() {
                    self.visit_expr(expr);
                }
                // The C# AddWriteLocation(node) shares ONE statement node for
                // all of the statement's variables.
                let stmt_node = self
                    .base
                    .make_node("AssignmentStatement", assignment.to_string());
                let stmt_pos = assignment
                    .variables()
                    .iter()
                    .next()
                    .and_then(|v| match v {
                        ast::Var::Name(t) => Some(t.token().start_position().bytes()),
                        _ => None,
                    })
                    .unwrap_or(0);
                self.record_statement_scope(&stmt_node, stmt_pos, &self.scope());
                for var in assignment.variables().iter() {
                    match var {
                        ast::Var::Name(token) => {
                            let name = token.token().to_string();
                            if name.trim().is_empty() {
                                continue;
                            }
                            let text = token.token().to_string();
                            let node = self.base.make_node("IdentifierName", text.clone());
                            self.record_identifier(node.clone(), token);
                            let variable = self.get_variable_or_create_global(&name);
                            self.variables.insert(node.clone(), variable.clone());
                            variable.borrow_mut().add_write_location(stmt_node.clone());
                            variable.borrow_mut().add_referencing_scope(self.scope());
                            Scope::add_referenced_variable_in(&self.scope(), &variable);
                        }
                        _ => self.visit_var(var),
                    }
                }
            }
            ast::Stmt::CompoundAssignment(ca) => {
                // C# VisitCompoundAssignmentStatement (…:168-193).
                self.visit_expr(ca.rhs());
                if let ast::Var::Name(token) = ca.lhs() {
                    let name = token.token().to_string();
                    if name.trim().is_empty() {
                        return;
                    }
                    let node = self.base.make_node("IdentifierName", name.clone());
                    self.record_identifier(node.clone(), token);
                    let variable = self.get_variable_or_create_global(&name);
                    self.variables.insert(node.clone(), variable.clone());
                    let stmt_node = self
                        .base
                        .make_node("CompoundAssignmentStatement", ca.to_string());
                    self.record_statement_scope(
                        &stmt_node,
                        token.token().start_position().bytes(),
                        &self.scope(),
                    );
                    variable.borrow_mut().add_write_location(stmt_node);
                    variable.borrow_mut().add_referencing_scope(self.scope());
                    Scope::add_referenced_variable_in(&self.scope(), &variable);
                } else {
                    self.visit_var(ca.lhs());
                }
            }
            ast::Stmt::NumericFor(nf) => {
                // C# VisitNumericForStatement (…:195-221). The C# creates ONE
                // statement node used for the block scope AND the iteration
                // variable's declaration.
                self.visit_expr(nf.start());
                self.visit_expr(nf.end());
                if let Some(step) = nf.step() {
                    self.visit_expr(step);
                }
                let node = self.base.make_node("NumericForStatement", nf.to_string());
                let scope = self.create_block_scope(node.clone());
                self.record_statement_scope(&node, nf.for_token().start_position().bytes(), &scope);
                let index_token = nf.index_variable().token().to_string();
                if !index_token.trim().is_empty() {
                    let variable = Scope::create_variable_in(
                        &scope,
                        VariableKind::Iteration,
                        &index_token,
                        Some(node.clone()),
                    );
                    let id_name = self.base.make_node("IdentifierName", index_token.clone());
                    self.record_identifier(id_name.clone(), nf.index_variable());
                    self.variables.insert(id_name, variable);
                }
                self.visit_block(nf.block());
                self.pop_scope(&scope);
            }
            ast::Stmt::GenericFor(gf) => {
                // C# VisitGenericForStatement (…:223-252). The C# creates ONE
                // statement node used for the block scope AND the iteration
                // variables' declarations.
                for expr in gf.expressions().iter() {
                    self.visit_expr(expr);
                }
                let node = self.base.make_node("GenericForStatement", gf.to_string());
                let scope = self.create_block_scope(node.clone());
                self.record_statement_scope(&node, gf.for_token().start_position().bytes(), &scope);
                for name in gf.names().iter() {
                    let identifier_name = name.token().to_string();
                    if identifier_name.trim().is_empty() {
                        continue;
                    }
                    let variable = Scope::create_variable_in(
                        &scope,
                        VariableKind::Iteration,
                        &identifier_name,
                        Some(node.clone()),
                    );
                    let name_node = self.base.make_node("IdentifierName", identifier_name);
                    self.record_identifier(name_node.clone(), name);
                    self.variables.insert(name_node, variable);
                }
                self.visit_block(gf.block());
                self.pop_scope(&scope);
            }
            ast::Stmt::While(w) => {
                // C# VisitWhileStatement (…:254-267).
                self.visit_expr(w.condition());
                let node = self.base.make_node("WhileStatement", w.to_string());
                let scope = self.create_block_scope(node);
                self.visit_block(w.block());
                self.pop_scope(&scope);
            }
            ast::Stmt::Repeat(r) => {
                // C# VisitRepeatUntilStatement (…:269-282).
                let node = self.base.make_node("RepeatUntilStatement", r.to_string());
                let scope = self.create_block_scope(node);
                self.visit_block(r.block());
                self.visit_expr(r.until());
                self.pop_scope(&scope);
            }
            ast::Stmt::If(if_stmt) => {
                // C# VisitIfStatement (…:284-325).
                self.visit_expr(if_stmt.condition());
                let node = self.base.make_node("IfStatement", if_stmt.to_string());
                let scope = self.create_block_scope(node);
                self.visit_block(if_stmt.block());
                self.pop_scope(&scope);
                if let Some(else_ifs) = if_stmt.else_if() {
                    for else_if in else_ifs {
                        self.visit_expr(else_if.condition());
                        let else_if_node = self.base.make_node("ElseIfClause", else_if.to_string());
                        let else_if_scope = self.create_block_scope(else_if_node);
                        self.visit_block(else_if.block());
                        self.pop_scope(&else_if_scope);
                    }
                }
                if let Some(else_block) = if_stmt.else_block() {
                    let else_node = self.base.make_node("ElseClause", else_block.to_string());
                    let else_scope = self.create_block_scope(else_node);
                    self.visit_block(else_block);
                    self.pop_scope(&else_scope);
                }
            }
            ast::Stmt::LocalAssignment(la) => {
                // C# VisitLocalVariableDeclarationStatement (…:327-345). The
                // C# creates ONE statement node shared by the declaration AND
                // the write location of every name.
                for expr in la.expressions().iter() {
                    self.visit_expr(expr);
                }
                let stmt_node = self
                    .base
                    .make_node("LocalVariableDeclarationStatement", la.to_string());
                self.record_statement_scope(
                    &stmt_node,
                    la.local_token().start_position().bytes(),
                    &self.scope(),
                );
                for name in la.names().iter() {
                    let identifier_name = name.token().to_string();
                    if identifier_name.trim().is_empty() {
                        continue;
                    }
                    let variable = Scope::create_variable_in(
                        &self.scope(),
                        VariableKind::Local,
                        &identifier_name,
                        Some(stmt_node.clone()),
                    );
                    let name_node = self.base.make_node("IdentifierName", identifier_name);
                    self.record_identifier(name_node.clone(), name);
                    self.variables.insert(name_node, variable.clone());
                    variable.borrow_mut().add_write_location(stmt_node.clone());
                    variable.borrow_mut().add_referencing_scope(self.scope());
                    Scope::add_referenced_variable_in(&self.scope(), &variable);
                }
            }
            ast::Stmt::LocalFunction(lf) => {
                // C# VisitLocalFunctionDeclarationStatement (…:347-374). The
                // C# creates ONE statement node shared by the declaration, the
                // write location, and the function scope; the name flows
                // through the rewriter's VisitSimpleFunctionName (renamed).
                // The node maps to the FUNCTION scope (the C#
                // CreateFunctionScope(node), ScopeAndVariableWalker.cs:326) —
                // no enclosing-scope record here, or the manager's
                // location_scopes merge would overwrite it (Finding 9).
                let node = self
                    .base
                    .make_node("LocalFunctionDeclarationStatement", lf.to_string());
                let name = lf.name().token().to_string();
                if !name.trim().is_empty() {
                    let variable = Scope::create_variable_in(
                        &self.scope(),
                        VariableKind::Local,
                        &name,
                        Some(node.clone()),
                    );
                    let name_node = self.base.make_node("IdentifierName", name);
                    self.record_identifier(name_node.clone(), lf.name());
                    self.variables.insert(name_node, variable.clone());
                    variable.borrow_mut().add_write_location(node.clone());
                    variable.borrow_mut().add_referencing_scope(self.scope());
                    Scope::add_referenced_variable_in(&self.scope(), &variable);
                }
                let scope = self.create_function_scope(node);
                for parameter in lf.body().parameters().iter() {
                    let parameter_variable = self.create_parameter(&scope, parameter);
                    self.variables.insert(
                        self.base.make_node("Parameter", parameter.to_string()),
                        parameter_variable,
                    );
                }
                self.visit_block(lf.body().block());
                self.pop_scope(&scope);
            }
            ast::Stmt::FunctionDeclaration(fd) => {
                // C# VisitFunctionDeclarationStatement (…:385-405).
                self.visit_function_name(fd.name());
                let node = self
                    .base
                    .make_node("FunctionDeclarationStatement", fd.to_string());
                let scope = self.create_function_scope(node);
                if fd.name().method_name().is_some() {
                    Scope::add_parameter_in(&scope, "self", None);
                }
                for parameter in fd.body().parameters().iter() {
                    let parameter_variable = self.create_parameter(&scope, parameter);
                    self.variables.insert(
                        self.base.make_node("Parameter", parameter.to_string()),
                        parameter_variable,
                    );
                }
                self.visit_block(fd.body().block());
                self.pop_scope(&scope);
            }
            ast::Stmt::Do(d) => {
                // C# VisitDoStatement (…:407-416).
                let node = self.base.make_node("DoStatement", d.to_string());
                let scope = self.create_block_scope(node);
                self.visit_block(d.block());
                self.pop_scope(&scope);
            }
            // The remaining statement kinds have no override: recurse into
            // their expressions via the default descent.
            ast::Stmt::FunctionCall(call) => {
                self.visit_function_call(call);
            }
            ast::Stmt::ConstAssignment(ca) => {
                for expr in ca.expressions().iter() {
                    self.visit_expr(expr);
                }
                for token in ca.names().iter() {
                    let name = token.token().to_string();
                    if name.trim().is_empty() {
                        continue;
                    }
                    let node = self.base.make_node("IdentifierName", name.clone());
                    let variable = self.get_variable_or_create_global(&name);
                    self.add_read(&node, &variable);
                }
            }
            ast::Stmt::ConstFunction(cf) => {
                self.visit_block(cf.body().block());
            }
            ast::Stmt::TypeFunction(tf) => {
                self.visit_type_function_body(tf.to_string(), tf.function_body());
            }
            ast::Stmt::ExportedTypeFunction(tf) => {
                self.visit_type_function_body(tf.to_string(), tf.type_function().function_body());
            }
            ast::Stmt::Label(label) => {
                // C# GotoLabelWalker.VisitGotoLabelStatement (the label
                // walker runs after the scope walk; the unified walk handles
                // it inline with the current scope).
                self.label_walker
                    .visit_goto_label_stmt(&self.scope(), label);
            }
            ast::Stmt::Goto(goto) => {
                // C# GotoWalker.VisitGotoStatement.
                self.goto_walker.visit_goto_stmt(
                    &self.scope(),
                    &goto.label_name().token().to_string(),
                    goto,
                );
            }
            ast::Stmt::TypeDeclaration(_) | ast::Stmt::ExportedTypeDeclaration(_) => {}
            _ => {}
        }
    }

    /// C# VisitTypeFunctionDeclarationStatement (…:407-...).
    fn visit_type_function_body(&mut self, text: String, body: &ast::FunctionBody) {
        let node = self
            .base
            .make_node("TypeFunctionDeclarationStatement", text);
        let scope = self.create_function_scope(node);
        for parameter in body.parameters().iter() {
            let parameter_variable = self.create_parameter(&scope, parameter);
            self.variables.insert(
                self.base.make_node("Parameter", parameter.to_string()),
                parameter_variable,
            );
        }
        self.visit_block(body.block());
        self.pop_scope(&scope);
    }

    /// C# VisitSimpleFunctionName (…:376-383): the referenced/declared
    /// function name. The C# SimpleFunctionNameSyntax nodes are: the plain
    /// function name, the FIRST part of a dotted name, and the prefix of a
    /// method name (the later dotted parts and the method name are not
    /// SimpleFunctionNames and create nothing — pinned by the reference
    /// probe). The C# write/read split depends on whether the parent is a
    /// FunctionDeclarationStatement — only the plain name has that parent.
    fn visit_function_name(&mut self, name: &ast::FunctionName) {
        let names: Vec<_> = name.names().iter().collect();
        let is_plain = names.len() == 1 && name.method_name().is_none();
        if let Some(first) = names.first() {
            let name_text = first.token().to_string();
            let node = self.base.make_node("SimpleFunctionName", name_text.clone());
            self.record_identifier(node.clone(), first);
            let variable = self.get_variable_or_create_global(&name_text);
            self.variables.insert(node.clone(), variable.clone());
            if is_plain {
                // The C# AddWriteLocation(node) shares the visited node.
                variable.borrow_mut().add_write_location(node);
            } else {
                variable.borrow_mut().add_read_location(node);
            }
            variable.borrow_mut().add_referencing_scope(self.scope());
            Scope::add_referenced_variable_in(&self.scope(), &variable);
        }
    }

    /// Visits an expression (the C# walker's expression overrides + the
    /// default descent).
    pub fn visit_expr(&mut self, expr: &ast::Expression) {
        match expr {
            ast::Expression::Var(ast::Var::Name(token)) => {
                // C# VisitIdentifierName (…:113-124).
                let name = token.token().to_string();
                if name.trim().is_empty() {
                    return;
                }
                let node = self.base.make_node("IdentifierName", name.clone());
                self.record_identifier(node.clone(), token);
                let variable = self.get_variable_or_create_global(&name);
                self.add_read(&node, &variable);
            }
            ast::Expression::Symbol(token) if token.is_symbol(Symbol::Ellipsis) => {
                // C# VisitVarArgExpression (…:100-111).
                let node = self.base.make_node("VarArgExpression", "...".to_string());
                let variable = self.get_variable_or_create_global("...");
                self.add_read(&node, &variable);
            }
            ast::Expression::Function(func) => {
                // C# VisitAnonymousFunctionExpression (…:105-121).
                let node = self
                    .base
                    .make_node("AnonymousFunctionExpression", func.to_string());
                let scope = self.create_function_scope(node);
                for parameter in func.body().parameters().iter() {
                    let parameter_variable = self.create_parameter(&scope, parameter);
                    self.variables.insert(
                        self.base.make_node("Parameter", parameter.to_string()),
                        parameter_variable,
                    );
                }
                self.visit_block(func.body().block());
                self.pop_scope(&scope);
            }
            ast::Expression::Var(var) => {
                self.visit_var(var);
            }
            _ => {
                self.visit_expr_children(expr);
            }
        }
    }

    /// The default descent into an expression's children (the C# rewriter's
    /// default visit).
    fn visit_expr_children(&mut self, expr: &ast::Expression) {
        match expr {
            ast::Expression::BinaryOperator { lhs, rhs, .. } => {
                self.visit_expr(lhs);
                self.visit_expr(rhs);
            }
            ast::Expression::Parentheses { expression, .. } => {
                self.visit_expr(expression);
            }
            ast::Expression::UnaryOperator { expression, .. } => {
                self.visit_expr(expression);
            }
            ast::Expression::FunctionCall(call) => {
                self.visit_function_call(call);
            }
            ast::Expression::TableConstructor(tc) => {
                for field in tc.fields().iter() {
                    match field {
                        ast::Field::NameKey { value, .. } => self.visit_expr(value),
                        ast::Field::ExpressionKey { key, value, .. } => {
                            self.visit_expr(key);
                            self.visit_expr(value);
                        }
                        ast::Field::NoKey(value) => self.visit_expr(value),
                        ast::Field::SetConstructor { .. } => {}
                        #[allow(unreachable_patterns)]
                        _ => {}
                    }
                }
            }
            ast::Expression::TypeAssertion { expression, .. } => {
                self.visit_expr(expression);
            }
            ast::Expression::IfExpression(if_expr) => {
                self.visit_expr(if_expr.condition());
                self.visit_expr(if_expr.if_expression());
                if let Some(else_ifs) = if_expr.else_if_expressions() {
                    for else_if in else_ifs {
                        self.visit_expr(else_if.condition());
                        self.visit_expr(else_if.expression());
                    }
                }
                self.visit_expr(if_expr.else_expression());
            }
            ast::Expression::InterpolatedString(interpolated) => {
                // C# default descent into an interpolated string's embedded
                // expressions (Finding 10): identifiers inside `{}` get
                // their read registrations and rename coverage.
                for expression in interpolated.expressions() {
                    self.visit_expr(expression);
                }
            }
            _ => {}
        }
    }

    /// Visits a var (the prefix + suffixes — the default descent).
    fn visit_var(&mut self, var: &ast::Var) {
        if let ast::Var::Expression(ve) = var {
            match ve.prefix() {
                ast::Prefix::Expression(e) => self.visit_expr(e),
                ast::Prefix::Name(token) => {
                    let name = token.token().to_string();
                    if name.trim().is_empty() {
                        return;
                    }
                    let node = self.base.make_node("IdentifierName", name.clone());
                    self.record_identifier(node.clone(), token);
                    let variable = self.get_variable_or_create_global(&name);
                    self.add_read(&node, &variable);
                }
                #[allow(unreachable_patterns)]
                _ => {}
            }
            for suffix in ve.suffixes() {
                if let ast::Suffix::Index(ast::Index::Brackets { expression, .. }) = suffix {
                    self.visit_expr(expression);
                }
            }
        }
    }

    /// Visits a function call (the prefix + args).
    fn visit_function_call(&mut self, call: &ast::FunctionCall) {
        match call.prefix() {
            ast::Prefix::Expression(e) => self.visit_expr(e),
            ast::Prefix::Name(token) => {
                let name = token.token().to_string();
                if name.trim().is_empty() {
                    return;
                }
                let node = self.base.make_node("IdentifierName", name.clone());
                self.record_identifier(node.clone(), token);
                let variable = self.get_variable_or_create_global(&name);
                self.add_read(&node, &variable);
            }
            #[allow(unreachable_patterns)]
            _ => {}
        }
        for suffix in call.suffixes() {
            match suffix {
                ast::Suffix::Index(ast::Index::Brackets { expression, .. }) => {
                    self.visit_expr(expression);
                }
                ast::Suffix::Call(c) => match c {
                    ast::Call::AnonymousCall(fa) => match fa {
                        ast::FunctionArgs::Parentheses { arguments, .. } => {
                            for arg in arguments.iter() {
                                self.visit_expr(arg);
                            }
                        }
                        ast::FunctionArgs::String(_) | ast::FunctionArgs::TableConstructor(_) => {}
                        #[allow(unreachable_patterns)]
                        _ => {}
                    },
                    ast::Call::MethodCall(mc) => match mc.args() {
                        ast::FunctionArgs::Parentheses { arguments, .. } => {
                            for arg in arguments.iter() {
                                self.visit_expr(arg);
                            }
                        }
                        ast::FunctionArgs::String(_) | ast::FunctionArgs::TableConstructor(_) => {}
                        #[allow(unreachable_patterns)]
                        _ => {}
                    },
                    #[allow(unreachable_patterns)]
                    _ => {}
                },
                _ => {}
            }
        }
    }

    /// The node -> variable map (the C# _variables dictionary).
    pub fn variables(&mut self) -> HashMap<Node, SharedVariable> {
        std::mem::take(&mut self.variables)
    }

    /// The node -> scope map (the BaseWalker._scopes).
    pub fn scopes(&mut self) -> HashMap<Node, Rc<RefCell<Scope>>> {
        std::mem::take(&mut self.base.scopes)
    }

    /// The node -> label map (the C# label walkers' maps, merged).
    pub fn labels(&mut self) -> HashMap<Node, Rc<RefCell<GotoLabel>>> {
        let mut labels = self.label_walker.labels();
        labels.extend(self.goto_walker.labels());
        labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoping::ifunctionscope::IFunctionScope;
    use crate::scoping::iscope::IScope;
    use crate::scoping::ivariable::IVariable;
    use crate::script::scopeandvariablemanager::manager::ScopeAndVariableManager;
    #[test]
    fn builds_the_scope_tree() {
        let mut manager = ScopeAndVariableManager::new(vec![
            "local a = 1\ndo\n\tlocal b = 2\nend\nprint(a)\n".to_string(),
        ]);
        let state = manager.get_lazy_state();
        let root = state.root_scope.borrow();
        assert_eq!(root.kind(), ScopeKind::Global);
        let globals: Vec<String> = root
            .declared_variables()
            .iter()
            .map(|v| v.borrow().name().to_string())
            .collect();
        // print is referenced -> the global "print" is created.
        assert!(globals.contains(&"print".to_string()));
        let root_contained = root.contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        assert_eq!(files.len(), 1);
        let file = files[0].borrow();
        assert_eq!(file.kind(), ScopeKind::File);
        let file_names: Vec<String> = file
            .declared_variables()
            .iter()
            .map(|v| v.borrow().name().to_string())
            .collect();
        assert!(file_names.contains(&"a".to_string()));
        assert!(file_names.contains(&"arg".to_string()));
        let file_contained = file.contained_scopes();
        let blocks: Vec<_> = file_contained.iter().collect();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].borrow().kind(), ScopeKind::Block);
    }

    #[test]
    fn vararg_parameters_are_real_parameters() {
        // C# CreateParameter maps VarArgParameter => "..." and adds it via
        // FunctionScope.AddParameter (ScopeAndVariableWalker.cs:71-82) — the
        // port must create the parameter instead of panicking (Finding 1).
        let mut manager = ScopeAndVariableManager::new(vec![
            "function f(...) end\nlocal g = function(...) end\n".to_string(),
        ]);
        let state = manager.get_lazy_state();
        let root = state.root_scope.borrow();
        let root_contained = root.contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        let file = files[0].borrow();
        let mut vararg_parameters = Vec::new();
        for scope in &file.contained_scopes() {
            let scope = scope.borrow();
            if scope.kind() == ScopeKind::Function {
                for parameter in scope.parameters() {
                    vararg_parameters.push(parameter.borrow().name().to_string());
                }
            }
        }
        assert_eq!(
            vararg_parameters,
            vec!["...".to_string(), "...".to_string()]
        );
    }

    #[test]
    fn multiple_trees_accumulate_state() {
        // Finding 5: the C# shared builder dictionaries accumulate across
        // trees (ScopeAndVariableManager.cs:35-47) — the port must not let
        // the last tree overwrite the maps, and node identities must stay
        // unique across trees.
        let mut single_manager = ScopeAndVariableManager::new(vec!["local a = 1\n".to_string()]);
        let single = single_manager.get_lazy_state();
        let mut manager = ScopeAndVariableManager::new(vec![
            "local a = 1\n".to_string(),
            "local b = 2\n".to_string(),
        ]);
        let state = manager.get_lazy_state();
        // Two trees hold exactly twice the single-tree entries (distinct
        // node ids per tree, no overwrite).
        assert_eq!(state.variables.len(), single.variables.len() * 2);
        assert_eq!(state.scopes.len(), single.scopes.len() * 2);
        assert_eq!(state.labels.len(), single.labels.len() * 2);
        // Both file scopes exist under the shared root, each declaring its
        // own variable.
        let root = state.root_scope.borrow();
        let root_contained = root.contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        assert_eq!(files.len(), 2);
        let file_a_names: Vec<String> = files[0]
            .borrow()
            .declared_variables()
            .iter()
            .map(|v| v.borrow().name().to_string())
            .collect();
        let file_b_names: Vec<String> = files[1]
            .borrow()
            .declared_variables()
            .iter()
            .map(|v| v.borrow().name().to_string())
            .collect();
        assert!(file_a_names.contains(&"a".to_string()));
        assert!(file_b_names.contains(&"b".to_string()));
        // The second tree's id base is the first tree's node count (every
        // node the walk creates lands in at least one map — the location
        // store included — so the distinct ids are contiguous 0..count) —
        // the rename rewriter's seed.
        let mut ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
        ids.extend(single.variables.keys().map(|n| n.id));
        ids.extend(single.scopes.keys().map(|n| n.id));
        ids.extend(single.labels.keys().map(|n| n.id));
        ids.extend(single.location_scopes.keys().map(|n| n.id));
        assert_eq!(state.tree_id_bases, vec![0, ids.len() as u64]);
    }

    #[test]
    fn forward_gotos_bind_to_the_label() {
        // Finding 6: `goto top` before `::top::` — the C# label walker
        // runs first so the goto binds to the label; the port's single
        // pass must bind the label statement to the forward goto's
        // same-scope placeholder instead of orphaning the jump.
        use crate::scoping::igotolabel::IGotoLabel;
        let mut manager =
            ScopeAndVariableManager::new(vec!["goto top\n::top::\ngoto top\n".to_string()]);
        let state = manager.get_lazy_state();
        let entries: Vec<_> = state.labels.values().collect();
        assert_eq!(entries.len(), 3); // two goto nodes + one label node
        for entry in &entries[1..] {
            assert!(
                std::rc::Rc::ptr_eq(entries[0], entry),
                "all three nodes must map to one label"
            );
        }
        let label = entries[0].borrow();
        assert_eq!(label.name(), "top");
        assert_eq!(
            label.jump_syntaxes().len(),
            2,
            "both gotos jump to the label"
        );
        // Finding 7: the label carries its statement's syntax — including
        // when the label statement bound to a forward-goto placeholder.
        assert!(
            label.label_syntax().is_some(),
            "the label must carry its statement's syntax"
        );
    }

    #[test]
    fn gotos_to_missing_labels_create_placeholder_labels() {
        // A goto with no matching label statement creates a placeholder
        // (the C# GetOrCreateLabel with a null syntax) — the label_syntax
        // stays None.
        use crate::scoping::igotolabel::IGotoLabel;
        let mut manager = ScopeAndVariableManager::new(vec!["goto nowhere\n".to_string()]);
        let state = manager.get_lazy_state();
        let entries: Vec<_> = state.labels.values().collect();
        assert_eq!(entries.len(), 1);
        let label = entries[0].borrow();
        assert_eq!(label.name(), "nowhere");
        assert_eq!(label.jump_syntaxes().len(), 1);
        assert!(label.label_syntax().is_none());
    }

    #[test]
    fn nested_block_labels_are_distinct() {
        // The C# label walker's CreateLabel targets only the current
        // scope (IScope.cs:226-231) — a label in a nested block is its
        // own label, never the outer one. The port's same-scope bind must
        // not ascend (Finding 6).
        let mut manager =
            ScopeAndVariableManager::new(vec!["do ::top:: do ::top:: end end\n".to_string()]);
        let state = manager.get_lazy_state();
        let entries: Vec<_> = state.labels.values().collect();
        assert_eq!(entries.len(), 2);
        assert!(
            !std::rc::Rc::ptr_eq(entries[0], entries[1]),
            "the nested label must be its own label"
        );
        // Finding 7: both labels carry their own statements' syntax.
        use crate::scoping::igotolabel::IGotoLabel;
        assert!(entries[0].borrow().label_syntax().is_some());
        assert!(entries[1].borrow().label_syntax().is_some());
    }

    #[test]
    fn function_scopes_capture_outer_variables() {
        // Finding 8: the C# FunctionScope.AddReferencedVariable override
        // (IFunctionScope.cs:55-62) captures variables referenced in a
        // function scope without being declared there, and the variable
        // records the capturing scope.
        use crate::scoping::ifunctionscope::IFunctionScope;
        use crate::scoping::ivariable::IVariable;
        let mut manager = ScopeAndVariableManager::new(vec![
            "local a = 1\nlocal f = function() print(a) end\n".to_string(),
        ]);
        let state = manager.get_lazy_state();
        let root = state.root_scope.borrow();
        let root_contained = root.contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        let file = files[0].borrow();
        let file_contained = file.contained_scopes();
        let function_scopes: Vec<_> = file_contained
            .iter()
            .filter(|s| s.borrow().kind() == ScopeKind::Function)
            .collect();
        assert_eq!(function_scopes.len(), 1);
        let captured: Vec<String> = function_scopes[0]
            .borrow()
            .captured_variables()
            .iter()
            .map(|v| v.borrow().name().to_string())
            .collect();
        assert!(
            captured.contains(&"a".to_string()),
            "a must be captured by the function scope: {captured:?}"
        );
        // The variable records the capturing scope.
        let a_variable = file
            .declared_variables()
            .iter()
            .find(|v| v.borrow().name() == "a")
            .expect("the a variable")
            .clone();
        let capturing = a_variable.borrow().capturing_scopes();
        assert!(
            capturing.iter().any(|s| Rc::ptr_eq(s, function_scopes[0])),
            "a must record the function scope as a capturing scope"
        );
    }

    #[test]
    fn local_function_statement_nodes_map_to_their_function_scope() {
        // Finding 9: the C# CreateFunctionScope(node) maps the local
        // function's statement node to the FUNCTION scope
        // (ScopeAndVariableWalker.cs:326) — the manager's location_scopes
        // merge must not overwrite it with the enclosing scope.
        let mut manager =
            ScopeAndVariableManager::new(vec!["local function f() end\n".to_string()]);
        let state = manager.get_lazy_state();
        let (node, scope) = state
            .scopes
            .iter()
            .find(|(node, _)| node.kind_name() == "LocalFunctionDeclarationStatement")
            .expect("the local-function statement node");
        assert_eq!(scope.borrow().kind(), ScopeKind::Function);
        let scope_ref = scope.borrow();
        let scope_node = scope_ref.node().expect("the function scope's node");
        assert_eq!(
            scope_node.id, node.id,
            "the function scope's node must be the statement node itself"
        );
    }

    #[test]
    fn referenced_variables_deduplicate() {
        // Finding 13: the C# _referencedVariables is an ISet (IScope.cs:124)
        // — a variable referenced twice is one entry.
        let mut manager =
            ScopeAndVariableManager::new(vec!["local a = 1\nprint(a)\nprint(a)\n".to_string()]);
        let state = manager.get_lazy_state();
        let root = state.root_scope.borrow();
        let root_contained = root.contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        let file = files[0].borrow();
        let referenced: Vec<String> = file
            .referenced_variables()
            .iter()
            .map(|v| v.borrow().name().to_string())
            .collect();
        // `a` is declared in this scope — the C# AddReferencedVariable
        // early-returns (IScope.cs:205) and never references it here.
        assert_eq!(
            referenced.iter().filter(|n| *n == "a").count(),
            0,
            "a is declared here, not referenced: {referenced:?}"
        );
        // `print` is referenced twice — the C# ISet keeps one entry
        // (Finding 13).
        assert_eq!(
            referenced.iter().filter(|n| *n == "print").count(),
            1,
            "print referenced twice must be one entry: {referenced:?}"
        );
    }

    #[test]
    fn interpolated_string_identifiers_are_registered() {
        // Finding 10: identifiers inside `{}` get read registrations (the
        // C# default descent visits the interpolated string's embedded
        // expressions; the port's manual descent must too).
        use crate::scoping::ivariable::IVariable;
        let mut manager = ScopeAndVariableManager::new(vec![
            "local name = \"x\"\nprint(`hello {name}`)\n".to_string(),
        ]);
        let state = manager.get_lazy_state();
        let root = state.root_scope.borrow();
        let root_contained = root.contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        let file = files[0].borrow();
        let declared = file.declared_variables();
        let name_variable = declared
            .iter()
            .find(|v| v.borrow().name() == "name")
            .expect("the name variable");
        let reads = name_variable.borrow().read_locations();
        let has_interpolated_read = reads
            .iter()
            .any(|n| n.kind_name() == "IdentifierName" && n.text == "name");
        assert!(
            has_interpolated_read,
            "the name identifier inside the backticks must be registered as a read"
        );
    }
}
