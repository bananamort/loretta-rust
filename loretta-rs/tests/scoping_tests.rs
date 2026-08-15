// Ported from Compilers/Lua/Test/Portable/Scoping (b767b4e) — the direct
// Scope/Variable primitives covered by CanBeAccessedInTests (via Script) and
// the FindVariable kind-limit semantics from IScope.cs. The Script-level
// tests (FindScopeTests/FindVariableTests/RenameVariableTests) land with the
// Script port.

use loretta::scoping::iscope::{IScope, Scope};
use loretta::scoping::ivariable::IVariable;
use loretta::scoping::scopekind::ScopeKind;
use loretta::scoping::variablekind::VariableKind;

#[test]
fn can_be_accessed_in_same_scope() {
    let block = Scope::new(ScopeKind::Block, Some(0), None);
    let variable =
        block
            .borrow_mut()
            .create_variable(&block, VariableKind::Local, "a".to_string(), Some(10));
    assert!(variable.borrow().can_be_accessed_in(&block));
}

#[test]
fn can_be_accessed_in_child_scope() {
    let file = Scope::new_file(None, None);
    let do_block = Scope::new(ScopeKind::Block, Some(4), Some(file.clone()));
    let variable =
        file.borrow_mut()
            .create_variable(&file, VariableKind::Local, "a".to_string(), Some(0));
    // The child scope's chain (do_block -> file) reaches the variable's scope.
    assert!(variable.borrow().can_be_accessed_in(&do_block));
}

#[test]
fn can_be_accessed_in_returns_false_when_scope_is_parent_of_parent() {
    let file = Scope::new_file(None, None);
    let do_block = Scope::new(ScopeKind::Block, Some(4), Some(file.clone()));
    let variable = do_block.borrow_mut().create_variable(
        &do_block,
        VariableKind::Local,
        "a".to_string(),
        Some(8),
    );
    // The file's chain (file -> global) never reaches the block.
    assert!(!variable.borrow().can_be_accessed_in(&file));
}

#[test]
fn find_variable_searches_by_scope_kind() {
    let global = Scope::new(ScopeKind::Global, None, None);
    let file = Scope::new_file(None, Some(global.clone()));
    let function = Scope::new_function(Some(2), Some(file.clone()));
    let block = Scope::new(ScopeKind::Block, Some(8), Some(function.clone()));
    function
        .borrow_mut()
        .create_variable(&function, VariableKind::Local, "a".to_string(), Some(3));

    // Block searches only blocks.
    assert!(block
        .borrow()
        .find_variable("a", ScopeKind::Block)
        .is_none());
    // Function searches functions and blocks.
    assert!(block
        .borrow()
        .find_variable("a", ScopeKind::Function)
        .is_some());
    // File/Global search everything up.
    assert!(block.borrow().find_variable("a", ScopeKind::File).is_some());
    assert!(block
        .borrow()
        .find_variable("a", ScopeKind::Global)
        .is_some());
}

#[test]
fn file_scope_has_implicit_variables() {
    let file = Scope::new_file(None, None);
    let file_borrowed = file.borrow();
    assert_eq!(file_borrowed.arg_variable().borrow().name(), "arg");
    assert_eq!(file_borrowed.var_arg_parameter().borrow().name(), "...");
    assert_eq!(
        file_borrowed.arg_variable().borrow().kind(),
        VariableKind::Parameter
    );
    // The implicit variables are declared in the file scope.
    assert!(file_borrowed
        .find_variable("arg", ScopeKind::Global)
        .is_some());
    // C# FindVariable("...") would throw (not a valid identifier), so the
    // vararg is verified by name only.
    assert_eq!(file_borrowed.var_arg_parameter().borrow().name(), "...");
}

#[test]
fn function_scope_parameters_and_capturing() {
    let global = Scope::new(ScopeKind::Global, None, None);
    let file = Scope::new_file(None, Some(global.clone()));
    let function = Scope::new_function(Some(2), Some(file.clone()));
    function
        .borrow_mut()
        .add_parameter(&function, "x".to_string(), Some(8));

    let parameter = function.borrow().parameters()[0].clone();
    assert_eq!(parameter.borrow().name(), "x");
    assert_eq!(parameter.borrow().kind(), VariableKind::Parameter);

    // A variable declared in the file scope and referenced from the function
    // becomes a captured variable of the function.
    let outer =
        file.borrow_mut()
            .create_variable(&file, VariableKind::Local, "outer".to_string(), Some(1));
    Scope::add_referenced_variable(&function, &outer);
    assert_eq!(function.borrow().captured_variables().len(), 1);
    assert!(Rc::ptr_eq(
        &function.borrow().captured_variables()[0],
        &outer
    ));
    assert_eq!(outer.borrow().capturing_scopes().len(), 1);
}

#[test]
fn find_variable_rejects_non_identifiers() {
    let block = Scope::new(ScopeKind::Block, Some(0), None);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        block
            .borrow()
            .find_variable("not an identifier", ScopeKind::Global)
    }));
    assert!(result.is_err());
}

use std::rc::Rc;
