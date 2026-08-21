// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Scoping.CanBeAccessedInTests (b767b4e):
// CanBeAccessedInTests
// C# source: src/Compilers/Lua/Test/Portable/Scoping/CanBeAccessedInTests.cs
//
// The 3 tests verify the Variable.CanBeAccessedIn semantics (IVariable.cs:
// 115-123 — the scope chain walk). The C# syntax-node references dock on the
// ported scope tree navigation (the walker's node kind names); the variables
// are found in the scopes' declared-variable lists.

use std::cell::RefCell;
use std::rc::Rc;

use loretta::scoping::iscope::{IScope, Scope};
use loretta::scoping::ivariable::{IVariable, SharedVariable};

use loretta_tests::scripttestsbase::ScriptTestsBase;

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

/// Finds the declared variable with the given name in the scope.
fn find_declared_variable(scope: &Rc<RefCell<Scope>>, name: &str) -> Option<SharedVariable> {
    scope
        .borrow()
        .declared_variables()
        .iter()
        .find(|v| v.borrow().name() == name)
        .cloned()
}

#[test]
fn script_can_be_accessed_in_returns_true_when_same_scope() {
    let (_, mut script) = ScriptTestsBase::parse_script_async("local a = 1 print(a)", None);
    let root = script.root_scope();
    // The port's walker tree: the Global root with the File scope child (the
    // C# root = the CompilationUnit — the port's "CompilationUnit" scope).
    let file_scope = find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    let variable = find_declared_variable(&file_scope, "a").expect("the variable a");
    assert!(
        variable.borrow().can_be_accessed_in(&file_scope),
        "the variable is accessible in its own scope"
    );
}

#[test]
fn script_can_be_accessed_in_returns_true_when_scope_is_child() {
    let (_, mut script) =
        ScriptTestsBase::parse_script_async("local a = 1\r\ndo\r\n    print(a)\r\nend", None);
    let root = script.root_scope();
    let file_scope = find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    let variable = find_declared_variable(&file_scope, "a").expect("the variable a");
    let do_scope = find_scope_by_kind(&root, "DoStatement").expect("the do scope");
    assert!(
        variable.borrow().can_be_accessed_in(&do_scope),
        "the variable is accessible in the child scope"
    );
}

#[test]
fn script_can_be_accessed_in_returns_false_when_scope_is_parent_of_parent() {
    let (_, mut script) = ScriptTestsBase::parse_script_async("do\r\n    local a = 1\r\nend", None);
    let root = script.root_scope();
    let file_scope = find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    let do_scope = find_scope_by_kind(&root, "DoStatement").expect("the do scope");
    let variable = find_declared_variable(&do_scope, "a").expect("the variable a");
    assert!(
        !variable.borrow().can_be_accessed_in(&file_scope),
        "the variable is not accessible in the parent-of-parent scope"
    );
}
