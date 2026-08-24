// Ported from Loretta.CodeAnalysis.Lua.LuaSyntaxOptions (b767b4e): LuaSyntaxOptions
// C# source: src/Compilers/Lua/Portable/LuaSyntaxOptions.cs

use crate::backtickstringtype::BacktickStringType;
use crate::continuetype::ContinueType;
use crate::integerformats::IntegerFormats;

/// The options used by Loretta to adapt to the syntax of the lua flavor being parsed.
///
/// "Accept" means not generating an error when parsing, but the syntax behind the option
/// will still be parsed normally.
#[derive(Debug, Clone)]
pub struct LuaSyntaxOptions {
    /// Whether to accept binary numbers (format: /0b[10]+/).
    pub accept_binary_numbers: bool,
    /// Whether to accept C comment syntax (formats: "//..." and "/* ... */").
    pub accept_c_comment_syntax: bool,
    /// Whether to accept compound assignment syntax.
    pub accept_compound_assignment: bool,
    /// Whether to accept empty statements (lone semicolons).
    pub accept_empty_statements: bool,
    /// Whether to accept C boolean operators (&&, ||, != and !).
    pub accept_c_boolean_operators: bool,
    /// Whether to accept goto labels and statements.
    pub accept_goto: bool,
    /// Whether to accept hexadecimal escapes in strings.
    pub accept_hex_escapes_in_strings: bool,
    /// Whether to accept hexadecimal floating point literals.
    pub accept_hex_float_literals: bool,
    /// Whether to accept octal numbers (format: /0o[0-7]+/).
    pub accept_octal_numbers: bool,
    /// Whether to accept shebangs (format: "#!...").
    pub accept_shebang: bool,
    /// Whether to accept underscores in any number literals.
    pub accept_underscore_in_number_literals: bool,
    /// Whether to use LuaJIT's identifier character rules.
    pub use_lua_jit_identifier_rules: bool,
    /// Whether to accept 5.3 bitwise operators.
    pub accept_bitwise_operators: bool,
    /// Whether to accept \\z escapes.
    pub accept_whitespace_escape: bool,
    /// Whether to accept Unicode (\\u{XXX}) escapes.
    pub accept_unicode_escape: bool,
    /// The type of continue to be recognized by the parser.
    pub continue_type: ContinueType,
    /// Whether to accept Luau if expressions.
    pub accept_if_expressions: bool,
    /// Whether to support the Lua 5.1 lexer bug where invalid escapes in strings are read as the character in the escape.
    pub accept_invalid_escapes: bool,
    /// Whether to accept Lua 5.4 variable attributes.
    pub accept_local_variable_attributes: bool,
    /// Format binary numeric literals are stored as.
    pub binary_integer_format: IntegerFormats,
    /// Format octal numeric literals are stored as.
    pub octal_integer_format: IntegerFormats,
    /// Format decimal integer literals are stored as.
    pub decimal_integer_format: IntegerFormats,
    /// Format hexadecimal integer literals are stored as.
    pub hex_integer_format: IntegerFormats,
    /// Whether to accept typed lua syntax or not.
    pub accept_typed_lua: bool,
    /// Whether to accept floor division or not.
    pub accept_floor_division: bool,
    /// Whether to accept LuaJIT number suffixes or not.
    pub accept_lua_jit_number_suffixes: bool,
    /// Whether to accept nesting of [[...]].
    pub accept_nesting_of_long_strings: bool,
    /// Defines how strings with backtick delimiters will be parsed.
    pub backtick_string_type: BacktickStringType,
}

