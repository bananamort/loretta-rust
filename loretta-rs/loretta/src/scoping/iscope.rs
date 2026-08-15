// Ported from Loretta.CodeAnalysis.Lua.IScope (b767b4e): IScope, IScopeInternal, Scope
// C# source: src/Compilers/Lua/Portable/Scoping/IScope.cs

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::ivariable::{IVariable, Variable, VariableRef};
use crate::scoping::scopekind::ScopeKind;
use crate::scoping::variablekind::VariableKind;
use crate::utilities::stringutils::StringUtils;
use full_moon::ast::lua52::Label;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// A shared reference to a [`Scope`].
pub type ScopeRef = Rc<RefCell<Scope>>;

/// The base interface for scopes.
pub trait IScope {
    /// The kind of scope.
    fn kind(&self) -> ScopeKind;

    /// The offset of the syntax node that originated this scope.
    /// Not supported for the global scope.
    /// (C# `SyntaxNode?` — projected as the source offset.)
    fn node(&self) -> Option<usize>;

    /// The parent scope (if any).
    fn containing_scope(&self) -> Option<ScopeRef>;

    /// Contains the variables declared within the scope.
    /// As variables can be shadowed/redeclared, there may be multiple
    /// variables with the same name.
    fn declared_variables(&self) -> Vec<VariableRef>;

    /// Variables that are directly referenced by this scope.
    fn referenced_variables(&self) -> Vec<VariableRef>;

    /// The goto labels contained within this scope.
    fn goto_labels(&self) -> Vec<Rc<RefCell<GotoLabel>>>;

    /// Returns the scopes directly contained within this scope.
    fn contained_scopes(&self) -> Vec<ScopeRef>;

    /// Attempts to find a variable with the given name.
    ///
    /// The `kind` parameter searches for a scope of the provided kind or a
    /// more specific one: `Block` searches only blocks, `Function` searches
    /// functions and blocks, `File` searches files, functions and blocks and
    /// `Global` searches everything.
    fn find_variable(&self, name: &str, kind: ScopeKind) -> Option<VariableRef>;
}

/// Whether a scope kind is the provided kind or a more specific one
/// (C# `Parent.Kind >= kind` with Global=0 < File=1 < Function=2 < Block=3).
fn is_at_least(actual: ScopeKind, wanted: ScopeKind) -> bool {
    (actual as u8) >= (wanted as u8)
}

/// The C# `Scope` class. The `FileScope`/`FunctionScope` subclasses are
/// flattened into this struct: the file-scope implicit variables and the
/// function-scope parameters/captured-variables are optional fields, and
/// `AddReferencedVariable` captures variables for function scopes.
pub struct Scope {
    kind: ScopeKind,
    node: Option<usize>,
    parent: Option<ScopeRef>,
    variables: HashMap<String, VariableRef>,
    declared_variables: Vec<VariableRef>,
    referenced_variables: Vec<VariableRef>,
    labels: HashMap<String, Rc<RefCell<GotoLabel>>>,
    contained_scopes: Vec<ScopeRef>,
    parameters: Vec<VariableRef>,
    captured_variables: Vec<VariableRef>,
    arg_variable: Option<VariableRef>,
    var_arg_parameter: Option<VariableRef>,
}

impl Scope {
    /// C# `Scope(ScopeKind, SyntaxNode?, IScopeInternal?)` ctor.
    pub fn new(kind: ScopeKind, node: Option<usize>, parent: Option<ScopeRef>) -> ScopeRef {
        Rc::new(RefCell::new(Self {
            kind,
            node,
            parent,
            variables: HashMap::new(),
            declared_variables: Vec::new(),
            referenced_variables: Vec::new(),
            labels: HashMap::new(),
            contained_scopes: Vec::new(),
            parameters: Vec::new(),
            captured_variables: Vec::new(),
            arg_variable: None,
            var_arg_parameter: None,
        }))
    }

