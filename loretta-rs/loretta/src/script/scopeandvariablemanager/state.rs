// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager.State (b767b4e): State
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.State.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::iscope::Scope;
use crate::scoping::ivariable::SharedVariable;
use crate::scoping::node::Node;

/// C# ScopeAndVariableManager.State (State.cs:7-29).
#[derive(Clone)]
pub struct State {
    /// C# State.RootScope (State.cs:22).
    pub root_scope: Rc<RefCell<Scope>>,
    /// C# State.Variables (State.cs:23) — the node -> variable map.
    pub variables: HashMap<Node, SharedVariable>,
    /// C# State.Scopes (State.cs:24) — the node -> scope map.
    pub scopes: HashMap<Node, Rc<RefCell<Scope>>>,
    /// C# State.Labels (State.cs:25) — the node -> label map.
    pub labels: HashMap<Node, Rc<RefCell<GotoLabel>>>,
    /// The shared node-id counter's value before each tree's walk (the
    /// rename rewriter reproduces a tree's node ids by seeding its walk
    /// with the tree's base — the port's C# node-identity emulation;
    /// Finding 5).
    pub tree_id_bases: Vec<u64>,
}

impl State {
    /// C# State(IScope, IImmutableDictionary, ...) (State.cs:10-20).
    pub fn new(
        root_scope: Rc<RefCell<Scope>>,
        variables: HashMap<Node, SharedVariable>,
        scopes: HashMap<Node, Rc<RefCell<Scope>>>,
        labels: HashMap<Node, Rc<RefCell<GotoLabel>>>,
        tree_id_bases: Vec<u64>,
    ) -> Self {
        State {
            root_scope,
            variables,
            scopes,
            labels,
            tree_id_bases,
        }
    }
}