/// The C# Equals (LuaSyntaxOptions.cs:660-688) deliberately omits
/// AcceptUnicodeEscape and AcceptInvalidEscapes — two options that do
/// not affect the resulting syntax tree — so the equality follows the
/// C# field list exactly (Finding 42).
impl PartialEq for LuaSyntaxOptions {
    fn eq(&self, other: &Self) -> bool {
        self.accept_binary_numbers == other.accept_binary_numbers
            && self.accept_c_comment_syntax == other.accept_c_comment_syntax
            && self.accept_compound_assignment == other.accept_compound_assignment
            && self.accept_empty_statements == other.accept_empty_statements
            && self.accept_c_boolean_operators == other.accept_c_boolean_operators
            && self.accept_goto == other.accept_goto
            && self.accept_hex_escapes_in_strings == other.accept_hex_escapes_in_strings
            && self.accept_hex_float_literals == other.accept_hex_float_literals
            && self.accept_octal_numbers == other.accept_octal_numbers
            && self.accept_shebang == other.accept_shebang
            && self.accept_underscore_in_number_literals
                == other.accept_underscore_in_number_literals
            && self.use_lua_jit_identifier_rules == other.use_lua_jit_identifier_rules
            && self.accept_bitwise_operators == other.accept_bitwise_operators
            && self.accept_whitespace_escape == other.accept_whitespace_escape
            && self.continue_type == other.continue_type
            && self.accept_if_expressions == other.accept_if_expressions
            && self.accept_local_variable_attributes == other.accept_local_variable_attributes
            && self.binary_integer_format == other.binary_integer_format
            && self.octal_integer_format == other.octal_integer_format
            && self.decimal_integer_format == other.decimal_integer_format
            && self.hex_integer_format == other.hex_integer_format
            && self.accept_typed_lua == other.accept_typed_lua
            && self.accept_floor_division == other.accept_floor_division
            && self.accept_lua_jit_number_suffixes == other.accept_lua_jit_number_suffixes
            && self.accept_nesting_of_long_strings == other.accept_nesting_of_long_strings
            && self.backtick_string_type == other.backtick_string_type
    }
}

impl Eq for LuaSyntaxOptions {}

/// The C# GetHashCode (LuaSyntaxOptions.cs:691-721) — the same field
/// list as the Equals (the two omitted fields excluded).
impl std::hash::Hash for LuaSyntaxOptions {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.accept_binary_numbers.hash(state);
        self.accept_c_comment_syntax.hash(state);
        self.accept_compound_assignment.hash(state);
        self.accept_empty_statements.hash(state);
        self.accept_c_boolean_operators.hash(state);
        self.accept_goto.hash(state);
        self.accept_hex_escapes_in_strings.hash(state);
        self.accept_hex_float_literals.hash(state);
        self.accept_octal_numbers.hash(state);
        self.accept_shebang.hash(state);
        self.accept_underscore_in_number_literals.hash(state);
        self.use_lua_jit_identifier_rules.hash(state);
        self.accept_bitwise_operators.hash(state);
        self.accept_whitespace_escape.hash(state);
        self.continue_type.hash(state);
        self.accept_if_expressions.hash(state);
        self.accept_local_variable_attributes.hash(state);
        self.binary_integer_format.hash(state);
        self.octal_integer_format.hash(state);
        self.decimal_integer_format.hash(state);
        self.hex_integer_format.hash(state);
        self.accept_typed_lua.hash(state);
        self.accept_floor_division.hash(state);
        self.accept_lua_jit_number_suffixes.hash(state);
        self.accept_nesting_of_long_strings.hash(state);
        self.backtick_string_type.hash(state);
    }
}

impl LuaSyntaxOptions {
    /// The Lua 5.1 preset.
    pub const LUA51: Self = Self {
        accept_binary_numbers: false,
        accept_c_comment_syntax: false,
        accept_compound_assignment: false,
        accept_empty_statements: false,
        accept_c_boolean_operators: false,
        accept_goto: false,
        accept_hex_escapes_in_strings: false,
        accept_hex_float_literals: false,
        accept_octal_numbers: false,
        accept_shebang: true,
        accept_underscore_in_number_literals: false,
        use_lua_jit_identifier_rules: false,
        accept_bitwise_operators: false,
        accept_whitespace_escape: false,
        accept_unicode_escape: false,
        continue_type: ContinueType::None,
        accept_if_expressions: false,
        accept_invalid_escapes: true,
        accept_local_variable_attributes: false,
        binary_integer_format: IntegerFormats::NotSupported,
        octal_integer_format: IntegerFormats::NotSupported,
        decimal_integer_format: IntegerFormats::NotSupported,
        hex_integer_format: IntegerFormats::NotSupported,
        accept_typed_lua: false,
        accept_floor_division: false,
        accept_lua_jit_number_suffixes: false,
        accept_nesting_of_long_strings: false,
        backtick_string_type: BacktickStringType::None,
    };

