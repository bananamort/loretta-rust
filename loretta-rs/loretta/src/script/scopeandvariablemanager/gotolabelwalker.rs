// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager.GotoLabelWalker (b767b4e)
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.GotoLabelWalker.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use full_moon::ast::lua52;

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
        next_id: std::rc::Rc<std::cell::Cell<u64>>,
    ) -> Self {
        GotoLabelWalker {
            labels,
            base: BaseWalker::with_next_id(scopes, next_id),
        }
    }

    /// C# VisitGotoLabelStatement (GotoLabelWalker.cs:20-24): creates the
    /// label in the nearest scope (the unified walk's current scope).
    pub fn visit_goto_label_stmt(&mut self, scope: &Rc<RefCell<Scope>>, label: &lua52::Label) {
        let name = label.name().token().to_string();
        let node = self.base.make_node("GotoLabelStatement", label.to_string());
        // Finding 6: the C# runs GotoLabelWalker before GotoWalker, so a
        // goto always binds to the already-created label. The port's
        // single pass can meet a forward goto first — it created a
        // placeholder in this scope (get_or_create_label_in). Bind to
        // that same-scope placeholder instead of creating a second label
        // that orphans the jump — without ascending, so a label in a
        // nested block still shadows an outer one (the C# CreateLabel
        // targets only the current scope, IScope.cs:226-231).
        //
        // Finding 7: the label carries its statement's syntax node (the
        // C# GotoLabelWalker passes it to CreateLabel,
        // GotoLabelWalker.cs:24) — including when binding to a
        // forward-goto placeholder (the port attaches it in place, the
        // shared placeholder must not be replaced).
        let existing = scope.borrow().try_get_label_in_scope(&name);
        let label_ref = match existing {
            Some(label_ref) => {
                label_ref.borrow_mut().set_label_syntax(label.clone());
                label_ref
            }
            None => Scope::create_label_in(scope, &name, Some(label.clone())),
        };
        self.labels.insert(node, label_ref);
    }

    /// The node -> label map.
    pub fn labels(&mut self) -> HashMap<Node, Rc<RefCell<GotoLabel>>> {
        std::mem::take(&mut self.labels)
    }
}
