// Ported from Loretta.CodeAnalysis.Lua.Experimental.ConstantFolder (b767b4e): ConstantFolder
// C# source: src/Compilers/Lua/Experimental/ConstantFolder.cs (+ ConstantFolder.ExpressionFlags.cs, ConstantFolder.NumberParsing.cs)

use crate::experimental::constantfoldingoptions::ConstantFoldingOptions;
use crate::symbol_display::objectdisplay::ObjectDisplay;
use crate::symbol_display::objectdisplayoptions::ObjectDisplayOptions;
use crate::utilities::hexfloat::HexFloat;
use crate::utilities::stringutils::StringUtils;
use full_moon::ast::span::ContainedSpan;
use full_moon::ast::{
    Ast, BinOp, Expression, Field, Index, Prefix, Suffix, TableConstructor, UnOp, Var,
    VarExpression,
};
use full_moon::node::Node;
use full_moon::tokenizer::{StringLiteralQuoteType, Symbol, Token, TokenReference, TokenType};
use full_moon::visitors::{VisitMut, VisitorMut};

/// C# `[Flags] ExpressionFlags` enum.
#[derive(Clone, Copy, PartialEq, Eq)]
struct ExpressionFlags(u16);

impl ExpressionFlags {
    const NONE: Self = Self(0);
    const IS_NIL: Self = Self(1 << 0);
    const IS_DOUBLE: Self = Self(1 << 1);
    const IS_STR: Self = Self(1 << 2);
    const IS_BOOL: Self = Self(1 << 3);
    const IS_TRUTHY: Self = Self(1 << 4);
    const IS_FALSEY: Self = Self(1 << 5);
    const IS_CONSTANT_TABLE: Self = Self(1 << 6);
    const IS_ANONYMOUS_FUNCTION: Self = Self(1 << 7);
    const IS_LONG: Self = Self(1 << 8);
    const IS_STRING_WITH_NUMBER: Self = Self(1 << 9);

    // C# composite flags (ConstantFolder.ExpressionFlags.cs lines 22-25).
    const CAN_CONVERT_TO_BOOL: Self = Self(Self::IS_TRUTHY.0 | Self::IS_FALSEY.0);
    const IS_SCALAR: Self = Self(
        Self::IS_NIL.0 | Self::IS_DOUBLE.0 | Self::IS_LONG.0 | Self::IS_STR.0 | Self::IS_BOOL.0,
    );
    const IS_CONSTANT: Self =
        Self(Self::IS_SCALAR.0 | Self::IS_CONSTANT_TABLE.0 | Self::IS_ANONYMOUS_FUNCTION.0);
    const IS_NUM: Self = Self(Self::IS_DOUBLE.0 | Self::IS_LONG.0 | Self::IS_STRING_WITH_NUMBER.0);

    fn has(&self, flag: Self) -> bool {
        self.0 & flag.0 != 0
    }

    fn or(&mut self, flag: Self) {
        self.0 |= flag.0;
    }
}

impl std::ops::BitOr for ExpressionFlags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

/// The C# `dynamic` values a literal can carry (Loretta lexer token values).
#[derive(Clone, Debug, PartialEq)]
enum Value {
    Nil,
    Long(i64),
    Ulong(u64),
    Double(f64),
    Str(String),
    Bool(bool),
}

/// Runs constant folding on an AST (C# `LuaSyntaxRewriter` replacement via
/// full_moon's `VisitorMut`; the C# `_exprFlags`/`_innerStringNumericValue`
/// caches are omitted since full_moon nodes carry no `Hash` — the flag and
/// extraction computations are pure and memoization is not observable).
#[derive(Clone)]
pub struct ConstantFolder {
    options: ConstantFoldingOptions,
}

impl ConstantFolder {
    /// C# ctor; the C# `ArgumentNullException` for a null options is vacuous
    /// for the value-typed port.
    pub fn new(options: ConstantFoldingOptions) -> Self {
        Self { options }
    }

    /// C# `FoldConstants(SyntaxNode, ConstantFoldingOptions)`: folds the
    /// whole AST rooted at the provided one.
    pub fn fold_ast(&self, ast: Ast) -> Ast {
        let mut folder = self.clone();
        folder.visit_ast(ast)
    }

    /// C# `GetInnerExpression(SyntaxNode)`.
    fn get_inner_expression(node: &Expression) -> &Expression {
        let mut current = node;
        while let Expression::Parentheses { expression, .. } = current {
            current = expression;
        }
        current
    }

    /// C# `GetFlags(SyntaxNode)` — pure (cache-free) computation.
    fn get_flags(&self, node: &Expression) -> ExpressionFlags {
        let inner = Self::get_inner_expression(node);
        let mut flags = ExpressionFlags::NONE;
        match inner {
            Expression::Symbol(token) => match token.token().token_type() {
                TokenType::Symbol {
                    symbol: Symbol::Nil,
                } => flags.or(ExpressionFlags::IS_NIL),
                TokenType::Symbol {
                    symbol: Symbol::True | Symbol::False,
                } => flags.or(ExpressionFlags::IS_BOOL),
                _ => {}
            },
            Expression::Number(_) => {
                let value = self.get_value(inner);
                match value {
                    Value::Double(_) => flags.or(ExpressionFlags::IS_DOUBLE),
                    _ => flags.or(ExpressionFlags::IS_LONG),
                }
            }
            Expression::String(_) => {
                flags.or(ExpressionFlags::IS_STR);
                if self.options.extract_numbers_from_strings
                    && self
                        .try_parse_number_in_string(&self.get_value_str(inner))
                        .is_some()
                {
                    flags.or(ExpressionFlags::IS_STRING_WITH_NUMBER);
                }
            }
            Expression::TableConstructor(table) => {
                if self.is_const_table(table) {
                    flags.or(ExpressionFlags::IS_CONSTANT_TABLE);
                }
            }
            Expression::Function(_) => flags.or(ExpressionFlags::IS_ANONYMOUS_FUNCTION),
            _ => {}
        }
        if Self::can_convert_to_boolean(inner) {
            // The C# composite `CanConvertToBool` flag is kept for parity.
            let _ = ExpressionFlags::CAN_CONVERT_TO_BOOL;
            if Self::is_falsey(inner) {
                flags.or(ExpressionFlags::IS_FALSEY);
            } else {
                flags.or(ExpressionFlags::IS_TRUTHY);
            }
        }
        flags
    }

    /// C# `IsConstTable(TableConstructorExpressionSyntax)`.
    fn is_const_table(&self, table: &TableConstructor) -> bool {
        fn is_const(folder: &ConstantFolder, node: &Expression) -> bool {
            folder
                .get_flags(node)
                .has(ExpressionFlags::IS_CONSTANT | ExpressionFlags::IS_CONSTANT_TABLE)
        }
        for field in table.fields().iter() {
            match field {
                Field::NameKey { value, .. } => {
                    if !is_const(self, value) {
                        return false;
                    }
                }
                Field::ExpressionKey { key, value, .. } => {
                    if !is_const(self, key) || !is_const(self, value) {
                        return false;
                    }
                }
                Field::NoKey(value) => {
                    if !is_const(self, value) {
                        return false;
                    }
                }
                // C# `default: throw ExceptionUtilities.UnexpectedValue(...)` —
                // the cfxlua SetConstructor field would throw; it is treated as
                // non-constant instead (documented adaptation).
                _ => return false,
            }
        }
        true
    }