    /// The Lua 5.2 preset.
    pub const LUA52: Self = Self {
        accept_empty_statements: true,
        accept_goto: true,
        accept_hex_escapes_in_strings: true,
        accept_hex_float_literals: true,
        accept_whitespace_escape: true,
        accept_invalid_escapes: false,
        accept_nesting_of_long_strings: true,
        ..Self::LUA51
    };

    /// The Lua 5.3 preset.
    pub const LUA53: Self = Self {
        accept_bitwise_operators: true,
        accept_unicode_escape: true,
        decimal_integer_format: IntegerFormats::Int64,
        hex_integer_format: IntegerFormats::Int64,
        accept_floor_division: true,
        ..Self::LUA52
    };

    /// The Lua 5.4 preset.
    pub const LUA54: Self = Self {
        accept_local_variable_attributes: true,
        ..Self::LUA53
    };

    /// The LuaJIT 2.0 preset.
    pub const LUAJIT20: Self = Self {
        accept_binary_numbers: false,
        accept_c_comment_syntax: false,
        accept_compound_assignment: false,
        accept_empty_statements: false,
        accept_c_boolean_operators: false,
        accept_goto: true,
        accept_hex_escapes_in_strings: true,
        accept_hex_float_literals: true,
        accept_octal_numbers: false,
        accept_shebang: true,
        accept_underscore_in_number_literals: false,
        use_lua_jit_identifier_rules: true,
        accept_bitwise_operators: false,
        accept_whitespace_escape: true,
        accept_unicode_escape: false,
        continue_type: ContinueType::None,
        accept_if_expressions: false,
        accept_invalid_escapes: false,
        accept_local_variable_attributes: false,
        binary_integer_format: IntegerFormats::NotSupported,
        octal_integer_format: IntegerFormats::NotSupported,
        decimal_integer_format: IntegerFormats::NotSupported,
        hex_integer_format: IntegerFormats::NotSupported,
        accept_typed_lua: false,
        accept_floor_division: false,
        accept_lua_jit_number_suffixes: true,
        accept_nesting_of_long_strings: true,
        backtick_string_type: BacktickStringType::None,
    };

    /// The LuaJIT 2.1-beta3 preset.
    pub const LUAJIT21: Self = Self {
        accept_binary_numbers: true,
        accept_unicode_escape: true,
        ..Self::LUAJIT20
    };

    /// The GLua preset.
    pub const GMOD: Self = Self {
        accept_c_comment_syntax: true,
        accept_c_boolean_operators: true,
        continue_type: ContinueType::Keyword,
        ..Self::LUAJIT20
    };

    /// The Luau preset.
    pub const LUAU: Self = Self {
        accept_binary_numbers: true,
        accept_c_comment_syntax: false,
        accept_compound_assignment: true,
        accept_empty_statements: false,
        accept_c_boolean_operators: false,
        accept_goto: false,
        accept_hex_escapes_in_strings: true,
        accept_hex_float_literals: false,
        accept_octal_numbers: false,
        accept_shebang: true,
        accept_underscore_in_number_literals: true,
        use_lua_jit_identifier_rules: false,
        accept_bitwise_operators: false,
        accept_whitespace_escape: true,
        accept_unicode_escape: true,
        continue_type: ContinueType::ContextualKeyword,
        accept_if_expressions: true,
        accept_invalid_escapes: true,
        accept_local_variable_attributes: false,
        binary_integer_format: IntegerFormats::Double,
        octal_integer_format: IntegerFormats::NotSupported,
        decimal_integer_format: IntegerFormats::NotSupported,
        hex_integer_format: IntegerFormats::Double,
        accept_typed_lua: true,
        accept_floor_division: true,
        accept_lua_jit_number_suffixes: false,
        accept_nesting_of_long_strings: true,
        backtick_string_type: BacktickStringType::InterpolatedStringLiteral,
    };

    /// The Roblox preset (alias for Luau).
    pub const ROBLOX: Self = Self::LUAU;

    /// The FiveM preset.
    pub const FIVEM: Self = Self {
        backtick_string_type: BacktickStringType::HashLiteral,
        ..Self::LUA53
    };

