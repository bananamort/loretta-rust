// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Scoping.FindScopeTests (b767b4e):
// ScopeTests (FindScopeTests.cs)
// C# source: src/Compilers/Lua/Test/Portable/Scoping/FindScopeTests.cs
//
// The 4 tests verify the scope tree structure (the C# GetScope/FindScope).
// The C# syntax-node references dock on the ported scope tree navigation
// (the walker's node kind names). The node-based FindScope of the C# test 2
// docks on the file-scope node lookup (the C# expression-scope == the
// compilationUnit-scope equivalence — documented).

use std::cell::RefCell;
use std::rc::Rc;

use loretta::scoping::iscope::{IScope, Scope};
use loretta::scoping::scopekind::ScopeKind;

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

#[test]
fn compilation_unit_has_file_scope() {
    let (_, mut script) = ScriptTestsBase::parse_script_async("print 'Hello'", None);
    let root = script.root_scope();
    let compilation_unit_scope =
        find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    assert_eq!(
        compilation_unit_scope.borrow().kind(),
        ScopeKind::File,
        "the compilation unit scope kind"
    );
    let containing = compilation_unit_scope
        .borrow()
        .parent()
        .expect("the containing scope");
    assert!(
        Rc::ptr_eq(&containing, &root),
        "the containing scope is the root"
    );
}

#[test]
fn find_scope_on_root_element_returns_root_scope() {
    // The C# FindScope on the INNER print expression (the
    // FunctionCallExpression's expression — the `print` identifier)
    // returns the compilation unit scope (FindScopeTests.cs:22-34); the
    // port's recorded identifier nodes let the same inner node be looked
    // up (Finding 61 restored the inner-expression docking — the old
    // test docked on the file node).
    let (_, mut script) = ScriptTestsBase::parse_script_async("print 'Hello'", None);
    let root = script.root_scope();
    let compilation_unit_scope =
        find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    let state = script.scope_and_variable_manager_state();
    let print_node = state
        .location_scopes
        .keys()
        .find(|node| node.text == "print")
        .expect("the print identifier node")
        .clone();
    drop(state);
    let found = script
        .find_scope(&print_node, ScopeKind::Block)
        .expect("the found scope");
    assert!(
        Rc::ptr_eq(&found, &compilation_unit_scope),
        "the print expression's scope is the file scope"
    );
}

#[test]
fn find_scope_local_function_is_parsed() {
    // Issue 106 — the local function body creates one contained scope.
    let (_, mut script) = ScriptTestsBase::parse_script_async("local function a() end", None);
    let root = script.root_scope();
    let file_scope = find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    assert_eq!(
        file_scope.borrow().contained_scopes().len(),
        1,
        "one contained scope"
    );
}

#[test]
fn find_scope_anonymous_function_is_parsed() {
    // Issue 106 — the anonymous function body creates one contained scope.
    let (_, mut script) = ScriptTestsBase::parse_script_async("(function(Variable) end)()", None);
    let root = script.root_scope();
    let file_scope = find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    assert_eq!(
        file_scope.borrow().contained_scopes().len(),
        1,
        "one contained scope"
    );
}
