// Ported from Loretta.CodeAnalysis.Lua.MessageProvider (b767b4e): MessageProvider
// C# source: src/Compilers/Lua/Portable/Errors/MessageProvider.cs

use crate::errors::errorcode::ErrorCode;
use crate::errors::luadiagnostic::{DiagnosticSeverity, LuaDiagnostic};
use crate::luaresources::LuaResources;

/// Provides messages for Lua diagnostics.
pub struct MessageProvider;

impl MessageProvider {
    /// The singleton instance.
    pub const INSTANCE: Self = Self;

    /// The code prefix for Lua diagnostics.
    pub const CODE_PREFIX: &'static str = "LUA";

    /// C# `ERR_BadDocumentationMode => (int) ErrorCode.ERR_BadDocumentationMode`.
    pub const ERR_BAD_DOCUMENTATION_MODE: i32 = ErrorCode::ErrBadDocumentationMode as i32;

    /// Gets the severity for the given error code.
    pub fn get_severity(code: ErrorCode) -> DiagnosticSeverity {
        match code {
            ErrorCode::WrnLineBreakMayAffectErrorReporting => DiagnosticSeverity::Warning,
            ErrorCode::Void | ErrorCode::Unknown => DiagnosticSeverity::Hidden,
            _ => DiagnosticSeverity::Error,
        }
    }

    /// Loads the message for the given error code.
    pub fn load_message(code: ErrorCode) -> String {
        Self::get_message_format(code)
    }