    /// The preset that sets everything to true.
    pub const ALL: Self = Self {
        accept_binary_numbers: true,
        accept_c_comment_syntax: true,
        accept_compound_assignment: true,
        accept_empty_statements: true,
        accept_c_boolean_operators: true,
        accept_goto: true,
        accept_hex_escapes_in_strings: true,
        accept_hex_float_literals: true,
        accept_octal_numbers: true,
        accept_shebang: true,
        accept_underscore_in_number_literals: true,
        use_lua_jit_identifier_rules: true,
        accept_bitwise_operators: true,
        accept_whitespace_escape: true,
        accept_unicode_escape: true,
        continue_type: ContinueType::ContextualKeyword,
        accept_if_expressions: true,
        accept_invalid_escapes: false,
        accept_local_variable_attributes: true,
        binary_integer_format: IntegerFormats::NotSupported,
        octal_integer_format: IntegerFormats::NotSupported,
        decimal_integer_format: IntegerFormats::NotSupported,
        hex_integer_format: IntegerFormats::NotSupported,
        accept_typed_lua: true,
        accept_floor_division: false,
        accept_lua_jit_number_suffixes: true,
        accept_nesting_of_long_strings: true,
        backtick_string_type: BacktickStringType::InterpolatedStringLiteral,
    };

    /// Same as All but with integer settings set to Int64.
    pub const ALL_WITH_INTEGERS: Self = Self {
        accept_c_comment_syntax: false,
        binary_integer_format: IntegerFormats::Int64,
        octal_integer_format: IntegerFormats::Int64,
        decimal_integer_format: IntegerFormats::Int64,
        hex_integer_format: IntegerFormats::Int64,
        accept_floor_division: true,
        ..Self::ALL
    };

