// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager.BaseWalker (b767b4e): BaseWalker
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.BaseWalker.cs

use std::cell::RefCell;
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
    /// The next node id (the C# node identity maps to the id counter).
    pub next_id: u64,
}

impl BaseWalker {
    /// C# BaseWalker(IDictionary<SyntaxNode, IScope>, SyntaxWalkerDepth)
    /// (BaseWalker.cs:9-17).
    pub fn new(scopes: HashMap<Node, Rc<RefCell<Scope>>>) -> Self {
        BaseWalker { scopes, next_id: 0 }
    }

    /// Creates a node with a unique id (the C# node identity).
    pub fn make_node(&mut self, kind: &str, text: String) -> Node {
        let node = Node::new_with_id(kind, text, self.next_id);
        self.next_id += 1;
        node
    }
}
