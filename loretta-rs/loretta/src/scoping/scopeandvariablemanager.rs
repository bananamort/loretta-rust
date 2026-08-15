// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager (b767b4e):
// ScopeAndVariableManager, State, BaseWalker, GotoLabelWalker, GotoWalker,
// ScopeAndVariableWalker
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager*.cs

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::igotolabel::IGotoLabelInternal;
use crate::scoping::iscope::{Scope, ScopeRef};
use crate::scoping::ivariable::VariableRef;
use crate::scoping::variablekind::VariableKind;
use full_moon::ast::lua52::{Goto, Label};
use full_moon::ast::{
    Ast, Call, CompoundAssignment, Do, FunctionArgs, FunctionBody, FunctionCall,
    FunctionDeclaration, GenericFor, If, Index, LocalAssignment, LocalFunction, NumericFor,
    Parameter, Prefix, Repeat, Suffix, Var, VarExpression, While,
};
use full_moon::node::Node;
use full_moon::tokenizer::{TokenReference, TokenType};
use full_moon::visitors::{VisitMut, VisitorMut};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// C# `ScopeAndVariableManager.State`.
pub struct State {
    /// The root (global) scope.
    pub root_scope: ScopeRef,
    /// C# `ImmutableDictionary<SyntaxNode, IVariable>` — keyed by the source
    /// offset of the node.
    pub variables: HashMap<usize, VariableRef>,
    /// C# `ImmutableDictionary<SyntaxNode, IScope>` — keyed by the source
    /// offset of the scope's origin node.
    pub scopes: HashMap<usize, ScopeRef>,
    /// C# `ImmutableDictionary<SyntaxNode, IGotoLabel>` — keyed by the source
    /// offset of the label/goto statement.
    pub labels: HashMap<usize, Rc<RefCell<GotoLabel>>>,
    /// The active scope for every non-trivia token offset (the port's
    /// `FindScope` projection: C# walks the syntax-tree ancestors).
    pub token_scopes: HashMap<usize, ScopeRef>,
}

impl State {
    fn new(root_scope: ScopeRef) -> Self {
        Self {
            root_scope,
            variables: HashMap::new(),
            scopes: HashMap::new(),
            labels: HashMap::new(),
            token_scopes: HashMap::new(),
        }
    }
}

/// C# `ScopeAndVariableManager` — `_trees` and the lazy `_state` (C#
/// `Interlocked.CompareExchange` — single-threaded `OnceCell` analog).
pub struct ScopeAndVariableManager {
    trees: Vec<Ast>,
    state: std::cell::OnceCell<State>,
}

impl ScopeAndVariableManager {
    /// C# ctor.
    pub fn new(trees: Vec<Ast>) -> Self {
        Self {
            trees,
            state: std::cell::OnceCell::new(),
        }
    }

    /// C# `GetLazyState()`.
    pub fn get_lazy_state(&self) -> &State {
        self.state
            .get_or_init(|| Self::calculate_state(&self.trees))
    }

    /// C# `CalculateState(ImmutableArray<SyntaxTree>)`.
    pub fn calculate_state(trees: &[Ast]) -> State {
        let root_scope = Scope::new(crate::scoping::scopekind::ScopeKind::Global, None, None);
        if trees.is_empty() {
            return State::new(root_scope);
        }

        let mut state = State::new(root_scope.clone());
        let mut gotos = Vec::new();

        // C# `foreach (var tree in trees) AddTree(...)`.
        for tree in trees {
            let mut walker = ScopeAndVariableWalker {
                root_scope: root_scope.clone(),
                state: &mut state,
                scope_stack: vec![root_scope.clone()],
                processed: HashSet::new(),
                gotos: &mut gotos,
            };
            walker.visit_ast(tree.clone());
        }

        // C# GotoWalker pass: labels were created by the label pass (inlined
        // into the main walk); gotos resolve after.
        for (label_name, scope, goto_stmt, goto_offset) in gotos {
            if label_name.trim().is_empty() {
                continue;
            }
            let label = scope.borrow_mut().get_or_create_label(label_name, None);
            label.borrow_mut().add_jump(goto_stmt);
            state.labels.insert(goto_offset, label);
        }

        state
    }
}