    /// C# `FileScope(SyntaxNode, IScopeInternal?)` ctor — creates the implicit
    /// `arg` and `...` parameters.
    pub fn new_file(node: Option<usize>, parent: Option<ScopeRef>) -> ScopeRef {
        let scope = Self::new(ScopeKind::File, node, parent);
        let arg = scope.borrow_mut().create_variable(
            &scope,
            VariableKind::Parameter,
            "arg".to_string(),
            None,
        );
        let var_arg = scope.borrow_mut().create_variable(
            &scope,
            VariableKind::Parameter,
            "...".to_string(),
            None,
        );
        scope.borrow_mut().arg_variable = Some(arg);
        scope.borrow_mut().var_arg_parameter = Some(var_arg);
        scope
    }

    /// C# `FunctionScope(SyntaxNode, IScopeInternal?)` ctor.
    pub fn new_function(node: Option<usize>, parent: Option<ScopeRef>) -> ScopeRef {
        Self::new(ScopeKind::Function, node, parent)
    }

    /// The parent scope (C# `Scope.Parent`).
    pub fn parent_ref(&self) -> Option<ScopeRef> {
        self.parent.clone()
    }

    /// C# `FileScope.ArgVariable`.
    pub fn arg_variable(&self) -> VariableRef {
        self.arg_variable
            .clone()
            .expect("file scope has an arg variable")
    }

    /// C# `FileScope.VarArgParameter`.
    pub fn var_arg_parameter(&self) -> VariableRef {
        self.var_arg_parameter
            .clone()
            .expect("file scope has a vararg parameter")
    }

    /// C# `FunctionScope.Parameters`.
    pub fn parameters(&self) -> &[VariableRef] {
        &self.parameters
    }

    /// C# `FunctionScope.CapturedVariables`.
    pub fn captured_variables(&self) -> &[VariableRef] {
        &self.captured_variables
    }

    /// C# `FunctionScope.AddParameter(string, SyntaxNode?)`.
    pub fn add_parameter(
        &mut self,
        self_ref: &ScopeRef,
        name: String,
        declaration: Option<usize>,
    ) -> VariableRef {
        let parameter = self.create_variable(self_ref, VariableKind::Parameter, name, declaration);
        self.parameters.push(parameter.clone());
        parameter
    }

    /// C# `FindVariable(string, ScopeKind)` — the C# `ArgumentNullException`
    /// is vacuous for `&str`; the invalid-identifier case throws the C#
    /// `ArgumentException` message.
    pub fn find_variable(&self, name: &str, kind: ScopeKind) -> Option<VariableRef> {
        if !StringUtils::is_identifier(name) {
            panic!("'name' must be a valid identifier.");
        }
        for variable in &self.declared_variables {
            if variable.borrow().name() == name {
                return Some(variable.clone());
            }
        }
        match &self.parent {
            Some(parent) if is_at_least(parent.borrow().kind, kind) => {
                parent.borrow().find_variable(name, kind)
            }
            _ => None,
        }
    }

    /// C# `TryGetVariable(string, out IVariableInternal?)`.
    pub fn try_get_variable(&self, name: &str) -> Option<VariableRef> {
        if let Some(variable) = self.variables.get(name) {
            return Some(variable.clone());
        }
        match &self.parent {
            Some(parent) => parent.borrow().try_get_variable(name),
            None => None,
        }
    }

    /// C# `GetOrCreateVariable(VariableKind, string, SyntaxNode?)`.
    pub fn get_or_create_variable(
        &mut self,
        self_ref: &ScopeRef,
        kind: VariableKind,
        name: String,
        declaration: Option<usize>,
    ) -> VariableRef {
        debug_assert!(self.kind == ScopeKind::Global || kind != VariableKind::Global);
        debug_assert!(!name.is_empty());

        let variable = match self.try_get_variable(&name) {
            Some(variable) => variable,
            None => self.create_variable(self_ref, kind, name, declaration),
        };

        if !self
            .referenced_variables
            .iter()
            .any(|v| Rc::ptr_eq(v, &variable))
        {
            self.referenced_variables.push(variable.clone());
        }
        debug_assert!(variable.borrow().kind() == kind);
        variable
    }

