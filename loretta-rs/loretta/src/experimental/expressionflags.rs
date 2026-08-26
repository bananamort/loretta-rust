// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFolder.ExpressionFlags (b767b4e)
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.ExpressionFlags.cs

use crate::experimental::constantfolder::{
    can_convert_to_boolean, get_inner_expression, is_falsey, number_is_double, string_value,
    ConstantFolder,
};
use crate::experimental::numparsing::try_parse_number_in_string;
use full_moon::ast;
use full_moon::tokenizer::Symbol;

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

impl ConstantFolder {
    /// C# GetFlags (ConstantFolder.ExpressionFlags.cs:31-77). Computed purely
    /// (the C# caches per SyntaxNode; the flags are a pure function of the
    /// inner expression's shape, so the cache is behavior-neutral).
    pub(crate) fn get_flags(&self, node: &ast::Expression) -> u16 {
        let inner = get_inner_expression(node);
        let mut flags: u16 = 0;
        match inner {
            ast::Expression::Symbol(t) if t.is_symbol(Symbol::Nil) => {
                flags |= FLAG_IS_NIL;
            }
            ast::Expression::Number(t) => {
                if number_is_double(&t.token().to_string()) {
                    flags |= FLAG_IS_DOUBLE;
                } else {
                    flags |= FLAG_IS_LONG;
                }
            }
            ast::Expression::String(t) => {
                flags |= FLAG_IS_STR;
                if self.options.extract_numbers_from_strings
                    && try_parse_number_in_string(&string_value(
                        t,
                        self.syntax_options.accept_invalid_escapes,
                    ))
                    .is_some()
                {
                    flags |= FLAG_IS_STRING_WITH_NUMBER;
                }
            }
            ast::Expression::Symbol(t)
                if t.is_symbol(Symbol::True) || t.is_symbol(Symbol::False) =>
            {
                flags |= FLAG_IS_BOOL;
            }
            _ => {}
        }
        if can_convert_to_boolean(inner) {
            flags |= if is_falsey(inner) {
                FLAG_IS_FALSEY
            } else {
                FLAG_IS_TRUTHY
            };
        }
        if let ast::Expression::TableConstructor(tc) = inner {
            if self.is_const_table(tc) {
                flags |= FLAG_IS_CONSTANT_TABLE;
            }
        }
        if matches!(inner, ast::Expression::Function(_)) {
            flags |= FLAG_IS_ANONYMOUS_FUNCTION;
        }
        flags
    }

    /// C# IsConstTable (ConstantFolder.ExpressionFlags.cs:79-118).
    fn is_const_table(&self, table_constructor: &ast::TableConstructor) -> bool {
        for field in table_constructor.fields().iter() {
            match field {
                ast::Field::NameKey { value, .. } => {
                    if !self.is_const(value) {
                        return false;
                    }
                }
                ast::Field::ExpressionKey { key, value, .. } => {
                    if !self.is_const(key) || !self.is_const(value) {
                        return false;
                    }
                }
                ast::Field::NoKey(value) => {
                    if !self.is_const(value) {
                        return false;
                    }
                }
                // C# SetConstructor has no counterpart (the C# switch default
                // throws UnexpectedValue); treat it as non-constant.
                ast::Field::SetConstructor { .. } => return false,
                #[allow(unreachable_patterns)]
                _ => return false,
            }
        }
        true
    }

    /// C# isConst local (ConstantFolder.ExpressionFlags.cs:116-117).
    fn is_const(&self, node: &ast::Expression) -> bool {
        has_e_flag(
            self.get_flags(node),
            FLAG_IS_CONSTANT | FLAG_IS_CONSTANT_TABLE,
        )
    }

    /// C# HasEFlag(SyntaxNode, ExpressionFlags) (ConstantFolder.ExpressionFlags.cs:122-123).
    pub(crate) fn has_e_flag(&self, node: &ast::Expression, wanted_flag: u16) -> bool {
        has_e_flag(self.get_flags(node), wanted_flag)
    }
}

/// C# HasEFlag(ExpressionFlags, ExpressionFlags) (ConstantFolder.ExpressionFlags.cs:120).
pub(crate) fn has_e_flag(flags: u16, wanted_flag: u16) -> bool {
    (flags & wanted_flag) != 0
}