    /// Gets the message format for the given error code.
    pub fn get_message_format(code: ErrorCode) -> String {
        match code {
            ErrorCode::ErrAmbiguousFunctionCallOrNewStatement => {
                LuaResources::ERR_AMBIGUOUS_FUNCTION_CALL_OR_NEW_STATEMENT
            }
            ErrorCode::ErrBadCharacter => LuaResources::ERR_BAD_CHARACTER,
            ErrorCode::ErrBadDocumentationMode => LuaResources::ERR_BAD_DOCUMENTATION_MODE,
            ErrorCode::ErrBinaryNumericLiteralNotSupportedInVersion => {
                LuaResources::ERR_BINARY_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrBitwiseOperatorsNotSupportedInVersion => {
                LuaResources::ERR_BITWISE_OPERATORS_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrCannotBeAssignedTo => LuaResources::ERR_CANNOT_BE_ASSIGNED_TO,
            ErrorCode::ErrCCommentsNotSupportedInVersion => {
                LuaResources::ERR_C_COMMENTS_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrCloseParenExpected => LuaResources::ERR_CLOSE_PAREN_EXPECTED,
            ErrorCode::ErrCompoundAssignmentNotSupportedInLuaVersion => {
                LuaResources::ERR_COMPOUND_ASSIGNMENT_NOT_SUPPORTED_IN_LUA_VERSION
            }
            ErrorCode::ErrDoubleBraceInInterpolation => {
                LuaResources::ERR_DOUBLE_BRACE_IN_INTERPOLATION
            }
            ErrorCode::ErrDoubleOverflow => LuaResources::ERR_DOUBLE_OVERFLOW,
            ErrorCode::ErrEscapeTooLarge => LuaResources::ERR_ESCAPE_TOO_LARGE,
            ErrorCode::ErrExpressionExpected => LuaResources::ERR_EXPRESSION_EXPECTED,
            ErrorCode::ErrGotoNotSupportedInLuaVersion => {
                LuaResources::ERR_GOTO_NOT_SUPPORTED_IN_LUA_VERSION
            }
            ErrorCode::ErrHexDigitExpected => LuaResources::ERR_HEX_DIGIT_EXPECTED,
            ErrorCode::ErrHexFloatLiteralNotSupportedInVersion => {
                LuaResources::ERR_HEX_FLOAT_LITERAL_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrHexStringEscapesNotSupportedInVersion => {
                LuaResources::ERR_HEX_STRING_ESCAPES_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrIdentifierExpected => LuaResources::ERR_IDENTIFIER_EXPECTED,
            ErrorCode::ErrIdentifierExpectedKw => LuaResources::ERR_IDENTIFIER_EXPECTED_KW,
            ErrorCode::ErrIfExpressionConditionExpected => {
                LuaResources::ERR_IF_EXPRESSION_CONDITION_EXPECTED
            }
            ErrorCode::ErrIfExpressionsNotSupportedInLuaVersion => {
                LuaResources::ERR_IF_EXPRESSIONS_NOT_SUPPORTED_IN_LUA_VERSION
            }
            ErrorCode::ErrInsufficientStack => LuaResources::ERR_INSUFFICIENT_STACK,
            ErrorCode::ErrInterpolatedStringMustStartWithBacktickCharacter => {
                LuaResources::ERR_INTERPOLATED_STRING_MUST_START_WITH_BACKTICK_CHARACTER
            }
            ErrorCode::ErrInterpolatedStringsNotSupportedInVersion => {
                LuaResources::ERR_INTERPOLATED_STRINGS_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrInvalidExpressionPart => LuaResources::ERR_INVALID_EXPRESSION_PART,
            ErrorCode::ErrInvalidNumber => LuaResources::ERR_INVALID_NUMBER,
            ErrorCode::ErrInvalidStatement => LuaResources::ERR_INVALID_STATEMENT,
            ErrorCode::ErrInvalidStringEscape => LuaResources::ERR_INVALID_STRING_ESCAPE,
            ErrorCode::ErrLbraceExpected => LuaResources::ERR_LBRACE_EXPECTED,
            ErrorCode::ErrLua51NestingInLongString => {
                LuaResources::ERR_LUA51_NESTING_IN_LONG_STRING
            }
            ErrorCode::ErrLuajitIdentifierRulesNotSupportedInVersion => {
                LuaResources::ERR_LUAJIT_IDENTIFIER_RULES_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrLuajitSuffixInFloat => LuaResources::ERR_LUAJIT_SUFFIX_IN_FLOAT,
            ErrorCode::ErrMixingNilableAndIntersectionNotAllowed => {
                LuaResources::ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED
            }
            ErrorCode::ErrMixingUnionsAndIntersectionsNotAllowed => {
                LuaResources::ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED
            }
            ErrorCode::ErrNonFunctionCallBeingUsedAsStatement => {
                LuaResources::ERR_NON_FUNCTION_CALL_BEING_USED_AS_STATEMENT
            }
            ErrorCode::ErrNormalTypeParametersComeBeforePacks => {
                LuaResources::ERR_NORMAL_TYPE_PARAMETERS_COME_BEFORE_PACKS
            }
            ErrorCode::ErrNumberSuffixNotSupportedInVersion => {
                LuaResources::ERR_NUMBER_SUFFIX_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrNumericLiteralTooLarge => LuaResources::ERR_NUMERIC_LITERAL_TOO_LARGE,
            ErrorCode::ErrOctalNumericLiteralNotSupportedInVersion => {
                LuaResources::ERR_OCTAL_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrOnlyOneTableTypeIndexerIsAllowed => {
                LuaResources::ERR_ONLY_ONE_TABLE_TYPE_INDEXER_IS_ALLOWED
            }
            ErrorCode::ErrRbraceExpected => LuaResources::ERR_RBRACE_EXPECTED,
            ErrorCode::ErrSemicolonExpected => LuaResources::ERR_SEMICOLON_EXPECTED,
            ErrorCode::ErrShebangNotSupportedInLuaVersion => {
                LuaResources::ERR_SHEBANG_NOT_SUPPORTED_IN_LUA_VERSION
            }
            ErrorCode::ErrSyntaxError => LuaResources::ERR_SYNTAX_ERROR,
            ErrorCode::ErrTypedLuaNotSupportedInLuaVersion => {
                LuaResources::ERR_TYPED_LUA_NOT_SUPPORTED_IN_LUA_VERSION
            }
            ErrorCode::ErrUnclosedExpressionHole => LuaResources::ERR_UNCLOSED_EXPRESSION_HOLE,
            ErrorCode::ErrUnderscoreInNumericLiteralNotSupportedInVersion => {
                LuaResources::ERR_UNDERSCORE_IN_NUMERIC_LITERAL_NOT_SUPPORTED_IN_VERSION
            }
            ErrorCode::ErrUnexpectedToken => LuaResources::ERR_UNEXPECTED_TOKEN,
            ErrorCode::ErrUnfinishedLongComment => LuaResources::ERR_UNFINISHED_LONG_COMMENT,
            ErrorCode::ErrUnfinishedString => LuaResources::ERR_UNFINISHED_STRING,
            ErrorCode::ErrUnicodeEscapeMissingCloseBrace => {
                LuaResources::ERR_UNICODE_ESCAPE_MISSING_CLOSE_BRACE
            }
            ErrorCode::ErrUnicodeEscapeMissingOpenBrace => {
                LuaResources::ERR_UNICODE_ESCAPE_MISSING_OPEN_BRACE
            }
            ErrorCode::ErrUnicodeEscapesNotSupportedLuaInVersion => {
                LuaResources::ERR_UNICODE_ESCAPES_NOT_SUPPORTED_LUA_IN_VERSION
            }
            ErrorCode::ErrWhitespaceEscapeNotSupportedInVersion => {
                LuaResources::ERR_WHITESPACE_ESCAPE_NOT_SUPPORTED_IN_VERSION
            }
            _ => "Unknown error",
        }
        .to_string()
    }

    /// Gets the description for the given error code.
    pub fn get_description(code: ErrorCode) -> String {
        match code {
            ErrorCode::ErrIfExpressionConditionExpected => {
                LuaResources::ERR_IF_EXPRESSION_CONDITION_EXPECTED_DESCRIPTION
            }
            ErrorCode::ErrMixingNilableAndIntersectionNotAllowed => {
                LuaResources::ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED_DESCRIPTION
            }
            ErrorCode::ErrMixingUnionsAndIntersectionsNotAllowed => {
                LuaResources::ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED_DESCRIPTION
            }
            ErrorCode::WrnLineBreakMayAffectErrorReporting => {
                LuaResources::WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING_DESCRIPTION
            }
            _ => "",
        }
        .to_string()
    }

    /// Gets the title for the given error code.
    pub fn get_title(code: ErrorCode) -> String {
        match code {
            ErrorCode::ErrIfExpressionConditionExpected => {
                LuaResources::ERR_IF_EXPRESSION_CONDITION_EXPECTED_TITLE
            }
            ErrorCode::ErrMixingNilableAndIntersectionNotAllowed => {
                LuaResources::ERR_MIXING_NILABLE_AND_INTERSECTION_NOT_ALLOWED_TITLE
            }
            ErrorCode::ErrMixingUnionsAndIntersectionsNotAllowed => {
                LuaResources::ERR_MIXING_UNIONS_AND_INTERSECTIONS_NOT_ALLOWED_TITLE
            }
            ErrorCode::WrnLineBreakMayAffectErrorReporting => {
                LuaResources::WRN_LINE_BREAK_MAY_AFFECT_ERROR_REPORTING_TITLE
            }
            _ => "",
        }
        .to_string()
    }

    /// Gets the help link for the given error code.
    pub fn get_help_link(_code: ErrorCode) -> String {
        String::new()
    }

    /// Gets the category for the given error code.
    pub fn get_category(_code: ErrorCode) -> String {
        String::new()
    }

    /// Gets the message prefix for a diagnostic.
    pub fn get_message_prefix(
        id: &str,
        severity: DiagnosticSeverity,
        is_warning_as_error: bool,
    ) -> String {
        let is_error = severity == DiagnosticSeverity::Error || is_warning_as_error;
        let prefix = if is_error { "error" } else { "warning" };
        format!("{prefix} {id}")
    }

    /// Gets the warning level for the given error code.
    pub fn get_warning_level(_code: ErrorCode) -> i32 {
        1
    }

    /// Creates a diagnostic from an error code and arguments.
    pub fn create_diagnostic(code: ErrorCode, args: &[&str]) -> LuaDiagnostic {
        let mut message = Self::get_message_format(code);
        for (i, arg) in args.iter().enumerate() {
            message = message.replace(&format!("{{{i}}}"), arg);
        }
        LuaDiagnostic::new(code, message, Self::get_severity(code), false)
    }

    /// Creates a diagnostic from a DiagnosticInfo.
    pub fn create_diagnostic_from_info(
        code: ErrorCode,
        message: String,
        severity: DiagnosticSeverity,
    ) -> LuaDiagnostic {
        LuaDiagnostic::new(code, message, severity, false)
    }
}
