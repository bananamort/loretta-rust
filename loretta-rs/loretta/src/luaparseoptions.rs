// Ported from Loretta.CodeAnalysis.Lua.LuaParseOptions (b767b4e): LuaParseOptions
// C# source: src/Compilers/Lua/Portable/LuaParseOptions.cs

use crate::luasyntaxoptions::LuaSyntaxOptions;

/// Stores source parsing related options and offers access to their values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaParseOptions {
    /// The LuaSyntaxOptions to use when parsing.
    pub syntax_options: LuaSyntaxOptions,
}

impl LuaParseOptions {
    /// The default parse options.
    pub fn default_options() -> Self {
        Self {
            syntax_options: LuaSyntaxOptions::ALL,
        }
    }

    /// Creates a new set of parse options.
    pub fn new(syntax_options: LuaSyntaxOptions) -> Self {
        Self { syntax_options }
    }

    /// Creates a new instance with the syntax options replaced by the provided ones.
    pub fn with_syntax_options(&self, syntax_options: LuaSyntaxOptions) -> Self {
        if self.syntax_options != syntax_options {
            Self { syntax_options }
        } else {
            self.clone()
        }
    }
}
