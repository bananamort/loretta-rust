// Ported from Loretta.CodeAnalysis.Lua.LuaResources (b767b4e): LuaResources
// C# source: src/Compilers/Lua/Portable/LuaResources.Designer.cs
// NOTE: ResourceManager/CultureInfo infrastructure is dropped. Static string constants instead.

/// Resource strings for Lua diagnostics.
pub struct LuaResources;

impl LuaResources {
    pub const ERR_AMBIGUOUS_FUNCTION_CALL_OR_NEW_STATEMENT: &'static str =
        "Syntax ambiguous between a function call and a new statement";
    pub const ERR_BAD_CHARACTER: &'static str = "Bad character input: '{0}'";
    pub const ERR_BAD_DOCUMENTATION_MODE: &'static str =
        "Provided documentation mode is unsupported or invalid: '{0}'";
    pub const ERR_BINARY_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Binary numeric literals are not supported in this lua version";
    pub const ERR_BITWISE_OPERATORS_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Bitwise operators are not supported in this lua version";
    pub const ERR_CANNOT_BE_ASSIGNED_TO: &'static str = "This expression cannot be assigned to";
    pub const ERR_C_COMMENTS_NOT_SUPPORTED_IN_VERSION: &'static str =
        "C comments are not supported in this lua version";
    pub const ERR_CLOSE_PAREN_EXPECTED: &'static str = ") expected";
    pub const ERR_COMPOUND_ASSIGNMENT_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Compound assignments are not supported in this lua version";
    pub const ERR_DOUBLE_OVERFLOW: &'static str = "Constant represents a value either too large or too small for a double precision floating-point number";
    pub const ERR_ESCAPE_TOO_LARGE: &'static str = "Escape is too large, the limit is {0}";
    pub const ERR_EXPRESSION_EXPECTED: &'static str = "Expression expected";
    pub const ERR_HEX_DIGIT_EXPECTED: &'static str = "Hexadecimal digit expected";
    pub const ERR_HEX_FLOAT_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Hexadecimal floating point numeric literals are not supported in this lua version";
    pub const ERR_HEX_STRING_ESCAPES_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Hexadecimal string escapes are not supported in this lua version";
    pub const ERR_IDENTIFIER_EXPECTED: &'static str = "Identifier expected";
    pub const ERR_IDENTIFIER_EXPECTED_KW: &'static str = "Identifier expected; '{1}' is a keyword";
    pub const ERR_IF_EXPRESSION_CONDITION_EXPECTED: &'static str =
        "Condition not found for if expression";
    pub const ERR_IF_EXPRESSION_CONDITION_EXPECTED_DESCRIPTION: &'static str =
        "If expressions require a condition but one was not found, did you perhaps forget to specify one?";
    pub const ERR_IF_EXPRESSION_CONDITION_EXPECTED_TITLE: &'static str =
        "If expressions require a condition";
    pub const ERR_IF_EXPRESSIONS_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "If expressions are not supported in this lua version";
    pub const ERR_INSUFFICIENT_STACK: &'static str =
        "An expression is too long or complex to compile";
    pub const ERR_INVALID_EXPRESSION_PART: &'static str = "Invalid expression part '{0}'";
    pub const ERR_INVALID_NUMBER: &'static str = "Invalid number";
    pub const ERR_INVALID_STATEMENT: &'static str = "Invalid statement";
    pub const ERR_INVALID_STRING_ESCAPE: &'static str = "Invalid string escape";
    pub const ERR_LBRACE_EXPECTED: &'static str = "{ expected";
    pub const ERR_LUA51_NESTING_IN_LONG_STRING: &'static str = "Nesting of [[...]] is deprecated";
    pub const ERR_LUAJIT_IDENTIFIER_RULES_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Identifiers containing characters with value above 0x7F are not supported in this lua version";
    pub const ERR_LUAJIT_SUFFIX_IN_FLOAT: &'static str =
        "LuaJIT suffixes cannot be used in floating point numbers";
    pub const ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED: &'static str =
        "Using nilable types directly in intersections is not allowed";
    pub const ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED_DESCRIPTION: &'static str =
        "Using nilable types directly in intersections is not allowed. The nilable types must be in parenthesis to be used in intersections";
    pub const ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED_TITLE: &'static str =
        "Using nilable types directly in intersections is not allowed";
    pub const ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED: &'static str =
        "Mixing union and intersection types is not allowed";
    pub const ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED_DESCRIPTION: &'static str =
        "Mixing union and intersection types is not allowed. The unions must be inside parenthesis to be used in intersections";
    pub const ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED_TITLE: &'static str =
        "Mixing union and intersection types is not allowed";
    pub const ERR_NON_FUNCTION_CALL_BEING_USED_AS_STATEMENT: &'static str =
        "Function calls are the only expressions that can be used as statements";
    pub const ERR_NORMAL_TYPE_PARAMETERS_COME_BEFORE_PACKS: &'static str =
        "Normal type parameters must come before pack type parameters";
    pub const ERR_NUMBER_SUFFIX_NOT_SUPPORTED_IN_VERSION: &'static str =
        "LuaJIT number suffixes are not supported in this lua version";
    pub const ERR_NUMERIC_LITERAL_TOO_LARGE: &'static str = "Numeric literal is too large";
    pub const ERR_OCTAL_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Octal numeric literals are not supported in this lua version";
    pub const ERR_ONLY_ONE_TABLE_TYPE_INDEXER_IS_ALLOWED: &'static str =
        "Only one indexer is allowed per table type";
    pub const ERR_RBRACE_EXPECTED: &'static str = "} expected";
    pub const ERR_SEMICOLON_EXPECTED: &'static str = "; expected";
    pub const ERR_SHEBANG_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Shebangs are not supported in this lua version";
    pub const ERR_SYNTAX_ERROR: &'static str = "Syntax error, '{0}' expected";
    pub const ERR_TYPED_LUA_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Typed lua is not supported in this lua version";
    pub const ERR_UNCLOSED_EXPRESSION_HOLE: &'static str =
        "Interpolated strings expressions must have a corresponding closing '}' for every opening '{'";
    pub const ERR_UNDERSCORE_IN_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Underscores in numeric literals are not supported in this lua version";
    pub const ERR_UNEXPECTED_TOKEN: &'static str = "Unexpected token '{0}'";
    pub const ERR_UNFINISHED_LONG_COMMENT: &'static str = "Unfinished multi-line comment";
    pub const ERR_UNFINISHED_STRING: &'static str = "Unfinished string";
    pub const ERR_UNICODE_ESCAPE_MISSING_CLOSE_BRACE: &'static str =
        "Unicode escape must have a closing brace ('}') after the hexadecimal number";
    pub const ERR_UNICODE_ESCAPE_MISSING_OPEN_BRACE: &'static str =
        "Unicode escape must have an opening brace ('{') after '\\u'";
    pub const ERR_UNICODE_ESCAPES_NOT_SUPPORTED_LUA_IN_VERSION: &'static str =
        "Unicode escapes are not supported in this lua version";
    pub const ERR_WHITESPACE_ESCAPE_NOT_SUPPORTED_IN_VERSION: &'static str =
        "The whitespace escape ('\\z') is not supported in this lua version";
    pub const THIS_METHOD_CAN_ONLY_BE_USED_TO_CREATE_TOKENS: &'static str =
        "This method can only be used to create tokens - {0} is not a token kind.";
    pub const USE_IDENTIFIER_TO_CREATE_IDENTIFIERS: &'static str =
        "Use Loretta.CodeAnalysis.Lua.SyntaxFactory.Identifier to create identifier tokens.";
    pub const USE_LITERAL_FOR_NUMERIC: &'static str =
        "Use Loretta.CodeAnalysis.Lua.SyntaxFactory.Literal to create numeric literal tokens.";
    pub const WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING: &'static str =
        "This line break (\\n\\r) may affect error reporting between the editor and lua";
    pub const WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING_DESCRIPTION: &'static str =
        "Lua considers '\\n\\r' a single line break so error reporting between the editor and Lua may differ; use \\n, \\r or \\r\\n instead";
    pub const WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING_TITLE: &'static str =
        "Line break may affect error reporting";
    pub const ERR_INTERPOLATED_STRING_MUST_START_WITH_BACKTICK_CHARACTER: &'static str =
        "Interpolated strings must start with the backtick character: `";
    pub const ERR_INTERPOLATED_STRINGS_NOT_SUPPORTED_IN_VERSION: &'static str =
        "Interpolated strings are not supported in this lua version";
    pub const ERR_DOUBLE_BRACE_IN_INTERPOLATION: &'static str =
        "Double braces have no meaning, did you mean to escape an opening brace with '\\{'?";
    pub const ERR_GOTO_NOT_SUPPORTED_IN_LUA_VERSION: &'static str =
        "Goto statements and labels are not supported in this lua version";
}