    /// C# `CanConvertToBoolean(SyntaxNode)`.
    fn can_convert_to_boolean(node: &Expression) -> bool {
        matches!(
            node,
            Expression::Symbol(_)
                | Expression::Number(_)
                | Expression::String(_)
                | Expression::Function(_)
        )
    }

    /// C# `IsFalsey(SyntaxNode)`.
    fn is_falsey(node: &Expression) -> bool {
        debug_assert!(Self::can_convert_to_boolean(node));
        matches!(
            node,
            Expression::Symbol(token)
                if matches!(
                    token.token().token_type(),
                    TokenType::Symbol {
                        symbol: Symbol::Nil | Symbol::False
                    }
                )
        )
    }

    /// C# `GetValue(SyntaxNode)` — the literal value of a literal expression.
    fn get_value(&self, node: &Expression) -> Value {
        let inner = Self::get_inner_expression(node);
        match inner {
            Expression::Number(token) => {
                Self::number_value(&token.token().to_string()).expect("numeric literal must parse")
            }
            Expression::String(token) => Value::Str(Self::string_value(token.token())),
            Expression::Symbol(token) => match token.token().token_type() {
                TokenType::Symbol {
                    symbol: Symbol::True,
                } => Value::Bool(true),
                TokenType::Symbol {
                    symbol: Symbol::False,
                } => Value::Bool(false),
                TokenType::Symbol {
                    symbol: Symbol::Nil,
                } => Value::Nil,
                _ => panic!("unexpected symbol in literal expression"),
            },
            _ => panic!("node is not a literal expression"),
        }
    }

    /// C# `GetValue<T>` for strings.
    fn get_value_str(&self, node: &Expression) -> String {
        match self.get_value(node) {
            Value::Str(s) => s,
            other => panic!("expected string value, got {other:?}"),
        }
    }

    /// C# `TryGetNumValue(SyntaxNode, out dynamic?)`.
    fn try_get_num_value(&self, node: &Expression) -> Option<Value> {
        let flags = self.get_flags(node);
        if flags.has(ExpressionFlags::IS_NUM) {
            if flags.has(ExpressionFlags::IS_STRING_WITH_NUMBER) {
                self.try_parse_number_in_string(&self.get_value_str(node))
            } else {
                match self.get_value(node) {
                    Value::Long(_) | Value::Ulong(_) | Value::Double(_) => {
                        Some(self.get_value(node))
                    }
                    _ => None,
                }
            }
        } else {
            None
        }
    }

    /// C# `TryGetInt32(SyntaxNode, out int)` — note the C# strict bounds check
    /// `converted64 is < int.MaxValue and > int.MinValue`.
    fn try_get_int32(&self, node: &Expression) -> Option<i32> {
        let converted64 = self.try_get_int64(node)?;
        let converted = converted64 as i32;
        if converted64 < i32::MAX as i64 && converted64 > i32::MIN as i64 {
            Some(converted)
        } else {
            None
        }
    }

