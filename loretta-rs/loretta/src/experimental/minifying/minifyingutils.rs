// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.MinifyingUtils (b767b4e): MinifyingUtils
// C# source: src/Compilers/Lua/Experimental/Minifying/MinifyingUtils.cs

use crate::scoping::variablekind::VariableKind;
use std::collections::BTreeSet;

/// The IScope surface used by MinifyingUtils.
/// IScope lands with the scoping SCC cluster; the variables are projected as
/// (name, kind, has_declaration) triples until then.
pub trait ScopeVariables {
    /// The scope's declared variables: (name, kind, has-declaration).
    fn declared_variables(&self) -> Vec<(String, VariableKind, bool)>;

    /// The containing scope, if any.
    fn containing_scope(&self) -> Option<&dyn ScopeVariables>;
}

/// A class with helper methods for minifying.
pub struct MinifyingUtils;

impl MinifyingUtils {
    /// Returns whether this is a variable we are able to rename or not.
    /// C# takes an `IVariable`; the port projects `Kind` and
    /// `Declaration is not null` (IVariable lands with the scoping SCC).
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
    pub fn get_unavailable_names(scopes: &[&dyn ScopeVariables]) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        for scope in scopes {
            result.extend(Self::get_unavailable_names_from_scope(*scope));
        }
        result
    }

    /// Returns the set of variable names that are <b>not</b> available in the
    /// provided scope (walking the containing-scope chain).
    pub fn get_unavailable_names_from_scope(scope: &dyn ScopeVariables) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut current = Some(scope);
        while let Some(scope) = current {
            for (name, kind, has_declaration) in scope.declared_variables() {
                if Self::can_rename(kind, has_declaration) {
                    continue;
                }
                result.insert(name);
            }
            match scope.containing_scope() {
                Some(containing) => current = Some(containing),
                None => break,
            }
        }
        result
    }
}
