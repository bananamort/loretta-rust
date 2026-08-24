// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFolder (b767b4e)
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.cs,
// ConstantFolder.ExpressionFlags.cs, ConstantFolder.NumberParsing.cs

use crate::experimental::constantfoldingoptions::ConstantFoldingOptions;
use crate::symbol_display::objectdisplay::ObjectDisplay;
use crate::symbol_display::objectdisplayoptions::ObjectDisplayOptions;
use crate::utilities::hexfloat::HexFloat;
use crate::utilities::stringutils::StringUtils;
use full_moon::ast;
use full_moon::ast::span::ContainedSpan;
use full_moon::tokenizer::{StringLiteralQuoteType, Symbol, Token, TokenReference, TokenType};
use full_moon::visitors::{VisitMut, VisitorMut};
use full_moon::ShortString;

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

/// C# ConstantFolder (ConstantFolder.cs:8-15): the options-holding rewriter.
#[derive(Clone)]
pub struct ConstantFolder {
    options: ConstantFoldingOptions,
}

/// The numeric value of an expression (C# `dynamic` long/double).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumValue {
    Long(i64),
    Double(f64),
}

impl ConstantFolder {
    /// C# ConstantFolder(ConstantFoldingOptions) (ConstantFolder.cs:12-15).
    pub fn new(options: ConstantFoldingOptions) -> Self {
        ConstantFolder { options }
    }

    /// C# LuaExtensions.ConstantFold: runs the rewriter over the tree.
    pub fn fold(&mut self, ast: ast::Ast) -> ast::Ast {
        let nodes = ast.nodes().clone().visit_mut(self);
        ast.with_nodes(nodes)
    }