    /// C# `TryGetInt64(SyntaxNode, out long)`.
    fn try_get_int64(&self, node: &Expression) -> Option<i64> {
        let value = self.try_get_num_value(node)?;
        match value {
            Value::Long(v) => Some(v),
            Value::Ulong(v) => {
                let tmp = v as f64;
                let converted = tmp as i64;
                if tmp == converted as f64 {
                    Some(converted)
                } else {
                    None
                }
            }
            Value::Double(v) => {
                let tmp = v;
                let converted = tmp as i64;
                if tmp == converted as f64 {
                    Some(converted)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// C# `TryConvertToBool(SyntaxNode, out bool)`.
    fn try_convert_to_bool(node: &Expression) -> Option<bool> {
        let inner = Self::get_inner_expression(node);
        if Self::can_convert_to_boolean(inner) {
            Some(!Self::is_falsey(inner))
        } else {
            None
        }
    }

    /// C# `TryConvertToDouble(long, out double)` — the C# implementation
    /// `converted = value; return value == converted;` always compares the
    /// promoted long against the just-assigned double, i.e. it is a tautology
    /// that always returns true; ported verbatim.
    fn try_convert_to_double(value: i64) -> f64 {
        value as f64
    }

    /// C# `TryParseNumberInString` — the four regexes are matched by hand
    /// (no regex crate in the workspace; they are simple patterns).
    /// The unanchored float regexes effectively require a full-string match
    /// because the .NET parsers consume the entire input.
    fn try_parse_number_in_string(&self, value: &str) -> Option<Value> {
        let value = StringUtils::trim(value);

        // s_decIntegerRegex ^[+\-]?\d+$ with AllowLeadingSign.
        if is_dec_integer(value) {
            if let Ok(parsed) = value.parse::<i64>() {
                return Some(Value::Long(parsed));
            }
        }
        // s_hexIntegerRegex ^[+\-]?0[xX][\da-fA-F]+$ with AllowHexSpecifier |
        // AllowLeadingSign (.NET 7+ hex parsing accepts the 0x prefix).
        if is_hex_integer(value) {
            if let Some(parsed) = parse_hex_integer_with_sign(value) {
                return Some(Value::Long(parsed));
            }
        }
        // s_decFloatRegex + RealParser.TryParseDouble.
        if is_dec_float(value) {
            if let Ok(parsed) = value.parse::<f64>() {
                return Some(Value::Double(parsed));
            }
        }
        // s_hexFloatRegex + HexFloat.DoubleFromHexString.
        if is_hex_float(value) {
            if let Ok(parsed) = HexFloat::double_from_hex_string(value) {
                return Some(Value::Double(parsed));
            }
        }
        None
    }

    /// Parses the value of a Lua number literal token (C# lexer token value).
    /// Handles decimal/hex/octal/binary integers, LuaJIT `LL`/`ULL` suffixes,
    /// the Luau `i` suffix, decimal floats and hexadecimal floats.
    fn number_value(text: &str) -> Option<Value> {
        let (text, suffix) = strip_number_suffix(text);
        let (sign, unsigned) = match text.strip_prefix('-') {
            Some(rest) => (-1i64, rest),
            None => (1i64, text.strip_prefix('+').unwrap_or(text)),
        };
        let is_hex = unsigned.starts_with("0x") || unsigned.starts_with("0X");
        let is_oct = unsigned.starts_with("0o") || unsigned.starts_with("0O");
        let is_bin = unsigned.starts_with("0b") || unsigned.starts_with("0B");

        let is_integer = if is_hex {
            all_hex_digits(unsigned.get(2..).unwrap_or(""))
        } else if is_oct {
            all_oct_digits(unsigned.get(2..).unwrap_or(""))
        } else if is_bin {
            all_bin_digits(unsigned.get(2..).unwrap_or(""))
        } else {
            all_dec_digits(unsigned)
        };

        if is_integer {
            let stripped: String = unsigned.chars().filter(|&c| c != '_').collect();
            let digits = if is_hex || is_oct || is_bin {
                stripped.get(2..).unwrap_or(&stripped)
            } else {
                &stripped
            };
            let radix = if is_hex {
                16
            } else if is_oct {
                8
            } else if is_bin {
                2
            } else {
                10
            };
            let parsed = i64::from_str_radix(digits, radix).ok()?;
            return match suffix {
                NumberSuffix::Ull => Some(Value::Ulong((sign as u64).wrapping_mul(parsed as u64))),
                _ => Some(Value::Long(sign.wrapping_mul(parsed))),
            };
        }

        if is_hex {
            // Hexadecimal float.
            return HexFloat::double_from_hex_string(unsigned)
                .ok()
                .map(|v| Value::Double(sign as f64 * v));
        }
        // Decimal float; strip separators and parse.
        let stripped: String = unsigned.chars().filter(|&c| c != '_').collect();
        stripped
            .parse::<f64>()
            .ok()
            .map(|v| Value::Double(sign as f64 * v))
    }

    /// The C# `SyntaxFactory.Literal` value of a string token (unescaped).
    fn string_value(token: &Token) -> String {
        let TokenType::StringLiteral {
            literal,
            multi_line_depth,
            quote_type,
        } = token.token_type()
        else {
            panic!("not a string literal token");
        };
        if *multi_line_depth > 0 {
            // Long strings carry no escape sequences.
            return literal.to_string();
        }
        let _ = quote_type;
        unescape_lua_string(literal)
    }

    /// C# `HasEFlag(ExpressionFlags, ExpressionFlags)`.
    fn has_eflag(flags: ExpressionFlags, wanted_flag: ExpressionFlags) -> bool {
        flags.has(wanted_flag)
    }

    /// The leading trivia of a node's first token and the trailing trivia of
    /// its last token (C# `GetLeadingTrivia`/`GetTrailingTrivia` of the node).
    fn surrounding_trivia(node: &Expression) -> (Vec<Token>, Vec<Token>) {
        let tokens: Vec<&TokenReference> = node.tokens().collect();
        let leading = tokens
            .first()
            .map(|t| t.leading_trivia().cloned().collect())
            .unwrap_or_default();
        let trailing = tokens
            .last()
            .map(|t| t.trailing_trivia().cloned().collect())
            .unwrap_or_default();
        (leading, trailing)
    }

    /// C# `WithTriviaFrom(SyntaxToken, SyntaxNode)`: a token with the
    /// container's leading and trailing trivia.
    fn token_with_trivia_from(
        token_type: TokenType,
        trivia_container: &Expression,
    ) -> TokenReference {
        let (leading, trailing) = Self::surrounding_trivia(trivia_container);
        TokenReference::new(leading, Token::new(token_type), trailing)
    }

    /// C# `WithTriviaFrom(SyntaxNode, SyntaxNode)`: a node with the
    /// container's leading trivia and its own trailing trivia.
    fn with_trivia_from(node: Expression, trivia_container: &Expression) -> Expression {
        let (leading, _) = Self::surrounding_trivia(trivia_container);
        let first = node
            .tokens()
            .next()
            .cloned()
            .expect("expression must have a first token");
        let replacement = TokenReference::new(
            leading,
            first.token().clone(),
            first.trailing_trivia().cloned().collect(),
        );
        FirstTokenReplacer::new(replacement).replace(node)
    }

    /// C# `LiteralExpressionWithTriviaFrom(long, SyntaxNode)`.
    fn literal_expression_with_trivia_from_long(
        value: i64,
        trivia_container: &Expression,
    ) -> Expression {
        let text = ObjectDisplay::format_literal_i64(value, ObjectDisplayOptions::NONE);
        Expression::Number(Self::token_with_trivia_from(
            TokenType::Number { text: text.into() },
            trivia_container,
        ))
    }

    /// C# `LiteralExpressionWithTriviaFrom(double, SyntaxNode)`.
    fn literal_expression_with_trivia_from_double(
        value: f64,
        trivia_container: &Expression,
    ) -> Expression {
        let text = ObjectDisplay::format_literal_f64(value, ObjectDisplayOptions::NONE);
        Expression::Number(Self::token_with_trivia_from(
            TokenType::Number { text: text.into() },
            trivia_container,
        ))
    }

    /// C# `LiteralExpressionWithTriviaFrom(string, SyntaxNode)`.
    fn literal_expression_with_trivia_from_str(
        value: &str,
        trivia_container: &Expression,
    ) -> Expression {
        let text = ObjectDisplay::format_literal_str(
            value,
            ObjectDisplayOptions::USE_QUOTES
                | ObjectDisplayOptions::ESCAPE_NON_PRINTABLE_CHARACTERS,
        );
        // The formatted text is always `"..."` (escape_non_printable prevents
        // the verbatim form); the token stores the text between the quotes.
        let inner = text
            .strip_prefix('"')
            .and_then(|t| t.strip_suffix('"'))
            .unwrap_or(&text);
        Expression::String(Self::token_with_trivia_from(
            TokenType::StringLiteral {
                literal: inner.into(),
                multi_line_depth: 0,
                quote_type: StringLiteralQuoteType::Double,
            },
            trivia_container,
        ))
    }

    /// C# `LiteralExpressionWithTriviaFrom(bool, SyntaxNode)`.
    fn literal_expression_with_trivia_from_bool(
        value: bool,
        trivia_container: &Expression,
    ) -> Expression {
        let symbol = if value { Symbol::True } else { Symbol::False };
        Expression::Symbol(Self::token_with_trivia_from(
            TokenType::Symbol { symbol },
            trivia_container,
        ))
    }

    /// C# `LiteralExpressionWithTriviaFrom(Value, SyntaxNode)` — dispatch for
    /// the dynamic numeric results.
    fn literal_expression_with_trivia_from_num(
        value: Value,
        trivia_container: &Expression,
    ) -> Expression {
        match value {
            Value::Long(v) => Self::literal_expression_with_trivia_from_long(v, trivia_container),
            Value::Ulong(v) => {
                let text = ObjectDisplay::format_literal_u64(v, ObjectDisplayOptions::NONE);
                Expression::Number(Self::token_with_trivia_from(
                    TokenType::Number { text: text.into() },
                    trivia_container,
                ))
            }
            Value::Double(v) => {
                Self::literal_expression_with_trivia_from_double(v, trivia_container)
            }
            _ => panic!("not a numeric value"),
        }
    }

    /// C# `VisitParenthesizedExpression` fold.
    fn fold_parentheses(&self, expr: Expression) -> Expression {
        let Expression::Parentheses {
            contained,
            expression,
        } = expr
        else {
            unreachable!()
        };
        let inner = *expression;
        if matches!(inner, Expression::Parentheses { .. }) {
            let outer = Expression::Parentheses {
                contained,
                expression: Box::new(inner.clone()),
            };
            Self::with_trivia_from(inner, &outer)
        } else {
            Expression::Parentheses {
                contained,
                expression: Box::new(inner),
            }
        }
    }

    /// C# `VisitUnaryExpression` fold.
    fn fold_unary(&self, expr: Expression) -> Expression {
        let Expression::UnaryOperator { unop, expression } = expr else {
            unreachable!()
        };
        let operand = *expression;
        let operand_flags = self.get_flags(&operand);
        match unop {
            UnOp::Minus(_) => {
                if let Some(value) = self.try_get_num_value(&operand) {
                    let negated = match value {
                        Value::Long(v) => Some(Value::Long(v.wrapping_neg())),
                        Value::Double(v) => Some(Value::Double(-v)),
                        // C# dynamic `-ulong` would throw a binder exception;
                        // left unfolded (documented adaptation).
                        _ => None,
                    };
                    if let Some(negated) = negated {
                        return Self::literal_expression_with_trivia_from_num(negated, &operand);
                    }
                }
                Expression::UnaryOperator {
                    unop,
                    expression: Box::new(operand),
                }
            }
            UnOp::Not(_) => {
                if let Some(value) = Self::try_convert_to_bool(&operand) {
                    return Self::literal_expression_with_trivia_from_bool(!value, &operand);
                }
                Expression::UnaryOperator {
                    unop,
                    expression: Box::new(operand),
                }
            }
            UnOp::Tilde(_) => {
                if Self::has_eflag(
                    operand_flags,
                    ExpressionFlags::IS_DOUBLE | ExpressionFlags::IS_STRING_WITH_NUMBER,
                ) {
                    if let Some(value) = self.try_get_int64(&operand) {
                        let result = !value;
                        let converted = Self::try_convert_to_double(result);
                        return Self::literal_expression_with_trivia_from_double(
                            converted, &operand,
                        );
                    }
                }
                if Self::has_eflag(operand_flags, ExpressionFlags::IS_LONG) {
                    if let Some(value) = self.try_get_int64(&operand) {
                        return Self::literal_expression_with_trivia_from_long(!value, &operand);
                    }
                }
                Expression::UnaryOperator {
                    unop,
                    expression: Box::new(operand),
                }
            }
            UnOp::Hash(_) => {
                if Self::has_eflag(operand_flags, ExpressionFlags::IS_STR) {
                    let len = self.get_value_str(&operand).encode_utf16().count() as f64;
                    return Self::literal_expression_with_trivia_from_double(len, &operand);
                }
                Expression::UnaryOperator {
                    unop,
                    expression: Box::new(operand),
                }
            }
            _ => Expression::UnaryOperator {
                unop,
                expression: Box::new(operand),
            },
        }
    }

    /// C# `VisitBinaryExpression` fold.
    fn fold_binary(&self, expr: Expression) -> Expression {
        let container = expr.clone();
        let Expression::BinaryOperator { lhs, binop, rhs } = expr else {
            unreachable!()
        };
        let left = *lhs;
        let right = *rhs;
        let left_flags = self.get_flags(&left);
        let right_flags = self.get_flags(&right);

        let is_nan_or_inf = |v: f64| v.is_nan() || v.is_infinite();

        let result: Option<Expression> = match &binop {
            BinOp::Plus(_) => {
                match (
                    self.try_get_num_value(&left),
                    self.try_get_num_value(&right),
                ) {
                    (Some(l), Some(r)) => match num_add(l, r) {
                        Some(result) => {
                            if let Value::Double(d) = result {
                                if is_nan_or_inf(d) {
                                    None
                                } else {
                                    Some(Self::literal_expression_with_trivia_from_num(
                                        result, &container,
                                    ))
                                }
                            } else {
                                Some(Self::literal_expression_with_trivia_from_num(
                                    result, &container,
                                ))
                            }
                        }
                        None => None,
                    },
                    _ => None,
                }
            }
            BinOp::Minus(_) => {
                match (
                    self.try_get_num_value(&left),
                    self.try_get_num_value(&right),
                ) {
                    (Some(l), Some(r)) => match num_sub(l, r) {
                        Some(result) => {
                            if let Value::Double(d) = result {
                                if is_nan_or_inf(d) {
                                    None
                                } else {
                                    Some(Self::literal_expression_with_trivia_from_num(
                                        result, &container,
                                    ))
                                }
                            } else {
                                Some(Self::literal_expression_with_trivia_from_num(
                                    result, &container,
                                ))
                            }
                        }
                        None => None,
                    },
                    _ => None,
                }
            }
            BinOp::Star(_) => {
                match (
                    self.try_get_num_value(&left),
                    self.try_get_num_value(&right),
                ) {
                    (Some(l), Some(r)) => match num_mul(l, r) {
                        Some(result) => {
                            if let Value::Double(d) = result {
                                if is_nan_or_inf(d) {
                                    None
                                } else {
                                    Some(Self::literal_expression_with_trivia_from_num(
                                        result, &container,
                                    ))
                                }
                            } else {
                                Some(Self::literal_expression_with_trivia_from_num(
                                    result, &container,
                                ))
                            }
                        }
                        None => None,
                    },
                    _ => None,
                }
            }
            BinOp::Slash(_) => {
                match (
                    self.try_get_num_value(&left),
                    self.try_get_num_value(&right),
                ) {
                    (Some(l), Some(r)) => {
                        let result = num_div(l, r);
                        if is_nan_or_inf(result) {
                            None
                        } else {
                            Some(Self::literal_expression_with_trivia_from_double(
                                result, &container,
                            ))
                        }
                    }
                    _ => None,
                }
            }
            BinOp::Percent(_) => {
                match (
                    self.try_get_num_value(&left),
                    self.try_get_num_value(&right),
                ) {
                    (Some(l), Some(r)) => {
                        // C# `long % 0` throws DivideByZeroException; the port
                        // leaves the expression unfolded instead (documented).
                        if matches!(&l, Value::Long(0)) || matches!(&r, Value::Long(0)) {
                            None
                        } else {
                            match num_mod(l, r) {
                                Some(result) => {
                                    if let Value::Double(d) = result {
                                        if is_nan_or_inf(d) {
                                            None
                                        } else {
                                            Some(Self::literal_expression_with_trivia_from_num(
                                                result, &container,
                                            ))
                                        }
                                    } else {
                                        Some(Self::literal_expression_with_trivia_from_num(
                                            result, &container,
                                        ))
                                    }
                                }
                                None => None,
                            }
                        }
                    }
                    _ => None,
                }
            }
            BinOp::Caret(_) => {
                match (
                    self.try_get_num_value(&left),
                    self.try_get_num_value(&right),
                ) {
                    (Some(l), Some(r)) => {
                        let result = num_as_double(l).powf(num_as_double(r));
                        if is_nan_or_inf(result) {
                            None
                        } else {
                            Some(Self::literal_expression_with_trivia_from_double(
                                result, &container,
                            ))
                        }
                    }
                    _ => None,
                }
            }
            BinOp::TwoDots(_) => {
                if Self::has_eflag(
                    left_flags,
                    ExpressionFlags::IS_STR | ExpressionFlags::IS_BOOL,
                ) && Self::has_eflag(
                    right_flags,
                    ExpressionFlags::IS_STR | ExpressionFlags::IS_BOOL,
                ) {
                    let left_str = match &left {
                        Expression::Symbol(t)
                            if matches!(
                                t.token().token_type(),
                                TokenType::Symbol {
                                    symbol: Symbol::True
                                }
                            ) =>
                        {
                            "true".to_string()
                        }
                        Expression::Symbol(t)
                            if matches!(
                                t.token().token_type(),
                                TokenType::Symbol {
                                    symbol: Symbol::False
                                }
                            ) =>
                        {
                            "false".to_string()
                        }
                        _ => self.get_value_str(&left),
                    };
                    let right_str = match &right {
                        Expression::Symbol(t)
                            if matches!(
                                t.token().token_type(),
                                TokenType::Symbol {
                                    symbol: Symbol::True
                                }
                            ) =>
                        {
                            "true".to_string()
                        }
                        Expression::Symbol(t)
                            if matches!(
                                t.token().token_type(),
                                TokenType::Symbol {
                                    symbol: Symbol::False
                                }
                            ) =>
                        {
                            "false".to_string()
                        }
                        _ => self.get_value_str(&right),
                    };
                    Some(Self::literal_expression_with_trivia_from_str(
                        &format!("{left_str}{right_str}"),
                        &container,
                    ))
                } else {
                    None
                }
            }
            BinOp::TwoEqual(_) => {
                if Self::has_eflag(left_flags, ExpressionFlags::IS_SCALAR)
                    && Self::has_eflag(right_flags, ExpressionFlags::IS_SCALAR)
                {
                    let result = Self::expr_equals(self, &left, &right, left_flags, right_flags);
                    Some(Self::literal_expression_with_trivia_from_bool(
                        result, &container,
                    ))
                } else {
                    None
                }
            }
            BinOp::TildeEqual(_) => {
                if Self::has_eflag(left_flags, ExpressionFlags::IS_SCALAR)
                    && Self::has_eflag(right_flags, ExpressionFlags::IS_SCALAR)
                {
                    let result = !Self::expr_equals(self, &left, &right, left_flags, right_flags);
                    Some(Self::literal_expression_with_trivia_from_bool(
                        result, &container,
                    ))
                } else {
                    None
                }
            }
            BinOp::LessThan(_) => {
                if Self::can_compare(left_flags, right_flags) {
                    let result = Self::compare(self, &left, &right, left_flags, right_flags);
                    Some(Self::literal_expression_with_trivia_from_bool(
                        result < 0,
                        &container,
                    ))
                } else {
                    None
                }
            }
            BinOp::LessThanEqual(_) => {
                if Self::can_compare(left_flags, right_flags) {
                    let result = Self::compare(self, &left, &right, left_flags, right_flags);
                    Some(Self::literal_expression_with_trivia_from_bool(
                        result <= 0,
                        &container,
                    ))
                } else {
                    None
                }
            }
            BinOp::GreaterThan(_) => {
                if Self::can_compare(left_flags, right_flags) {
                    let result = Self::compare(self, &left, &right, left_flags, right_flags);
                    Some(Self::literal_expression_with_trivia_from_bool(
                        result > 0,
                        &container,
                    ))
                } else {
                    None
                }
            }
            BinOp::GreaterThanEqual(_) => {
                if Self::can_compare(left_flags, right_flags) {
                    let result = Self::compare(self, &left, &right, left_flags, right_flags);
                    Some(Self::literal_expression_with_trivia_from_bool(
                        result >= 0,
                        &container,
                    ))
                } else {
                    None
                }
            }
            BinOp::And(_) => Self::try_convert_to_bool(&left).map(|result| {
                if result {
                    right.clone()
                } else {
                    left.clone()
                }
            }),
            BinOp::Or(_) => Self::try_convert_to_bool(&left).map(|result| {
                if !result {
                    right.clone()
                } else {
                    left.clone()
                }
            }),
            BinOp::Pipe(_) => self.fold_bitwise(
                &left,
                &right,
                left_flags,
                right_flags,
                &container,
                |l, r| l | r,
            ),
            BinOp::Ampersand(_) => self.fold_bitwise(
                &left,
                &right,
                left_flags,
                right_flags,
                &container,
                |l, r| l & r,
            ),
            BinOp::Tilde(_) => self.fold_bitwise(
                &left,
                &right,
                left_flags,
                right_flags,
                &container,
                |l, r| l ^ r,
            ),
            BinOp::DoubleGreaterThan(_) => self.fold_shift(
                &left,
                &right,
                left_flags,
                right_flags,
                &container,
                |l, r| l >> r,
            ),
            BinOp::DoubleLessThan(_) => self.fold_shift(
                &left,
                &right,
                left_flags,
                right_flags,
                &container,
                |l, r| l << r,
            ),
            _ => None,
        };

        match result {
            Some(folded) => folded,
            None => Expression::BinaryOperator {
                lhs: Box::new(left),
                binop,
                rhs: Box::new(right),
            },
        }
    }

    /// C# bitwise binary fold (Or/And/Xor).
    fn fold_bitwise(
        &self,
        left: &Expression,
        right: &Expression,
        left_flags: ExpressionFlags,
        right_flags: ExpressionFlags,
        container: &Expression,
        op: impl FnOnce(i64, i64) -> i64,
    ) -> Option<Expression> {
        if !Self::has_eflag(left_flags, ExpressionFlags::IS_NUM)
            || !Self::has_eflag(right_flags, ExpressionFlags::IS_NUM)
        {
            return None;
        }
        let left_val = self.try_get_int64(left)?;
        let right_val = self.try_get_int64(right)?;
        let result = op(left_val, right_val);
        if Self::has_eflag(left_flags, ExpressionFlags::IS_LONG)
            || Self::has_eflag(right_flags, ExpressionFlags::IS_LONG)
        {
            Some(Self::literal_expression_with_trivia_from_long(
                result, container,
            ))
        } else {
            // C# TryConvertToDouble always returns true (see its port).
            let converted = Self::try_convert_to_double(result);
            Some(Self::literal_expression_with_trivia_from_double(
                converted, container,
            ))
        }
    }

    /// C# shift binary fold (Right/Left).
    fn fold_shift(
        &self,
        left: &Expression,
        right: &Expression,
        left_flags: ExpressionFlags,
        right_flags: ExpressionFlags,
        container: &Expression,
        op: impl FnOnce(i64, u32) -> i64,
    ) -> Option<Expression> {
        if !Self::has_eflag(left_flags, ExpressionFlags::IS_NUM)
            || !Self::has_eflag(right_flags, ExpressionFlags::IS_NUM)
        {
            return None;
        }
        let left_val = self.try_get_int64(left)?;
        let right_val = self.try_get_int32(right)?;
        let result = op(left_val, right_val as u32);
        if Self::has_eflag(left_flags, ExpressionFlags::IS_LONG)
            || Self::has_eflag(right_flags, ExpressionFlags::IS_LONG)
        {
            Some(Self::literal_expression_with_trivia_from_long(
                result, container,
            ))
        } else {
            let converted = Self::try_convert_to_double(result);
            Some(Self::literal_expression_with_trivia_from_double(
                converted, container,
            ))
        }
    }

    /// C# local `exprEquals`.
    fn expr_equals(
        &self,
        left: &Expression,
        right: &Expression,
        left_flags: ExpressionFlags,
        right_flags: ExpressionFlags,
    ) -> bool {
        let mut result = false;
        if Self::has_eflag(left_flags, ExpressionFlags::IS_NIL)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_NIL)
        {
            result = true;
        } else if Self::has_eflag(left_flags, ExpressionFlags::IS_STR)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_STR)
        {
            result = self.get_value_str(left) == self.get_value_str(right);
        } else if Self::has_eflag(left_flags, ExpressionFlags::IS_BOOL)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_BOOL)
        {
            result = Self::has_eflag(left_flags, ExpressionFlags::IS_TRUTHY)
                == Self::has_eflag(right_flags, ExpressionFlags::IS_TRUTHY);
        } else if let (Some(l), Some(r)) =
            (self.try_get_num_value(left), self.try_get_num_value(right))
        {
            result = num_equal(l, r);
        }
        result
    }

    /// C# local `canCompare`.
    fn can_compare(left_flags: ExpressionFlags, right_flags: ExpressionFlags) -> bool {
        (Self::has_eflag(left_flags, ExpressionFlags::IS_NUM)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_NUM))
            || (Self::has_eflag(left_flags, ExpressionFlags::IS_STR)
                && Self::has_eflag(right_flags, ExpressionFlags::IS_STR))
    }

