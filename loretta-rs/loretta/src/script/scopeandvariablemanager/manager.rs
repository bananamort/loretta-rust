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
            return State::new(
                root_scope,
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                Vec::new(),
                HashMap::new(),
            );
        }

        // The shared node-id counter spans every tree's walk (the C#
        // SyntaxNode reference identity across the accumulated state —
        // Finding 5); the per-tree bases let the rename rewriter reproduce
        // each tree's node ids.
        let next_id: std::rc::Rc<std::cell::Cell<u64>> = std::rc::Rc::new(std::cell::Cell::new(0));
        let mut tree_id_bases: Vec<u64> = Vec::new();
        let mut variables: HashMap<Node, SharedVariable> = HashMap::new();
        let mut scopes: HashMap<Node, Rc<RefCell<Scope>>> = HashMap::new();
        let mut labels: HashMap<Node, Rc<RefCell<GotoLabel>>> = HashMap::new();
        let mut location_scopes: HashMap<Node, Rc<RefCell<Scope>>> = HashMap::new();

        for tree in trees {
            tree_id_bases.push(next_id.get());
            Self::add_tree(
                tree,
                &root_scope,
                &mut variables,
                &mut scopes,
                &mut labels,
                &mut location_scopes,
                &next_id,
            );
        }

        State::new(
            root_scope,
            variables,
            scopes,
            labels,
            tree_id_bases,
            location_scopes,
        )
    }

    /// C# AddTree (ScopeAndVariableManager.cs:56-74): the three walkers over
    /// the tree root. The C# walkers run sequentially with the shared
    /// accumulating maps; the port's unified walk covers the
    /// scope/variable/label/goto logic in one pass and accumulates each
    /// tree's entries into the shared maps (the observable maps and tree
    /// are identical).
    fn add_tree(
        tree: &str,
        root_scope: &Rc<RefCell<Scope>>,
        variables: &mut HashMap<Node, SharedVariable>,
        scopes: &mut HashMap<Node, Rc<RefCell<Scope>>>,
        labels: &mut HashMap<Node, Rc<RefCell<GotoLabel>>>,
        location_scopes: &mut HashMap<Node, Rc<RefCell<Scope>>>,
        next_id: &std::rc::Rc<std::cell::Cell<u64>>,
    ) {
        let full_ast =
            match full_moon::parse_fallible(tree, full_moon::LuaVersion::new()).into_result() {
                Ok(ast) => ast,
                Err(_) => {
                    // The C# (LuaSyntaxTree) never fails to produce a tree —
                    // parse errors become error nodes the walkers handle.
                    // full_moon returns no AST on parse failure, so the tree
                    // is dropped and contributes nothing to the accumulated
                    // state (structural boundary, Finding 5).
                    return;
                }
            };
        let mut walker = ScopeAndVariableWalker::new(
            root_scope.clone(),
            HashMap::new(),
            HashMap::new(),
            next_id.clone(),
        );
        walker.visit_ast(&full_ast);
        let walked_variables = walker.variables();
        let walked_scopes = walker.scopes();
        let walked_labels = walker.labels();
        // C# shared accumulating builders (ScopeAndVariableManager.cs:35-47):
        // each tree's entries join the shared maps instead of replacing them
        // (Finding 5 — the last tree must not win).
        variables.extend(walked_variables);
        labels.extend(walked_labels);
        scopes.extend(walked_scopes);
        // The identifier + statement location store stays SEPARATE from the
        // scopes map (Finding 14): the C# _scopes holds only scope-created
        // nodes, so Script.GetScope(identifier) must return null; the port's
        // FindScope resolves the precomputed enclosing scopes here (the C#
        // walks the node's parents; the port precomputes them).
        for (node, (_, scope)) in walker.location_scopes {
            location_scopes.insert(node, scope);
        }
        let _ = walker;
    }
}

impl State {
    /// The tree-dump accessor used by the scope op (C# ScopeToJson).
    pub fn root_scope(&self) -> &Rc<RefCell<Scope>> {
        &self.root_scope
    }
}
