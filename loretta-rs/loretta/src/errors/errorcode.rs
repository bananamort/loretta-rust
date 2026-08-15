// Ported from Loretta.CodeAnalysis.Lua.ErrorCode (b767b4e): ErrorCode
// C# source: src/Compilers/Lua/Portable/Errors/ErrorCode.cs

/// Error codes for Lua diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ErrorCode {
    /// The code was lazily determined and does not need to be reported.
    Void = -2,

    /// The code has yet to be determined.
    Unknown = -1,

    // Lexer Errors
    /// C#: ERR_InvalidStringEscape
    ErrInvalidStringEscape = 1,
    /// C#: ERR_UnfinishedString
    ErrUnfinishedString = 3,
    /// C#: ERR_InvalidNumber
    ErrInvalidNumber = 4,
    /// C#: ERR_NumericLiteralTooLarge
    ErrNumericLiteralTooLarge = 5,
    /// C#: ERR_UnfinishedLongComment
    ErrUnfinishedLongComment = 6,
    /// C#: ERR_ShebangNotSupportedInLuaVersion
    ErrShebangNotSupportedInLuaVersion = 7,
    /// C#: ERR_BinaryNumericLiteralNotSupportedInVersion
    ErrBinaryNumericLiteralNotSupportedInVersion = 8,
    /// C#: ERR_OctalNumericLiteralNotSupportedInVersion
    ErrOctalNumericLiteralNotSupportedInVersion = 9,
    /// C#: ERR_HexFloatLiteralNotSupportedInVersion
    ErrHexFloatLiteralNotSupportedInVersion = 10,
    /// C#: ERR_UnderscoreInNumericLiteralNotSupportedInVersion
    ErrUnderscoreInNumericLiteralNotSupportedInVersion = 11,
    /// C#: ERR_CCommentsNotSupportedInVersion
    ErrCCommentsNotSupportedInVersion = 12,
    /// C#: ERR_LuajitIdentifierRulesNotSupportedInVersion
    ErrLuajitIdentifierRulesNotSupportedInVersion = 13,
    /// C#: ERR_BadCharacter
    ErrBadCharacter = 14,
    /// C#: ERR_UnexpectedToken
    ErrUnexpectedToken = 15,
    /// C#: ERR_HexStringEscapesNotSupportedInVersion
    ErrHexStringEscapesNotSupportedInVersion = 16,
    /// C#: ERR_AmbiguousFunctionCallOrNewStatement
    ErrAmbiguousFunctionCallOrNewStatement = 17,
    /// C#: ERR_NonFunctionCallBeingUsedAsStatement
    ErrNonFunctionCallBeingUsedAsStatement = 18,
    /// C#: ERR_CannotBeAssignedTo
    ErrCannotBeAssignedTo = 19,
    /// C#: ERR_DoubleOverflow
    ErrDoubleOverflow = 20,
    /// C#: ERR_BitwiseOperatorsNotSupportedInVersion
    ErrBitwiseOperatorsNotSupportedInVersion = 21,
    /// C#: WRN_LineBreakMayAffectErrorReporting
    WrnLineBreakMayAffectErrorReporting = 22,
    /// C#: ERR_WhitespaceEscapeNotSupportedInVersion
    ErrWhitespaceEscapeNotSupportedInVersion = 23,
    /// C#: ERR_UnicodeEscapeMissingOpenBrace
    ErrUnicodeEscapeMissingOpenBrace = 24,
    /// C#: ERR_UnicodeEscapeMissingCloseBrace
    ErrUnicodeEscapeMissingCloseBrace = 25,
    /// C#: ERR_EscapeTooLarge
    ErrEscapeTooLarge = 26,
    /// C#: ERR_HexDigitExpected
    ErrHexDigitExpected = 27,
    /// C#: ERR_UnicodeEscapesNotSupportedLuaInVersion
    ErrUnicodeEscapesNotSupportedLuaInVersion = 28,
    /// C#: ERR_NumberSuffixNotSupportedInVersion
    ErrNumberSuffixNotSupportedInVersion = 30,
    /// C#: ERR_LuajitSuffixInFloat
    ErrLuajitSuffixInFloat = 31,
    /// C#: ERR_Lua51NestingInLongString
    ErrLua51NestingInLongString = 32,
    /// C#: ERR_InterpolatedStringMustStartWithBacktickCharacter
    ErrInterpolatedStringMustStartWithBacktickCharacter = 33,
    /// C#: ERR_UnclosedExpressionHole
    ErrUnclosedExpressionHole = 34,
    /// C#: ERR_DoubleBraceInInterpolation
    ErrDoubleBraceInInterpolation = 35,
    /// C#: ERR_InterpolatedStringsNotSupportedInVersion
    ErrInterpolatedStringsNotSupportedInVersion = 36,

    // Parser Errors
    /// C#: ERR_IdentifierExpectedKW
    ErrIdentifierExpectedKw = 1000,
    /// C#: ERR_IdentifierExpected
    ErrIdentifierExpected = 1001,
    /// C#: ERR_SemicolonExpected
    ErrSemicolonExpected = 1002,
    /// C#: ERR_CloseParenExpected
    ErrCloseParenExpected = 1003,
    /// C#: ERR_LbraceExpected
    ErrLbraceExpected = 1004,
    /// C#: ERR_RbraceExpected
    ErrRbraceExpected = 1005,
    /// C#: ERR_SyntaxError
    ErrSyntaxError = 1006,
    /// C#: ERR_InsufficientStack
    ErrInsufficientStack = 1007,
    /// C#: ERR_IfExpressionsNotSupportedInLuaVersion
    ErrIfExpressionsNotSupportedInLuaVersion = 1008,
    /// C#: ERR_IfExpressionConditionExpected
    ErrIfExpressionConditionExpected = 1009,
    /// C#: ERR_ExpressionExpected
    ErrExpressionExpected = 1010,
    /// C#: ERR_InvalidExpressionPart
    ErrInvalidExpressionPart = 1011,
    /// C#: ERR_InvalidStatement
    ErrInvalidStatement = 1012,
    /// C#: ERR_CompoundAssignmentNotSupportedInLuaVersion
    ErrCompoundAssignmentNotSupportedInLuaVersion = 1013,
    /// C#: ERR_MixingNilableAndIntersectionNotAllowed
    ErrMixingNilableAndIntersectionNotAllowed = 1014,
    /// C#: ERR_MixingUnionsAndIntersectionsNotAllowed
    ErrMixingUnionsAndIntersectionsNotAllowed = 1015,
    /// C#: ERR_TypedLuaNotSupportedInLuaVersion
    ErrTypedLuaNotSupportedInLuaVersion = 1016,
    /// C#: ERR_OnlyOneTableTypeIndexerIsAllowed
    ErrOnlyOneTableTypeIndexerIsAllowed = 1017,
    /// C#: ERR_NormalTypeParametersComeBeforePacks
    ErrNormalTypeParametersComeBeforePacks = 1018,
    /// C#: ERR_GotoNotSupportedInLuaVersion
    ErrGotoNotSupportedInLuaVersion = 1019,

    // MessageProvider stuff
    /// C#: ERR_BadDocumentationMode
    ErrBadDocumentationMode = 2000,
}