    /// C# local `compare` — ordinal string comparison is done over UTF-16
    /// code units to mirror `string.CompareOrdinal`.
    fn compare(
        &self,
        left: &Expression,
        right: &Expression,
        left_flags: ExpressionFlags,
        right_flags: ExpressionFlags,
    ) -> i32 {
        if Self::has_eflag(left_flags, ExpressionFlags::IS_DOUBLE)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_DOUBLE)
        {
            return compare_f64(self.get_value_f64(left), self.get_value_f64(right));
        }
        if Self::has_eflag(left_flags, ExpressionFlags::IS_LONG)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_LONG)
        {
            return compare_i64(self.get_value_i64(left), self.get_value_i64(right));
        }
        if Self::has_eflag(left_flags, ExpressionFlags::IS_DOUBLE)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_LONG)
        {
            return compare_f64(self.get_value_f64(left), self.get_value_i64(right) as f64);
        }
        if Self::has_eflag(left_flags, ExpressionFlags::IS_LONG)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_DOUBLE)
        {
            return compare_f64(self.get_value_i64(left) as f64, self.get_value_f64(right));
        }
        if Self::has_eflag(left_flags, ExpressionFlags::IS_STR)
            && Self::has_eflag(right_flags, ExpressionFlags::IS_STR)
        {
            let l: Vec<u16> = self.get_value_str(left).encode_utf16().collect();
            let r: Vec<u16> = self.get_value_str(right).encode_utf16().collect();
            return match l.cmp(&r) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };
        }
        panic!("Both expressions must have the same type.");
    }

    fn get_value_f64(&self, node: &Expression) -> f64 {
        match self.get_value(node) {
            Value::Double(v) => v,
            Value::Long(v) => v as f64,
            Value::Ulong(v) => v as f64,
            other => panic!("expected numeric value, got {other:?}"),
        }
    }

    fn get_value_i64(&self, node: &Expression) -> i64 {
        match self.get_value(node) {
            Value::Long(v) => v,
            other => panic!("expected long value, got {other:?}"),
        }
    }

    /// C# `VisitMemberAccessExpression`/`VisitElementAccessExpression` fold
    /// applied to full_moon `Var` expressions.
    fn fold_var(&self, var: Var) -> Expression {
        let container = var.clone();
        let Var::Expression(var_expr) = var else {
            return Expression::Var(Var::Name(var_expr_name(&container)));
        };
        // Only the first suffix participates when the prefix base is a
        // constant table; deeper suffixes need C#'s Parenthesized wrapping,
        // which full_moon renders structurally (documented adaptation).
        let first_suffix = var_expr.suffixes().next().cloned();
        let base = match var_expr.prefix() {
            Prefix::Expression(expr) => Some((**expr).clone()),
            Prefix::Name(_) => None,
            _ => None,
        };

        if let (Some(base), Some(first_suffix)) = (base, first_suffix) {
            if self
                .get_flags(&base)
                .has(ExpressionFlags::IS_CONSTANT_TABLE)
            {
                let table = match Self::get_inner_expression(&base) {
                    Expression::TableConstructor(table) => table.clone(),
                    _ => unreachable!(),
                };
                let folded: Option<Expression> = match &first_suffix {
                    Suffix::Index(Index::Dot { name, .. }) => {
                        let member_name = name.token().to_string();
                        Self::fold_member_access(self, &table, &member_name)
                    }
                    Suffix::Index(Index::Brackets { expression, .. }) => {
                        Self::fold_element_access(self, &table, expression)
                    }
                    _ => None,
                };
                if let Some(value) = folded {
                    let value = Self::with_trivia_from(value, &var_expr_expression(&container));
                    let remaining: Vec<Suffix> = var_expr.suffixes().skip(1).cloned().collect();
                    if remaining.is_empty() {
                        return value;
                    }
                    // C# wraps the folded value in parentheses unless it is a
                    // prefix expression; in full_moon the base is structural.
                    let base_expr = match &value {
                        Expression::Var(Var::Name(_)) | Expression::Var(Var::Expression(_)) => {
                            value
                        }
                        _ => Self::parenthesize(value),
                    };
                    return Expression::Var(Var::Expression(Box::new(
                        VarExpression::new(Prefix::Expression(Box::new(base_expr)))
                            .with_suffixes(remaining),
                    )));
                }
            }
        }

        Expression::Var(Var::Expression(var_expr))
    }

    /// C# `VisitMemberAccessExpression` field lookup (C# iterates the fields
    /// in reverse).
    fn fold_member_access(
        &self,
        table: &TableConstructor,
        member_name: &str,
    ) -> Option<Expression> {
        let fields: Vec<&Field> = table.fields().iter().collect();
        for field in fields.into_iter().rev() {
            match field {
                Field::NameKey { key, value, .. } => {
                    if key.token().to_string() == member_name {
                        return Some(value.clone());
                    }
                }
                Field::ExpressionKey { key, value, .. }
                    if {
                        self.get_flags(key).has(ExpressionFlags::IS_STR)
                            && self.get_value_str(key) == member_name
                    } =>
                {
                    return Some(value.clone());
                }
                _ => {}
            }
        }
        None
    }

    /// C# `VisitElementAccessExpression` field lookup.
    fn fold_element_access(
        &self,
        table: &TableConstructor,
        key_expression: &Expression,
    ) -> Option<Expression> {
        let fields: Vec<&Field> = table.fields().iter().collect();
        for field in fields.into_iter().rev() {
            match field {
                Field::NameKey { key, value, .. } => {
                    if self.get_flags(key_expression).has(ExpressionFlags::IS_STR)
                        && self.get_value_str(key_expression) == key.token().to_string()
                    {
                        return Some(value.clone());
                    }
                }
                Field::ExpressionKey { key, value, .. } if key.similar(key_expression) => {
                    return Some(value.clone());
                }
                _ => {}
            }
        }
        None
    }

    /// C# `ParenthesizedExpression(expression)` — fresh parentheses tokens.
    fn parenthesize(expr: Expression) -> Expression {
        Expression::Parentheses {
            contained: ContainedSpan::new(
                TokenReference::new(
                    Vec::new(),
                    Token::new(TokenType::Symbol {
                        symbol: Symbol::LeftParen,
                    }),
                    Vec::new(),
                ),
                TokenReference::new(
                    Vec::new(),
                    Token::new(TokenType::Symbol {
                        symbol: Symbol::RightParen,
                    }),
                    Vec::new(),
                ),
            ),
            expression: Box::new(expr),
        }
    }

    /// Applies the C# visitor fold rules to a fully-visited expression.
    fn fold(&self, expr: Expression) -> Expression {
        match expr {
            Expression::Parentheses { .. } => self.fold_parentheses(expr),
            Expression::UnaryOperator { .. } => self.fold_unary(expr),
            Expression::BinaryOperator { .. } => self.fold_binary(expr),
            Expression::Var(var) => self.fold_var(var),
            other => other,
        }
    }
}

