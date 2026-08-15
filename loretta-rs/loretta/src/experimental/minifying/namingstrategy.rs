// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.NamingStrategy (b767b4e): NamingStrategy
// C# source: src/Compilers/Lua/Experimental/Minifying/NamingStrategy.cs

/// The naming strategy to use to convert a slot into a variable name.
/// Uses the provided scope to check if the variable name is not being used already.
///
/// C# original: `delegate string NamingStrategy(int slot, IEnumerable<IScope> scopes)`
/// IScope is not yet ported — this will be updated when IScope lands.
/// The boxed-closure representation allows capture-capable factories such as
/// `NamingStrategies::sequential`.
pub type NamingStrategy = Box<dyn Fn(i32) -> String>;
