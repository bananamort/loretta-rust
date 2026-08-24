// Ported from Loretta.CodeAnalysis.Lua.IScope (b767b4e): IScope, IScopeInternal, Scope
// C# source: src/Compilers/Lua/Portable/Scoping/IScope.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use full_moon::ast::lua52;

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::ivariable::{IVariable, IVariableInternal, SharedVariable, Variable};
use crate::scoping::node::Node;
use crate::scoping::scopekind::ScopeKind;
use crate::scoping::variablekind::VariableKind;
use crate::utilities::stringutils::StringUtils;

/// The base interface for scopes (C# IScope).
pub trait IScope {
    /// The kind of scope.
    fn kind(&self) -> ScopeKind;

    /// The syntax node that originated this scope (not supported for the
    /// global scope).
    fn node(&self) -> Option<&Node>;

    /// The parent scope (if any).
    fn containing_scope(&self) -> Option<Rc<RefCell<Scope>>>;

    /// The variables declared within the scope (shadowing/redeclaration can
    /// produce multiple variables with the same name).
    fn declared_variables(&self) -> Vec<SharedVariable>;

    /// Variables that are directly referenced by this scope.
    fn referenced_variables(&self) -> Vec<SharedVariable>;

    /// The goto labels contained within this scope.
    fn goto_labels(&self) -> Vec<Rc<RefCell<GotoLabel>>>;

    /// The scopes directly contained within this scope.
    fn contained_scopes(&self) -> Vec<Rc<RefCell<Scope>>>;

    /// Attempts to find a variable with the given name (searching up to the
    /// provided scope kind).
    fn find_variable(&self, name: &str, kind: ScopeKind) -> Option<SharedVariable>;
}

/// The internal class Scope (C# IScope.cs:120-238). The C# internal
/// interface's mutators map to the struct methods; the ones needing the
/// shared handle take the `Rc` explicitly (the C# `this` reference maps to
/// the Rc in the port).
///
/// The C# FileScope/FunctionScope subclasses map to the optional data below
/// (the C# subclass instances are the only implementations, so the
/// observable surface is preserved).
pub struct Scope {
    kind: ScopeKind,
    node: Option<Node>,
    parent: Option<Rc<RefCell<Scope>>>,
    variables: HashMap<String, SharedVariable>,
    declared_variables: Vec<SharedVariable>,
    referenced_variables: Vec<SharedVariable>,
    labels: HashMap<String, Rc<RefCell<GotoLabel>>>,
    contained_scopes: Vec<Rc<RefCell<Scope>>>,
    /// C# FileScope data (IFileScope.cs:34-40).
    file_data: Option<crate::scoping::ifilescope::FileScopeData>,
    /// C# FunctionScope data (IFunctionScope.cs:29-44).
    function_data: Option<crate::scoping::ifunctionscope::FunctionScopeData>,
}

