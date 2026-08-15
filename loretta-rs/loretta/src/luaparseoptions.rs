// Ported from Loretta.CodeAnalysis.Lua.LuaParseOptions (b767b4e): LuaParseOptions
// C# source: src/Compilers/Lua/Portable/LuaParseOptions.cs

use crate::luasyntaxoptions::LuaSyntaxOptions;

/// Stores source parsing related options and offers access to their values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaParseOptions {
    /// The LuaSyntaxOptions to use when parsing.
    pub syntax_options: LuaSyntaxOptions,
    /// C# `_features`: ImmutableDictionary<string, string>, always empty by
    /// default ("the features flag doesn't do anything currently").
    pub features: Vec<(String, String)>,
}

impl LuaParseOptions {
    /// The default parse options.
    pub fn default_options() -> Self {
        Self {
            syntax_options: LuaSyntaxOptions::ALL,
            features: Vec::new(),
        }
    }

    /// The documentation mode. C# `DocumentationMode` "does nothing
    /// currently"; the base ParseOptions ctor fixes it to Parse.
    pub fn documentation_mode(&self) -> &'static str {
        "Parse"
    }

    /// The language name (C# `LanguageNames.Lua`).
    pub fn language(&self) -> &'static str {
        "Lua"
    }

    /// Validates the options, appending diagnostics to the builder.
    /// C# delegates to the Core ParseOptions.ValidateOptions; with
    /// DocumentationMode = Parse (fixed by the ctor) and empty features it
    /// adds nothing — the docs say the options "don't do anything currently".
    pub fn validate_options(&self, _builder: &mut Vec<String>) {}

    /// Creates a new set of parse options.
    pub fn new(syntax_options: LuaSyntaxOptions) -> Self {
        Self {
            syntax_options,
            features: Vec::new(),
        }
    }

    /// Creates a new instance with the features replaced by the provided ones.
    /// C# `WithFeatures` -> `new LuaParseOptions(this) { _features = ... }`.
    pub fn with_features(&self, features: Vec<(String, String)>) -> Self {
        Self {
            syntax_options: self.syntax_options.clone(),
            features,
        }
    }

    /// Creates a new instance with the syntax options replaced by the provided ones.
    pub fn with_syntax_options(&self, syntax_options: LuaSyntaxOptions) -> Self {
        if self.syntax_options != syntax_options {
            Self {
                syntax_options,
                features: self.features.clone(),
            }
        } else {
            self.clone()
        }
    }
}