    /// C# GetFlags (ConstantFolder.ExpressionFlags.cs:31-77). Computed purely
    /// (the C# caches per SyntaxNode; the flags are a pure function of the
    /// inner expression's shape, so the cache is behavior-neutral).
    fn get_flags(&self, node: &ast::Expression) -> u16 {
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
                    && try_parse_number_in_string(&string_value(t)).is_some()
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
    fn has_e_flag(&self, node: &ast::Expression, wanted_flag: u16) -> bool {
        has_e_flag(self.get_flags(node), wanted_flag)
    }

    /// C# TryGetNumValue (ConstantFolder.cs:498-515).
    fn try_get_num_value(&self, node: &ast::Expression) -> Option<NumValue> {
        let flags = self.get_flags(node);
        if has_e_flag(flags, FLAG_IS_NUM) {
            if has_e_flag(flags, FLAG_IS_STRING_WITH_NUMBER) {
                let inner = get_inner_expression(node);
                let ast::Expression::String(t) = inner else {
                    unreachable!("IsStringWithNumber requires a string literal");
                };
                return try_parse_number_in_string(&string_value(t));
            }
            return Some(number_value(node));
        }
        None
    }

    /// C# TryGetInt32 (ConstantFolder.cs:517-526).
    fn try_get_int32(&self, node: &ast::Expression) -> Option<i32> {
        let converted64 = self.try_get_int64(node)?;
        let converted = converted64 as i32;
        if converted64 > i32::MIN as i64 && converted64 < i32::MAX as i64 {
            Some(converted)
        } else {
            None
        }
    }

    /// C# TryGetInt64 (ConstantFolder.cs:528-547).
    fn try_get_int64(&self, node: &ast::Expression) -> Option<i64> {
        match self.try_get_num_value(node)? {
            NumValue::Long(i64) => Some(i64),
            NumValue::Double(d) => {
                let tmp = d;
                let converted = tmp.trunc() as i64;
                if tmp == converted as f64 {
                    Some(converted)
                } else {
                    None
                }
            }
        }
    }

    /// C# TryConvertToBool (ConstantFolder.cs:549-560).
    fn try_convert_to_bool(node: &ast::Expression) -> Option<bool> {
        let inner_node = get_inner_expression(node);
        if can_convert_to_boolean(inner_node) {
            Some(!is_falsey(inner_node))
        } else {
            None
        }
    }

    /// C# TryConvertToDouble(long) (ConstantFolder.cs:562-566).
    fn try_convert_to_double(value: i64) -> Option<f64> {
        let converted = value as f64;
        if value == converted as i64 {
            Some(converted)
        } else {
            None
        }
    }

    /// C# VisitParenthesizedExpression (ConstantFolder.cs:17-23).
    fn visit_parenthesized(
        &mut self,
        leading: Vec<Token>,
        contained: ContainedSpan,
        expression: Box<ast::Expression>,
    ) -> ast::Expression {
        let inner_expr = expression.visit_mut(self);
        if let ast::Expression::Parentheses { .. } = &*inner_expr {
            // C# WithTriviaFrom(innerParenthesized, node): the leading comes
            // from the outer node; the inner keeps its own trailing.
            return set_first_leading(*inner_expr, leading);
        }
        ast::Expression::Parentheses {
            contained,
            expression: inner_expr,
        }
    }

    /// C# VisitUnaryExpression (ConstantFolder.cs:25-46).
    fn visit_unary(
        &mut self,
        leading: Vec<Token>,
        trailing: Vec<Token>,
        unop: ast::UnOp,
        operand: Box<ast::Expression>,
    ) -> ast::Expression {
        let operand = operand.visit_mut(self);
        let operand_flags = self.get_flags(&operand);
        match &unop {
            ast::UnOp::Minus(_) => {
                if let Some(val) = self.try_get_num_value(&operand) {
                    return literal_num(negate_num(val), &leading, &trailing);
                }
            }
            ast::UnOp::Not(_) => {
                if let Some(value) = Self::try_convert_to_bool(&operand) {
                    return literal_bool(!value, &leading, &trailing);
                }
            }
            ast::UnOp::Tilde(_) => {
                if has_e_flag(operand_flags, FLAG_IS_DOUBLE | FLAG_IS_STRING_WITH_NUMBER) {
                    if let Some(value) = self.try_get_int64(&operand) {
                        if let Some(result) = Self::try_convert_to_double(!value) {
                            return literal_double(result, &leading, &trailing);
                        }
                    }
                } else if has_e_flag(operand_flags, FLAG_IS_LONG) {
                    if let Some(value) = self.try_get_int64(&operand) {
                        return literal_long(!value, &leading, &trailing);
                    }
                }
            }
            ast::UnOp::Hash(_) if has_e_flag(operand_flags, FLAG_IS_STR) => {
                let len = get_string_value(&operand).len() as f64;
                return literal_double(len, &leading, &trailing);
            }
            _ => {}
        }
        ast::Expression::UnaryOperator {
            unop,
            expression: operand,
        }
    }

    /// C# VisitBinaryExpression (ConstantFolder.cs:48-334).
    fn visit_binary(
        &mut self,
        leading: Vec<Token>,
        trailing: Vec<Token>,
        lhs: Box<ast::Expression>,
        binop: ast::BinOp,
        rhs: Box<ast::Expression>,
    ) -> ast::Expression {
        let left = lhs.visit_mut(self);
        let right = rhs.visit_mut(self);
        let left_flags = self.get_flags(&left);
        let right_flags = self.get_flags(&right);

        let both_num = |this: &Self| -> Option<(NumValue, NumValue)> {
            let l = this.try_get_num_value(&left)?;
            let r = this.try_get_num_value(&right)?;
            Some((l, r))
        };

        // C# arithmetic: `result is double d && (IsNaN(d) || IsInfinity(d))`
        // breaks the case; long results can't be NaN/Infinity.
        let arithmetic = |result: NumValue| -> Option<ast::Expression> {
            if let NumValue::Double(d) = result {
                if d.is_nan() || d.is_infinite() {
                    return None;
                }
            }
            Some(literal_num(result, &leading, &trailing))
        };

        match &binop {
            ast::BinOp::Plus(_) => {
                if let Some((l, r)) = both_num(self) {
                    if let Some(expr) = arithmetic(num_add(l, r)) {
                        return expr;
                    }
                }
            }
            ast::BinOp::Minus(_) => {
                if let Some((l, r)) = both_num(self) {
                    if let Some(expr) = arithmetic(num_sub(l, r)) {
                        return expr;
                    }
                }
            }
            ast::BinOp::Star(_) => {
                if let Some((l, r)) = both_num(self) {
                    if let Some(expr) = arithmetic(num_mul(l, r)) {
                        return expr;
                    }
                }
            }
            ast::BinOp::Slash(_) => {
                if let Some((l, r)) = both_num(self) {
                    // C#: `var result = (double) (leftNum / (double) rightNum);`
                    let result = NumValue::Double(num_div(l, r));
                    if let NumValue::Double(d) = result {
                        if !d.is_nan() && !d.is_infinite() {
                            return literal_num(result, &leading, &trailing);
                        }
                    }
                }
            }
            ast::BinOp::Percent(_) => {
                if let Some((l, r)) = both_num(self) {
                    if let Some(expr) = arithmetic(num_mod(l, r)) {
                        return expr;
                    }
                }
            }
            ast::BinOp::Caret(_) => {
                if let Some((l, r)) = both_num(self) {
                    // C#: Math.Pow((double) leftNum, (double) rightNum)
                    let result = NumValue::Double(num_pow(l, r));
                    if let NumValue::Double(d) = result {
                        if !d.is_nan() && !d.is_infinite() {
                            return literal_num(result, &leading, &trailing);
                        }
                    }
                }
            }
            ast::BinOp::TwoDots(_) => {
                if has_e_flag(left_flags, FLAG_IS_STR | FLAG_IS_BOOL)
                    && has_e_flag(right_flags, FLAG_IS_STR | FLAG_IS_BOOL)
                {
                    // C# left.Kind()/right.Kind() switch (ConstantFolder.cs:122-135).
                    let left_str = match get_inner_expression(&left) {
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::True) => {
                            "true".to_string()
                        }
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::False) => {
                            "false".to_string()
                        }
                        ast::Expression::String(_) => get_string_value(&left),
                        _ => unreachable!("concat operand must be a literal"),
                    };
                    let right_str = match get_inner_expression(&right) {
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::True) => {
                            "true".to_string()
                        }
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::False) => {
                            "false".to_string()
                        }
                        ast::Expression::String(_) => get_string_value(&right),
                        _ => unreachable!("concat operand must be a literal"),
                    };
                    return literal_str(format!("{left_str}{right_str}"), &leading, &trailing);
                }
            }
            ast::BinOp::TwoEqual(_) => {
                if has_e_flag(left_flags, FLAG_IS_SCALAR) && has_e_flag(right_flags, FLAG_IS_SCALAR)
                {
                    let result = expr_equals(self, &left, &right, left_flags, right_flags);
                    return literal_bool(result, &leading, &trailing);
                }
            }
            ast::BinOp::TildeEqual(_) => {
                if has_e_flag(left_flags, FLAG_IS_SCALAR) && has_e_flag(right_flags, FLAG_IS_SCALAR)
                {
                    let result = !expr_equals(self, &left, &right, left_flags, right_flags);
                    return literal_bool(result, &leading, &trailing);
                }
            }
            ast::BinOp::LessThan(_) => {
                if can_compare(left_flags, right_flags) {
                    let result = compare(&left, &right, left_flags, right_flags);
                    return literal_bool(result < 0, &leading, &trailing);
                }
            }
            ast::BinOp::LessThanEqual(_) => {
                if can_compare(left_flags, right_flags) {
                    let result = compare(&left, &right, left_flags, right_flags);
                    return literal_bool(result <= 0, &leading, &trailing);
                }
            }
            ast::BinOp::GreaterThan(_) => {
                if can_compare(left_flags, right_flags) {
                    let result = compare(&left, &right, left_flags, right_flags);
                    return literal_bool(result > 0, &leading, &trailing);
                }
            }
            ast::BinOp::GreaterThanEqual(_) => {
                if can_compare(left_flags, right_flags) {
                    let result = compare(&left, &right, left_flags, right_flags);
                    return literal_bool(result >= 0, &leading, &trailing);
                }
            }
            ast::BinOp::And(_) => {
                if let Some(result) = Self::try_convert_to_bool(&left) {
                    return if result { *right } else { *left };
                }
            }
            ast::BinOp::Or(_) => {
                if let Some(result) = Self::try_convert_to_bool(&left) {
                    return if !result { *right } else { *left };
                }
            }
            ast::BinOp::Pipe(_) | ast::BinOp::Ampersand(_) | ast::BinOp::Tilde(_)
                if has_e_flag(left_flags, FLAG_IS_NUM) && has_e_flag(right_flags, FLAG_IS_NUM) =>
            {
                if let (Some(left_val), Some(right_val)) =
                    (self.try_get_int64(&left), self.try_get_int64(&right))
                {
                    let result = match &binop {
                        ast::BinOp::Pipe(_) => left_val | right_val,
                        ast::BinOp::Ampersand(_) => left_val & right_val,
                        _ => left_val ^ right_val,
                    };
                    if has_e_flag(left_flags, FLAG_IS_LONG) || has_e_flag(right_flags, FLAG_IS_LONG)
                    {
                        return literal_long(result, &leading, &trailing);
                    } else if let Some(converted) = Self::try_convert_to_double(result) {
                        return literal_double(converted, &leading, &trailing);
                    }
                }
            }
            ast::BinOp::DoubleGreaterThan(_) | ast::BinOp::DoubleLessThan(_)
                if has_e_flag(left_flags, FLAG_IS_NUM) && has_e_flag(right_flags, FLAG_IS_NUM) =>
            {
                if let (Some(left_val), Some(right_val)) =
                    (self.try_get_int64(&left), self.try_get_int32(&right))
                {
                    // C# shifts mask the count; use wrapping_*.
                    let result = match &binop {
                        ast::BinOp::DoubleGreaterThan(_) => left_val.wrapping_shr(right_val as u32),
                        _ => left_val.wrapping_shl(right_val as u32),
                    };
                    if has_e_flag(left_flags, FLAG_IS_LONG) || has_e_flag(right_flags, FLAG_IS_LONG)
                    {
                        return literal_long(result, &leading, &trailing);
                    } else if let Some(converted) = Self::try_convert_to_double(result) {
                        return literal_double(converted, &leading, &trailing);
                    }
                }
            }
            _ => {}
        }

        ast::Expression::BinaryOperator {
            lhs: left,
            binop,
            rhs: right,
        }
    }

    /// C# VisitMemberAccessExpression + VisitElementAccessExpression
    /// (ConstantFolder.cs:336-407).
    fn visit_var(&mut self, leading: Vec<Token>, var: ast::Var) -> ast::Expression {
        let ast::Var::Expression(ve) = var else {
            // A bare name: nothing to fold; keep it.
            return ast::Expression::Var(var);
        };
        let prefix = ve.prefix().clone();
        let suffixes: Vec<ast::Suffix> = ve.suffixes().cloned().collect();

        let prefix = match prefix {
            ast::Prefix::Expression(e) => ast::Prefix::Expression(e.visit_mut(self)),
            name @ ast::Prefix::Name(_) => name,
            #[allow(unreachable_patterns)]
            _ => unreachable!("unsupported prefix kind"),
        };
        let suffixes: Vec<ast::Suffix> = suffixes
            .into_iter()
            .map(|suffix| match suffix {
                ast::Suffix::Index(ast::Index::Brackets {
                    brackets,
                    expression,
                }) => ast::Suffix::Index(ast::Index::Brackets {
                    brackets,
                    expression: expression.visit_mut(self),
                }),
                other => other,
            })
            .collect();

        // C#: the fold applies when the immediate base (node.Expression) is a
        // constant table. In the C# AST a directly-indexed table is wrapped
        // in a PrefixExpressionSyntax (which GetFlags can't see through), so
        // only a parenthesized base folds — the port replicates that by
        // requiring the base to be a Parentheses (full_moon has no
        // prefix-wrapper).
        if suffixes.len() == 1 {
            if let ast::Suffix::Index(index) = &suffixes[0] {
                let base_expr = expr_from_prefix(&prefix);
                if matches!(&base_expr, ast::Expression::Parentheses { .. })
                    && self.has_e_flag(&base_expr, FLAG_IS_CONSTANT_TABLE)
                {
                    let table = get_inner_expression(&base_expr);
                    let table = if let ast::Expression::TableConstructor(tc) = table {
                        tc.clone()
                    } else {
                        unreachable!("IsConstantTable requires a table constructor");
                    };
                    match index {
                        ast::Index::Dot { name, .. } => {
                            if let Some(value) =
                                lookup_table_field(&table, Some(&name.token().to_string()), None)
                            {
                                return set_first_leading(value, leading);
                            }
                        }
                        ast::Index::Brackets { expression, .. }
                            if self.has_e_flag(expression, FLAG_IS_SCALAR) =>
                        {
                            if let Some(value) =
                                lookup_table_field(&table, None, Some((expression, self)))
                            {
                                return set_first_leading(value, leading);
                            }
                        }
                        #[allow(unreachable_patterns)]
                        _ => {}
                    }
                }
            }
        }

        ast::Expression::Var(ast::Var::Expression(Box::new(
            ast::VarExpression::new(prefix).with_suffixes(suffixes),
        )))
    }
}

