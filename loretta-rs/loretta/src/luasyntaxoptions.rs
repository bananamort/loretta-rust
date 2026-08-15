// Ported from Loretta.CodeAnalysis.Lua.LuaSyntaxOptions (b767b4e): LuaSyntaxOptions
// C# source: src/Compilers/Lua/Portable/LuaSyntaxOptions.cs

use crate::backtickstringtype::BacktickStringType;
use crate::continuetype::ContinueType;
use crate::integerformats::IntegerFormats;

/// The options used by Loretta to adapt to the syntax of the lua flavor being parsed.
///
/// "Accept" means not generating an error when parsing, but the syntax behind the option
/// will still be parsed normally.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
