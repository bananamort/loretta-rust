// Ported from Loretta.CodeAnalysis.Lua.Script (b767b4e): Script
// C# source: src/Compilers/Lua/Portable/Script/Script.cs

use std::cell::RefCell;
use std::rc::Rc;

use crate::luasyntaxoptions::LuaSyntaxOptions;
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
    /// C# _scopeAndVariableManager (Script.cs:15). Boxed to keep the
    /// RenameResult::Ok variant small (the port's no-#[allow] rule).
    scope_and_variable_manager: Box<ScopeAndVariableManager>,
    /// The per-tree syntax options (the C# SyntaxTree carries its
    /// LuaParseOptions; the port's trees are strings, so the options are
    /// stored alongside — the rename gate's UseLuaJitIdentifierRules
    /// check, Script.cs:158-165).
    tree_options: Vec<LuaSyntaxOptions>,
}

impl Script {
    /// C# Script.Empty (Script.cs:13) — an empty script.
    pub fn empty() -> Script {
        Script::new(Vec::new())
    }

    /// C# Script() + Script(ImmutableArray<SyntaxTree>) (Script.cs:20-41).
    /// The C# default-array ArgumentException maps to the empty-trees
    /// interpretation (the port has no default-array state). Trees without
    /// explicit options parse with LuaParseOptions.Default = All
    /// (LuaParseOptions.cs:15).
    pub fn new(trees: Vec<String>) -> Script {
        Script::new_with_options(trees, LuaSyntaxOptions::ALL)
    }

    /// C# Script(ImmutableArray<SyntaxTree>) with one option set for every
    /// tree (the C# tests parse each tree with the same LuaParseOptions).
    pub fn new_with_options(trees: Vec<String>, options: LuaSyntaxOptions) -> Script {
        let tree_count = trees.len();
        let scope_and_variable_manager = ScopeAndVariableManager::new(trees.clone());
        Script {
            trees,
            scope_and_variable_manager: Box::new(scope_and_variable_manager),
            tree_options: vec![options; tree_count],
        }
    }

    /// C# Script.SyntaxTrees (Script.cs:43).
    pub fn syntax_trees(&self) -> &[String] {
        &self.trees
    }