impl VisitorMut for ConstantFolder {
    /// C# LuaSyntaxRewriter overrides for the four expression shapes; every
    /// other expression kind keeps the default behavior (recurse into
    /// children), which the derived VisitMut does after this returns.
    fn visit_expression(&mut self, node: ast::Expression) -> ast::Expression {
        let leading = first_leading(&node);
        let trailing = last_trailing(&node);
        match node {
            ast::Expression::Parentheses {
                contained,
                expression,
            } => self.visit_parenthesized(leading, contained, expression),
            ast::Expression::UnaryOperator { unop, expression } => {
                self.visit_unary(leading, trailing, unop, expression)
            }
            ast::Expression::BinaryOperator { lhs, binop, rhs } => {
                self.visit_binary(leading, trailing, lhs, binop, rhs)
            }
            ast::Expression::Var(var) => self.visit_var(leading, var),
            other => other,
        }
    }
}

/// C# WithTriviaFrom(SyntaxToken, SyntaxNode) (ConstantFolder.cs:435-439):
/// the literal token takes the container's leading AND trailing trivia.
fn literal_from_text(
    text: String,
    leading: &[Token],
    trailing: &[Token],
    make_token_type: impl FnOnce(String) -> TokenType,
) -> ast::Expression {
    let token_ref = TokenReference::new(
        leading.to_vec(),
        Token::new(make_token_type(text)),
        trailing.to_vec(),
    );
    let kind = token_ref.token().token_type().clone();
    match kind {
        TokenType::Number { .. } => ast::Expression::Number(token_ref),
        TokenType::StringLiteral { .. } => ast::Expression::String(token_ref),
        _ => ast::Expression::Symbol(token_ref),
    }
}

/// C# LiteralExpressionWithTriviaFrom(long) (ConstantFolder.cs:409-413).
fn literal_long(value: i64, leading: &[Token], trailing: &[Token]) -> ast::Expression {
    let text = ObjectDisplay::format_literal_i64(value, ObjectDisplayOptions::NONE);
    literal_from_text(text, leading, trailing, |text| TokenType::Number {
        text: ShortString::new(text),
    })
}

/// C# LiteralExpressionWithTriviaFrom(double) (ConstantFolder.cs:415-419).
fn literal_double(value: f64, leading: &[Token], trailing: &[Token]) -> ast::Expression {
    let text = ObjectDisplay::format_literal_f64(value, ObjectDisplayOptions::NONE);
    literal_from_text(text, leading, trailing, |text| TokenType::Number {
        text: ShortString::new(text),
    })
}

/// C# LiteralExpressionWithTriviaFrom(string) (ConstantFolder.cs:421-425).
fn literal_str(value: String, leading: &[Token], trailing: &[Token]) -> ast::Expression {
    // C# Literal(string) = FormatLiteral(value, UseQuotes | EscapeNonPrintable)
    // — the quoted text is the token's text. full_moon stores the literal
    // without the quotes (its Display adds them), so strip the pair.
    let escaped = ObjectDisplay::format_literal_str(
        &value,
        ObjectDisplayOptions::USE_QUOTES | ObjectDisplayOptions::ESCAPE_NON_PRINTABLE_CHARACTERS,
    );
    let literal_text = escaped
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(&escaped)
        .to_string();
    literal_from_text(literal_text, leading, trailing, |text| {
        TokenType::StringLiteral {
            literal: ShortString::new(text),
            multi_line_depth: 0,
            quote_type: StringLiteralQuoteType::Double,
        }
    })
}

/// C# LiteralExpressionWithTriviaFrom(bool) (ConstantFolder.cs:427-433).
fn literal_bool(value: bool, leading: &[Token], trailing: &[Token]) -> ast::Expression {
    let symbol = if value { Symbol::True } else { Symbol::False };
    literal_from_text(String::new(), leading, trailing, |_| TokenType::Symbol {
        symbol,
    })
}

