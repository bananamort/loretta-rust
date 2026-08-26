// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFolder (b767b4e)
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.cs
// (the ExpressionFlags members live in expressionflags.rs; the number
// parsing lives in numparsing.rs — mirroring the C# partial-class files.)

use crate::experimental::constantfoldingoptions::ConstantFoldingOptions;
use crate::experimental::expressionflags::{
    has_e_flag, FLAG_IS_ANONYMOUS_FUNCTION, FLAG_IS_BOOL, FLAG_IS_CONSTANT, FLAG_IS_CONSTANT_TABLE,
    FLAG_IS_DOUBLE, FLAG_IS_FALSEY, FLAG_IS_LONG, FLAG_IS_NIL, FLAG_IS_NUM, FLAG_IS_SCALAR,
    FLAG_IS_STR, FLAG_IS_STRING_WITH_NUMBER, FLAG_IS_TRUTHY,
};
use crate::experimental::numparsing::{number_is_double, number_value, try_parse_number_in_string};
use crate::symbol_display::objectdisplay::ObjectDisplay;
use crate::symbol_display::objectdisplayoptions::ObjectDisplayOptions;
use full_moon::ast;
use full_moon::ast::span::ContainedSpan;
use full_moon::tokenizer::{StringLiteralQuoteType, Symbol, Token, TokenReference, TokenType};
use full_moon::visitors::{VisitMut, VisitorMut};
use full_moon::ShortString;

/// C# ConstantFolder (ConstantFolder.cs:8-15): the options-holding rewriter.
#[derive(Clone)]
pub struct ConstantFolder {
    options: ConstantFoldingOptions,
    /// The syntax options the tree was parsed with — the C# token values
    /// are computed by the lexer with the LuaParseOptions, so the escape
    /// echo/skip is preset-dependent (Finding 36).
    syntax_options: crate::luasyntaxoptions::LuaSyntaxOptions,
}

/// The numeric value of an expression (C# `dynamic` long/double).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumValue {
    Long(i64),
    Double(f64),
}

