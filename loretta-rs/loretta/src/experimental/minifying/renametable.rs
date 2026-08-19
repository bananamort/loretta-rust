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
    last_use_cache: HashMap<u64, Option<u64>>,
    /// C# _variableMap (RenameTable.cs:11) — (slot, newName).
    variable_map: HashMap<u64, (i32, String)>,
    /// C# _slotAllocator (RenameTable.cs:12).
    slot_allocator: Box<dyn ISlotAllocator>,
    /// The identifier records of the current walk (node id -> (node,
    /// position, scope)) — used for the last-use ordering and the scope
    /// collection.
    records: HashMap<u64, (Node, usize, Rc<RefCell<Scope>>)>,
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
            records: HashMap::new(),
        }
    }

    /// Prepares the identifier records for the walk (the location scopes and
    /// positions).
    pub fn prepare(&mut self, records: &[IdentifierRecord]) {
        self.records = records
            .iter()
            .map(|(node, pos, scope)| (node.id, (node.clone(), *pos, scope.clone())))
            .collect();
    }

    /// The variable's identity key (the declaration node id — unique per
    /// variable; every renameable variable has a declaration).
    fn variable_key(variable: &SharedVariable) -> u64 {
        variable
            .borrow()
            .declaration()
            .expect("renameable variables have a declaration")
            .id
    }

    /// C# GetLastUse (RenameTable.cs:27-41): the location the variable is
    /// last used — the read/write location with the highest source span, or
    /// the declaration.
    pub fn get_last_use(&mut self, variable: &SharedVariable) -> Option<u64> {
        let key = Self::variable_key(variable);
        if !self.last_use_cache.contains_key(&key) {
            let mut best: Option<(usize, u64)> = None;
            for node in variable.borrow().read_locations() {
                if let Some((_, pos, _)) = self.records.get(&node.id) {
                    if best.map(|(b, _)| *pos > b).unwrap_or(true) {
                        best = Some((*pos, node.id));
                    }
                }
            }
            for node in variable.borrow().write_locations() {
                if let Some((_, pos, _)) = self.records.get(&node.id) {
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
            // per location). The statement nodes the variable carries (the
            // declaration and the write-location statements) are not
            // identifier records — the C# FindScope of a statement resolves
            // to the scope of the names it declares, which the variable's
            // own identifier records carry, so a missing location falls back
            // to those record scopes (the C# node identity is shared; the
            // port resolves the statement's scope via the variable).
            let mut scopes: Vec<Rc<RefCell<Scope>>> = Vec::new();
            let mut seen: Vec<usize> = Vec::new();
            let mut fallback: Vec<Rc<RefCell<Scope>>> = Vec::new();
            {
                let snapshot: Vec<(Node, Rc<RefCell<Scope>>)> = self
                    .records
                    .values()
                    .map(|(node, _, scope)| (node.clone(), scope.clone()))
                    .collect();
                for (node, scope) in snapshot {
                    if let Some(v) = self.script.get_variable(&node) {
                        if Rc::ptr_eq(&v, &variable)
                            && !fallback.iter().any(|s| Rc::ptr_eq(s, &scope))
                        {
                            fallback.push(scope);
                        }
                    }
                }
            }
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
                match self.records.get(&location.id) {
                    Some((_, _, scope)) => push_scope(scope, &mut scopes, &mut seen),
                    None => {
                        for scope in &fallback {
                            push_scope(scope, &mut scopes, &mut seen);
                        }
                    }
                }
            }
            for location in variable.borrow().write_locations() {
                match self.records.get(&location.id) {
                    Some((_, _, scope)) => push_scope(scope, &mut scopes, &mut seen),
                    None => {
                        for scope in &fallback {
                            push_scope(scope, &mut scopes, &mut seen);
                        }
                    }
                }
            }
            if let Some(declaration) = variable.borrow().declaration() {
                match self.records.get(&declaration.id) {
                    Some((_, _, scope)) => push_scope(scope, &mut scopes, &mut seen),
                    None => {
                        for scope in &fallback {
                            push_scope(scope, &mut scopes, &mut seen);
                        }
                    }
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
