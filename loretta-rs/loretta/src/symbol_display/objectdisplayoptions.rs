// Ported from Loretta.CodeAnalysis.ObjectDisplayOptions (b767b4e): ObjectDisplayOptions
// C# source: src/Compilers/Core/Portable/SymbolDisplay/ObjectDisplayOptions.cs

/// Specifies the options for how objects are displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjectDisplayOptions(u8);

impl ObjectDisplayOptions {
    /// Format object using default options.
    pub const NONE: Self = Self(0);

    /// Whether or not to display integral literals in hexadecimal.
    pub const USE_HEXADECIMAL_NUMBERS: Self = Self(1 << 0);

    /// Whether or not to quote string literals.
    pub const USE_QUOTES: Self = Self(1 << 1);

    /// Replace non-printable (e.g. control) characters with dedicated (e.g. \t) or unicode (\u0001) escape sequences.
    pub const ESCAPE_NON_PRINTABLE_CHARACTERS: Self = Self(1 << 2);

    /// Escapes characters using their UTF8 encoding instead of unicode escapes.
    pub const ESCAPE_WITH_UTF8: Self = Self(1 << 3);

    /// Determines if a flag is set.
    #[inline]
    pub fn includes_option(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }
}

impl std::ops::BitOr for ObjectDisplayOptions {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}
