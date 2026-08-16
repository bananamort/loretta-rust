// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.MinifyingUtils (b767b4e): MinifyingUtils
// C# source: src/Compilers/Lua/Experimental/Minifying/MinifyingUtils.cs

use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use crate::scoping::iscope::{IScope, Scope};
use crate::scoping::ivariable::{IVariable, SharedVariable};
use crate::scoping::variablekind::VariableKind;

/// A class with helper methods for minifying (C# MinifyingUtils.cs:6).
pub struct MinifyingUtils;

impl MinifyingUtils {
    /// C# CanRename (MinifyingUtils.cs:13-19): whether the variable is
    /// renameable (a local/parameter/iteration with a declaration).
    pub fn can_rename(variable: &SharedVariable) -> bool {
        if !matches!(
            variable.borrow().kind(),
            VariableKind::Iteration | VariableKind::Local | VariableKind::Parameter
        ) {
            return false;
        }
        if variable.borrow().declaration().is_none() {
            return false;
        }
        true
    }

    /// C# GetUnavailableNames(IEnumerable<IScope>) (MinifyingUtils.cs:21-29).
    pub fn get_unavailable_names(scopes: &[Rc<RefCell<Scope>>]) -> HashSet<String> {
        let mut set = HashSet::new();
        for scope in scopes {
            set.extend(Self::get_unavailable_names_in_scope(scope));
        }
        set
    }

    /// C# GetUnavailableNames(IScope) (MinifyingUtils.cs:31-48): the names
    /// of the non-renameable declared variables along the containing chain.
    pub fn get_unavailable_names_in_scope(scope: &Rc<RefCell<Scope>>) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut current = Some(scope.clone());
        while let Some(s) = current {
            for variable in s.borrow().declared_variables() {
                if Self::can_rename(&variable) {
                    continue;
                }
                result.insert(variable.borrow().name().to_string());
            }
            current = s.borrow().containing_scope();
        }
        result
    }
}
