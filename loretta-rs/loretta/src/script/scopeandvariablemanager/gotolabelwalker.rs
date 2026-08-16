// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager.GotoLabelWalker (b767b4e)
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.GotoLabelWalker.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::iscope::Scope;
use crate::scoping::node::Node;
use crate::script::scopeandvariablemanager::basewalker::BaseWalker;

/// C# GotoLabelWalker (GotoLabelWalker.cs:7-24): creates the labels from the
/// `::label::` statements. The C# FindScope (the ancestor walk over the
/// scope map) maps to the current scope of the unified walk — the walker
/// receives it per visit.
pub struct GotoLabelWalker {
    /// C# _labels (GotoLabelWalker.cs:9).
    labels: HashMap<Node, Rc<RefCell<GotoLabel>>>,
    base: BaseWalker,
}

impl GotoLabelWalker {
    /// C# GotoLabelWalker(IDictionary, IDictionary) (GotoLabelWalker.cs:12-18).
    pub fn new(
        scopes: HashMap<Node, Rc<RefCell<Scope>>>,
        labels: HashMap<Node, Rc<RefCell<GotoLabel>>>,
    ) -> Self {
        GotoLabelWalker {
            labels,
            base: BaseWalker::new(scopes),
        }
    }

    /// C# VisitGotoLabelStatement (GotoLabelWalker.cs:20-24): creates the
    /// label in the nearest scope (the unified walk's current scope).
    pub fn visit_goto_label_stmt(
        &mut self,
        scope: &Rc<RefCell<Scope>>,
        name: &str,
        stmt_text: String,
    ) {
        let node = self.base.make_node("GotoLabelStatement", stmt_text);
        let label = Scope::create_label_in(scope, name);
        self.labels.insert(node, label);
    }

    /// The node -> label map.
    pub fn labels(&mut self) -> HashMap<Node, Rc<RefCell<GotoLabel>>> {
        std::mem::take(&mut self.labels)
    }
}