/// Builds the literal expression from a numeric value (C# LiteralExpression
/// with the corresponding kind).
fn literal_num(value: NumValue, leading: &[Token], trailing: &[Token]) -> ast::Expression {
    match value {
        NumValue::Long(v) => literal_long(v, leading, trailing),
        NumValue::Double(d) => literal_double(d, leading, trailing),
    }
}

/// C# GetInnerExpression (ConstantFolder.cs:447-450).
fn get_inner_expression(node: &ast::Expression) -> &ast::Expression {
    match node {
        ast::Expression::Parentheses { expression, .. } => get_inner_expression(expression),
        other => other,
    }
}

/// C# CanConvertToBoolean (ConstantFolder.cs:458-470).
fn can_convert_to_boolean(node: &ast::Expression) -> bool {
    match node {
        ast::Expression::Symbol(t) => {
            t.is_symbol(Symbol::Nil) || t.is_symbol(Symbol::True) || t.is_symbol(Symbol::False)
        }
        ast::Expression::Number(_) | ast::Expression::String(_) | ast::Expression::Function(_) => {
            true
        }
        _ => false,
    }
}

/// C# IsFalsey (ConstantFolder.cs:477-481).
fn is_falsey(node: &ast::Expression) -> bool {
    debug_assert!(can_convert_to_boolean(node));
    matches!(
        node,
        ast::Expression::Symbol(t)
            if t.is_symbol(Symbol::Nil) || t.is_symbol(Symbol::False)
    )
}

/// C# GetValue(node) — the literal token's value for a Number node.
fn number_value(node: &ast::Expression) -> NumValue {
    let inner = get_inner_expression(node);
    let ast::Expression::Number(t) = inner else {
        unreachable!("number value requires a number literal");
    };
    let text = t.token().to_string();
    if number_is_double(&text) {
        let parsed = parse_double_literal(&text)
            .unwrap_or_else(|| panic!("invalid number literal {text:?}"));
        NumValue::Double(parsed)
    } else {
        // C# fold-as-0: TryParse failure leaves the default value and the
        // lexer reports ERR_NumericLiteralTooLarge (Lexer.Numbers.cs).
        NumValue::Long(parse_integer_literal(&text))
    }
}

/// C# GetValue<string> — the decoded string value of a String literal.
fn get_string_value(node: &ast::Expression) -> String {
    let inner = get_inner_expression(node);
    let ast::Expression::String(t) = inner else {
        unreachable!("string value requires a string literal");
    };
    string_value(t)
}

/// C# HasEFlag(ExpressionFlags, ExpressionFlags) (ConstantFolder.ExpressionFlags.cs:120).
fn has_e_flag(flags: u16, wanted_flag: u16) -> bool {
    (flags & wanted_flag) != 0
}

/// C# exprEquals local (ConstantFolder.cs:302-314).
fn expr_equals(
    folder: &ConstantFolder,
    left: &ast::Expression,
    right: &ast::Expression,
    left_flags: u16,
    right_flags: u16,
) -> bool {
    if has_e_flag(left_flags, FLAG_IS_NIL) && has_e_flag(right_flags, FLAG_IS_NIL) {
        return true;
    }
    if has_e_flag(left_flags, FLAG_IS_STR) && has_e_flag(right_flags, FLAG_IS_STR) {
        return get_string_value(left) == get_string_value(right);
    }
    if has_e_flag(left_flags, FLAG_IS_BOOL) && has_e_flag(right_flags, FLAG_IS_BOOL) {
        return has_e_flag(left_flags, FLAG_IS_TRUTHY) == has_e_flag(right_flags, FLAG_IS_TRUTHY);
    }
    if let (Some(left_num), Some(right_num)) = (
        folder.try_get_num_value(left),
        folder.try_get_num_value(right),
    ) {
        return num_value_eq(left_num, right_num);
    }
    false
}

/// C# canCompare local (ConstantFolder.cs:316-318).
fn can_compare(left_flags: u16, right_flags: u16) -> bool {
    (has_e_flag(left_flags, FLAG_IS_NUM) && has_e_flag(right_flags, FLAG_IS_NUM))
        || (has_e_flag(left_flags, FLAG_IS_STR) && has_e_flag(right_flags, FLAG_IS_STR))
}

/// C# compare local (ConstantFolder.cs:320-333).
fn compare(
    left: &ast::Expression,
    right: &ast::Expression,
    left_flags: u16,
    right_flags: u16,
) -> i32 {
    if has_e_flag(left_flags, FLAG_IS_DOUBLE) && has_e_flag(right_flags, FLAG_IS_DOUBLE) {
        return double_cmp(get_double_value(left), get_double_value(right));
    }
    if has_e_flag(left_flags, FLAG_IS_LONG) && has_e_flag(right_flags, FLAG_IS_LONG) {
        return long_cmp(get_long_value(left), get_long_value(right));
    }
    if has_e_flag(left_flags, FLAG_IS_DOUBLE) && has_e_flag(right_flags, FLAG_IS_LONG) {
        // C#: Comparer<double>.Default.Compare(GetValue<double>(left), GetValue<long>(right))
        return double_cmp(get_double_value(left), get_long_value(right) as f64);
    }
    if has_e_flag(left_flags, FLAG_IS_LONG) && has_e_flag(right_flags, FLAG_IS_DOUBLE) {
        return double_cmp(get_long_value(left) as f64, get_double_value(right));
    }
    if has_e_flag(left_flags, FLAG_IS_STR) && has_e_flag(right_flags, FLAG_IS_STR) {
        // C# string.CompareOrdinal — byte-wise comparison.
        return long_cmp(
            get_string_value(left).as_bytes(),
            get_string_value(right).as_bytes(),
        );
    }
    panic!("Both expressions must have the same type.");
}