    /// All presets that are preconfigured.
    pub const ALL_PRESETS: &'static [Self] = &[
        Self::LUA51,
        Self::LUA52,
        Self::LUA53,
        Self::LUA54,
        Self::LUAJIT20,
        Self::LUAJIT21,
        Self::GMOD,
        Self::LUAU,
        Self::FIVEM,
        Self::ALL,
        Self::ALL_WITH_INTEGERS,
    ];

    /// C# obsolete `AcceptHashStrings => BacktickStringType == BacktickStringType.None`
    /// (LuaSyntaxOptions.cs:412-413) — the obsolete warning attribute has no Rust equivalent.
    pub fn accept_hash_strings(&self) -> bool {
        self.backtick_string_type == BacktickStringType::None
    }

    /// Creates a new LuaSyntaxOptions with the provided values.
    /// C# ctor throws ArgumentException("AcceptFloorDivision and AcceptCCommentSyntax
    /// cannot be enabled simultaneously.") when both are enabled (LuaSyntaxOptions.cs:284-288).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        accept_binary_numbers: bool,
        accept_c_comment_syntax: bool,
        accept_compound_assignment: bool,
        accept_empty_statements: bool,
        accept_c_boolean_operators: bool,
        accept_goto: bool,
        accept_hex_escapes_in_strings: bool,
        accept_hex_float_literals: bool,
        accept_octal_numbers: bool,
        accept_shebang: bool,
        accept_underscore_in_number_literals: bool,
        use_lua_jit_identifier_rules: bool,
        accept_bitwise_operators: bool,
        accept_whitespace_escape: bool,
        accept_unicode_escape: bool,
        continue_type: ContinueType,
        accept_if_expressions: bool,
        accept_invalid_escapes: bool,
        accept_local_variable_attributes: bool,
        binary_integer_format: IntegerFormats,
        octal_integer_format: IntegerFormats,
        decimal_integer_format: IntegerFormats,
        hex_integer_format: IntegerFormats,
        accept_typed_lua: bool,
        accept_floor_division: bool,
        accept_lua_jit_number_suffixes: bool,
        accept_nesting_of_long_strings: bool,
        backtick_string_type: BacktickStringType,
    ) -> Self {
        assert!(
            !(accept_floor_division && accept_c_comment_syntax),
            "AcceptFloorDivision and AcceptCCommentSyntax cannot be enabled simultaneously."
        );
        Self {
            accept_binary_numbers,
            accept_c_comment_syntax,
            accept_compound_assignment,
            accept_empty_statements,
            accept_c_boolean_operators,
            accept_goto,
            accept_hex_escapes_in_strings,
            accept_hex_float_literals,
            accept_octal_numbers,
            accept_shebang,
            accept_underscore_in_number_literals,
            use_lua_jit_identifier_rules,
            accept_bitwise_operators,
            accept_whitespace_escape,
            accept_unicode_escape,
            continue_type,
            accept_if_expressions,
            accept_invalid_escapes,
            accept_local_variable_attributes,
            binary_integer_format,
            octal_integer_format,
            decimal_integer_format,
            hex_integer_format,
            accept_typed_lua,
            accept_floor_division,
            accept_lua_jit_number_suffixes,
            accept_nesting_of_long_strings,
            backtick_string_type,
        }
    }

    /// Creates a new LuaSyntaxOptions changing the provided fields (C# `With`
    /// with Option<T> defaults; `None` keeps the current value).
    #[allow(clippy::too_many_arguments)]
    pub fn with(
        &self,
        accept_binary_numbers: Option<bool>,
        accept_c_comment_syntax: Option<bool>,
        accept_compound_assignment: Option<bool>,
        accept_empty_statements: Option<bool>,
        accept_c_boolean_operators: Option<bool>,
        accept_goto: Option<bool>,
        accept_hex_escapes_in_strings: Option<bool>,
        accept_hex_float_literals: Option<bool>,
        accept_octal_numbers: Option<bool>,
        accept_shebang: Option<bool>,
        accept_underscore_in_number_literals: Option<bool>,
        use_lua_jit_identifier_rules: Option<bool>,
        accept_bitwise_operators: Option<bool>,
        accept_whitespace_escape: Option<bool>,
        accept_unicode_escape: Option<bool>,
        continue_type: Option<ContinueType>,
        accept_if_expressions: Option<bool>,
        accept_invalid_escapes: Option<bool>,
        accept_local_variable_attributes: Option<bool>,
        binary_integer_format: Option<IntegerFormats>,
        octal_integer_format: Option<IntegerFormats>,
        decimal_integer_format: Option<IntegerFormats>,
        hex_integer_format: Option<IntegerFormats>,
        accept_typed_lua: Option<bool>,
        accept_floor_division: Option<bool>,
        accept_lua_jit_number_suffixes: Option<bool>,
        accept_nesting_of_long_strings: Option<bool>,
        backtick_string_type: Option<BacktickStringType>,
    ) -> Self {
        Self::new(
            accept_binary_numbers.unwrap_or(self.accept_binary_numbers),
            accept_c_comment_syntax.unwrap_or(self.accept_c_comment_syntax),
            accept_compound_assignment.unwrap_or(self.accept_compound_assignment),
            accept_empty_statements.unwrap_or(self.accept_empty_statements),
            accept_c_boolean_operators.unwrap_or(self.accept_c_boolean_operators),
            accept_goto.unwrap_or(self.accept_goto),
            accept_hex_escapes_in_strings.unwrap_or(self.accept_hex_escapes_in_strings),
            accept_hex_float_literals.unwrap_or(self.accept_hex_float_literals),
            accept_octal_numbers.unwrap_or(self.accept_octal_numbers),
            accept_shebang.unwrap_or(self.accept_shebang),
            accept_underscore_in_number_literals
                .unwrap_or(self.accept_underscore_in_number_literals),
            use_lua_jit_identifier_rules.unwrap_or(self.use_lua_jit_identifier_rules),
            accept_bitwise_operators.unwrap_or(self.accept_bitwise_operators),
            accept_whitespace_escape.unwrap_or(self.accept_whitespace_escape),
            accept_unicode_escape.unwrap_or(self.accept_unicode_escape),
            continue_type.unwrap_or(self.continue_type),
            accept_if_expressions.unwrap_or(self.accept_if_expressions),
            accept_invalid_escapes.unwrap_or(self.accept_invalid_escapes),
            accept_local_variable_attributes.unwrap_or(self.accept_local_variable_attributes),
            binary_integer_format.unwrap_or(self.binary_integer_format),
            octal_integer_format.unwrap_or(self.octal_integer_format),
            decimal_integer_format.unwrap_or(self.decimal_integer_format),
            hex_integer_format.unwrap_or(self.hex_integer_format),
            accept_typed_lua.unwrap_or(self.accept_typed_lua),
            accept_floor_division.unwrap_or(self.accept_floor_division),
            accept_lua_jit_number_suffixes.unwrap_or(self.accept_lua_jit_number_suffixes),
            accept_nesting_of_long_strings.unwrap_or(self.accept_nesting_of_long_strings),
            backtick_string_type.unwrap_or(self.backtick_string_type),
        )
    }

    /// Returns a string representation of the preset.
    pub fn preset_name(&self) -> Option<&'static str> {
        if *self == Self::LUA51 {
            Some("Lua 5.1")
        } else if *self == Self::LUA52 {
            Some("Lua 5.2")
        } else if *self == Self::LUA53 {
            Some("Lua 5.3")
        } else if *self == Self::LUA54 {
            Some("Lua 5.4")
        } else if *self == Self::LUAJIT20 {
            Some("LuaJIT 2.0")
        } else if *self == Self::LUAJIT21 {
            Some("LuaJIT 2.1")
        } else if *self == Self::GMOD {
            Some("GLua")
        } else if *self == Self::LUAU {
            Some("Luau")
        } else if *self == Self::FIVEM {
            Some("FiveM")
        } else if *self == Self::ALL {
            Some("All (without integers)")
        } else if *self == Self::ALL_WITH_INTEGERS {
            Some("All (with integers)")
        } else {
            None
        }
    }
}

