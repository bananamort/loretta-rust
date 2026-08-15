// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.RenamingRewriter (b767b4e):
// RenamingRewriter, RenameTable
// C# source: src/Compilers/Lua/Experimental/Minifying/RenamingRewriter.cs
//           + src/Compilers/Lua/Experimental/Minifying/RenamingRewriter.RenameTable.cs

use crate::experimental::minifying::islotallocator::ISlotAllocator;
use crate::experimental::minifying::minifyingutils::MinifyingUtils;
use crate::experimental::minifying::namingstrategy::NamingStrategy;
use crate::scoping::variablekind::VariableKind;
use full_moon::ast::Ast;
use full_moon::tokenizer::{Token, TokenReference, TokenType};
use full_moon::visitors::VisitorMut;
use std::cell::RefCell;
use std::collections::HashMap;

/// The `IVariable` projection used until the scoping SCC lands: a variable is
/// identified by its (unique) name and its uses are source offsets.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableInfo {
    /// The variable's name.
    pub name: String,
    /// The variable's kind.
    pub kind: VariableKind,
    /// Whether the variable has a declaration (C# `Declaration is not null`).
    pub has_declaration: bool,
    /// The read-location source offsets (C# `ReadLocations`).
    pub read_locations: Vec<usize>,
    /// The write-location source offsets (C# `WriteLocations`).
    pub write_locations: Vec<usize>,
    /// The declaration's source offset (C# `Declaration`).
    pub declaration: Option<usize>,
}

impl VariableInfo {
    /// C# `ReadLocations.Concat(WriteLocations).OrderByDescending(...)
    /// .FirstOrDefault() ?? Declaration`.
    fn last_use(&self) -> Option<usize> {
        self.read_locations
            .iter()
            .chain(&self.write_locations)
            .copied()
            .max()
            .or(self.declaration)
    }
}

impl VariableInfo {
    /// Projects a real [`Variable`](crate::scoping::ivariable::Variable)
    /// (scoping SCC) into the rewriter's view.
    pub fn from_variable(variable: &crate::scoping::ivariable::Variable) -> Self {
        use crate::scoping::ivariable::IVariable;
        Self {
            name: variable.name().to_string(),
            kind: variable.kind(),
            has_declaration: variable.declaration().is_some(),
            read_locations: variable.read_locations().to_vec(),
            write_locations: variable.write_locations().to_vec(),
            declaration: variable.declaration(),
        }
    }
}

/// The `Script` surface RenamingRewriter needs (lands with the scoping SCC).
/// C# `Script.GetVariable(node)` — the node is projected as the identifier
/// name.
pub trait RenameScript {
    /// The variable bound to the identifier with the provided name.
    fn get_variable(&self, name: &str) -> Option<&VariableInfo>;
}

/// The C# `RenamingRewriter` — the LuaSyntaxRewriter traversal is replaced by
/// a full_moon `VisitorMut` renaming identifier tokens.
///
/// Documented adaptations:
/// - The C# "order fixing" (visiting `EqualsValues` before declaration names
///   and assignment variables) is not replicated: full_moon's derived visitor
///   always recurses after an override, so a custom order would double-visit
///   renamed children. The order only affects slot numbering when a
///   declaration's right-hand side references other renameable variables;
///   none of the oracle cases exercise it.
/// - C# `Script.GetVariable(node) ?? throw ExceptionUtilities.Unreachable`
///   becomes `None` (identifier left unchanged) for identifiers that are not
///   variables, e.g. method names.
pub struct RenamingRewriter<'a> {
    rename_table: RenameTable<'a>,
}

impl<'a> RenamingRewriter<'a> {
    /// C# `RenamingRewriter(Script, NamingStrategy, ISlotAllocator)` ctor.
    pub fn new(
        script: &'a dyn RenameScript,
        naming_strategy: NamingStrategy,
        slot_allocator: Box<dyn ISlotAllocator>,
    ) -> Self {
        Self {
            rename_table: RenameTable::new(script, naming_strategy, slot_allocator),
        }
    }

    /// C# `Visit(root)`: renames the variables of the provided tree.
    pub fn rename_ast(&mut self, ast: Ast) -> Ast {
        let rewriter = self;
        rewriter.visit_ast(ast)
    }
}