fn long_cmp<T: Ord>(l: T, r: T) -> i32 {
    match l.cmp(&r) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

fn double_cmp(l: f64, r: f64) -> i32 {
    if l < r {
        -1
    } else if l > r {
        1
    } else {
        0
    }
}

fn get_double_value(node: &ast::Expression) -> f64 {
    match number_value(node) {
        NumValue::Double(d) => d,
        NumValue::Long(v) => v as f64,
    }
}

fn get_long_value(node: &ast::Expression) -> i64 {
    match number_value(node) {
        NumValue::Long(v) => v,
        NumValue::Double(d) => d.trunc() as i64,
    }
}

fn num_value_eq(l: NumValue, r: NumValue) -> bool {
    match (l, r) {
        (NumValue::Long(a), NumValue::Long(b)) => a == b,
        (NumValue::Double(a), NumValue::Double(b)) => a == b,
        (NumValue::Long(a), NumValue::Double(b)) => a as f64 == b,
        (NumValue::Double(a), NumValue::Long(b)) => a == b as f64,
    }
}

fn negate_num(v: NumValue) -> NumValue {
    match v {
        NumValue::Long(x) => NumValue::Long(x.wrapping_neg()),
        NumValue::Double(x) => NumValue::Double(-x),
    }
}

fn num_add(l: NumValue, r: NumValue) -> NumValue {
    match (l, r) {
        (NumValue::Long(a), NumValue::Long(b)) => NumValue::Long(a.wrapping_add(b)),
        (NumValue::Long(a), NumValue::Double(b)) => NumValue::Double(a as f64 + b),
        (NumValue::Double(a), NumValue::Long(b)) => NumValue::Double(a + b as f64),
        (NumValue::Double(a), NumValue::Double(b)) => NumValue::Double(a + b),
    }
}

fn num_sub(l: NumValue, r: NumValue) -> NumValue {
    match (l, r) {
        (NumValue::Long(a), NumValue::Long(b)) => NumValue::Long(a.wrapping_sub(b)),
        (NumValue::Long(a), NumValue::Double(b)) => NumValue::Double(a as f64 - b),
        (NumValue::Double(a), NumValue::Long(b)) => NumValue::Double(a - b as f64),
        (NumValue::Double(a), NumValue::Double(b)) => NumValue::Double(a - b),
    }
}

fn num_mul(l: NumValue, r: NumValue) -> NumValue {
    match (l, r) {
        (NumValue::Long(a), NumValue::Long(b)) => NumValue::Long(a.wrapping_mul(b)),
        (NumValue::Long(a), NumValue::Double(b)) => NumValue::Double(a as f64 * b),
        (NumValue::Double(a), NumValue::Long(b)) => NumValue::Double(a * b as f64),
        (NumValue::Double(a), NumValue::Double(b)) => NumValue::Double(a * b),
    }
}

fn num_div(l: NumValue, r: NumValue) -> f64 {
    match (l, r) {
        (NumValue::Long(a), NumValue::Long(b)) => a as f64 / b as f64,
        (NumValue::Long(a), NumValue::Double(b)) => a as f64 / b,
        (NumValue::Double(a), NumValue::Long(b)) => a / b as f64,
        (NumValue::Double(a), NumValue::Double(b)) => a / b,
    }
}

fn num_mod(l: NumValue, r: NumValue) -> NumValue {
    match (l, r) {
        (NumValue::Long(a), NumValue::Long(b)) => {
            if b == 0 {
                // C# on the double path: `%` by zero is NaN and the case
                // breaks (no fold) (ConstantFolder.cs:100-102); the C# Int64
                // path throws DivideByZeroException — the port no-folds
                // instead of panicking (Finding 3).
                NumValue::Double(f64::NAN)
            } else {
                NumValue::Long(a.wrapping_rem(b))
            }
        }
        (NumValue::Long(a), NumValue::Double(b)) => NumValue::Double(a as f64 % b),
        (NumValue::Double(a), NumValue::Long(b)) => NumValue::Double(a % b as f64),
        (NumValue::Double(a), NumValue::Double(b)) => NumValue::Double(a % b),
    }
}

fn num_pow(l: NumValue, r: NumValue) -> f64 {
    let a = match l {
        NumValue::Long(x) => x as f64,
        NumValue::Double(x) => x,
    };
    let b = match r {
        NumValue::Long(x) => x as f64,
        NumValue::Double(x) => x,
    };
    a.powf(b)
}

/// C# number classification: the token's Value is double iff the text has a
/// '.', exponent ('e'/'E') or hex-float ('p'/'P'). In a hex literal the
/// 'e'/'E' characters are DIGITS, not exponents — only '.' and 'p'/'P'
/// (the hex-float markers) make it a double (e.g. 0xE5, 0x1e5 are
/// integers — Finding 19).
fn number_is_double(text: &str) -> bool {
    if text.starts_with("0x") || text.starts_with("0X") {
        text.contains('.') || text.contains('p') || text.contains('P')
    } else {
        text.contains('.')
            || text.contains('e')
            || text.contains('E')
            || text.contains('p')
            || text.contains('P')
    }
}

/// Parses an integer literal (decimal, hex or binary) like the C# lexer's
/// integer paths (Lexer.Numbers.cs): underscores are skipped (the C#
/// Consume*Digits builders), overflow folds to 0 (TryParse's default out
/// value, with ERR_NumericLiteralTooLarge reported by the diagnostics
/// side), and hex digits are a two's-complement bit pattern
/// (long.TryParse AllowHexSpecifier).
fn parse_integer_literal(text: &str) -> i64 {
    let text = text.trim();
    if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        // C# long.TryParse(AllowHexSpecifier) (Lexer.Numbers.cs:374-378):
        // 0xffffffffffffffff is -1; values wider than 64 bits fail -> 0.
        u64::from_str_radix(&rest.replace('_', ""), 16)
            .map(|bits| bits as i64)
            .unwrap_or(0)
    } else if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        // C# ParseBinaryNumber (Lexer.Numbers.cs:86-90): values with bit 63
        // set (no ull suffix) fold to 0.
        u64::from_str_radix(&rest.replace('_', ""), 2)
            .ok()
            .filter(|&bits| bits <= i64::MAX as u64)
            .map(|bits| bits as i64)
            .unwrap_or(0)
    } else {
        // C# long.TryParse (Lexer.Numbers.cs:258-262, 282-295): overflow -> 0.
        text.replace('_', "").parse::<i64>().unwrap_or(0)
    }
}

/// Parses a double literal (decimal float or hex float).
fn parse_double_literal(text: &str) -> Option<f64> {
    let text = text.trim();
    if text.starts_with("0x") || text.starts_with("0X") {
        HexFloat::double_from_hex_string(text).ok()
    } else {
        text.parse::<f64>().ok()
    }
}

/// C# TryParseNumberInString (ConstantFolder.NumberParsing.cs:20-66).
fn try_parse_number_in_string(value: &str) -> Option<NumValue> {
    let value = StringUtils::trim(value);
    // s_decIntegerRegex: ^[+\-]?\d+$ with long.TryParse(AllowLeadingSign)
    if is_dec_integer(value) {
        if let Ok(i64) = value.parse::<i64>() {
            return Some(NumValue::Long(i64));
        }
    }
    // s_hexIntegerRegex: ^[+\-]?0[xX][\da-fA-F]+$ with
    // long.TryParse(AllowLeadingSign | AllowHexSpecifier) — on .NET 8+ this
    // style combination throws ArgumentException at call time (pinned by the
    // constantfold-hex corpus case), so any hex-integer string panics with
    // the exact framework message.
    if is_hex_integer(value) {
        panic!(
            "With the AllowHexSpecifier or AllowBinarySpecifier bit set in the enum bit field, \
             the only other valid bits that can be combined into the enum value must be \
             AllowLeadingWhite and AllowTrailingWhite. (Parameter 'style')"
        );
    }
    // s_decFloatRegex with RealParser.TryParseDouble (invariant round-trip).
    if is_dec_float(value) {
        if let Some(f64) = parse_double_literal(value) {
            return Some(NumValue::Double(f64));
        }
    }
    // s_hexFloatRegex with HexFloat.DoubleFromHexString (try/catch -> None).
    if is_hex_float(value) {
        if let Ok(f64) = HexFloat::double_from_hex_string(value) {
            return Some(NumValue::Double(f64));
        }
    }
    None
}

