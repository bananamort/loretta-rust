// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.NamingStrategy (b767b4e): NamingStrategy
// C# source: src/Compilers/Lua/Experimental/Minifying/NamingStrategy.cs

/// The naming strategy to use to convert a slot into a variable name.
/// Uses the provided scope to check if the variable name is not being used already.
///
/// C# original: `delegate string NamingStrategy(int slot, IEnumerable<IScope> scopes)`
/// IScope is not yet ported — this will be updated when IScope lands.
pub type NamingStrategy = fn(slot: i32) -> String;