    /// C# `CreateVariable(VariableKind, string, SyntaxNode?)`.
    pub fn create_variable(
        &mut self,
        self_ref: &ScopeRef,
        kind: VariableKind,
        name: String,
        declaration: Option<usize>,
    ) -> VariableRef {
        debug_assert!(self.kind == ScopeKind::Global || kind != VariableKind::Global);
        debug_assert!(!name.is_empty());

        let variable = Rc::new(RefCell::new(Variable::new(
            kind,
            self_ref.clone(),
            name.clone(),
            declaration,
        )));
        self.variables.insert(name, variable.clone());
        self.declared_variables.push(variable.clone());
        variable
    }

    /// C# `AddReferencedVariable(IVariableInternal)` — for function scopes
    /// this also records the variable as captured (C# `FunctionScope`
    /// override).
    pub fn add_referenced_variable(self_ref: &ScopeRef, variable: &VariableRef) {
        let mut this = self_ref.borrow_mut();
        if this
            .declared_variables
            .iter()
            .any(|d| Rc::ptr_eq(d, variable))
        {
            return;
        }
        if this.kind == ScopeKind::Function {
            if !this
                .captured_variables
                .iter()
                .any(|c| Rc::ptr_eq(c, variable))
            {
                this.captured_variables.push(variable.clone());
            }
            variable.borrow_mut().add_capturing_scope(self_ref);
        }
        if !this
            .referenced_variables
            .iter()
            .any(|r| Rc::ptr_eq(r, variable))
        {
            this.referenced_variables.push(variable.clone());
        }
        let parent = this.parent.clone();
        if let Some(parent) = parent {
            Self::add_referenced_variable(&parent, variable);
        }
    }

    /// C# `TryGetLabel(string, out IGotoLabelInternal?)`.
    pub fn try_get_label(&self, name: &str) -> Option<Rc<RefCell<GotoLabel>>> {
        if let Some(label) = self.labels.get(name) {
            return Some(label.clone());
        }
        if self.kind == ScopeKind::Block {
            if let Some(parent) = &self.parent {
                if let Some(label) = parent.borrow().try_get_label(name) {
                    return Some(label);
                }
            }
        }
        None
    }

    /// C# `GetOrCreateLabel(string, GotoLabelStatementSyntax?)`.
    pub fn get_or_create_label(
        &mut self,
        name: String,
        label_syntax: Option<Label>,
    ) -> Rc<RefCell<GotoLabel>> {
        debug_assert!(!name.is_empty());
        debug_assert!(label_syntax.is_some());

        match self.try_get_label(&name) {
            Some(label) => label,
            None => self.create_label(name, label_syntax),
        }
    }

    /// C# `CreateLabel(string, GotoLabelStatementSyntax?)`.
    pub fn create_label(
        &mut self,
        name: String,
        label_syntax: Option<Label>,
    ) -> Rc<RefCell<GotoLabel>> {
        let label = Rc::new(RefCell::new(GotoLabel::new(name.clone(), label_syntax)));
        self.labels.insert(name, label.clone());
        label
    }

    /// C# `AddChildScope(IScopeInternal)`.
    pub fn add_child_scope(&mut self, self_ref: &ScopeRef, scope: &ScopeRef) {
        debug_assert!(match scope.borrow().parent_ref() {
            Some(parent) => Rc::ptr_eq(&parent, self_ref),
            None => false,
        });
        self.contained_scopes.push(scope.clone());
    }
}

impl IScope for Scope {
    fn kind(&self) -> ScopeKind {
        self.kind
    }

    fn node(&self) -> Option<usize> {
        self.node
    }

    fn containing_scope(&self) -> Option<ScopeRef> {
        self.parent.clone()
    }

    fn declared_variables(&self) -> Vec<VariableRef> {
        self.declared_variables.clone()
    }

    fn referenced_variables(&self) -> Vec<VariableRef> {
        self.referenced_variables.clone()
    }

    fn goto_labels(&self) -> Vec<Rc<RefCell<GotoLabel>>> {
        self.labels.values().cloned().collect()
    }

    fn contained_scopes(&self) -> Vec<ScopeRef> {
        self.contained_scopes.clone()
    }

    fn find_variable(&self, name: &str, kind: ScopeKind) -> Option<VariableRef> {
        Scope::find_variable(self, name, kind)
    }
}
