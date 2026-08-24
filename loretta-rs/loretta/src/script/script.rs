// Ported from Loretta.CodeAnalysis.Lua.Script (b767b4e): Script
// C# source: src/Compilers/Lua/Portable/Script/Script.cs

use std::cell::RefCell;
use std::rc::Rc;

use crate::scoping::igotolabel::GotoLabel;
use crate::scoping::iscope::{IScope, Scope};
use crate::scoping::ivariable::{IVariable, SharedVariable};
use crate::scoping::node::Node;
use crate::scoping::scopekind::ScopeKind;
use crate::script::renameerrors::RenameError;
use crate::script::scopeandvariablemanager::manager::ScopeAndVariableManager;

/// The rename result (C# Tsu.Result<Script, IEnumerable<RenameError>>).
pub enum RenameResult {
    /// C# Ok(Script).
    Ok(Script),
    /// C# Err(errors).
    Err(Vec<RenameError>),
}

impl std::fmt::Debug for RenameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenameResult::Ok(_) => f.write_str("Ok(Script)"),
            RenameResult::Err(errors) => f.debug_tuple("Err").field(errors).finish(),
        }
    }
}

/// A script containing one or more files (C# Script.cs:12-...).
pub struct Script {
    /// C# Script.SyntaxTrees (Script.cs:43) — the dropped SyntaxTree maps
    /// to the tree texts.
    trees: Vec<String>,
    /// C# _scopeAndVariableManager (Script.cs:15).
    scope_and_variable_manager: ScopeAndVariableManager,
}

impl Script {
    /// C# Script.Empty (Script.cs:13) — an empty script.
    pub fn empty() -> Script {
        Script::new(Vec::new())
    }

    /// C# Script() + Script(ImmutableArray<SyntaxTree>) (Script.cs:20-41).
    /// The C# default-array ArgumentException maps to the empty-trees
    /// interpretation (the port has no default-array state).
    pub fn new(trees: Vec<String>) -> Script {
        let scope_and_variable_manager = ScopeAndVariableManager::new(trees.clone());
        Script {
            trees,
            scope_and_variable_manager,
        }
    }

    /// C# Script.SyntaxTrees (Script.cs:43).
    pub fn syntax_trees(&self) -> &[String] {
        &self.trees
    }

    /// The memoized state (for the rename rewriter).
    pub(crate) fn scope_and_variable_manager_state(
        &mut self,
    ) -> crate::script::scopeandvariablemanager::state::State {
        self.scope_and_variable_manager.get_lazy_state()
    }

    /// C# Script.RootScope (Script.cs:48).
    pub fn root_scope(&mut self) -> Rc<RefCell<Scope>> {
        self.scope_and_variable_manager.get_lazy_state().root_scope
    }

    /// C# Script.GetScope (Script.cs:55-58).
    pub fn get_scope(&mut self, node: &Node) -> Option<Rc<RefCell<Scope>>> {
        self.scope_and_variable_manager
            .get_lazy_state()
            .scopes
            .get(node)
            .cloned()
    }

    /// C# Script.FindScope (Script.cs:96-112): walks the node's ancestors
    /// looking for the nearest scope of the provided kind or a more generic
    /// one (the C# `scope.Kind <= kind` — the scope-kind ordering).
    pub fn find_scope(&mut self, node: &Node, kind: ScopeKind) -> Option<Rc<RefCell<Scope>>> {
        let state = self.scope_and_variable_manager.get_lazy_state();
        let mut current = Some(node.clone());
        while let Some(n) = current {
            if let Some(scope) = state.scopes.get(&n) {
                if scope.borrow().kind() as u8 <= kind as u8 {
                    return Some(scope.clone());
                }
            }
            // The C# AncestorsAndSelf walk over the scope map — the parent
            // scope lookup happens through the containing chain.
            current = match state.scopes.get(&n) {
                Some(scope) => scope.borrow().containing_scope(),
                None => None,
            }
            .and_then(|parent| parent.borrow().node().cloned());
        }
        None
    }

    /// C# Script.GetVariable (Script.cs:114-117).
    pub fn get_variable(&mut self, node: &Node) -> Option<SharedVariable> {
        self.scope_and_variable_manager
            .get_lazy_state()
            .variables
            .get(node)
            .cloned()
    }

    /// C# Script.GetLabel (Script.cs:125-128).
    pub fn get_label(&mut self, node: &Node) -> Option<Rc<RefCell<GotoLabel>>> {
        self.scope_and_variable_manager
            .get_lazy_state()
            .labels
            .get(node)
            .cloned()
    }