impl ConstantFolder {
    /// C# ConstantFolder(ConstantFoldingOptions) (ConstantFolder.cs:12-15).
    pub fn new(
        options: ConstantFoldingOptions,
        syntax_options: crate::luasyntaxoptions::LuaSyntaxOptions,
    ) -> Self {
        ConstantFolder {
            options,
            syntax_options,
        }
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
                return try_parse_number_in_string(&string_value(
                    t,
                    self.syntax_options.accept_invalid_escapes,
                ));
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
    fn visit_unary(&mut self, unop: ast::UnOp, operand: Box<ast::Expression>) -> ast::Expression {
        let operand = operand.visit_mut(self);
        let operand_flags = self.get_flags(&operand);
        // C# LiteralExpressionWithTriviaFrom(value, operand)
        // (ConstantFolder.cs:31-43): the folded literal takes the
        // OPERAND's leading and trailing trivia — not the whole unary
        // node's (the operator's; Finding 27). In full_moon the
        // whitespace/comments between the operator and the operand ride
        // the operator's trailing trivia, so the operand's leading is
        // normally empty — the shape still mirrors the C# exactly.
        let leading = first_leading(&operand);
        let trailing = last_trailing(&operand);
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
                // C# ConstantFolder.cs:43: (double) GetValue<string>
                // (operand).Length — the .NET string Length is the
                // UTF-16 code-unit count, so #"é" is 1 and #"😀" is 2
                // (Finding 32) — the port's .len() was the UTF-8 byte
                // count.
                let len = utf16_len(&get_string_value(
                    &operand,
                    self.syntax_options.accept_invalid_escapes,
                )) as f64;
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
                    // The C# left.Kind()/right.Kind() switch
                    // (ConstantFolder.cs:122-135) checks the DIRECT kind
                    // and throws ExceptionUtilities.Unreachable for
                    // anything else — a parenthesized operand crashes the
                    // C# fold; the port's paren-stripping folded it (a
                    // crash/success asymmetry — Finding 38).
                    let left_str = match left.as_ref() {
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::True) => {
                            "true".to_string()
                        }
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::False) => {
                            "false".to_string()
                        }
                        ast::Expression::String(_) => {
                            get_string_value(&left, self.syntax_options.accept_invalid_escapes)
                        }
                        _ => unreachable!("concat operand must be a direct literal"),
                    };
                    let right_str = match right.as_ref() {
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::True) => {
                            "true".to_string()
                        }
                        ast::Expression::Symbol(t) if t.is_symbol(Symbol::False) => {
                            "false".to_string()
                        }
                        ast::Expression::String(_) => {
                            get_string_value(&right, self.syntax_options.accept_invalid_escapes)
                        }
                        _ => unreachable!("concat operand must be a direct literal"),
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
                    let result = compare(self, &left, &right, left_flags, right_flags);
                    return literal_bool(result < 0, &leading, &trailing);
                }
            }
            ast::BinOp::LessThanEqual(_) => {
                if can_compare(left_flags, right_flags) {
                    let result = compare(self, &left, &right, left_flags, right_flags);
                    return literal_bool(result <= 0, &leading, &trailing);
                }
            }
            ast::BinOp::GreaterThan(_) => {
                if can_compare(left_flags, right_flags) {
                    let result = compare(self, &left, &right, left_flags, right_flags);
                    return literal_bool(result > 0, &leading, &trailing);
                }
            }
            ast::BinOp::GreaterThanEqual(_) => {
                if can_compare(left_flags, right_flags) {
                    let result = compare(self, &left, &right, left_flags, right_flags);
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

        // C#: the fold applies per access on the immediate base
        // (ConstantFolder.cs:336-407), bottom-up — the chained accesses
        // fold inner-first (Finding 37; the port used to gate the lookup
        // on suffixes.len() == 1). The FIRST access keeps the C#
        // PrefixExpression parity (only a parenthesized base folds — the
        // port's old check); each subsequent access checks the folded
        // value instead.
        let mut folded = expr_from_prefix(&prefix);
        let mut remaining: Vec<ast::Suffix> = Vec::new();
        for (i, suffix) in suffixes.into_iter().enumerate() {
            let ast::Suffix::Index(index) = &suffix else {
                remaining.push(suffix);
                continue;
            };
            if self.has_e_flag(&folded, FLAG_IS_CONSTANT_TABLE)
                && (i > 0 || matches!(&folded, ast::Expression::Parentheses { .. }))
            {
                let table = get_inner_expression(&folded);
                let table = if let ast::Expression::TableConstructor(tc) = table {
                    tc.clone()
                } else {
                    unreachable!("IsConstantTable requires a table constructor");
                };
                let found = match index {
                    ast::Index::Dot { name, .. } => {
                        lookup_table_field(&table, Some(&name.token().to_string()), None, self)
                    }
                    ast::Index::Brackets { expression, .. }
                        if self.has_e_flag(expression, FLAG_IS_SCALAR) =>
                    {
                        lookup_table_field(&table, None, Some(expression), self)
                    }
                    #[allow(unreachable_patterns)]
                    _ => None,
                };
                if let Some(value) = found {
                    folded = value;
                    continue;
                }
            }
            remaining.push(suffix);
        }

        if remaining.is_empty() {
            // The C# WithTriviaFrom(value, node): the folded value takes
            // the var's leading trivia.
            return set_first_leading(folded, leading);
        }

        ast::Expression::Var(ast::Var::Expression(Box::new(
            ast::VarExpression::new(prefix).with_suffixes(remaining),
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
                self.visit_unary(unop, expression)
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
pub(crate) fn get_inner_expression(node: &ast::Expression) -> &ast::Expression {
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

/// C# GetValue<string> — the decoded string value of a String literal.
fn get_string_value(node: &ast::Expression, accept_invalid_escapes: bool) -> String {
    let inner = get_inner_expression(node);
    let ast::Expression::String(t) = inner else {
        unreachable!("string value requires a string literal");
    };
    string_value(t, accept_invalid_escapes)
}

/// The .NET string Length — the UTF-16 code-unit count (a char beyond
/// 0xFFFF is a surrogate pair, two units).
fn utf16_len(s: &str) -> usize {
    s.chars()
        .map(|c| if c as u32 > 0xFFFF { 2 } else { 1 })
        .sum()
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
        return get_string_value(left, folder.syntax_options.accept_invalid_escapes)
            == get_string_value(right, folder.syntax_options.accept_invalid_escapes);
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
    folder: &ConstantFolder,
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
            get_string_value(left, folder.syntax_options.accept_invalid_escapes).as_bytes(),
            get_string_value(right, folder.syntax_options.accept_invalid_escapes).as_bytes(),
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

/// C# Math.Pow((double) leftNum, (double) rightNum) (ConstantFolder.cs —
/// the Caret case). The port uses f64::powf — the same platform pow, and
/// the corpus-visible cases agree; last-ulp differences vs the .NET
/// runtime are theoretically possible on some inputs (Finding 66 — note
/// only, the corpus agrees).
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

/// Decodes the value of a string literal token (C# token.Value). Bracketed
/// (long) strings do not process escapes; quoted strings do.
fn string_value(token_ref: &TokenReference, accept_invalid_escapes: bool) -> String {
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
    unescape_lua_string(text, accept_invalid_escapes)
}

/// Lua escape decoding for quoted strings (\a \b \f \n \r \t \v \\ \" \' \z
/// \xXX \u{...} \ddd) — the `accept_invalid_escapes` flag carries the C#
/// LuaSyntaxOptions.AcceptInvalidEscapes (the lexer's preset-dependent
/// echo/skip, ShortString.cs:199-205 — Finding 36).
fn unescape_lua_string(text: &str, accept_invalid_escapes: bool) -> String {
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
                // \z skips the following whitespace — the C#
                // CharUtils.IsWhitespace set [ \t\n\v\f\r]
                // (ShortString.cs:141; the Rust is_ascii_whitespace
                // excludes '\v' — Finding 35).
                loop {
                    match chars.clone().next() {
                        Some(n) if n == ' ' || ('\t'..='\r').contains(&n) => {
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
                // \u{XXXX} — the C# keeps BMP codepoints as the raw code
                // unit (a lone surrogate included, ShortString.cs:285-292)
                // and astral codepoints as a UTF-16 surrogate pair
                // (ShortString.cs:294-297); the port's Rust string keeps
                // the astral as a single char (the utf16_len count
                // handles it) and represents a lone surrogate as U+FFFD —
                // the closest legal Rust char (the raw \uD800 cannot
                // exist in a Rust string; the C#-equality residual is
                // documented — Finding 34).
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
                    if v > 0x10FFFF {
                        // The C# ERR_EscapeTooLarge + the sentinel: the
                        // escape is skipped entirely (ShortString.cs:
                        // 279-283).
                    } else if let Some(c) = char::from_u32(v) {
                        out.push(c);
                    } else {
                        // The lone surrogate (0xD800-0xDFFF): the C#
                        // keeps the raw code unit; the port uses U+FFFD.
                        out.push('\u{FFFD}');
                    }
                }
            }
            Some(d) if d.is_ascii_digit() => {
                // \ddd up to three decimal digits — the C# keeps values
                // up to 255 (ParseDecimalInteger, ShortString.cs:223-226);
                // larger values report ERR_InvalidStringEscape and the
                // escape char is the sentinel, so the escape is SKIPPED
                // entirely in the string value (Finding 33) — the port
                // used to push the decoded char.
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
                    if v <= 255 {
                        out.push(char::from_u32(v).expect("values up to 255 are valid chars"));
                    }
                }
            }
            Some(other) => {
                // The C# default escape case (ShortString.cs:199-205):
                // with AcceptInvalidEscapes the character is echoed;
                // without it the escape is skipped entirely (the C#
                // sentinel).
                if accept_invalid_escapes {
                    out.push(other);
                }
            }
        }
    }
    out
}

/// C# member/element lookup over the table's fields (Reverse order —
/// ConstantFolder.cs:342, 377). `name` is the member name for `.x`
/// accesses: BOTH the IdentifierKeyed and the ExpressionKeyed C# checks
/// run on the dot path (ConstantFolder.cs:344-358). `key_expression` is
/// the key expression for `[k]` accesses. The folder supplies the option
/// flags and the string values (Finding A — the Dot arm used to pass
/// brackets=None, making the ExpressionKeyed check unreachable there).
fn lookup_table_field(
    table: &ast::TableConstructor,
    name: Option<&str>,
    key_expression: Option<&ast::Expression>,
    folder: &ConstantFolder,
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
                } else if let Some(key_expr) = key_expression {
                    // C#: HasEFlag(keyExpression, IsStr) && GetValue == identifier
                    if folder.has_e_flag(key_expr, FLAG_IS_STR)
                        && get_string_value(key_expr, folder.syntax_options.accept_invalid_escapes)
                            == key.token().to_string()
                    {
                        return Some(value.clone());
                    }
                }
            }
            ast::Field::ExpressionKey { key, value, .. } => {
                if let Some(name) = name {
                    // C#: key IsStr && GetValue(key) == member name
                    // (ConstantFolder.cs:350-357) — the ExpressionKeyed
                    // check runs on the dot path too.
                    if is_str_with_value(key, name, folder.syntax_options.accept_invalid_escapes) {
                        return Some(value.clone());
                    }
                } else if let Some(key_expr) = key_expression {
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
fn is_str_with_value(key: &ast::Expression, name: &str, accept_invalid_escapes: bool) -> bool {
    let inner = get_inner_expression(key);
    match inner {
        ast::Expression::String(t) => string_value(t, accept_invalid_escapes) == name,
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
        let mut folder = ConstantFolder::new(
            ConstantFoldingOptions {
                extract_numbers_from_strings: false,
            },
            crate::luasyntaxoptions::LuaSyntaxOptions::ALL_WITH_INTEGERS,
        );
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