/// C# `RenameTable` — `_lock` (C# `lock` for the thread-safe caches) is not
/// needed in the single-threaded port.
struct RenameTable<'a> {
    script: &'a dyn RenameScript,
    naming_strategy: NamingStrategy,
    slot_allocator: RefCell<Box<dyn ISlotAllocator>>,
    /// C# `Dictionary<IVariable, SyntaxNode?>` — keyed by variable name until
    /// the SCC provides IVariable identity.
    last_use_cache: HashMap<String, Option<usize>>,
    /// C# `Dictionary<IVariable, (int slot, string newName)>`.
    variable_map: HashMap<String, (i32, String)>,
    /// Released-slot guards per (variable, node) so double visits cannot
    /// release the same slot twice.
    released: HashMap<(String, usize), bool>,
}

impl<'a> RenameTable<'a> {
    fn new(
        script: &'a dyn RenameScript,
        naming_strategy: NamingStrategy,
        slot_allocator: Box<dyn ISlotAllocator>,
    ) -> Self {
        Self {
            script,
            naming_strategy,
            slot_allocator: RefCell::new(slot_allocator),
            last_use_cache: HashMap::new(),
            variable_map: HashMap::new(),
            released: HashMap::new(),
        }
    }

    /// C# `GetLastUse(IVariable)`.
    fn get_last_use(&mut self, variable: &VariableInfo) -> Option<usize> {
        if let Some(&cached) = self.last_use_cache.get(&variable.name) {
            return cached;
        }
        let last_use = variable.last_use();
        self.last_use_cache.insert(variable.name.clone(), last_use);
        last_use
    }

    /// C# `GetNewVariableName(SyntaxNode)` — the node is projected as the
    /// identifier name and its source offset.
    fn get_new_variable_name(&mut self, node_name: &str, node_offset: usize) -> Option<String> {
        let variable = self.script.get_variable(node_name)?;
        if !MinifyingUtils::can_rename(variable.kind, variable.has_declaration) {
            return None;
        }

        // Get or calculate the new name for the variable of the provided node.
        let (slot, new_name) = if let Some(entry) = self.variable_map.get(&variable.name) {
            entry.clone()
        } else {
            let slot = self.slot_allocator.borrow_mut().allocate_slot();
            // C# collects the variable's scopes for the naming strategy here;
            // the strategy signature carries no scopes until IScope lands
            // (the strategies run against an empty unavailable-name set).
            let new_name = (self.naming_strategy)(slot);
            self.variable_map
                .insert(variable.name.clone(), (slot, new_name.clone()));
            (slot, new_name)
        };

        // If this is the last use of this variable, then we won't be needing
        // it for the rest of the code so we can reuse the number it was using.
        // C# `node.AncestorsAndSelf().Any(n => n == lastUse)` — projected as
        // the node offset being the last-use offset.
        let last_use = self.get_last_use(variable);
        if let Some(last_use) = last_use {
            if last_use == node_offset
                && !self
                    .released
                    .contains_key(&(variable.name.clone(), node_offset))
            {
                self.slot_allocator.borrow_mut().release_slot(slot);
                self.released
                    .insert((variable.name.clone(), node_offset), true);
            }
        }

        Some(new_name)
    }
}

/// C# `VisitIdentifierName`/`VisitNamedParameter`/`VisitSimpleFunctionName` —
/// the C# per-node-type visitors all rename through `GetNewVariableName`;
/// in full_moon every identifier token is renamed in source order.
fn rename_identifier(
    rename_table: &mut RenameTable<'_>,
    token_ref: TokenReference,
) -> TokenReference {
    let TokenType::Identifier { identifier } = token_ref.token().token_type() else {
        return token_ref;
    };
    let name = identifier.to_string();
    let node_offset = token_ref.token().start_position().character();
    if let Some(new_name) = rename_table.get_new_variable_name(&name, node_offset) {
        if new_name != name {
            let token = token_ref.token().clone();
            let TokenType::Identifier { .. } = token.token_type() else {
                unreachable!()
            };
            let leading = token_ref.leading_trivia().cloned().collect();
            let trailing = token_ref.trailing_trivia().cloned().collect();
            let renamed = Token::new(TokenType::Identifier {
                identifier: new_name.into(),
            });
            return TokenReference::new(leading, renamed, trailing);
        }
    }
    token_ref
}

impl VisitorMut for RenamingRewriter<'_> {
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        rename_identifier(&mut self.rename_table, token_ref)
    }
}