/// C# `BaseWalker.FindScope(SyntaxNode)` — projected: the active scope of a
/// token is recorded during the walk; the C# ancestor-walk is replaced by the
/// `token_scopes` map (see `ScopeAndVariableWalker`).
fn node_offset(node: &impl Node) -> usize {
    node.start_position().map(|p| p.bytes()).unwrap_or_default()
}

fn token_offset(token: &TokenReference) -> usize {
    token.token().start_position().bytes()
}

/// The C# node-kind names for scope origin nodes (C# `node.Kind().ToString()`).
fn node_kind_name(origin: ScopeOrigin) -> &'static str {
    match origin {
        ScopeOrigin::CompilationUnit => "CompilationUnit",
        ScopeOrigin::FunctionDeclarationStatement => "FunctionDeclarationStatement",
        ScopeOrigin::LocalFunctionDeclarationStatement => "LocalFunctionDeclarationStatement",
        ScopeOrigin::AnonymousFunctionExpression => "AnonymousFunctionExpression",
        ScopeOrigin::NumericForStatement => "NumericForStatement",
        ScopeOrigin::GenericForStatement => "GenericForStatement",
        ScopeOrigin::WhileStatement => "WhileStatement",
        ScopeOrigin::RepeatUntilStatement => "RepeatUntilStatement",
        ScopeOrigin::IfStatement => "IfStatement",
        ScopeOrigin::ElseIfClause => "ElseIfClause",
        ScopeOrigin::ElseClause => "ElseClause",
        ScopeOrigin::DoStatement => "DoStatement",
    }
}

#[derive(Clone, Copy)]
enum ScopeOrigin {
    CompilationUnit,
    FunctionDeclarationStatement,
    LocalFunctionDeclarationStatement,
    AnonymousFunctionExpression,
    NumericForStatement,
    GenericForStatement,
    WhileStatement,
    RepeatUntilStatement,
    IfStatement,
    ElseIfClause,
    ElseClause,
    DoStatement,
}

/// C# `ScopeAndVariableWalker` (+ the inline label walker) — the full_moon
/// `VisitorMut` traversal. Overridden nodes register their declarations and
/// uses manually (C# visit order) and mark their whole subtree as processed
/// so the derived recursion does not re-register.
struct ScopeAndVariableWalker<'a> {
    root_scope: ScopeRef,
    state: &'a mut State,
    scope_stack: Vec<ScopeRef>,
    processed: HashSet<usize>,
    gotos: &'a mut Vec<(String, ScopeRef, Goto, usize)>,
}

impl<'a> ScopeAndVariableWalker<'a> {
    /// The overrides are invoked again by the derived recursion after they
    /// return (the derived `visit_mut` always recurses into children); the
    /// first visit marks the whole node as processed, and the second visit is
    /// skipped entirely via this guard.
    fn already_processed(&self, node: &impl Node) -> bool {
        self.processed.contains(&node_offset(node))
    }

    fn current_scope(&self) -> ScopeRef {
        self.scope_stack
            .last()
            .cloned()
            .expect("scope stack is never empty")
    }