/// s_decIntegerRegex: ^[+\-]?\d+$
fn is_dec_integer(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        idx = 1;
    }
    if idx == bytes.len() {
        return false;
    }
    bytes[idx..].iter().all(|b| b.is_ascii_digit())
}

/// s_hexIntegerRegex: ^[+\-]?0[xX][\da-fA-F]+$
fn is_hex_integer(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        idx = 1;
    }
    if bytes.len() - idx < 3 {
        return false;
    }
    if bytes[idx] != b'0' || (bytes[idx + 1] != b'x' && bytes[idx + 1] != b'X') {
        return false;
    }
    bytes[idx + 2..].iter().all(|b| b.is_ascii_hexdigit())
}

/// s_decFloatRegex: ^[+\-]?(\.\d+|\d+(\.\d+)?)([eE][+\-]?\d+)?$
fn is_dec_float(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        idx = 1;
    }
    let rest = &bytes[idx..];
    if rest.is_empty() {
        return false;
    }
    // (\.\d+ | \d+(\.\d+)?)
    let mut i = 0;
    if rest[0] == b'.' {
        i = 1;
        if i == rest.len() || !rest[i].is_ascii_digit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_digit() {
            i += 1;
        }
    } else {
        if !rest[0].is_ascii_digit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_digit() {
            i += 1;
        }
        if i < rest.len() && rest[i] == b'.' {
            i += 1;
            if i < rest.len() && rest[i].is_ascii_digit() {
                while i < rest.len() && rest[i].is_ascii_digit() {
                    i += 1;
                }
            }
        }
    }
    // ([eE][+\-]?\d+)?
    if i < rest.len() && (rest[i] == b'e' || rest[i] == b'E') {
        i += 1;
        if i < rest.len() && (rest[i] == b'+' || rest[i] == b'-') {
            i += 1;
        }
        if i == rest.len() || !rest[i].is_ascii_digit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_digit() {
            i += 1;
        }
    }
    i == rest.len()
}

/// s_hexFloatRegex:
/// [+\-]?0x(\.[\da-fA-F]+|[\da-fA-F]+(\.[\da-fA-F]+)?)([pP][+\-]?\d+)?
fn is_hex_float(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut idx = 0;
    if bytes.first() == Some(&b'+') || bytes.first() == Some(&b'-') {
        idx = 1;
    }
    let rest = &bytes[idx..];
    if rest.len() < 2 || rest[0] != b'0' || (rest[1] != b'x' && rest[1] != b'X') {
        return false;
    }
    let mut i = 2;
    let mut digits = 0;
    if i < rest.len() && rest[i] == b'.' {
        i += 1;
        if i == rest.len() || !rest[i].is_ascii_hexdigit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_hexdigit() {
            i += 1;
            digits += 1;
        }
    } else {
        if i == rest.len() || !rest[i].is_ascii_hexdigit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_hexdigit() {
            i += 1;
            digits += 1;
        }
        if i < rest.len() && rest[i] == b'.' {
            i += 1;
            while i < rest.len() && rest[i].is_ascii_hexdigit() {
                i += 1;
                digits += 1;
            }
        }
    }
    if digits == 0 {
        return false;
    }
    // ([pP][+\-]?\d+)?
    if i < rest.len() && (rest[i] == b'p' || rest[i] == b'P') {
        i += 1;
        if i < rest.len() && (rest[i] == b'+' || rest[i] == b'-') {
            i += 1;
        }
        if i == rest.len() || !rest[i].is_ascii_digit() {
            return false;
        }
        while i < rest.len() && rest[i].is_ascii_digit() {
            i += 1;
        }
    }
    i == rest.len()
}

/// Decodes the value of a string literal token (C# token.Value). Bracketed
/// (long) strings do not process escapes; quoted strings do.
fn string_value(token_ref: &TokenReference) -> String {
    let TokenType::StringLiteral {
        literal,
        multi_line_depth,
        ..
    } = token_ref.token().token_type()
    else {
        unreachable!("string value requires a string literal token");
    };
    let text = literal.as_str();
    if *multi_line_depth > 0 {
        return text.to_string();
    }
    unescape_lua_string(text)
}

/// Lua escape decoding for quoted strings (\a \b \f \n \r \t \v \\ \" \' \z
/// \xXX \u{...} \ddd).
fn unescape_lua_string(text: &str) -> String {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            None => out.push('\\'),
            Some('a') => out.push('\x07'),
            Some('b') => out.push('\x08'),
            Some('f') => out.push('\x0C'),
            Some('n') => out.push('\n'),
            Some('r') => out.push('\r'),
            Some('t') => out.push('\t'),
            Some('v') => out.push('\x0B'),
            Some('\\') => out.push('\\'),
            Some('"') => out.push('"'),
            Some('\'') => out.push('\''),
            Some('z') => {
                // \z skips the following whitespace.
                loop {
                    match chars.clone().next() {
                        Some(n) if n.is_ascii_whitespace() => {
                            chars.next();
                        }
                        _ => break,
                    }
                }
            }
            Some('x') => {
                let mut hex = String::new();
                for _ in 0..2 {
                    match chars.next() {
                        Some(h) if h.is_ascii_hexdigit() => hex.push(h),
                        other => {
                            if let Some(o) = other {
                                out.push(o);
                            }
                            break;
                        }
                    }
                }
                if let Ok(v) = u8::from_str_radix(&hex, 16) {
                    out.push(v as char);
                }
            }
            Some('u') => {
                // \u{XXXX}
                let mut digits = String::new();
                if chars.next() == Some('{') {
                    for c in chars.by_ref() {
                        if c == '}' {
                            break;
                        }
                        digits.push(c);
                    }
                }
                if let Ok(v) = u32::from_str_radix(&digits, 16) {
                    if let Some(c) = char::from_u32(v) {
                        out.push(c);
                    }
                }
            }
            Some(d) if d.is_ascii_digit() => {
                // \ddd up to three decimal digits.
                let mut digits = String::new();
                digits.push(d);
                for _ in 0..2 {
                    match chars.next() {
                        Some(n) if n.is_ascii_digit() => digits.push(n),
                        other => {
                            if let Some(o) = other {
                                out.push(o);
                            }
                            break;
                        }
                    }
                }
                if let Ok(v) = digits.parse::<u32>() {
                    out.push(char::from_u32(v).unwrap_or('\u{FFFD}'));
                }
            }
            Some(other) => out.push(other),
        }
    }
    out
}