    /// The memoized state (for the rename rewriter and the FindScope
    /// tests' recorded-node lookups — Finding 61).
    pub fn scope_and_variable_manager_state(
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
    /// one (the C# `scope.Kind <= kind` — the scope-kind ordering). The
    /// identifier/statement location store is consulted first (the port's
    /// precomputed enclosing scopes), then the scope-created nodes' map
    /// (Finding 14 keeps the two separate).
    pub fn find_scope(&mut self, node: &Node, kind: ScopeKind) -> Option<Rc<RefCell<Scope>>> {
        let state = self.scope_and_variable_manager.get_lazy_state();
        let mut current = Some(node.clone());
        while let Some(n) = current {
            let scope = state
                .location_scopes
                .get(&n)
                .or_else(|| state.scopes.get(&n));
            if let Some(scope) = scope {
                if scope.borrow().kind() as u8 <= kind as u8 {
                    return Some(scope.clone());
                }
            }
            // The C# AncestorsAndSelf walk over the scope map — the parent
            // scope lookup happens through the containing chain.
            current = match scope {
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
    /// provided variable with the new name. The C# argument checks are the
    /// nulls (Script.cs:143-144) — there is NO empty-string check: the
    /// empty name flows into the location handling, where the C#
    /// FindVariable rejects it with an ArgumentException (IScope.cs:166-167)
    /// and the port's find_variable panics with the same message (Finding
    /// 44); a variable without locations renames to an empty string with no
    /// changes, like the C# Ok result.
    pub fn rename_variable(&mut self, variable: &SharedVariable, new_name: &str) -> RenameResult {
        let mut errors: Vec<RenameError> = Vec::new();
        let mut trees_with_locations: Vec<usize> = Vec::new();

        let handle_location =
            |this: &mut Script,
             location: &Node,
             tree_id_bases: &[u64],
             errors: &mut Vec<RenameError>,
             trees_with_locations: &mut Vec<usize>| {
                // C# trees.Add(location.SyntaxTree) (Script.cs:180-187):
                // the node's own tree. The port recovers it exactly from
                // the node id against the state's per-tree id bases
                // (Finding 12) — no substring search, which misattributed
                // shared text to the first matching tree (defaulting to
                // tree 0).
                let tree_idx = tree_id_bases
                    .iter()
                    .rposition(|&base| base <= location.id)
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
        let tree_id_bases = state.tree_id_bases.clone();
        let read_locations = variable.borrow().read_locations();
        let write_locations = variable.borrow().write_locations();
        let declaration = variable.borrow().declaration().cloned();
        drop(state);

        for location in &read_locations {
            handle_location(
                self,
                location,
                &tree_id_bases,
                &mut errors,
                &mut trees_with_locations,
            );
        }
        for location in &write_locations {
            handle_location(
                self,
                location,
                &tree_id_bases,
                &mut errors,
                &mut trees_with_locations,
            );
        }
        if let Some(declaration) = &declaration {
            handle_location(
                self,
                declaration,
                &tree_id_bases,
                &mut errors,
                &mut trees_with_locations,
            );
        }

        if new_name.chars().any(|c| c as u32 >= 0x7F) {
            // C# Script.cs:158-165: only the AFFECTED trees — and only
            // those without LuaJIT identifier rules — report the error.
            for &idx in &trees_with_locations {
                if let Some(options) = self.tree_options.get(idx) {
                    if !options.use_lua_jit_identifier_rules {
                        errors.push(RenameError::IdentifierNameNotSupported {
                            tree_without_support: self.trees[idx].clone(),
                        });
                    }
                }
            }
        }

        if !errors.is_empty() {
            return RenameResult::Err(errors);
        }

        let new_trees = self.rename_in_trees(variable, new_name, &trees_with_locations);
        // The C# rewritten trees keep their options (Script.cs:170-178).
        RenameResult::Ok(Script {
            trees: new_trees.clone(),
            scope_and_variable_manager: Box::new(ScopeAndVariableManager::new(new_trees)),
            tree_options: self.tree_options.clone(),
        })
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
    fn empty_rename_names_follow_the_csharp_validation() {
        // Finding 44: the port's top-level panic on an empty new name
        // was not the C# behavior — RenameVariable has no empty-string
        // check (Script.cs:143-144); the empty name flows into the
        // location handling, where the C# FindVariable rejects it with
        // an ArgumentException ("'name' must be a valid identifier.",
        // IScope.cs:166-167) and the port's find_variable panics with
        // the same message.
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
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            script.rename_variable(&variable, "")
        }));
        assert!(
            result.is_err(),
            "the empty name must panic like the C# ArgumentException"
        );
    }

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

    #[test]
    fn renames_to_luajit_identifier_names_when_the_tree_allows_them() {
        // Finding 11: the C# gate is per-tree UseLuaJitIdentifierRules
        // (Script.cs:158-165) — trees with LuaJIT identifier rules accept
        // non-ASCII names.
        let mut script = Script::new_with_options(
            vec!["local a = 1\n".to_string()],
            LuaSyntaxOptions::LUAJIT20,
        );
        let root = script.root_scope();
        let file = root.borrow().contained_scopes()[0].clone();
        let variable = file
            .borrow()
            .declared_variables()
            .iter()
            .find(|v| v.borrow().name() == "a")
            .expect("the a variable")
            .clone();
        let result = script.rename_variable(&variable, "\u{30EB}");
        match result {
            RenameResult::Ok(new_script) => {
                assert!(new_script.syntax_trees()[0].contains("local \u{30EB} = 1"));
            }
            RenameResult::Err(errors) => panic!("expected the rename to succeed: {errors:?}"),
        }
    }

    #[test]
    fn unsupported_identifier_error_only_for_affected_trees() {
        // Finding 11: the C# gate reports only the trees with the
        // variable's locations (Script.cs:158-165) — not every tree.
        let mut script = Script::new_with_options(
            vec!["local a = 1\n".to_string(), "local b = 2\n".to_string()],
            LuaSyntaxOptions::LUA51,
        );
        let root = script.root_scope();
        let root_contained = root.borrow().contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        let variable = files[0]
            .borrow()
            .declared_variables()
            .iter()
            .find(|v| v.borrow().name() == "a")
            .expect("the a variable")
            .clone();
        let result = script.rename_variable(&variable, "\u{FEFF}");
        match result {
            RenameResult::Err(errors) => {
                assert_eq!(errors.len(), 1, "only the affected tree: {errors:?}");
            }
            other => panic!("expected the error: {other:?}"),
        }
    }

    #[test]
    fn rename_attributes_locations_to_the_nodes_own_tree() {
        // Finding 12: the tree attribution is exact (the C#
        // location.SyntaxTree, Script.cs:180-187) — the substring search
        // misattributed text shared with another tree to the first
        // matching tree.
        let mut script = Script::new_with_options(
            vec![
                "local a = 1\n".to_string(),
                "local a = 1\nprint(a)\n".to_string(),
            ],
            LuaSyntaxOptions::LUA51,
        );
        let root = script.root_scope();
        let root_contained = root.borrow().contained_scopes();
        let files: Vec<_> = root_contained.iter().collect();
        assert_eq!(files.len(), 2);
        let variable = files[1]
            .borrow()
            .declared_variables()
            .iter()
            .find(|v| v.borrow().name() == "a")
            .expect("the second tree's a variable")
            .clone();
        let result = script.rename_variable(&variable, "b");
        match result {
            RenameResult::Ok(new_script) => {
                assert_eq!(new_script.syntax_trees()[0], "local a = 1\n");
                assert!(new_script.syntax_trees()[1].contains("local b = 1"));
                assert!(new_script.syntax_trees()[1].contains("print(b)"));
            }
            RenameResult::Err(errors) => panic!("rename failed: {errors:?}"),
        }
    }

    #[test]
    fn get_scope_returns_null_for_identifier_nodes() {
        // Finding 14: the C# _scopes map holds only scope-created nodes —
        // GetScope(identifier) is null (Script.cs:55-59); the location
        // store is separate.
        let mut script = Script::new(vec!["print(a)\n".to_string()]);
        let state = script.scope_and_variable_manager_state();
        let (identifier, _) = state
            .location_scopes
            .iter()
            .find(|(node, _)| node.kind_name() == "IdentifierName")
            .expect("the identifier node");
        assert!(
            script.get_scope(identifier).is_none(),
            "identifiers are not in the scopes map"
        );
    }

    #[test]
    fn find_scope_resolves_identifiers_via_the_location_store() {
        // Finding 14: find_scope keeps resolving identifiers (the C#
        // ancestor walk equivalent) after the location store split.
        let mut script = Script::new(vec!["do print(a) end\n".to_string()]);
        let state = script.scope_and_variable_manager_state();
        let (identifier, scope) = state
            .location_scopes
            .iter()
            .find(|(node, _)| node.kind_name() == "IdentifierName")
            .expect("the identifier node");
        assert_eq!(scope.borrow().kind(), ScopeKind::Block);
        let found = script
            .find_scope(identifier, ScopeKind::Block)
            .expect("the found scope");
        assert!(Rc::ptr_eq(&found, scope));
    }
}