/// C# `Var::Name` helper.
fn var_expr_name(var: &Var) -> TokenReference {
    match var {
        Var::Name(token) => token.clone(),
        _ => unreachable!(),
    }
}

/// The `Var` expression as an `Expression` (for trivia containers).
fn var_expr_expression(var: &Var) -> Expression {
    match var {
        Var::Name(token) => Expression::Var(Var::Name(token.clone())),
        Var::Expression(var_expr) => Expression::Var(Var::Expression(var_expr.clone())),
        _ => unreachable!(),
    }
}

/// Replaces the first token of an expression (C# `WithTriviaFrom` node form).
struct FirstTokenReplacer {
    replacement: TokenReference,
    done: bool,
}

impl FirstTokenReplacer {
    fn new(replacement: TokenReference) -> Self {
        Self {
            replacement,
            done: false,
        }
    }

    fn replace(self, expr: Expression) -> Expression {
        expr.visit_mut(&mut self.clone())
    }
}

impl Clone for FirstTokenReplacer {
    fn clone(&self) -> Self {
        Self {
            replacement: self.replacement.clone(),
            done: false,
        }
    }
}

impl VisitorMut for FirstTokenReplacer {
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        if !self.done {
            self.done = true;
            self.replacement.clone()
        } else {
            token_ref
        }
    }
}

