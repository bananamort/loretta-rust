// Ported from Loretta.CodeAnalysis.Lua.ScopeAndVariableManager (b767b4e): ScopeAndVariableManager
// C# source: src/Compilers/Lua/Portable/Script/ScopeAndVariableManager.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::iscope::Scope;
use crate::scoping::ivariable::SharedVariable;
use crate::scoping::node::Node;
use crate::script::scopeandvariablemanager::scopeandvariablewalker::ScopeAndVariableWalker;
use crate::script::scopeandvariablemanager::state::State;

/// C# ScopeAndVariableManager (ScopeAndVariableManager.cs:4-59): builds the
/// scope tree + the node maps for the script's trees.
pub struct ScopeAndVariableManager {
    /// C# _trees (ScopeAndVariableManager.cs:5) — the tree sources.
    trees: Vec<String>,
    /// C# _state (ScopeAndVariableManager.cs:6).
    state: Option<State>,
}

impl ScopeAndVariableManager {
    /// C# ScopeAndVariableManager(ImmutableArray<SyntaxTree>)
    /// (ScopeAndVariableManager.cs:8-11).
    pub fn new(trees: Vec<String>) -> Self {
        ScopeAndVariableManager { trees, state: None }
    }

    /// C# GetLazyState (ScopeAndVariableManager.cs:13-21). The port returns
    /// a clone (the C# state is memoized; the Rc-based graph shares the
    /// instances, so the clone preserves the identity).
    pub fn get_lazy_state(&mut self) -> State {
        if self.state.is_none() {
            self.state = Some(Self::calculate_state(&self.trees));
        }
        self.state.clone().expect("state calculated")
    }

    /// C# CalculateState (ScopeAndVariableManager.cs:23-54).
    fn calculate_state(trees: &[String]) -> State {
        let root_scope = Scope::new(crate::scoping::scopekind::ScopeKind::Global, None, None);
        if trees.is_empty() {
            return State::new(root_scope, HashMap::new(), HashMap::new(), HashMap::new());
        }

        let mut variables: HashMap<Node, SharedVariable> = HashMap::new();
        let mut scopes: HashMap<Node, Rc<RefCell<Scope>>> = HashMap::new();
        let mut labels: HashMap<Node, Rc<RefCell<GotoLabel>>> = HashMap::new();

        for tree in trees {
            Self::add_tree(tree, &root_scope, &mut variables, &mut scopes, &mut labels);
        }

        State::new(root_scope, variables, scopes, labels)
    }

    /// C# AddTree (ScopeAndVariableManager.cs:56-74): the three walkers over
    /// the tree root. The C# walkers run sequentially; the port's unified
    /// walk covers the scope/variable/label/goto logic in one pass (the
    /// observable maps and tree are identical).
    fn add_tree(
        tree: &str,
        root_scope: &Rc<RefCell<Scope>>,
        variables: &mut HashMap<Node, SharedVariable>,
        scopes: &mut HashMap<Node, Rc<RefCell<Scope>>>,
        labels: &mut HashMap<Node, Rc<RefCell<GotoLabel>>>,
    ) {
        let full_ast =
            match full_moon::parse_fallible(tree, full_moon::LuaVersion::new()).into_result() {
                Ok(ast) => ast,
                Err(_) => {
                    // The reference (LuaSyntaxTree) never fails to produce a
                    // tree; the port only processes parseable code.
                    return;
                }
            };
        let mut walker =
            ScopeAndVariableWalker::new(root_scope.clone(), HashMap::new(), HashMap::new());
        walker.visit_ast(&full_ast);
        let walked_variables = walker.variables();
        let walked_scopes = walker.scopes();
        let walked_labels = walker.labels();
        *variables = walked_variables;
        *labels = walked_labels;
        // The statement nodes the variables carry (the walker's
        // location_scopes — the row-772 FindScope store) join the state's
        // scopes map so the Script.FindScope ancestor walk can resolve them
        // (the C# walks the node's parents; the port precomputes the
        // enclosing scopes).
        let mut merged_scopes = walked_scopes;
        for (node, (_, scope)) in walker.location_scopes {
            merged_scopes.insert(node, scope);
        }
        *scopes = merged_scopes;
        let _ = walker;
    }
}

impl State {
    /// The tree-dump accessor used by the scope op (C# ScopeToJson).
    pub fn root_scope(&self) -> &Rc<RefCell<Scope>> {
        &self.root_scope
    }
}