impl Scope {
    /// C# Scope(ScopeKind, SyntaxNode?, IScopeInternal?) (IScope.cs:128-137).
    pub fn new(
        kind: ScopeKind,
        node: Option<Node>,
        parent: Option<Rc<RefCell<Scope>>>,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Scope {
            kind,
            node,
            parent,
            variables: HashMap::new(),
            declared_variables: Vec::new(),
            referenced_variables: Vec::new(),
            labels: HashMap::new(),
            contained_scopes: Vec::new(),
            file_data: None,
            function_data: None,
        }))
    }

    /// C# FileScope's base constructor (IFileScope.cs:28-32) — the scope
    /// with its implicit arg/... variables.
    pub fn new_file_scope(
        node: Option<Node>,
        parent: Option<Rc<RefCell<Scope>>>,
    ) -> Rc<RefCell<Self>> {
        let scope = Self::new(ScopeKind::File, node, parent);
        let arg = Self::create_variable_in(&scope, VariableKind::Parameter, "arg", None);
        let vararg = Self::create_variable_in(&scope, VariableKind::Parameter, "...", None);
        scope.borrow_mut().file_data = Some(crate::scoping::ifilescope::FileScopeData {
            arg_variable: arg,
            vararg_parameter: vararg,
        });
        scope
    }

    /// C# FunctionScope's base constructor (IFunctionScope.cs:31-33) — the
    /// scope with its parameter/captured lists.
    pub fn new_function_scope(
        node: Option<Node>,
        parent: Option<Rc<RefCell<Scope>>>,
    ) -> Rc<RefCell<Self>> {
        let scope = Self::new(ScopeKind::Function, node, parent);
        scope.borrow_mut().function_data =
            Some(crate::scoping::ifunctionscope::FunctionScopeData::default());
        scope
    }

    /// C# FileScope data accessor.
    pub fn file_data(&self) -> Option<&crate::scoping::ifilescope::FileScopeData> {
        self.file_data.as_ref()
    }

    /// C# FunctionScope data accessor.
    pub fn function_data(&self) -> Option<&crate::scoping::ifunctionscope::FunctionScopeData> {
        self.function_data.as_ref()
    }

    /// C# FunctionScope data mutable accessor.
    pub fn function_data_mut(
        &mut self,
    ) -> Option<&mut crate::scoping::ifunctionscope::FunctionScopeData> {
        self.function_data.as_mut()
    }

    /// C# FindVariable (IScope.cs:164-174).
    pub fn find_variable(&self, name: &str, kind: ScopeKind) -> Option<SharedVariable> {
        // The C# null-check (IScope.cs:166) maps to the port's &str —
        // an empty name is NOT the null case: it falls through to the
        // identifier validation and panics like the C# ArgumentException
        // (IScope.cs:167 — Finding 44).
        if !StringUtils::is_identifier(name) {
            panic!("'{name}' must be a valid identifier.");
        }
        for variable in &self.declared_variables {
            if variable.borrow().name() == name {
                return Some(variable.clone());
            }
        }
        match &self.parent {
            Some(parent) if parent.borrow().kind() as u8 >= kind as u8 => {
                parent.borrow().find_variable(name, kind)
            }
            _ => None,
        }
    }

    /// C# TryGetVariable (IScope.cs:176-177).
    pub fn try_get_variable(&self, name: &str) -> Option<SharedVariable> {
        self.variables.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|p| p.borrow().try_get_variable(name))
        })
    }

    /// C# GetOrCreateVariable (IScope.cs:179-190) — through the shared
    /// handle (the C# `this`).
    pub fn get_or_create_variable_in(
        scope: &Rc<RefCell<Scope>>,
        kind: VariableKind,
        name: &str,
        declaration: Option<Node>,
    ) -> SharedVariable {
        debug_assert!(
            scope.borrow().kind == ScopeKind::Global || kind != VariableKind::Global,
            "global variables can only be created in the global scope"
        );
        debug_assert!(!name.is_empty(), "variable name must not be empty");

        let variable = {
            let existing = scope.borrow().try_get_variable(name);
            match existing {
                Some(variable) => variable,
                None => Self::create_variable_in(scope, kind, name, declaration),
            }
        };
        // C# _referencedVariables is an ISet (IScope.cs:124): adding the
        // same variable again is a no-op (Finding 13).
        let mut scope_ref = scope.borrow_mut();
        if !scope_ref
            .referenced_variables
            .iter()
            .any(|v| Rc::ptr_eq(v, &variable))
        {
            scope_ref.referenced_variables.push(variable.clone());
        }
        debug_assert!(variable.borrow().kind() == kind);
        variable
    }

    /// C# CreateVariable (IScope.cs:192-201) — through the shared handle.
    pub fn create_variable_in(
        scope: &Rc<RefCell<Scope>>,
        kind: VariableKind,
        name: &str,
        declaration: Option<Node>,
    ) -> SharedVariable {
        debug_assert!(
            scope.borrow().kind == ScopeKind::Global || kind != VariableKind::Global,
            "global variables can only be created in the global scope"
        );
        debug_assert!(!name.is_empty(), "variable name must not be empty");

        let variable = Rc::new(RefCell::new(Variable::new(
            kind,
            Rc::downgrade(scope),
            name.to_string(),
            declaration,
        )));
        let mut scope = scope.borrow_mut();
        scope.variables.insert(name.to_string(), variable.clone());
        scope.declared_variables.push(variable.clone());
        variable
    }

    /// C# AddReferencedVariable (IScope.cs:203-209) with the FunctionScope
    /// override (IFunctionScope.cs:55-62) — the shared handle is passed
    /// explicitly (the C# `this`; the override records the capturing scope
    /// on the variable). Variables referenced in a function scope without
    /// being declared there are captured (the C# captured-variable set).
    pub fn add_referenced_variable_in(scope: &Rc<RefCell<Scope>>, variable: &SharedVariable) {
        let mut scope_ref = scope.borrow_mut();
        if scope_ref
            .declared_variables
            .iter()
            .any(|v| Rc::ptr_eq(v, variable))
        {
            return;
        }
        // C# FunctionScope.AddReferencedVariable (IFunctionScope.cs:55-62):
        // the captured and referenced sets are HashSets — a variable
        // referenced twice is captured/referenced once (Finding 13).
        if let Some(data) = scope_ref.function_data.as_mut() {
            if !data
                .captured_variables
                .iter()
                .any(|v| Rc::ptr_eq(v, variable))
            {
                data.captured_variables.push(variable.clone());
            }
            variable.borrow_mut().add_capturing_scope(scope.clone());
        }
        if !scope_ref
            .referenced_variables
            .iter()
            .any(|v| Rc::ptr_eq(v, variable))
        {
            scope_ref.referenced_variables.push(variable.clone());
        }
        if let Some(parent) = &scope_ref.parent {
            Scope::add_referenced_variable_in(parent, variable);
        }
    }

    /// C# TryGetLabel (IScope.cs:211-213).
    pub fn try_get_label(&self, name: &str) -> Option<Rc<RefCell<GotoLabel>>> {
        self.labels.get(name).cloned().or_else(|| {
            if self.kind == ScopeKind::Block {
                self.parent
                    .as_ref()
                    .and_then(|p| p.borrow().try_get_label(name))
            } else {
                None
            }
        })
    }

    /// The scope's own label map lookup — no block ascent (the C# label
    /// walker's CreateLabel targets only the current scope; Finding 6).
    pub fn try_get_label_in_scope(&self, name: &str) -> Option<Rc<RefCell<GotoLabel>>> {
        self.labels.get(name).cloned()
    }

    /// C# GetOrCreateLabel (IScope.cs:215-224).
    pub fn get_or_create_label_in(
        scope: &Rc<RefCell<Scope>>,
        name: &str,
        label_syntax: Option<lua52::Label>,
    ) -> Rc<RefCell<GotoLabel>> {
        debug_assert!(!name.is_empty(), "label name must not be empty");
        let existing = scope.borrow().try_get_label(name);
        match existing {
            Some(label) => label,
            None => Self::create_label_in(scope, name, label_syntax),
        }
    }

    /// C# CreateLabel (IScope.cs:226-231).
    pub fn create_label_in(
        scope: &Rc<RefCell<Scope>>,
        name: &str,
        label_syntax: Option<lua52::Label>,
    ) -> Rc<RefCell<GotoLabel>> {
        let label = Rc::new(RefCell::new(GotoLabel::new(name.to_string(), label_syntax)));
        scope
            .borrow_mut()
            .labels
            .insert(name.to_string(), label.clone());
        label
    }

    /// C# AddChildScope (IScope.cs:233-237). The C# assert on the
    /// containing-scope identity is guaranteed by the builder (the port's Rc
    /// model can't compare against `self` without its own handle).
    pub fn add_child_scope(&mut self, scope: Rc<RefCell<Scope>>) {
        self.contained_scopes.push(scope);
    }

    /// C# IScope.ContainingScope — the parent scope.
    pub fn parent(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent.clone()
    }
}

