// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFolder.ExpressionFlags (b767b4e):
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.ExpressionFlags.cs
// (the ExpressionFlags enum and its HasEFlag helper; the GetFlags/IsConstTable
// extensions need the folder's options, so they remain ConstantFolder methods).

/// C# ConstantFolder.ExpressionFlags (ConstantFolder.ExpressionFlags.cs:8-26).
pub const FLAG_IS_NIL: u16 = 1 << 0;
pub const FLAG_IS_DOUBLE: u16 = 1 << 1;
pub const FLAG_IS_STR: u16 = 1 << 2;
pub const FLAG_IS_BOOL: u16 = 1 << 3;
pub const FLAG_IS_TRUTHY: u16 = 1 << 4;
pub const FLAG_IS_FALSEY: u16 = 1 << 5;
pub const FLAG_IS_CONSTANT_TABLE: u16 = 1 << 6;
pub const FLAG_IS_ANONYMOUS_FUNCTION: u16 = 1 << 7;
pub const FLAG_IS_LONG: u16 = 1 << 8;
pub const FLAG_IS_STRING_WITH_NUMBER: u16 = 1 << 9;

pub const FLAG_CAN_CONVERT_TO_BOOL: u16 = FLAG_IS_TRUTHY | FLAG_IS_FALSEY;
pub const FLAG_IS_SCALAR: u16 =
    FLAG_IS_NIL | FLAG_IS_DOUBLE | FLAG_IS_LONG | FLAG_IS_STR | FLAG_IS_BOOL;
pub const FLAG_IS_CONSTANT: u16 =
    FLAG_IS_SCALAR | FLAG_IS_CONSTANT_TABLE | FLAG_IS_ANONYMOUS_FUNCTION;
pub const FLAG_IS_NUM: u16 = FLAG_IS_DOUBLE | FLAG_IS_LONG | FLAG_IS_STRING_WITH_NUMBER;

/// C# HasEFlag(ExpressionFlags, ExpressionFlags) (ConstantFolder.ExpressionFlags.cs:120).
pub fn has_e_flag(flags: u16, wanted_flag: u16) -> bool {
    (flags & wanted_flag) != 0
}