    /// C# `CreateFileScope`/`CreateFunctionScope`/`CreateBlockScope`.
    fn create_scope(&mut self, origin: ScopeOrigin, node: &impl Node) -> ScopeRef {
        let parent = self.current_scope();
        let scope = match origin {
            ScopeOrigin::CompilationUnit => {
                Scope::new_file(Some(node_offset(node)), Some(parent.clone()))
            }
            _ => Scope::new(
                match origin {
                    ScopeOrigin::FunctionDeclarationStatement
                    | ScopeOrigin::LocalFunctionDeclarationStatement
                    | ScopeOrigin::AnonymousFunctionExpression => {
                        crate::scoping::scopekind::ScopeKind::Function
                    }
                    _ => crate::scoping::scopekind::ScopeKind::Block,
                },
                Some(node_offset(node)),
                Some(parent.clone()),
            ),
        };
        self.state.scopes.insert(node_offset(node), scope.clone());
        {
            scope
                .borrow_mut()
                .set_node_kind(Some(node_kind_name(origin).to_string()));
        }
        parent.borrow_mut().add_child_scope(&parent, &scope);
        // Every token of the scope-origin node maps to this scope; nested
        // scope-creating nodes run later and overwrite their own subtrees
        // (the C# visit order assigns the innermost scope).
        for token in node.tokens() {
            self.state
                .token_scopes
                .insert(token_offset(token), scope.clone());
        }
        self.scope_stack.push(scope.clone());
        scope
    }

    fn pop_scope(&mut self, scope: &ScopeRef) {
        let popped = self.scope_stack.pop().expect("scope stack");
        debug_assert!(Rc::ptr_eq(&popped, scope));
    }

    /// C# `GetVariableOrCreateGlobal`.
    fn get_variable_or_create_global(&mut self, name: &str) -> VariableRef {
        let scope = self.current_scope();
        let found = scope.borrow().try_get_variable(name);
        match found {
            Some(variable) => variable,
            None => self.root_scope.borrow_mut().create_variable(
                &self.root_scope,
                VariableKind::Global,
                name.to_string(),
                None,
            ),
        }
    }

    /// C# `VisitIdentifierName`/`VisitVarArgExpression` use registration.
    /// Tokens already handled by an override (the derived recursion re-visits
    /// the whole subtree after the override, possibly in a popped scope) are
    /// skipped.
    fn register_use(&mut self, name: &str, offset: usize) {
        if name.trim().is_empty() || self.processed.contains(&offset) {
            return;
        }
        let scope = self.current_scope();
        let variable = self.get_variable_or_create_global(name);
        self.state.variables.insert(offset, variable.clone());
        variable.borrow_mut().add_read_location(offset);
        variable.borrow_mut().add_referencing_scope(&scope);
        Scope::add_referenced_variable(&scope, &variable);
    }

    /// C# `VisitSimpleFunctionName` for the single-part function-declaration
    /// name — parent is the declaration, so it is a write.
    fn register_function_name_part(&mut self, name: &str, offset: usize) {
        if self.processed.contains(&offset) {
            return;
        }
        let scope = self.current_scope();
        let variable = self.get_variable_or_create_global(name);
        self.state.variables.insert(offset, variable.clone());
        variable.borrow_mut().add_write_location(offset);
        variable.borrow_mut().add_referencing_scope(&scope);
        Scope::add_referenced_variable(&scope, &variable);
    }

    /// C# `VisitIdentifierName` for the first part of a member/method
    /// function name — a read.
    fn register_function_name_read(&mut self, name: &str, offset: usize) {
        if self.processed.contains(&offset) {
            return;
        }
        let scope = self.current_scope();
        let variable = self.get_variable_or_create_global(name);
        self.state.variables.insert(offset, variable.clone());
        variable.borrow_mut().add_read_location(offset);
        variable.borrow_mut().add_referencing_scope(&scope);
        Scope::add_referenced_variable(&scope, &variable);
    }

    /// C# `CreateParameter(FunctionScope, ParameterSyntax)`.
    fn create_parameter(
        &mut self,
        scope: &ScopeRef,
        name: &str,
        declaration: Option<usize>,
    ) -> VariableRef {
        scope
            .borrow_mut()
            .add_parameter(scope, name.to_string(), declaration)
    }

    /// C# `CreateParameter(FunctionScope, string)` (the `self` method
    /// parameter — no `_variables` entry).
    fn create_self_parameter(&mut self, scope: &ScopeRef) {
        self.create_parameter(scope, "self", None);
    }