/// full_moon `VisitorMut` traversal (C# `LuaSyntaxRewriter`): children are
/// visited first, then the C# fold rules are applied.
impl VisitorMut for ConstantFolder {
    fn visit_expression(&mut self, expr: Expression) -> Expression {
        let expr = match expr {
            Expression::BinaryOperator { lhs, binop, rhs } => Expression::BinaryOperator {
                lhs: Box::new(self.visit_expression(*lhs)),
                binop,
                rhs: Box::new(self.visit_expression(*rhs)),
            },
            Expression::Parentheses {
                contained,
                expression,
            } => Expression::Parentheses {
                contained,
                expression: Box::new(self.visit_expression(*expression)),
            },
            Expression::UnaryOperator { unop, expression } => Expression::UnaryOperator {
                unop,
                expression: Box::new(self.visit_expression(*expression)),
            },
            Expression::Function(body) => {
                Expression::Function(Box::new(self.visit_anonymous_function(*body)))
            }
            Expression::FunctionCall(call) => {
                Expression::FunctionCall(self.visit_function_call(call))
            }
            Expression::TableConstructor(table) => {
                Expression::TableConstructor(self.visit_table_constructor(table))
            }
            Expression::Var(var) => Expression::Var(self.visit_var(var)),
            Expression::IfExpression(if_expr) => {
                Expression::IfExpression(self.visit_if_expression(if_expr))
            }
            Expression::InterpolatedString(s) => {
                Expression::InterpolatedString(self.visit_interpolated_string(s))
            }
            Expression::TypeAssertion {
                expression,
                type_assertion,
            } => Expression::TypeAssertion {
                expression: Box::new(self.visit_expression(*expression)),
                type_assertion,
            },
            other => other,
        };
        self.fold(expr)
    }
}

