use std::cell::RefCell;
use std::rc::Rc;

use loretta::scoping::iscope::{IScope, Scope};
use loretta::scoping::ivariable::SharedVariable;
use loretta::script::script::{RenameResult, Script};
use loretta_fuzz::{fuzz_input, run_iters};

/// Returns the first declared variable found in the scope tree, if any.
fn find_first_variable(scope: &Rc<RefCell<Scope>>) -> Option<SharedVariable> {
    let borrowed = scope.borrow();
    if let Some(variable) = borrowed.declared_variables().first() {
        return Some(variable.clone());
    }
    for child in borrowed.contained_scopes() {
        if let Some(variable) = find_first_variable(&child) {
            return Some(variable);
        }
    }
    None
}

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // host = embed loretta as a host library: build a Script from the input,
    // locate a declared variable in the scope tree and rename it, then
    // materialize the renamed syntax trees (the IDE-host workflow).
    let mut script = Script::new(vec![text.into_owned()]);
    let root = script.root_scope();
    if let Some(variable) = find_first_variable(&root) {
        match script.rename_variable(&variable, "renamed") {
            RenameResult::Ok(new_script) => {
                let _ = new_script.syntax_trees();
            }
            RenameResult::Err(errors) => {
                let _ = errors.len();
            }
        }
    }
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("host", iters, seed, target);
}
