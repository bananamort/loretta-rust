// Pending port — Loretta.CodeAnalysis.Lua scoping (b767b4e). One file per graph node will land here.

pub mod ifilescope;
pub mod ifunctionscope;
pub mod igotolabel;
pub mod iscope;
pub mod ivariable;
pub mod node;
pub mod scopekind;
pub mod variablekind;

pub use ifilescope::{FileScopeData, IFileScope};
pub use ifunctionscope::{FunctionScopeData, IFunctionScope};
pub use iscope::{IScope, Scope};
pub use ivariable::{IVariable, IVariableInternal, SharedVariable, Variable};
pub use node::Node;
pub use scopekind::ScopeKind;
pub use variablekind::VariableKind;