    /// Visits the function-body parameters and block (C# `Visit(node.Body)`).
    fn visit_function_body(&mut self, body: &FunctionBody, scope: &ScopeRef) {
        for parameter in body.parameters().iter() {
            let (name, declaration) = match parameter {
                Parameter::Name(token) => (token.token().to_string(), Some(token_offset(token))),
                Parameter::Ellipsis(token) => ("...".to_string(), Some(token_offset(token))),
                _ => continue,
            };
            let variable = self.create_parameter(scope, &name, declaration);
            self.state
                .variables
                .insert(declaration.expect("declaration set above"), variable);
        }
        body.block().clone().visit_mut(self);
    }

    /// Marks every token of the node as processed so the derived recursion
    /// does not re-register uses or declarations.
    fn mark_processed(&mut self, node: &impl Node) {
        for token in node.tokens() {
            self.processed.insert(token_offset(token));
        }
    }

    /// Visits call arguments (C# traversal of function calls); the method
    /// names of `MethodCall`s are not variables.
    fn visit_call_args(&mut self, call: &Call) {
        match call {
            Call::AnonymousCall(args) => {
                self.visit_function_args(args);
            }
            Call::MethodCall(method_call) => {
                self.visit_function_args(method_call.args());
            }
            _ => {}
        }
    }

    fn visit_function_args(&mut self, args: &FunctionArgs) {
        match args {
            FunctionArgs::Parentheses { arguments, .. } => {
                for expression in arguments.iter() {
                    expression.clone().visit_mut(self);
                }
            }
            FunctionArgs::TableConstructor(table) => {
                table.clone().visit_mut(self);
            }
            _ => {}
        }
    }

    /// C# `VisitAssignmentStatement`/`VisitCompoundAssignmentStatement`
    /// variable handling: a bare identifier is a write; anything else is
    /// visited (its identifiers become reads).
    fn register_write_variable(&mut self, var: &Var) {
        match var {
            Var::Name(token) => {
                let name = token.token().to_string();
                if name.trim().is_empty() {
                    return;
                }
                let offset = token_offset(token);
                if self.processed.contains(&offset) {
                    return;
                }
                let scope = self.current_scope();
                let variable = self.get_variable_or_create_global(&name);
                self.state.variables.insert(offset, variable.clone());
                variable.borrow_mut().add_write_location(offset);
                variable.borrow_mut().add_referencing_scope(&scope);
                Scope::add_referenced_variable(&scope, &variable);
            }
            _ => {
                var.clone().visit_mut(self);
            }
        }
    }
}

