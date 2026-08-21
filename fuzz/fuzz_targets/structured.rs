use std::cell::RefCell;
use std::rc::Rc;

use loretta::scoping::iscope::{IScope, Scope};
use loretta::script::script::Script;
use loretta_fuzz::{fuzz_input, run_iters};

/// Recursively counts the scopes and declared variables in the scope tree.
fn count(scope: &Rc<RefCell<Scope>>) -> (usize, usize) {
    let mut scopes = 1;
    let mut variables = 0;
    let borrowed = scope.borrow();
    variables += borrowed.declared_variables().len();
    for child in borrowed.contained_scopes() {
        let (s, v) = count(&child);
        scopes += s;
        variables += v;
    }
    (scopes, variables)
}

fn target(data: &[u8]) {
    let text = String::from_utf8_lossy(data);
    // structured = scope-tree fuzzing: build the ported Script and walk the
    // full scope hierarchy the way an IDE outline would.
    let mut script = Script::new(vec![text.into_owned()]);
    let root = script.root_scope();
    let (scopes, variables) = count(&root);
    let _ = (scopes, variables);
}

fn main() {
    let (iters, seed) = fuzz_input();
    run_iters("structured", iters, seed, target);
}