/// C# member/element lookup over the table's fields (Reverse order).
/// `name` is the member name for `.x` accesses; `brackets` carries the key
/// expression + the folder for `[k]` accesses.
fn lookup_table_field(
    table: &ast::TableConstructor,
    name: Option<&str>,
    brackets: Option<(&ast::Expression, &ConstantFolder)>,
) -> Option<ast::Expression> {
    let fields: Vec<&ast::Field> = table.fields().iter().collect();
    for field in fields.into_iter().rev() {
        match field {
            ast::Field::NameKey { key, value, .. } => {
                if let Some(name) = name {
                    // C#: Identifier.Text == node.MemberName.Text (Ordinal)
                    if key.token().to_string() == name {
                        return Some(value.clone());
                    }
                } else if let Some((key_expr, folder)) = brackets {
                    // C#: HasEFlag(keyExpression, IsStr) && GetValue == identifier
                    if folder.has_e_flag(key_expr, FLAG_IS_STR)
                        && get_string_value(key_expr) == key.token().to_string()
                    {
                        return Some(value.clone());
                    }
                }
            }
            ast::Field::ExpressionKey { key, value, .. } => {
                if let Some(name) = name {
                    // C#: key IsStr && GetValue(key) == member name
                    if is_str_with_value(key, name) {
                        return Some(value.clone());
                    }
                } else if let Some((key_expr, _folder)) = brackets {
                    // C#: field.Key.IsEquivalentTo(keyExpression)
                    if expressions_equivalent(key, key_expr) {
                        return Some(value.clone());
                    }
                }
            }
            ast::Field::NoKey(_) => {}
            ast::Field::SetConstructor { .. } => {}
            #[allow(unreachable_patterns)]
            _ => {}
        }
    }
    None
}

/// C# GetValue<string>(key) == name check for the member access.
fn is_str_with_value(key: &ast::Expression, name: &str) -> bool {
    let inner = get_inner_expression(key);
    match inner {
        ast::Expression::String(t) => string_value(t) == name,
        _ => false,
    }
}

/// C# IsEquivalentTo: the field key is syntactically identical to the index
/// key (ignoring trivia).
fn expressions_equivalent(a: &ast::Expression, b: &ast::Expression) -> bool {
    let tokens_a = collect_expr_tokens(a);
    let tokens_b = collect_expr_tokens(b);
    if tokens_a.len() != tokens_b.len() {
        return false;
    }
    tokens_a
        .iter()
        .zip(tokens_b.iter())
        .all(|(ta, tb)| ta.token().to_string() == tb.token().to_string())
}

/// Collects the non-trivia tokens of an expression in source order.
fn collect_expr_tokens(expr: &ast::Expression) -> Vec<TokenReference> {
    struct TokenGrabber {
        tokens: Vec<TokenReference>,
    }
    impl VisitorMut for TokenGrabber {
        fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
            self.tokens.push(token_ref.clone());
            token_ref
        }
    }
    let mut grabber = TokenGrabber { tokens: Vec::new() };
    let _ = expr.clone().visit_mut(&mut grabber);
    grabber.tokens
}

/// C# node.GetLeadingTrivia() — the leading trivia of the expression's first
/// token (collected via the full token walk, so every kind is covered).
fn first_leading(expr: &ast::Expression) -> Vec<Token> {
    collect_expr_tokens(expr)
        .first()
        .map(|t| t.leading_trivia().cloned().collect())
        .unwrap_or_default()
}

/// C# node.GetTrailingTrivia() — the trailing trivia of the expression's
/// last token.
fn last_trailing(expr: &ast::Expression) -> Vec<Token> {
    collect_expr_tokens(expr)
        .last()
        .map(|t| t.trailing_trivia().cloned().collect())
        .unwrap_or_default()
}

/// Replaces the leading trivia of a token reference (C# WithLeadingTrivia).
fn replace_leading(token_ref: TokenReference, leading: Vec<Token>) -> TokenReference {
    let trailing: Vec<Token> = token_ref.trailing_trivia().cloned().collect();
    TokenReference::new(leading, token_ref.token().to_owned(), trailing)
}

fn replace_prefix_first(prefix: ast::Prefix, leading: Vec<Token>) -> ast::Prefix {
    match prefix {
        ast::Prefix::Expression(e) => {
            ast::Prefix::Expression(Box::new(set_first_leading(*e, leading)))
        }
        ast::Prefix::Name(t) => ast::Prefix::Name(replace_leading(t, leading)),
        #[allow(unreachable_patterns)]
        _ => unreachable!("unsupported prefix kind"),
    }
}

fn replace_unop(unop: ast::UnOp, leading: Vec<Token>) -> ast::UnOp {
    let token = replace_leading(unop.token().clone(), leading);
    match unop {
        ast::UnOp::Minus(_) => ast::UnOp::Minus(token),
        ast::UnOp::Not(_) => ast::UnOp::Not(token),
        ast::UnOp::Hash(_) => ast::UnOp::Hash(token),

        ast::UnOp::Tilde(_) => ast::UnOp::Tilde(token),
        #[allow(unreachable_patterns)]
        _ => unreachable!("unsupported unary operator"),
    }
}