impl std::fmt::Display for LuaSyntaxOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = self.preset_name() {
            write!(f, "{name}")
        } else {
            write!(
                f,
                "{{ AcceptBinaryNumbers = {}, AcceptCCommentSyntax = {}, AcceptCompoundAssignment = {}, AcceptEmptyStatements = {}, AcceptCBooleanOperators = {}, AcceptGoto = {}, AcceptHexEscapesInStrings = {}, AcceptHexFloatLiterals = {}, AcceptOctalNumbers = {}, AcceptShebang = {}, AcceptUnderscoreInNumberLiterals = {}, UseLuaJitIdentifierRules = {}, AcceptBitwiseOperators = {}, AcceptWhitespaceEscape = {}, ContinueType = {:?}, AcceptIfExpressions = {}, AcceptLocalVariableAttributes = {}, BinaryIntegerFormat = {:?}, OctalIntegerFormat = {:?}, DecimalIntegerFormat = {:?}, HexIntegerFormat = {:?}, AcceptTypedLua = {}, AcceptFloorDivision = {}, AcceptLuaJITNumberSuffixes = {}, AcceptNestingOfLongStrings = {}, BacktickStringType = {:?} }}",
                self.accept_binary_numbers,
                self.accept_c_comment_syntax,
                self.accept_compound_assignment,
                self.accept_empty_statements,
                self.accept_c_boolean_operators,
                self.accept_goto,
                self.accept_hex_escapes_in_strings,
                self.accept_hex_float_literals,
                self.accept_octal_numbers,
                self.accept_shebang,
                self.accept_underscore_in_number_literals,
                self.use_lua_jit_identifier_rules,
                self.accept_bitwise_operators,
                self.accept_whitespace_escape,
                self.continue_type,
                self.accept_if_expressions,
                self.accept_local_variable_attributes,
                self.binary_integer_format,
                self.octal_integer_format,
                self.decimal_integer_format,
                self.hex_integer_format,
                self.accept_typed_lua,
                self.accept_floor_division,
                self.accept_lua_jit_number_suffixes,
                self.accept_nesting_of_long_strings,
                self.backtick_string_type,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equality_omits_the_escape_fields() {
        // Finding 42: the C# Equals/GetHashCode deliberately omit
        // AcceptUnicodeEscape and AcceptInvalidEscapes
        // (LuaSyntaxOptions.cs:660-721) — two options that do not affect
        // the resulting syntax tree.
        let base = LuaSyntaxOptions::ALL;
        let with_unicode = LuaSyntaxOptions {
            accept_unicode_escape: false,
            ..LuaSyntaxOptions::ALL
        };
        let with_invalid = LuaSyntaxOptions {
            accept_invalid_escapes: false,
            ..LuaSyntaxOptions::ALL
        };
        assert_eq!(base, with_unicode);
        assert_eq!(base, with_invalid);
        // The hash agrees (a HashSet lookup uses both).
        let mut set = std::collections::HashSet::new();
        set.insert(base.clone());
        assert!(set.contains(&with_unicode));
        assert!(set.contains(&with_invalid));
        // Any OTHER field difference still differs.
        let different = LuaSyntaxOptions {
            accept_goto: false,
            ..LuaSyntaxOptions::ALL
        };
        assert_ne!(base, different);
        assert!(!set.contains(&different));
    }
}
