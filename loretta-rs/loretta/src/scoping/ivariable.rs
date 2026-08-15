// Ported from Loretta.CodeAnalysis.Lua.IVariable (b767b4e): IVariable, IVariableInternal, Variable
// C# source: src/Compilers/Lua/Portable/Scoping/IVariable.cs

use crate::scoping::iscope::ScopeRef;
use crate::scoping::variablekind::VariableKind;
use std::cell::RefCell;
use std::rc::Rc;

/// A shared reference to a [`Variable`].
pub type VariableRef = Rc<RefCell<Variable>>;

/// The base interface for variables.
pub trait IVariable {
    /// The kind of this variable.
    fn kind(&self) -> VariableKind;

    /// The containing scope.
    fn containing_scope(&self) -> ScopeRef;

    /// The variable's name.
    fn name(&self) -> &str;

    /// The offset of the node where this variable is declared.
    /// None if it is a global or implicit variable.
    /// (C# `SyntaxNode?` — projected as the source offset.)
    fn declaration(&self) -> Option<usize>;

    /// The scopes that reference this variable.
    fn referencing_scopes(&self) -> Vec<ScopeRef>;

    /// All scopes that capture this variable as an upvalue.
    fn capturing_scopes(&self) -> Vec<ScopeRef>;

    /// All locations this variable is read from.
    /// (C# `IEnumerable<SyntaxNode>` — projected as source offsets.)
    fn read_locations(&self) -> &[usize];

    /// All locations this variable is written to.
    /// (C# `IEnumerable<SyntaxNode>` — projected as source offsets.)
    fn write_locations(&self) -> &[usize];

    /// Returns whether this variable can be accessed in the provided scope.
    fn can_be_accessed_in(&self, scope: &ScopeRef) -> bool;
}

/// The C# `IVariableInternal` — the mutating operations are flattened into
/// inherent methods on [`Variable`] (internal interface, documented drop).
pub struct Variable {
    kind: VariableKind,
    containing_scope: ScopeRef,
    name: String,
    declaration: Option<usize>,
    referencing_scopes: Vec<ScopeRef>,
    capturing_scopes: Vec<ScopeRef>,
    read_locations: Vec<usize>,
    write_locations: Vec<usize>,
}

impl Variable {
    /// C# `Variable(VariableKind, IScopeInternal, string, SyntaxNode?)` ctor.
    pub fn new(
        kind: VariableKind,
        containing_scope: ScopeRef,
        name: String,
        declaration: Option<usize>,
    ) -> Self {
        Self {
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

    /// C# `AddReferencingScope(IScopeInternal)`.
    pub fn add_referencing_scope(&mut self, scope: &ScopeRef) {
        if !self.referencing_scopes.iter().any(|s| Rc::ptr_eq(s, scope)) {
            self.referencing_scopes.push(scope.clone());
        }
    }

    /// C# `AddCapturingScope(IScopeInternal)`.
    pub fn add_capturing_scope(&mut self, scope: &ScopeRef) {
        if !self.capturing_scopes.iter().any(|s| Rc::ptr_eq(s, scope)) {
            self.capturing_scopes.push(scope.clone());
        }
    }

    /// C# `AddReadLocation(SyntaxNode)`.
    pub fn add_read_location(&mut self, node: usize) {
        self.read_locations.push(node);
    }

    /// C# `AddWriteLocation(SyntaxNode)`.
    pub fn add_write_location(&mut self, node: usize) {
        self.write_locations.push(node);
    }
}

impl IVariable for Variable {
    fn kind(&self) -> VariableKind {
        self.kind
    }

    fn containing_scope(&self) -> ScopeRef {
        self.containing_scope.clone()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn declaration(&self) -> Option<usize> {
        self.declaration
    }

    fn referencing_scopes(&self) -> Vec<ScopeRef> {
        self.referencing_scopes.clone()
    }

    fn capturing_scopes(&self) -> Vec<ScopeRef> {
        self.capturing_scopes.clone()
    }

    fn read_locations(&self) -> &[usize] {
        &self.read_locations
    }

    fn write_locations(&self) -> &[usize] {
        &self.write_locations
    }

    fn can_be_accessed_in(&self, scope: &ScopeRef) -> bool {
        let mut current = Some(scope.clone());
        while let Some(curr_scope) = current {
            if Rc::ptr_eq(&self.containing_scope, &curr_scope) {
                return true;
            }
            current = curr_scope.borrow().parent_ref();
        }
        false
    }
}
