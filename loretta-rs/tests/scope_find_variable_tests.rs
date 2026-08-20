// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Scoping.FindVariableTests (b767b4e):
// FindVariableTests
// C# source: src/Compilers/Lua/Test/Portable/Scoping/FindVariableTests.cs
//
// The 2 tests verify the Scope.FindVariable semantics (the C# IScope.
// FindVariable). The C# syntax-node references dock on the ported scope tree
// navigation (the walker's node kind names); the innermost scope is the do
// block's scope.

use std::cell::RefCell;
use std::rc::Rc;

use loretta::scoping::iscope::{IScope, Scope};
use loretta::scoping::scopekind::ScopeKind;
use loretta::script::script::Script;

use loretta_tests::scripttestsbase::ScriptTestsBase;

/// The setup: the two-tree script with the innermost do-block scope (the C#
/// SetupScriptAsync).
fn setup_script() -> (Script, Rc<RefCell<Scope>>) {
    let mut script = ScriptTestsBase::parse_script_async_many(
        &loretta::luasyntaxoptions::LuaSyntaxOptions::ALL,
        &[
            "local a = 1\nfunction f(b)\n    print(b)\n    do\n        local c = 3\n    end\nend",
            "glob = 2",
        ],
    );
    let root = script.root_scope();
    let inner_most = find_scope_by_kind(&root, "DoStatement").expect("the do scope");
    (script, inner_most)
}

/// Finds the scope with the given walker node kind name.
fn find_scope_by_kind(scope: &Rc<RefCell<Scope>>, kind_name: &str) -> Option<Rc<RefCell<Scope>>> {
    if scope.borrow().node().map(|n| n.kind_name()) == Some(kind_name) {
        return Some(scope.clone());
    }
    for child in scope.borrow().contained_scopes() {
        if let Some(found) = find_scope_by_kind(&child, kind_name) {
            return Some(found);
        }
    }
    None
}

#[test]
fn script_find_variable_returns_null_when_no_variable_is_available() {
    let (_, inner_most_scope) = setup_script();
    let cases: &[(ScopeKind, &str)] = &[
        (ScopeKind::File, "glob"),
        (ScopeKind::Function, "a"),
        (ScopeKind::Block, "b"),
    ];
    for (scope_kind, name) in cases {
        assert!(
            inner_most_scope
                .borrow()
                .find_variable(name, *scope_kind)
                .is_none(),
            "find_variable({name:?}, {scope_kind:?}) must be none"
        );
    }
}

#[test]
fn script_find_variable_returns_variable_when_variable_is_available() {
    let (_, inner_most_scope) = setup_script();
    let cases: &[(ScopeKind, &str)] = &[
        (ScopeKind::Global, "glob"),
        (ScopeKind::Global, "a"),
        (ScopeKind::File, "a"),
        (ScopeKind::Global, "b"),
        (ScopeKind::File, "b"),
        (ScopeKind::Function, "b"),
        (ScopeKind::Global, "c"),
        (ScopeKind::File, "c"),
        (ScopeKind::Function, "c"),
        (ScopeKind::Block, "c"),
    ];
    for (scope_kind, name) in cases {
        assert!(
            inner_most_scope
                .borrow()
                .find_variable(name, *scope_kind)
                .is_some(),
            "find_variable({name:?}, {scope_kind:?}) must be some"
        );
    }
}