    /// C# Script.RenameVariable (Script.cs:141-188): attempts to rename the
    /// provided variable with the new name.
    pub fn rename_variable(&mut self, variable: &SharedVariable, new_name: &str) -> RenameResult {
        if new_name.is_empty() {
            panic!("newName cannot be null or empty");
        }
        let mut errors: Vec<RenameError> = Vec::new();
        let mut trees_with_locations: Vec<usize> = Vec::new();

        let handle_location =
            |this: &mut Script,
             location: &Node,
             errors: &mut Vec<RenameError>,
             trees_with_locations: &mut Vec<usize>| {
                let tree_idx = this
                    .trees
                    .iter()
                    .position(|t| t.contains(&location.text))
                    .unwrap_or(0);
                if !trees_with_locations.contains(&tree_idx) {
                    trees_with_locations.push(tree_idx);
                }
                if let Some(scope) = this.find_scope(location, ScopeKind::Block) {
                    if let Some(conflicting) =
                        scope.borrow().find_variable(new_name, ScopeKind::Global)
                    {
                        // The C# HashSet<RenameError> deduplicates the
                        // conflicts (the shared declaration/write statement
                        // node visits twice — Script.cs:145, 180-187).
                        let already_present = errors.iter().any(|e| {
                            matches!(
                                e,
                                RenameError::VariableConflict {
                                    variable_being_conflicted_with: existing
                                } if Rc::ptr_eq(existing, &conflicting)
                            )
                        });
                        if !already_present {
                            errors.push(RenameError::VariableConflict {
                                variable_being_conflicted_with: conflicting,
                            });
                        }
                    }
                }
            };

        let state = self.scope_and_variable_manager.get_lazy_state();
        let read_locations = variable.borrow().read_locations();
        let write_locations = variable.borrow().write_locations();
        let declaration = variable.borrow().declaration().cloned();
        drop(state);

        for location in &read_locations {
            handle_location(self, location, &mut errors, &mut trees_with_locations);
        }
        for location in &write_locations {
            handle_location(self, location, &mut errors, &mut trees_with_locations);
        }
        if let Some(declaration) = &declaration {
            handle_location(self, declaration, &mut errors, &mut trees_with_locations);
        }

        if new_name.chars().any(|c| c as u32 >= 0x7F) {
            // C#: the per-tree IdentifierNameNotSupportedError for trees
            // without LuaJIT identifier rules (the dropped options map to
            // the port's single tree mode).
            for tree in &self.trees {
                let _ = tree;
                errors.push(RenameError::IdentifierNameNotSupported {
                    tree_without_support: tree.clone(),
                });
            }
        }

        if !errors.is_empty() {
            return RenameResult::Err(errors);
        }

        let new_trees = self.rename_in_trees(variable, new_name, &trees_with_locations);
        RenameResult::Ok(Script::new(new_trees))
    }

    /// C# RenameVariable's final loop: the RenameRewriter over each affected
    /// tree.
    fn rename_in_trees(
        &mut self,
        variable: &SharedVariable,
        new_name: &str,
        tree_indices: &[usize],
    ) -> Vec<String> {
        let mut new_trees = self.trees.clone();
        for &idx in tree_indices {
            if let Some(tree) = new_trees.get(idx) {
                let rewritten = crate::script::scriptrenamerewriter::rename_in_tree(
                    idx, tree, variable, new_name, self,
                );
                new_trees[idx] = rewritten;
            }
        }
        new_trees
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renames_a_local_variable() {
        let mut script = Script::new(vec!["local a = 1\nprint(a)\n".to_string()]);
        let root = script.root_scope();
        let file = root.borrow().contained_scopes()[0].clone();
        let variable = file
            .borrow()
            .declared_variables()
            .iter()
            .find(|v| v.borrow().name() == "a")
            .expect("the a variable")
            .clone();
        let result = script.rename_variable(&variable, "renamed");
        match result {
            RenameResult::Ok(new_script) => {
                let text = &new_script.syntax_trees()[0];
                assert!(text.contains("local renamed = 1"));
                assert!(text.contains("print(renamed)"));
                assert!(!text.contains("local a = 1"));
            }
            RenameResult::Err(errors) => panic!("rename failed: {errors:?}"),
        }
    }

    #[test]
    fn renames_a_variable_declared_in_the_second_tree() {
        // Finding 5: with the shared node-id counter, the rename rewriter
        // must reproduce the second tree's node ids via its recorded id
        // base.
        let mut script = Script::new(vec![
            "local a = 1\n".to_string(),
            "local b = 2\nprint(b)\n".to_string(),
        ]);
        let root = script.root_scope();
        let root_contained = root.borrow().contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        let variable = files[1]
            .borrow()
            .declared_variables()
            .iter()
            .find(|v| v.borrow().name() == "b")
            .expect("the b variable")
            .clone();
        let result = script.rename_variable(&variable, "renamed");
        match result {
            RenameResult::Ok(new_script) => {
                assert_eq!(new_script.syntax_trees()[0], "local a = 1\n");
                assert!(new_script.syntax_trees()[1].contains("local renamed = 2"));
                assert!(new_script.syntax_trees()[1].contains("print(renamed)"));
            }
            RenameResult::Err(errors) => panic!("rename failed: {errors:?}"),
        }
    }
}
