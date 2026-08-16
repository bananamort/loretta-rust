// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.NamingStrategy (b767b4e): NamingStrategy
// C# source: src/Compilers/Lua/Experimental/Minifying/NamingStrategy.cs

use std::cell::RefCell;
use std::rc::Rc;

use crate::scoping::iscope::Scope;

/// The naming strategy to use to convert a slot into a variable name.
/// Uses the provided scope to check if the variable name is not being used
/// already.
///
/// C# original: `delegate string NamingStrategy(int slot, IEnumerable<IScope> scopes)`
pub type NamingStrategy = Box<dyn Fn(i32, &[Rc<RefCell<Scope>>]) -> String>;
