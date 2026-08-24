// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Scoping.RenameVariableTests (b767b4e):
// RenameVariableTests
// C# source: src/Compilers/Lua/Test/Portable/Scoping/RenameVariableTests.cs
//
// The 3 tests verify the Script.RenameVariable results (the ported
// rename_variable + the RenameError variants — renameerrors.rs). The C#
// syntax-node references dock on the declared-variable lookup in the file
// scope (the port's walker tree: the Global root with the File scope child).

use std::cell::RefCell;
use std::rc::Rc;

use loretta::luasyntaxoptions::LuaSyntaxOptions;
use loretta::scoping::iscope::{IScope, Scope};
use loretta::scoping::ivariable::{IVariable, SharedVariable};
use loretta::script::renameerrors::RenameError;
use loretta::script::script::{RenameResult, Script};

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

/// Finds the declared variable with the given name in the file scope.
fn find_variable(script: &mut Script, name: &str) -> SharedVariable {
    let root = script.root_scope();
    let file_scope = find_scope_by_kind(&root, "CompilationUnit").expect("the file scope");
    let variable = file_scope
        .borrow()
        .declared_variables()
        .iter()
        .find(|v| v.borrow().name() == name)
        .cloned()
        .expect("the variable");
    drop(file_scope);
    variable
}

#[test]
fn script_rename_variable_returns_error_for_unsupported_identifier() {
    let (_, mut script) =
        ScriptTestsBase::parse_script_async("local a = 2", Some(&LuaSyntaxOptions::LUA51));
    let variable = find_variable(&mut script, "a");
    let result = script.rename_variable(&variable, "\u{FEFF}");
    match result {
        RenameResult::Err(errors) => {
            assert_eq!(errors.len(), 1, "one error: {errors:?}");
            match &errors[0] {
                RenameError::IdentifierNameNotSupported {
                    tree_without_support,
                } => {
                    // The C# asserts the TreeWithoutSupport payload is the
                    // affected tree (RenameVariableTests.cs:16-18) —
                    // Finding 61 restored the payload assertion.
                    assert_eq!(
                        tree_without_support, "local a = 2",
                        "the tree without support"
                    );
                }
                other => panic!("not an identifier error: {other:?}"),
            }
        }
        other => panic!("expected the error: {other:?}"),
    }
}

#[test]
fn script_rename_variable_returns_error_for_conflicting_variable() {
    let (_, mut script) =
        ScriptTestsBase::parse_script_async("local a, b = 2, 3", Some(&LuaSyntaxOptions::LUA51));
    let variable_a = find_variable(&mut script, "a");
    let variable_b = find_variable(&mut script, "b");
    let result = script.rename_variable(&variable_a, "b");
    match result {
        RenameResult::Err(errors) => {
            assert_eq!(errors.len(), 1, "one error: {errors:?}");
            match &errors[0] {
                RenameError::VariableConflict {
                    variable_being_conflicted_with,
                    ..
                } => {
                    assert!(
                        Rc::ptr_eq(variable_being_conflicted_with, &variable_b),
                        "the conflicting variable is b"
                    );
                }
                other => panic!("not a conflict error: {other:?}"),
            }
        }
        other => panic!("expected the error: {other:?}"),
    }
}

#[test]
fn script_rename_variable_returns_correctly_renamed_script() {
    let (_, mut script) = ScriptTestsBase::parse_script_async(
        "local a = 2\r\nlocal function a() end",
        Some(&LuaSyntaxOptions::LUA51),
    );
    let variable = find_variable(&mut script, "a");
    let result = script.rename_variable(&variable, "b");
    match result {
        RenameResult::Ok(new_script) => {
            let trees = new_script.syntax_trees();
            assert_eq!(trees.len(), 1, "one tree");
            assert_eq!(
                trees[0], "local b = 2\r\nlocal function a() end",
                "the renamed text"
            );
        }
        other => panic!("expected the renamed script: {other:?}"),
    }
}
