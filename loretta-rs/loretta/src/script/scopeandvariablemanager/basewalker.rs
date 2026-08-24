// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager.BaseWalker (b767b4e): BaseWalker
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.BaseWalker.cs

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::scoping::iscope::Scope;
use crate::scoping::node::Node;

/// C# BaseWalker (BaseWalker.cs:5-25): the common scope-map holder for the
/// three walkers. The C# `_scopes` map keyed by SyntaxNode maps to the
/// port's Node-keyed map (the walkers fill it while building the tree).
pub struct BaseWalker {
    /// C# BaseWalker._scopes (BaseWalker.cs:7).
    pub scopes: HashMap<Node, Rc<RefCell<Scope>>>,
    /// The next node id (the C# node identity maps to the id counter). The
    /// counter is shared by every walker and spans all trees' walks, so
    /// node identities stay unique across the accumulated state (the C#
    /// SyntaxNode reference identity, Finding 5).
    pub next_id: Rc<Cell<u64>>,
}

impl BaseWalker {
    /// C# BaseWalker(IDictionary<SyntaxNode, IScope>, SyntaxWalkerDepth)
    /// (BaseWalker.cs:9-17) — with a fresh node-id counter (single-walk
    /// uses, e.g. the minifier's rename re-walk).
    pub fn new(scopes: HashMap<Node, Rc<RefCell<Scope>>>) -> Self {
        BaseWalker {
            scopes,
            next_id: Rc::new(Cell::new(0)),
        }
    }

    /// The BaseWalker with a shared node-id counter (the manager's
    /// multi-tree walks — the counter continues across trees).
    pub fn with_next_id(scopes: HashMap<Node, Rc<RefCell<Scope>>>, next_id: Rc<Cell<u64>>) -> Self {
        BaseWalker { scopes, next_id }
    }

    /// Creates a node with a unique id (the C# node identity).
    pub fn make_node(&mut self, kind: &str, text: String) -> Node {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        Node::new_with_id(kind, text, id)
    }
}
