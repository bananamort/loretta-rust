// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.MinifyingUtils (b767b4e): MinifyingUtils
// C# source: src/Compilers/Lua/Experimental/Minifying/MinifyingUtils.cs

use crate::scoping::iscope::{IScope, ScopeRef};
use crate::scoping::ivariable::IVariable;
use crate::scoping::variablekind::VariableKind;
use std::collections::BTreeSet;

/// The IScope surface used by MinifyingUtils; the variables are projected as
/// (name, kind, has_declaration) triples.
pub trait ScopeVariables {
    /// The scope's declared variables: (name, kind, has-declaration).
    fn declared_variables(&self) -> Vec<(String, VariableKind, bool)>;

    /// The containing scope, if any.
    fn containing_scope(&self) -> Option<ScopeRef>;
}

/// A class with helper methods for minifying.
pub struct MinifyingUtils;

impl MinifyingUtils {
    /// Returns whether this is a variable we are able to rename or not.
    /// C# takes an `IVariable`; the port projects `Kind` and
    /// `Declaration is not null`.
    pub fn can_rename(kind: VariableKind, has_declaration: bool) -> bool {
        if !matches!(
            kind,
            VariableKind::Iteration | VariableKind::Local | VariableKind::Parameter
        ) {
            return false;
        }
        if !has_declaration {
            return false;
        }
        true
    }

    /// Returns the set of variable names that are <b>not</b> available in the
    /// provided scopes.
    pub fn get_unavailable_names(scopes: &[ScopeRef]) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        for scope in scopes {
            result.extend(Self::get_unavailable_names_from_scope(scope));
        }
        result
    }

    /// Returns the set of variable names that are <b>not</b> available in the
    /// provided scope (walking the containing-scope chain).
    pub fn get_unavailable_names_from_scope(scope: &ScopeRef) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut current: Option<ScopeRef> = Some(scope.clone());
        while let Some(scope_ref) = current {
            let scope = scope_ref.borrow();
            for (name, kind, has_declaration) in scope_ref.declared_variables() {
                if Self::can_rename(kind, has_declaration) {
                    continue;
                }
                result.insert(name);
            }
            match scope.parent_ref() {
                Some(parent) => current = Some(parent),
                None => break,
            }
        }
        result
    }
}

/// The concrete [`Scope`](crate::scoping::iscope::Scope) projection for the
/// minifying utilities (C# `IScope.DeclaredVariables` + `ContainingScope`).
impl ScopeVariables for ScopeRef {
    fn declared_variables(&self) -> Vec<(String, VariableKind, bool)> {
        self.borrow()
            .declared_variables()
            .iter()
            .map(|v| {
                let v = v.borrow();
                (v.name().to_string(), v.kind(), v.declaration().is_some())
            })
            .collect()
    }

    fn containing_scope(&self) -> Option<ScopeRef> {
        self.borrow().parent_ref()
    }
}