/// C# dynamic numeric binary operations; mixed unsigned/signed cases would
/// throw C# binder exceptions and are returned as `None` (documented).
fn num_add(l: Value, r: Value) -> Option<Value> {
    match (l, r) {
        (Value::Long(l), Value::Long(r)) => Some(Value::Long(l.wrapping_add(r))),
        (Value::Ulong(l), Value::Ulong(r)) => Some(Value::Ulong(l.wrapping_add(r))),
        (Value::Long(l), Value::Double(r)) => Some(Value::Double(l as f64 + r)),
        (Value::Double(l), Value::Long(r)) => Some(Value::Double(l + r as f64)),
        (Value::Double(l), Value::Double(r)) => Some(Value::Double(l + r)),
        _ => None,
    }
}

fn num_sub(l: Value, r: Value) -> Option<Value> {
    match (l, r) {
        (Value::Long(l), Value::Long(r)) => Some(Value::Long(l.wrapping_sub(r))),
        (Value::Ulong(l), Value::Ulong(r)) => Some(Value::Ulong(l.wrapping_sub(r))),
        (Value::Long(l), Value::Double(r)) => Some(Value::Double(l as f64 - r)),
        (Value::Double(l), Value::Long(r)) => Some(Value::Double(l - r as f64)),
        (Value::Double(l), Value::Double(r)) => Some(Value::Double(l - r)),
        _ => None,
    }
}

fn num_mul(l: Value, r: Value) -> Option<Value> {
    match (l, r) {
        (Value::Long(l), Value::Long(r)) => Some(Value::Long(l.wrapping_mul(r))),
        (Value::Ulong(l), Value::Ulong(r)) => Some(Value::Ulong(l.wrapping_mul(r))),
        (Value::Long(l), Value::Double(r)) => Some(Value::Double(l as f64 * r)),
        (Value::Double(l), Value::Long(r)) => Some(Value::Double(l * r as f64)),
        (Value::Double(l), Value::Double(r)) => Some(Value::Double(l * r)),
        _ => None,
    }
}

/// C# `(double) (leftNum / (double) rightNum)` — always a double.
fn num_div(l: Value, r: Value) -> f64 {
    num_as_double(l) / num_as_double(r)
}

fn num_mod(l: Value, r: Value) -> Option<Value> {
    match (l, r) {
        (Value::Long(l), Value::Long(r)) => Some(Value::Long(l.wrapping_rem(r))),
        (Value::Ulong(l), Value::Ulong(r)) => Some(Value::Ulong(l.wrapping_rem(r))),
        (Value::Long(l), Value::Double(r)) => Some(Value::Double(l as f64 % r)),
        (Value::Double(l), Value::Long(r)) => Some(Value::Double(l % r as f64)),
        (Value::Double(l), Value::Double(r)) => Some(Value::Double(l % r)),
        _ => None,
    }
}

fn num_as_double(v: Value) -> f64 {
    match v {
        Value::Long(v) => v as f64,
        Value::Ulong(v) => v as f64,
        Value::Double(v) => v,
        _ => f64::NAN,
    }
}

fn num_equal(l: Value, r: Value) -> bool {
    match (l, r) {
        (Value::Long(l), Value::Long(r)) => l == r,
        (Value::Ulong(l), Value::Ulong(r)) => l == r,
        (Value::Long(l), Value::Double(r)) => l as f64 == r,
        (Value::Double(l), Value::Long(r)) => l == r as f64,
        (Value::Double(l), Value::Double(r)) => l == r,
        (Value::Ulong(l), Value::Long(r)) => l as f64 == r as f64,
        (Value::Long(l), Value::Ulong(r)) => l as f64 == r as f64,
        (Value::Ulong(l), Value::Double(r)) => l as f64 == r,
        (Value::Double(l), Value::Ulong(r)) => l == r as f64,
        _ => false,
    }
}

/// C# `Comparer<double>.Default.Compare` (no NaN literals exist, so
/// partial_cmp is total here).
fn compare_f64(l: f64, r: f64) -> i32 {
    match l.partial_cmp(&r) {
        Some(std::cmp::Ordering::Less) => -1,
        Some(std::cmp::Ordering::Equal) => 0,
        Some(std::cmp::Ordering::Greater) => 1,
        None => 0,
    }
}

