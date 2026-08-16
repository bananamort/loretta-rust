// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager.GotoWalker (b767b4e)
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.GotoWalker.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::scoping::igotolabel::{GotoLabel, IGotoLabelInternal};
use crate::scoping::iscope::Scope;
use crate::scoping::node::Node;
use crate::script::scopeandvariablemanager::basewalker::BaseWalker;

/// C# GotoWalker (GotoWalker.cs:7-28): links the `goto` statements to their
/// labels. The C# FindScope maps to the unified walk's current scope.
pub struct GotoWalker {
    /// C# _labels (GotoWalker.cs:9).
    labels: HashMap<Node, Rc<RefCell<GotoLabel>>>,
    base: BaseWalker,
}

impl GotoWalker {
    /// C# GotoWalker(IDictionary, IDictionary) (GotoWalker.cs:12-18).
    pub fn new(
        scopes: HashMap<Node, Rc<RefCell<Scope>>>,
        labels: HashMap<Node, Rc<RefCell<GotoLabel>>>,
    ) -> Self {
        GotoWalker {
            labels,
            base: BaseWalker::new(scopes),
        }
    }

    /// C# VisitGotoStatement (GotoWalker.cs:20-27): gets-or-creates the
    /// label and adds the jump.
    pub fn visit_goto_stmt(
        &mut self,
        scope: &Rc<RefCell<Scope>>,
        name: &str,
        stmt: &full_moon::ast::lua52::Goto,
    ) {
        if name.trim().is_empty() {
            return;
        }
        let node = self.base.make_node("GotoStatement", stmt.to_string());
        let label = Scope::get_or_create_label_in(scope, name);
        label.borrow_mut().add_jump(stmt.clone());
        self.labels.insert(node, label);
    }

    /// The node -> label map.
    pub fn labels(&mut self) -> HashMap<Node, Rc<RefCell<GotoLabel>>> {
        std::mem::take(&mut self.labels)
    }
}
