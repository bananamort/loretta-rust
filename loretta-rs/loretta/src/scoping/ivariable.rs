// Ported from Loretta.CodeAnalysis.Lua.IVariable (b767b4e): IVariable, IVariableInternal, Variable
// C# source: src/Compilers/Lua/Portable/Scoping/IVariable.cs

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::scoping::iscope::Scope;
use crate::scoping::node::Node;
use crate::scoping::variablekind::VariableKind;

/// The base interface for variables (C# IVariable).
pub trait IVariable {
    /// The kind of this variable.
    fn kind(&self) -> VariableKind;

    /// The containing scope.
    fn containing_scope(&self) -> Rc<RefCell<Scope>>;

    /// The variable's name.
    fn name(&self) -> &str;

    /// The node where this variable is declared (null if it is a global or
    /// implicit variable).
    fn declaration(&self) -> Option<&Node>;

    /// The scopes that reference this variable.
    fn referencing_scopes(&self) -> Vec<Rc<RefCell<Scope>>>;

    /// All scopes that capture this variable as an upvalue.
    fn capturing_scopes(&self) -> Vec<Rc<RefCell<Scope>>>;

    /// All locations this variable is read from.
    fn read_locations(&self) -> Vec<Node>;

    /// All locations this variable is written to.
    fn write_locations(&self) -> Vec<Node>;

    /// Returns whether this variable can be accessed in the provided scope.
    fn can_be_accessed_in(&self, scope: &Rc<RefCell<Scope>>) -> bool;
}

/// The internal interface for variables (C# IVariableInternal).
pub trait IVariableInternal: IVariable {
    fn add_referencing_scope(&mut self, scope: Rc<RefCell<Scope>>);
    fn add_capturing_scope(&mut self, scope: Rc<RefCell<Scope>>);
    fn add_read_location(&mut self, node: Node);
    fn add_write_location(&mut self, node: Node);
}

/// A shared variable (the C# reference identity maps to Rc<RefCell<..>>).
pub type SharedVariable = Rc<RefCell<Variable>>;

/// The internal class Variable (C# IVariable.cs:71-136).
pub struct Variable {
    kind: VariableKind,
    containing_scope: Weak<RefCell<Scope>>,
    name: String,
    declaration: Option<Node>,
    referencing_scopes: Vec<Rc<RefCell<Scope>>>,
    capturing_scopes: Vec<Rc<RefCell<Scope>>>,
    read_locations: Vec<Node>,
    write_locations: Vec<Node>,
}

impl Variable {
    /// C# Variable(VariableKind, IScopeInternal, string, SyntaxNode?)
    /// (IVariable.cs:78-91). The containing scope arrives as a weak handle
    /// (the C# reference is strong; the port's scope graph owns the scopes,
    /// so the weak handle avoids cycles).
    pub fn new(
        kind: VariableKind,
        containing_scope: Weak<RefCell<Scope>>,
        name: String,
        declaration: Option<Node>,
    ) -> Self {
        assert!(!name.is_empty(), "variable name must not be null or empty");
        Variable {
            kind,
            containing_scope,
            name,
            declaration,
            referencing_scopes: Vec::new(),
            capturing_scopes: Vec::new(),
            read_locations: Vec::new(),
            write_locations: Vec::new(),
        }
    }
}

impl IVariable for Variable {
    fn kind(&self) -> VariableKind {
        self.kind
    }

    fn containing_scope(&self) -> Rc<RefCell<Scope>> {
        self.containing_scope
            .upgrade()
            .expect("the containing scope must outlive its variables")
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn declaration(&self) -> Option<&Node> {
        self.declaration.as_ref()
    }

    fn referencing_scopes(&self) -> Vec<Rc<RefCell<Scope>>> {
        self.referencing_scopes.clone()
    }

    fn capturing_scopes(&self) -> Vec<Rc<RefCell<Scope>>> {
        self.capturing_scopes.clone()
    }

    fn read_locations(&self) -> Vec<Node> {
        self.read_locations.clone()
    }

    fn write_locations(&self) -> Vec<Node> {
        self.write_locations.clone()
    }

    /// C# CanBeAccessedIn (IVariable.cs:115-123): walks up the scope chain.
    fn can_be_accessed_in(&self, scope: &Rc<RefCell<Scope>>) -> bool {
        let containing = self.containing_scope();
        let mut curr_scope = Some(scope.clone());
        while let Some(curr) = curr_scope {
            if Rc::ptr_eq(&containing, &curr) {
                return true;
            }
            curr_scope = curr.borrow().parent();
        }
        false
    }
}

impl IVariableInternal for Variable {
    /// C# AddReferencingScope (IVariable.cs:125-126).
    fn add_referencing_scope(&mut self, scope: Rc<RefCell<Scope>>) {
        if !self
            .referencing_scopes
            .iter()
            .any(|s| Rc::ptr_eq(s, &scope))
        {
            self.referencing_scopes.push(scope);
        }
    }

    /// C# AddCapturingScope (IVariable.cs:128-129).
    fn add_capturing_scope(&mut self, scope: Rc<RefCell<Scope>>) {
        if !self.capturing_scopes.iter().any(|s| Rc::ptr_eq(s, &scope)) {
            self.capturing_scopes.push(scope);
        }
    }

    /// C# AddReadLocation (IVariable.cs:131-132).
    fn add_read_location(&mut self, node: Node) {
        self.read_locations.push(node);
    }

    /// C# AddWriteLocation (IVariable.cs:134-135).
    fn add_write_location(&mut self, node: Node) {
        self.write_locations.push(node);
    }
}