fn compare_i64(l: i64, r: i64) -> i32 {
    match l.cmp(&r) {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

/// C# number literal suffixes (`LL`, `ULL`, `i`).
#[derive(Clone, Copy, PartialEq)]
enum NumberSuffix {
    None,
    Ll,
    Ull,
    I,
}

fn strip_number_suffix(text: &str) -> (&str, NumberSuffix) {
    if let Some(stripped) = text
        .strip_suffix("ULL")
        .or_else(|| text.strip_suffix("ull"))
    {
        (stripped, NumberSuffix::Ull)
    } else if let Some(stripped) = text.strip_suffix("LL").or_else(|| text.strip_suffix("ll")) {
        (stripped, NumberSuffix::Ll)
    } else if let Some(stripped) = text.strip_suffix('i') {
        (stripped, NumberSuffix::I)
    } else {
        (text, NumberSuffix::None)
    }
}

fn all_dec_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b == b'_' || b.is_ascii_digit())
}

fn all_hex_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b == b'_' || b.is_ascii_hexdigit())
}

fn all_oct_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b == b'_' || (b'0'..=b'7').contains(&b))
}

fn all_bin_digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b == b'_' || b == b'0' || b == b'1')
}

/// C# `s_decIntegerRegex` `^[+\-]?\d+$`.
fn is_dec_integer(value: &str) -> bool {
    let digits = value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// C# `s_hexIntegerRegex` `^[+\-]?0[xX][\da-fA-F]+$`.
fn is_hex_integer(value: &str) -> bool {
    let digits = value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value);
    let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    else {
        return false;
    };
    !hex.is_empty() && hex.bytes().all(|b| b.is_ascii_hexdigit())
}

/// .NET `long.TryParse(value, AllowHexSpecifier | AllowLeadingSign)`.
fn parse_hex_integer_with_sign(value: &str) -> Option<i64> {
    let (sign, digits) = match value.strip_prefix('-') {
        Some(rest) => (-1i64, rest),
        None => (1i64, value.strip_prefix('+').unwrap_or(value)),
    };
    let hex = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))?;
    let parsed = i64::from_str_radix(hex, 16).ok()?;
    Some(sign * parsed)
}

/// C# `s_decFloatRegex`
/// `[+\-]?(\.\d+|\d+(\.\d+)?)([eE][+\-]?\d+)?` — the regex is unanchored but
/// the .NET parser consumes the whole input, so a full match is equivalent.
fn is_dec_float(value: &str) -> bool {
    let digits = value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value);
    let (mantissa, exponent_ok) = if let Some(idx) = digits.find(['e', 'E']) {
        let (mantissa, exponent) = digits.split_at(idx);
        let exponent = exponent
            .get(1..)
            .and_then(|e| e.strip_prefix('+').or_else(|| e.strip_prefix('-')))
            .unwrap_or(exponent.get(1..).unwrap_or(""));
        (
            mantissa,
            !exponent.is_empty() && exponent.bytes().all(|b| b.is_ascii_digit()),
        )
    } else {
        (digits, true)
    };
    if !exponent_ok {
        return false;
    }
    let mantissa_ok = if let Some(dot) = mantissa.find('.') {
        let (int_part, frac_part) = mantissa.split_at(dot);
        let frac_part = frac_part.get(1..).unwrap_or("");
        (int_part.is_empty() || int_part.bytes().all(|b| b.is_ascii_digit()))
            && (frac_part.is_empty() || frac_part.bytes().all(|b| b.is_ascii_digit()))
            && (!int_part.is_empty() || !frac_part.is_empty())
    } else {
        !mantissa.is_empty() && mantissa.bytes().all(|b| b.is_ascii_digit())
    };
    mantissa_ok
}

/// C# `s_hexFloatRegex`
/// `[+\-]?0x(\.[\da-fA-F]+|[\da-fA-F]+(\.[\da-fA-F]+)?)([pP][+\-]?\d+)?`.
fn is_hex_float(value: &str) -> bool {
    let digits = value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value);
    let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    else {
        return false;
    };
    let (mantissa, exponent_ok) = if let Some(idx) = hex.find(['p', 'P']) {
        let (mantissa, exponent) = hex.split_at(idx);
        let exponent = exponent
            .get(1..)
            .and_then(|e| e.strip_prefix('+').or_else(|| e.strip_prefix('-')))
            .unwrap_or(exponent.get(1..).unwrap_or(""));
        (
            mantissa,
            !exponent.is_empty() && exponent.bytes().all(|b| b.is_ascii_digit()),
        )
    } else {
        (hex, true)
    };
    if !exponent_ok {
        return false;
    }
    let mantissa_ok = if let Some(dot) = mantissa.find('.') {
        let (int_part, frac_part) = mantissa.split_at(dot);
        let frac_part = frac_part.get(1..).unwrap_or("");
        (int_part.is_empty() || int_part.bytes().all(|b| b.is_ascii_hexdigit()))
            && (frac_part.is_empty() || frac_part.bytes().all(|b| b.is_ascii_hexdigit()))
            && (!int_part.is_empty() || !frac_part.is_empty())
    } else {
        !mantissa.is_empty() && mantissa.bytes().all(|b| b.is_ascii_hexdigit())
    };
    mantissa_ok
}

/// Unescapes a Lua string literal body (Loretta lexer value rules).
fn unescape_lua_string(raw: &str) -> String {
    let mut result = String::new();
    let mut chars = raw.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            result.push(c);
            continue;
        }
        let Some(escaped) = chars.next() else {
            result.push('\\');
            break;
        };
        match escaped {
            'a' => result.push('\x07'),
            'b' => result.push('\x08'),
            'f' => result.push('\x0C'),
            'n' => result.push('\n'),
            'r' => result.push('\r'),
            't' => result.push('\t'),
            'v' => result.push('\x0B'),
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            '\'' => result.push('\''),
            'x' => {
                let mut hex = String::new();
                for _ in 0..2 {
                    if let Some(&h) = chars.peek() {
                        if h.is_ascii_hexdigit() {
                            hex.push(h);
                            chars.next();
                        }
                    }
                }
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(char::from(byte));
                    }
                } else {
                    result.push('\\');
                    result.push('x');
                    result.push_str(&hex);
                }
            }
            'u' => {
                if chars.next_if_eq(&'{').is_some() {
                    let mut hex = String::new();
                    loop {
                        match chars.peek() {
                            Some(&'}') => {
                                chars.next();
                                break;
                            }
                            Some(&h) if h.is_ascii_hexdigit() => {
                                hex.push(h);
                                chars.next();
                            }
                            _ => {
                                hex.clear();
                                break;
                            }
                        }
                    }
                    if !hex.is_empty() {
                        if let Ok(codepoint) = u32::from_str_radix(&hex, 16) {
                            if let Some(c) = char::from_u32(codepoint) {
                                result.push(c);
                            }
                        }
                    }
                } else {
                    result.push_str("\\u");
                }
            }
            'z' => {
                while let Some(&w) = chars.peek() {
                    if w.is_whitespace() {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            '\n' => {
                // C# Lua 5.4 `\<newline>` line continuation.
            }
            '\r' => {
                if chars.next_if_eq(&'\n').is_some() {
                    // \r\n continuation.
                }
            }
            '0'..='9' => {
                let mut decimal = String::from(escaped);
                for _ in 0..2 {
                    if let Some(&d) = chars.peek() {
                        if d.is_ascii_digit() {
                            decimal.push(d);
                            chars.next();
                        }
                    }
                }
                if let Ok(byte) = decimal.parse::<u8>() {
                    result.push(char::from(byte));
                } else {
                    result.push('\\');
                    result.push_str(&decimal);
                }
            }
            other => {
                result.push('\\');
                result.push(other);
            }
        }
    }
    result
}