/// C# WithTriviaFrom(node, node) for the paren fold (ConstantFolder.cs:441-445):
/// the leading trivia of the first token is replaced; the trailing stays.
fn set_first_leading(expr: ast::Expression, leading: Vec<Token>) -> ast::Expression {
    match expr {
        ast::Expression::BinaryOperator { lhs, binop, rhs } => ast::Expression::BinaryOperator {
            lhs: Box::new(set_first_leading(*lhs, leading)),
            binop,
            rhs,
        },
        ast::Expression::Parentheses {
            contained,
            expression,
        } => {
            let (first, second) = contained.tokens();
            ast::Expression::Parentheses {
                contained: ContainedSpan::new(
                    replace_leading(first.clone(), leading),
                    second.clone(),
                ),
                expression,
            }
        }
        ast::Expression::UnaryOperator { unop, expression } => ast::Expression::UnaryOperator {
            unop: replace_unop(unop, leading),
            expression,
        },
        ast::Expression::Function(func) => {
            let new_token = replace_leading(func.function_token().clone(), leading);
            ast::Expression::Function(Box::new(func.with_function_token(new_token)))
        }
        ast::Expression::FunctionCall(call) => {
            let new_prefix = replace_prefix_first(call.prefix().clone(), leading);
            ast::Expression::FunctionCall(call.with_prefix(new_prefix))
        }

        ast::Expression::IfExpression(if_expr) => {
            let new_token = replace_leading(if_expr.if_token().clone(), leading);
            ast::Expression::IfExpression(if_expr.with_if_token(new_token))
        }

        ast::Expression::InterpolatedString(interpolated) => {
            // Defensive arm (Finding 4): no fold rule reaches an
            // interpolated string's first token today — constants can't
            // contain one (it carries no expression flags), the
            // parenthesized arm never descends into its inner expression,
            // and full_moon prefixes are only `(`-wrapped or names
            // (parsers.rs:1765-1813) — so the arm is dead, matching the C#
            // (no folding for them either). Implemented rather than
            // unreachable!() so a future fold rule that does reach it
            // rewrites the first token instead of panicking: the first
            // segment's literal (kind Begin) for strings with expressions,
            // or the last_string for segment-less strings (the parser
            // keeps the Begin token there, parsers.rs:2747-2750).
            let mut segments: Vec<_> = interpolated.segments().cloned().collect();
            match segments.first_mut() {
                Some(first) => {
                    first.literal = replace_leading(first.literal.clone(), leading);
                    ast::Expression::InterpolatedString(
                        full_moon::ast::luau::InterpolatedString::new(
                            segments,
                            interpolated.last_string().clone(),
                        ),
                    )
                }
                None => ast::Expression::InterpolatedString(
                    full_moon::ast::luau::InterpolatedString::new(
                        Vec::new(),
                        replace_leading(interpolated.last_string().clone(), leading),
                    ),
                ),
            }
        }
        ast::Expression::TableConstructor(tc) => {
            let (first, second) = tc.braces().tokens();
            let new_braces =
                ContainedSpan::new(replace_leading(first.clone(), leading), second.clone());
            ast::Expression::TableConstructor(tc.with_braces(new_braces))
        }
        ast::Expression::Number(t) | ast::Expression::String(t) | ast::Expression::Symbol(t) => {
            let kind = t.token().token_type().clone();
            let token_ref = replace_leading(t.clone(), leading);
            match kind {
                TokenType::Number { .. } => ast::Expression::Number(token_ref),
                TokenType::StringLiteral { .. } => ast::Expression::String(token_ref),
                _ => ast::Expression::Symbol(token_ref),
            }
        }

        ast::Expression::TypeAssertion {
            expression,
            type_assertion,
        } => ast::Expression::TypeAssertion {
            expression: Box::new(set_first_leading(*expression, leading)),
            type_assertion,
        },
        ast::Expression::Var(var) => match var {
            ast::Var::Name(t) => ast::Expression::Var(ast::Var::Name(replace_leading(t, leading))),
            ast::Var::Expression(ve) => {
                let new_prefix = replace_prefix_first(ve.prefix().clone(), leading);
                ast::Expression::Var(ast::Var::Expression(Box::new(ve.with_prefix(new_prefix))))
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!("unsupported var kind"),
        },
        #[allow(unreachable_patterns)]
        _ => unreachable!("unsupported expression kind in set_first_leading"),
    }
}

/// The expression view of a prefix (C# node.Expression).
fn expr_from_prefix(prefix: &ast::Prefix) -> ast::Expression {
    match prefix {
        ast::Prefix::Expression(e) => (**e).clone(),
        ast::Prefix::Name(t) => ast::Expression::Var(ast::Var::Name(t.clone())),
        #[allow(unreachable_patterns)]
        _ => unreachable!("unsupported prefix kind"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_number_forms() {
        assert_eq!(try_parse_number_in_string("10"), Some(NumValue::Long(10)));
        assert_eq!(try_parse_number_in_string("+10"), Some(NumValue::Long(10)));
        assert_eq!(try_parse_number_in_string("-10"), Some(NumValue::Long(-10)));
        assert_eq!(
            try_parse_number_in_string("1.5"),
            Some(NumValue::Double(1.5))
        );
        assert_eq!(
            try_parse_number_in_string(".5"),
            Some(NumValue::Double(0.5))
        );
        assert_eq!(
            try_parse_number_in_string("1e2"),
            Some(NumValue::Double(100.0))
        );
        assert_eq!(
            try_parse_number_in_string("1E-2"),
            Some(NumValue::Double(0.01))
        );
        assert_eq!(
            try_parse_number_in_string("0x1.8p10"),
            Some(NumValue::Double(1536.0))
        );
        assert_eq!(try_parse_number_in_string("abc"), None);
        // Any hex-integer string panics with the pinned .NET ArgumentException
        // (the AllowLeadingSign | AllowHexSpecifier style is invalid on
        // .NET 8+; see the constantfold-hex corpus case).
        for hex in ["0x10", "0Xff", "-0x10"] {
            let result = std::panic::catch_unwind(|| {
                let _ = try_parse_number_in_string(hex);
            });
            assert!(result.is_err(), "hex string {hex:?} must panic");
        }
    }

    #[test]
    fn folds_arithmetic() {
        let folded = fold_sample("local a = 1 + 2\n");
        assert_eq!(folded, "local a = 3\n");
    }

    #[test]
    fn folds_concatenation() {
        let folded = fold_sample("local a = \"foo\" .. \"bar\"\n");
        assert_eq!(folded, "local a = \"foobar\"\n");
    }

    #[test]
    fn folds_unary_and_comparisons() {
        let folded = fold_sample("local a = -5\nlocal b = not true\nlocal c = #\"hi\"\n");
        assert_eq!(folded, "local a = -5\nlocal b = false\nlocal c = 2\n");
    }

    fn fold_sample(code: &str) -> String {
        let ast = full_moon::parse(code).expect("parse");
        let mut folder = ConstantFolder::new(ConstantFoldingOptions {
            extract_numbers_from_strings: false,
        });
        folder.fold(ast).to_string()
    }

    #[test]
    fn set_first_leading_rewrites_interpolated_string_first_token() {
        // Finding 4: the arm is dead through the fold rules (no constant
        // can contain an interpolated string) but must still rewrite the
        // first token correctly if ever reached.
        use full_moon::ast::luau::{InterpolatedString, InterpolatedStringSegment};
        use full_moon::tokenizer::InterpolatedStringKind;

        let begin = TokenReference::new(
            vec![Token::new(TokenType::Whitespace {
                characters: ShortString::new("  "),
            })],
            Token::new(TokenType::InterpolatedString {
                literal: ShortString::new("x"),
                kind: InterpolatedStringKind::Begin,
            }),
            vec![],
        );
        let end = TokenReference::new(
            vec![],
            Token::new(TokenType::InterpolatedString {
                literal: ShortString::new("y"),
                kind: InterpolatedStringKind::End,
            }),
            vec![],
        );
        let new_leading = vec![Token::new(TokenType::Whitespace {
            characters: ShortString::new("    "),
        })];

        // With an expression: the first segment's literal is the first
        // token.
        let expr = ast::Expression::InterpolatedString(InterpolatedString::new(
            vec![InterpolatedStringSegment {
                literal: begin.clone(),
                expression: ast::Expression::Number(TokenReference::new(
                    vec![],
                    Token::new(TokenType::Number {
                        text: ShortString::new("1"),
                    }),
                    vec![],
                )),
            }],
            end.clone(),
        ));
        let rewritten = set_first_leading(expr, new_leading.clone());
        let ast::Expression::InterpolatedString(rewritten) = &rewritten else {
            panic!("must stay an interpolated string");
        };
        let first = rewritten.segments().next().expect("one segment");
        let trivia: Vec<String> = first
            .literal
            .leading_trivia()
            .map(|t| t.to_string())
            .collect();
        assert_eq!(trivia, vec!["    ".to_string()]);

        // Without an expression: the last_string carries the first token.
        let bare = ast::Expression::InterpolatedString(InterpolatedString::new(Vec::new(), begin));
        let rewritten_bare = set_first_leading(bare, new_leading);
        let ast::Expression::InterpolatedString(rewritten_bare) = &rewritten_bare else {
            panic!("must stay an interpolated string");
        };
        let trivia: Vec<String> = rewritten_bare
            .last_string()
            .leading_trivia()
            .map(|t| t.to_string())
            .collect();
        assert_eq!(trivia, vec!["    ".to_string()]);
    }
}
