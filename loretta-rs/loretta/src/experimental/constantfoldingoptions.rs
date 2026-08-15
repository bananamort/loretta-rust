// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFoldingOptions (b767b4e): ConstantFoldingOptions
// C# source: src/Compilers/Lua/Experimental/ConstantFoldingOptions.cs

/// Settings to use when constant folding.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConstantFoldingOptions {
    pub extract_numbers_from_strings: bool,
}

impl ConstantFoldingOptions {
    /// The default, most conservative, preset.
    pub const DEFAULT: Self = Self {
        extract_numbers_from_strings: false,
    };

    /// The preset with everything set to true.
    pub const ALL: Self = Self {
        extract_numbers_from_strings: true,
    };
}
