// Ported from Loretta.CodeAnalysis.Lua.LuaResources (b767b4e): LuaResources
// C# source: src/Compilers/Lua/Portable/LuaResources.Designer.cs
// NOTE: ResourceManager/CultureInfo infrastructure is dropped. Static string constants instead.

/// Resource strings for Lua diagnostics.
pub struct LuaResources;

impl LuaResources {
    pub const ERR_AMBIGUOUS_FUNCTION_CALL_OR_NEW_STATEMENT: &'static str =
        "Ambiguous function call or new statement. Please use parentheses to disambiguate.";
    pub const ERR_BAD_CHARACTER: &'static str = "Bad character '{0}' encountered.";
    pub const ERR_BAD_DOCUMENTATION_MODE: &'static str = "Invalid documentation mode '{0}'.";
    pub const ERR_BINARY_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Binary numeric literals are not supported in Lua {0}.";
    pub const ERR_BITWISE_OPERATORS_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Bitwise operators are not supported in Lua {0}.";
    pub const ERR_CANNOT_BE_ASSIGNED_TO: &'static str =
        "The expression '{0}' cannot be assigned to.";
    pub const ERR_C_COMMENTS_NOT_SUPPORTED_IN_VERSION: &'static str =
        "C-style comments are not supported in Lua {0}.";
    pub const ERR_CLOSE_PAREN_EXPECTED: &'static str = "Close parenthesis expected.";
    pub const ERR_COMPOUND_ASSIGNMENT_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Compound assignments are not supported in Lua {0}.";
    pub const ERR_DOUBLE_BRACE_IN_INTERPOLATION: &'static str =
        "Double braces in interpolated strings are not allowed. Use a single brace.";
    pub const ERR_DOUBLE_OVERFLOW: &'static str = "The double value '{0}' is too large.";
    pub const ERR_ESCAPE_TOO_LARGE: &'static str = "The escape sequence is too large.";
    pub const ERR_EXPRESSION_EXPECTED: &'static str = "Expression expected.";
    pub const ERR_GOTO_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Goto statements are not supported in Lua {0}.";
    pub const ERR_HEX_DIGIT_EXPECTED: &'static str = "Hexadecimal digit expected.";
    pub const ERR_HEX_FLOAT_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Hexadecimal float literals are not supported in Lua {0}.";
    pub const ERR_HEX_STRING_ESCAPES_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Hexadecimal string escapes are not supported in Lua {0}.";
    pub const ERR_IDENTIFIER_EXPECTED: &'static str = "Identifier expected.";
    pub const ERR_IDENTIFIER_EXPECTED_KW: &'static str = "Identifier expected, got keyword '{0}'.";
    pub const ERR_IF_EXPRESSION_CONDITION_EXPECTED: &'static str =
        "If expression condition expected.";
    pub const ERR_IF_EXPRESSION_CONDITION_EXPECTED_DESCRIPTION: &'static str =
        "The condition of an if expression is required.";
    pub const ERR_IF_EXPRESSION_CONDITION_EXPECTED_TITLE: &'static str =
        "If expression condition expected.";
    pub const ERR_IF_EXPRESSIONS_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "If expressions are not supported in Lua {0}.";
    pub const ERR_INSUFFICIENT_STACK: &'static str = "Insufficient stack to continue parsing.";
    pub const ERR_INTERPOLATED_STRING_MUST_START_WITH_BACKTICK_CHARACTER: &'static str =
        "Interpolated strings must start with a backtick character.";
    pub const ERR_INTERPOLATED_STRINGS_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Interpolated strings are not supported in Lua {0}.";
    pub const ERR_INVALID_EXPRESSION_PART: &'static str = "Invalid expression part '{0}'.";
    pub const ERR_INVALID_NUMBER: &'static str = "The number '{0}' is not valid.";
    pub const ERR_INVALID_STATEMENT: &'static str = "Invalid statement '{0}'.";
    pub const ERR_INVALID_STRING_ESCAPE: &'static str = "Invalid string escape '{0}'.";
    pub const ERR_LBRACE_EXPECTED: &'static str = "Left brace expected.";
    pub const ERR_LUA51_NESTING_IN_LONG_STRING: &'static str =
        "Nesting in long strings is not supported in Lua 5.1.";
    pub const ERR_LUAJIT_IDENTIFIER_RULES_NOT_SUPPORTED_IN_VERSION: &'static str =
        "LuaJIT identifier rules are not supported in Lua {0}.";
    pub const ERR_LUAJIT_SUFFIX_IN_FLOAT: &'static str =
        "LuaJIT number suffixes are not supported in float literals.";
    pub const ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED: &'static str =
        "Mixing nilable and intersection types is not allowed.";
    pub const ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED_DESCRIPTION: &'static str =
        "Mixing nilable (?) and intersection (&) types is not allowed.";
    pub const ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED_TITLE: &'static str =
        "Mixing nilable and intersection types is not allowed.";
    pub const ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED: &'static str =
        "Mixing union and intersection types is not allowed.";
    pub const ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED_DESCRIPTION: &'static str =
        "Mixing union (|) and intersection (&) types is not allowed without parentheses.";
    pub const ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED_TITLE: &'static str =
        "Mixing union and intersection types is not allowed.";
    pub const ERR_NON_FUNCTION_CALL_BEING_USED_AS_STATEMENT: &'static str =
        "Non-function call expression used as a statement.";
    pub const ERR_NORMAL_TYPE_PARAMETERS_COME_BEFORE_PACKS: &'static str =
        "Normal type parameters must come before type packs.";
    pub const ERR_NUMBER_SUFFIX_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Number suffix '{0}' is not supported in Lua {1}.";
    pub const ERR_NUMERIC_LITERAL_TOO_LARGE: &'static str =
        "The numeric literal '{0}' is too large.";
    pub const ERR_OCTAL_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Octal numeric literals are not supported in Lua {0}.";
    pub const ERR_ONLY_ONE_TABLE_TYPE_INDEXER_IS_ALLOWED: &'static str =
        "Only one table type indexer is allowed.";
    pub const ERR_RBRACE_EXPECTED: &'static str = "Right brace expected.";
    pub const ERR_SEMICOLON_EXPECTED: &'static str = "Semicolon expected.";
    pub const ERR_SHEBANG_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Shebangs are not supported in Lua {0}.";
    pub const ERR_SYNTAX_ERROR: &'static str = "Syntax error: {0}.";
    pub const ERR_TYPED_LUA_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Typed Lua is not supported in Lua {0}.";
    pub const ERR_UNCLOSED_EXPRESSION_HOLE: &'static str =
        "Unclosed expression hole in interpolated string.";
    pub const ERR_UNDERSCORE_IN_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Underscores in numeric literals are not supported in Lua {0}.";
    pub const ERR_UNEXPECTED_TOKEN: &'static str = "Unexpected token '{0}'.";
    pub const ERR_UNFINISHED_LONG_COMMENT: &'static str = "Unfinished long comment.";
    pub const ERR_UNFINISHED_STRING: &'static str = "Unfinished string.";
    pub const ERR_UNICODE_ESCAPE_MISSING_CLOSE_BRACE: &'static str =
        "Unicode escape sequence is missing a closing brace.";
    pub const ERR_UNICODE_ESCAPE_MISSING_OPEN_BRACE: &'static str =
        "Unicode escape sequence is missing an opening brace.";
    pub const ERR_UNICODE_ESCAPES_NOT_SUPPORTED_LUA_IN_VERSION: &'static str =
        "Unicode escapes are not supported in Lua {0}.";
    pub const ERR_WHITESPACE_ESCAPE_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Whitespace escapes are not supported in Lua {0}.";
    pub const THIS_METHOD_CAN_ONLY_BE_USED_TO_CREATE_TOKENS: &'static str =
        "This method can only be used to create tokens.";
    pub const USE_IDENTIFIER_TO_CREATE_IDENTIFIERS: &'static str =
        "Use an identifier to create identifiers.";
    pub const USE_LITERAL_FOR_NUMERIC: &'static str = "Use a literal for numeric values.";
    pub const WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING: &'static str =
        "Line break may affect error reporting.";
    pub const WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING_DESCRIPTION: &'static str =
        "A line break was found between the start of a statement and the start of the next token.";
    pub const WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING_TITLE: &'static str =
        "Line break may affect error reporting.";
}