impl IScope for Scope {
    fn kind(&self) -> ScopeKind {
        self.kind
    }

    fn node(&self) -> Option<&Node> {
        self.node.as_ref()
    }

    fn containing_scope(&self) -> Option<Rc<RefCell<Scope>>> {
        self.parent.clone()
    }

    fn declared_variables(&self) -> Vec<SharedVariable> {
        self.declared_variables.clone()
    }

    fn referenced_variables(&self) -> Vec<SharedVariable> {
        self.referenced_variables.clone()
    }

    fn goto_labels(&self) -> Vec<Rc<RefCell<GotoLabel>>> {
        self.labels.values().cloned().collect()
    }

    fn contained_scopes(&self) -> Vec<Rc<RefCell<Scope>>> {
        self.contained_scopes.clone()
    }

    fn find_variable(&self, name: &str, kind: ScopeKind) -> Option<SharedVariable> {
        self.find_variable(name, kind)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoping::ifilescope::IFileScope;
    use crate::scoping::ifunctionscope::IFunctionScope;

    #[test]
    fn file_scope_builds_implicit_variables() {
        let file = Scope::new_file_scope(None, None);
        assert_eq!(file.borrow().kind(), ScopeKind::File);
        assert_eq!(file.borrow().arg_variable().borrow().name(), "arg");
        assert_eq!(file.borrow().vararg_parameter().borrow().name(), "...");
        assert_eq!(
            file.borrow().arg_variable().borrow().kind(),
            VariableKind::Parameter
        );
    }

    #[test]
    fn function_scope_parameters_and_find() {
        let file = Scope::new_file_scope(None, None);
        let function = Scope::new_function_scope(None, Some(file.clone()));
        file.borrow_mut().add_child_scope(function.clone());
        Scope::add_parameter_in(&function, "x", None);
        assert_eq!(function.borrow().parameters().len(), 1);
        assert_eq!(function.borrow().parameters()[0].borrow().name(), "x");
        // find_variable walks up to the file scope (kind File covers
        // Function + Block).
        let found = function.borrow().find_variable("arg", ScopeKind::File);
        assert!(found.is_some());
        assert_eq!(found.unwrap().borrow().name(), "arg");
        // the found variable is accessible in the function scope.
        let arg = file.borrow().arg_variable();
        assert!(arg.borrow().can_be_accessed_in(&function));
    }

    #[test]
    fn get_or_create_variable_references_are_deduplicated() {
        // Finding 13: the C# _referencedVariables is an ISet (IScope.cs:124)
        // — GetOrCreateVariable adds the same variable only once.
        let global = Scope::new(ScopeKind::Global, None, None);
        let v1 = Scope::get_or_create_variable_in(&global, VariableKind::Global, "x", None);
        let v2 = Scope::get_or_create_variable_in(&global, VariableKind::Global, "x", None);
        assert!(Rc::ptr_eq(&v1, &v2));
        assert_eq!(global.borrow().referenced_variables().len(), 1);
    }
}
