// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.RenamingRewriter.RenameTable (b767b4e)
// C# source: src/Compilers/Lua/Experimental/Minifying/RenamingRewriter.RenameTable.cs

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::experimental::minifying::islotallocator::ISlotAllocator;
use crate::experimental::minifying::minifyingutils::MinifyingUtils;
use crate::experimental::minifying::namingstrategy::NamingStrategy;
use crate::scoping::iscope::Scope;
use crate::scoping::ivariable::{IVariable, SharedVariable};
use crate::scoping::node::Node;
use crate::script::script::Script;

/// The identifier walk records (the node, its token's byte position, and the
/// scope the identifier is in) — the port's equivalent of the C# node
/// locations with their source spans.
pub type IdentifierRecord = (Node, usize, Rc<RefCell<Scope>>);

/// C# RenameTable (RenameTable.cs:5-86).
pub struct RenameTable {
    /// C# _lock (RenameTable.cs:7) — the port has no threads.
    /// C# _script (RenameTable.cs:8).
    script: Script,
    /// C# _namingStrategy (RenameTable.cs:9).
    naming_strategy: NamingStrategy,
    /// C# _lastUseCache (RenameTable.cs:10) — keyed by the variable's
    /// declaration node id (the C# reference-identity key).
    last_use_cache: HashMap<usize, Option<u64>>,
    /// C# _variableMap (RenameTable.cs:11) — (slot, newName).
    variable_map: HashMap<usize, (i32, String)>,
    /// C# _slotAllocator (RenameTable.cs:12).
    slot_allocator: Box<dyn ISlotAllocator>,
    /// The location scopes of the current walk (node -> (position, scope))
    /// — the port's FindScope store for the nodes the variables carry (the
    /// identifier records and the statement nodes that become
    /// declaration/write locations). Used for the last-use ordering and the
    /// scope collection.
    location_scopes: HashMap<Node, (usize, Rc<RefCell<Scope>>)>,
}

impl RenameTable {
    /// C# RenameTable(Script, NamingStrategy, ISlotAllocator)
    /// (RenameTable.cs:14-25).
    pub fn new(
        script: Script,
        naming_strategy: NamingStrategy,
        slot_allocator: Box<dyn ISlotAllocator>,
    ) -> Self {
        RenameTable {
            script,
            naming_strategy,
            last_use_cache: HashMap::new(),
            variable_map: HashMap::new(),
            slot_allocator,
            location_scopes: HashMap::new(),
        }
    }

    /// Prepares the location scopes for the walk (the C# FindScope of every
    /// node the variables carry).
    pub fn prepare(&mut self, location_scopes: &HashMap<Node, (usize, Rc<RefCell<Scope>>)>) {
        self.location_scopes = location_scopes.clone();
    }

    /// The variable's identity key (the C# IVariable reference identity — a
    /// shared declaration node must not conflate its variables).
    fn variable_key(variable: &SharedVariable) -> usize {
        Rc::as_ptr(variable) as usize
    }

    /// C# GetLastUse (RenameTable.cs:27-41): the location the variable is
    /// last used — the read/write location with the highest source span, or
    /// the declaration.
    pub fn get_last_use(&mut self, variable: &SharedVariable) -> Option<u64> {
        let key = Self::variable_key(variable);
        if !self.last_use_cache.contains_key(&key) {
            let mut best: Option<(usize, u64)> = None;
            for node in variable.borrow().read_locations() {
                if let Some((pos, _)) = self.location_scopes.get(&node) {
                    if best.map(|(b, _)| *pos > b).unwrap_or(true) {
                        best = Some((*pos, node.id));
                    }
                }
            }
            for node in variable.borrow().write_locations() {
                if let Some((pos, _)) = self.location_scopes.get(&node) {
                    if best.map(|(b, _)| *pos > b).unwrap_or(true) {
                        best = Some((*pos, node.id));
                    }
                }
            }
            let use_node = match best {
                Some((_, id)) => Some(id),
                None => variable.borrow().declaration().map(|node| node.id),
            };
            self.last_use_cache.insert(key, use_node);
        }
        self.last_use_cache[&key]
    }

    /// C# GetNewVariableName (RenameTable.cs:51-80).
    pub fn get_new_variable_name(&mut self, node: &Node) -> Option<String> {
        let variable = self
            .script
            .get_variable(node)
            .expect("the node must map to a variable (C# ExceptionUtilities.Unreachable)");
        if !MinifyingUtils::can_rename(&variable) {
            return None;
        }
        let key = Self::variable_key(&variable);

        if !self.variable_map.contains_key(&key) {
            let slot = self.slot_allocator.allocate_slot();

            // The scopes the variable's locations live in (the C# FindScope
            // per location — the walker precomputes the enclosing scope of
            // every node the variable carries).
            let mut scopes: Vec<Rc<RefCell<Scope>>> = Vec::new();
            let mut seen: Vec<usize> = Vec::new();
            let push_scope = |scope: &Rc<RefCell<Scope>>,
                              scopes: &mut Vec<Rc<RefCell<Scope>>>,
                              seen: &mut Vec<usize>| {
                let ptr = Rc::as_ptr(scope) as usize;
                if !seen.contains(&ptr) {
                    seen.push(ptr);
                    scopes.push(scope.clone());
                }
            };
            for location in variable.borrow().read_locations() {
                if let Some((_, scope)) = self.location_scopes.get(&location) {
                    push_scope(scope, &mut scopes, &mut seen);
                }
            }
            for location in variable.borrow().write_locations() {
                if let Some((_, scope)) = self.location_scopes.get(&location) {
                    push_scope(scope, &mut scopes, &mut seen);
                }
            }
            if let Some(declaration) = variable.borrow().declaration() {
                if let Some((_, scope)) = self.location_scopes.get(declaration) {
                    push_scope(scope, &mut scopes, &mut seen);
                }
            }
            let name = (slot, (self.naming_strategy)(slot, &scopes));
            self.variable_map.insert(key, name);
        }

        let (slot, new_name) = self.variable_map[&key].clone();

        // If this is the last use of this variable, then we won't be needing
        // it for the rest of the code so we can reuse the number it was
        // using (the C# AncestorsAndSelf check maps to the node identity —
        // the visits are identifier-level).
        let last_use = self.get_last_use(&variable);
        if let Some(last_use_id) = last_use {
            if node.id == last_use_id {
                self.slot_allocator.release_slot(slot);
            }
        }

        Some(new_name)
    }
}