impl VisitorMut for ScopeAndVariableWalker<'_> {
    /// C# `VisitCompilationUnit`.
    fn visit_ast(&mut self, ast: Ast) -> Ast {
        let scope = self.create_scope(ScopeOrigin::CompilationUnit, &ast);
        {
            let nodes = ast.nodes().clone();
            nodes.visit_mut(self);
        }
        self.pop_scope(&scope);
        // A scope-creating first statement shares the compilation unit's
        // offset 0 and would overwrite the file scope in the map; the file
        // scope wins the key (C# keys by node identity).
        self.state.scopes.insert(0, scope);
        ast
    }

    /// C# `VisitLocalVariableDeclarationStatement` — values first.
    fn visit_local_assignment(&mut self, node: LocalAssignment) -> LocalAssignment {
        if self.already_processed(&node) {
            return node;
        }
        for expression in node.expressions().iter() {
            expression.clone().visit_mut(self);
        }
        let scope = self.current_scope();
        let node_origin = node_offset(&node);
        for name in node.names().iter() {
            let name_text = name.token().to_string();
            if name_text.trim().is_empty() {
                continue;
            }
            let variable = scope.borrow_mut().create_variable(
                &scope,
                VariableKind::Local,
                name_text.clone(),
                Some(node_origin),
            );
            let offset = token_offset(name);
            self.state.variables.insert(offset, variable.clone());
            variable.borrow_mut().add_write_location(node_origin);
            variable.borrow_mut().add_referencing_scope(&scope);
            Scope::add_referenced_variable(&scope, &variable);
        }
        self.mark_processed(&node);
        node
    }

    /// C# `VisitAssignmentStatement` — values first.
    fn visit_assignment(&mut self, node: full_moon::ast::Assignment) -> full_moon::ast::Assignment {
        if self.already_processed(&node) {
            return node;
        }
        for expression in node.expressions().iter() {
            expression.clone().visit_mut(self);
        }
        for variable in node.variables().iter() {
            self.register_write_variable(variable);
        }
        self.mark_processed(&node);
        node
    }

    /// C# `VisitCompoundAssignmentStatement` — expression first.
    fn visit_compound_assignment(&mut self, node: CompoundAssignment) -> CompoundAssignment {
        if self.already_processed(&node) {
            return node;
        }
        node.rhs().clone().visit_mut(self);
        match node.lhs() {
            Var::Name(token) => {
                let name = token.token().to_string();
                if !name.trim().is_empty() {
                    let scope = self.current_scope();
                    let variable = self.get_variable_or_create_global(&name);
                    let offset = token_offset(token);
                    self.state.variables.insert(offset, variable.clone());
                    variable.borrow_mut().add_write_location(offset);
                    variable.borrow_mut().add_referencing_scope(&scope);
                    Scope::add_referenced_variable(&scope, &variable);
                }
            }
            variable => {
                variable.clone().visit_mut(self);
            }
        }
        self.mark_processed(&node);
        node
    }

    /// C# `VisitNumericForStatement`.
    fn visit_numeric_for(&mut self, node: NumericFor) -> NumericFor {
        if self.already_processed(&node) {
            return node;
        }
        node.start().clone().visit_mut(self);
        node.end().clone().visit_mut(self);
        if let Some(step) = node.step() {
            step.clone().visit_mut(self);
        }

        let scope = self.create_scope(ScopeOrigin::NumericForStatement, &node);
        {
            let name_text = node.index_variable().token().to_string();
            if !name_text.trim().is_empty() {
                let variable = scope.borrow_mut().create_variable(
                    &scope,
                    VariableKind::Iteration,
                    name_text.clone(),
                    Some(node_offset(&node)),
                );
                let offset = token_offset(node.index_variable());
                self.state.variables.insert(offset, variable.clone());
                node.block().clone().visit_mut(self);
            }
        }
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitGenericForStatement`.
    fn visit_generic_for(&mut self, node: GenericFor) -> GenericFor {
        if self.already_processed(&node) {
            return node;
        }
        for expression in node.expressions().iter() {
            expression.clone().visit_mut(self);
        }
        let scope = self.create_scope(ScopeOrigin::GenericForStatement, &node);
        {
            for name in node.names().iter() {
                let name_text = name.token().to_string();
                if name_text.trim().is_empty() {
                    continue;
                }
                let variable = scope.borrow_mut().create_variable(
                    &scope,
                    VariableKind::Iteration,
                    name_text.clone(),
                    Some(node_offset(&node)),
                );
                let offset = token_offset(name);
                self.state.variables.insert(offset, variable.clone());
            }
            node.block().clone().visit_mut(self);
        }
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitWhileStatement` — condition outside the block scope.
    fn visit_while(&mut self, node: While) -> While {
        if self.already_processed(&node) {
            return node;
        }
        node.condition().clone().visit_mut(self);
        let scope = self.create_scope(ScopeOrigin::WhileStatement, &node);
        node.block().clone().visit_mut(self);
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitRepeatUntilStatement` — body and condition inside the scope.
    fn visit_repeat(&mut self, node: Repeat) -> Repeat {
        if self.already_processed(&node) {
            return node;
        }
        let scope = self.create_scope(ScopeOrigin::RepeatUntilStatement, &node);
        node.block().clone().visit_mut(self);
        node.until().clone().visit_mut(self);
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitIfStatement`.
    fn visit_if(&mut self, node: If) -> If {
        if self.already_processed(&node) {
            return node;
        }
        node.condition().clone().visit_mut(self);
        let scope = self.create_scope(ScopeOrigin::IfStatement, &node);
        node.block().clone().visit_mut(self);
        self.pop_scope(&scope);

        if let Some(else_ifs) = node.else_if() {
            for else_if in else_ifs {
                else_if.condition().clone().visit_mut(self);
                let scope = self.create_scope(ScopeOrigin::ElseIfClause, else_if);
                else_if.block().clone().visit_mut(self);
                self.pop_scope(&scope);
            }
        }

        if let Some(else_block) = node.else_block() {
            let scope = self.create_scope(ScopeOrigin::ElseClause, else_block);
            else_block.clone().visit_mut(self);
            self.pop_scope(&scope);
        }
        self.mark_processed(&node);
        node
    }

    /// C# `VisitDoStatement`.
    fn visit_do(&mut self, node: Do) -> Do {
        if self.already_processed(&node) {
            return node;
        }
        let scope = self.create_scope(ScopeOrigin::DoStatement, &node);
        node.block().clone().visit_mut(self);
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitFunctionDeclarationStatement` + `VisitSimpleFunctionName`.
    /// For a simple name the part is a write (C# checks the parent is the
    /// declaration); for member/method names only the first part is visited
    /// (a read via `VisitIdentifierName` — the remaining parts are member-name
    /// tokens, not identifier nodes).
    fn visit_function_declaration(&mut self, node: FunctionDeclaration) -> FunctionDeclaration {
        if self.already_processed(&node) {
            return node;
        }
        let names: Vec<&TokenReference> = node.name().names().iter().collect();
        let first = names.first().map(|n| (*n).clone());
        match node.name().method_colon() {
            Some(_) => {
                if let Some(first) = first {
                    let name_text = first.token().to_string();
                    if !name_text.trim().is_empty() {
                        self.register_function_name_read(&name_text, token_offset(&first));
                    }
                }
            }
            None => {
                if names.len() == 1 {
                    if let Some(first) = first {
                        let name_text = first.token().to_string();
                        if !name_text.trim().is_empty() {
                            self.register_function_name_part(&name_text, token_offset(&first));
                        }
                    }
                } else if let Some(first) = first {
                    let name_text = first.token().to_string();
                    if !name_text.trim().is_empty() {
                        self.register_function_name_read(&name_text, token_offset(&first));
                    }
                }
            }
        }

        let scope = self.create_scope(ScopeOrigin::FunctionDeclarationStatement, &node);
        {
            // C# `if (node.Name.IsKind(MethodFunctionName)) CreateParameter(scope, "self");`
            if node.name().method_colon().is_some() {
                self.create_self_parameter(&scope);
            }
            self.visit_function_body(node.body(), &scope);
        }
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitLocalFunctionDeclarationStatement`.
    fn visit_local_function(&mut self, node: LocalFunction) -> LocalFunction {
        if self.already_processed(&node) {
            return node;
        }
        let name_text = node.name().token().to_string();
        if !name_text.trim().is_empty() {
            let scope = self.current_scope();
            let variable = scope.borrow_mut().create_variable(
                &scope,
                VariableKind::Local,
                name_text.clone(),
                Some(node_offset(&node)),
            );
            let offset = token_offset(node.name());
            self.state.variables.insert(offset, variable.clone());
            variable.borrow_mut().add_write_location(node_offset(&node));
            variable.borrow_mut().add_referencing_scope(&scope);
            Scope::add_referenced_variable(&scope, &variable);
        }

        let scope = self.create_scope(ScopeOrigin::LocalFunctionDeclarationStatement, &node);
        self.visit_function_body(node.body(), &scope);
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitAnonymousFunctionExpression`.
    fn visit_anonymous_function(
        &mut self,
        node: full_moon::ast::AnonymousFunction,
    ) -> full_moon::ast::AnonymousFunction {
        if self.already_processed(&node) {
            return node;
        }
        let scope = self.create_scope(ScopeOrigin::AnonymousFunctionExpression, &node);
        self.visit_function_body(node.body(), &scope);
        self.pop_scope(&scope);
        self.mark_processed(&node);
        node
    }

    /// C# `VisitIdentifierName` for var chains: the base is a read; field
    /// names (`.x`) and method names (`:m()`) are not variables; index
    /// expressions and call arguments are read contexts.
    fn visit_var(&mut self, var: Var) -> Var {
        if self.already_processed(&var) {
            return var;
        }
        match &var {
            Var::Name(token) => {
                self.register_use(&token.token().to_string(), token_offset(token));
            }
            Var::Expression(var_expr) => {
                self.visit_var_expression(var_expr);
            }
            _ => {}
        }
        self.mark_processed(&var);
        var
    }

    /// C# `VisitFunctionCall` — the prefix is a read context; method names
    /// are not variables.
    fn visit_function_call(&mut self, node: FunctionCall) -> FunctionCall {
        if self.already_processed(&node) {
            return node;
        }
        match node.prefix() {
            Prefix::Name(token) => {
                self.register_use(&token.token().to_string(), token_offset(token));
            }
            Prefix::Expression(expression) => {
                expression.clone().visit_mut(self);
            }
            _ => {}
        }
        for suffix in node.suffixes() {
            if let Suffix::Call(call) = suffix {
                self.visit_call_args(call);
            }
        }
        self.mark_processed(&node);
        node
    }

    /// C# `VisitGotoLabelStatement` — the label pass runs inline (before the
    /// goto pass, matching the C# walker order).
    fn visit_label(&mut self, node: Label) -> Label {
        if self.already_processed(&node) {
            return node;
        }
        let scope = self.current_scope();
        let name = node.name().to_string();
        if !name.trim().is_empty() {
            let label = scope
                .borrow_mut()
                .create_label(name.clone(), Some(node.clone()));
            let offset = node_offset(&node);
            self.state.labels.insert(offset, label);
        }
        self.mark_processed(&node);
        node
    }

    /// C# `VisitGotoStatement` — deferred to the goto pass.
    fn visit_goto(&mut self, node: Goto) -> Goto {
        if self.already_processed(&node) {
            return node;
        }
        let label_name = node.label_name().to_string();
        let offset = node_offset(&node);
        self.gotos
            .push((label_name, self.current_scope(), node.clone(), offset));
        self.mark_processed(&node);
        node
    }

    /// The derived recursion visits every token; uses are registered here for
    /// tokens that were not handled by the overrides above.
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        let offset = token_offset(&token_ref);
        if self.processed.contains(&offset) {
            // The override already recorded the C#-order scope assignments
            // for the whole subtree; the derived re-visit must not overwrite.
            return token_ref;
        }
        let scope = self.current_scope();
        self.state.token_scopes.insert(offset, scope);
        if let TokenType::Identifier { identifier } = token_ref.token().token_type() {
            self.register_use(&identifier.to_string(), offset);
        }
        token_ref
    }
}

impl ScopeAndVariableWalker<'_> {
    /// The C# `Visit(node.Expression)`/suffix traversal for a var chain.
    fn visit_var_expression(&mut self, var_expr: &VarExpression) {
        match var_expr.prefix() {
            Prefix::Name(token) => {
                self.register_use(&token.token().to_string(), token_offset(token));
            }
            Prefix::Expression(expression) => {
                expression.clone().visit_mut(self);
            }
            _ => {}
        }
        for suffix in var_expr.suffixes() {
            match suffix {
                Suffix::Index(Index::Brackets { expression, .. }) => {
                    expression.clone().visit_mut(self);
                }
                Suffix::Index(Index::Dot { .. }) => {}
                Suffix::Call(call) => self.visit_call_args(call),
                _ => {}
            }
        }
    }
}
