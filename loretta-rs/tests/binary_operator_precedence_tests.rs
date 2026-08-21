// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Parsing.BinaryOperatorPrecedenceTests (b767b4e):
// BinaryOperatorPrecedenceTests
// C# source: src/Compilers/Lua/Test/Portable/Parsing/BinaryOperatorPrecedenceTests.cs
//
// The data-driven test parses `a OP1 b OP2 c` for every operator pair and
// verifies the binding follows the C# precedence table. The dropped
// SyntaxKind expression kinds (GetBinaryExpressionKinds, the generated
// SyntaxFacts.g.cs:1527-1551) dock on the full_moon BinOp values; the C#
// precedence table (SyntaxFacts.g.cs:73-127) matches the full_moon
// precedence_of_token table (ast/mod.rs:2301-2333) exactly (verified — the
// relative order is identical), and the C# right-associativity (the `^` and
// `..`, SyntaxFacts.cs:23-24) matches the full_moon parser. The TypeCast
// expression kind is absent from the full_moon BinOp set (like the C#
// untested exclusion of the FloorDivide).

use full_moon::tokenizer::Symbol;

/// The binary operator kinds under test (the C# GetBinaryExpressionKinds
/// minus the untested TypeCast/FloorDivide — the TypeCast has no full_moon
/// BinOp).
const BINARY_OPS: &[BinOpToken] = &[
    BinOpToken::Caret,
    BinOpToken::Percent,
    BinOpToken::Slash,
    BinOpToken::Star,
    BinOpToken::Minus,
    BinOpToken::Plus,
    BinOpToken::TwoDots,
    BinOpToken::DoubleLessThan,
    BinOpToken::DoubleGreaterThan,
    BinOpToken::Ampersand,
    BinOpToken::Tilde,
    BinOpToken::Pipe,
    BinOpToken::GreaterThan,
    BinOpToken::GreaterThanEqual,
    BinOpToken::LessThan,
    BinOpToken::LessThanEqual,
    BinOpToken::TildeEqual,
    BinOpToken::TwoEqual,
    BinOpToken::And,
    BinOpToken::Or,
];

/// The operator identity (the C# SyntaxKind docked on the full_moon symbol).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum BinOpToken {
    Caret,
    Percent,
    Slash,
    Star,
    Minus,
    Plus,
    TwoDots,
    DoubleLessThan,
    DoubleGreaterThan,
    Ampersand,
    Tilde,
    Pipe,
    GreaterThan,
    GreaterThanEqual,
    LessThan,
    LessThanEqual,
    TildeEqual,
    TwoEqual,
    And,
    Or,
}

impl BinOpToken {
    fn symbol(self) -> Symbol {
        match self {
            BinOpToken::Caret => Symbol::Caret,
            BinOpToken::Percent => Symbol::Percent,
            BinOpToken::Slash => Symbol::Slash,
            BinOpToken::Star => Symbol::Star,
            BinOpToken::Minus => Symbol::Minus,
            BinOpToken::Plus => Symbol::Plus,
            BinOpToken::TwoDots => Symbol::TwoDots,
            BinOpToken::DoubleLessThan => Symbol::DoubleLessThan,
            BinOpToken::DoubleGreaterThan => Symbol::DoubleGreaterThan,
            BinOpToken::Ampersand => Symbol::Ampersand,
            BinOpToken::Tilde => Symbol::Tilde,
            BinOpToken::Pipe => Symbol::Pipe,
            BinOpToken::GreaterThan => Symbol::GreaterThan,
            BinOpToken::GreaterThanEqual => Symbol::GreaterThanEqual,
            BinOpToken::LessThan => Symbol::LessThan,
            BinOpToken::LessThanEqual => Symbol::LessThanEqual,
            BinOpToken::TildeEqual => Symbol::TildeEqual,
            BinOpToken::TwoEqual => Symbol::TwoEqual,
            BinOpToken::And => Symbol::And,
            BinOpToken::Or => Symbol::Or,
        }
    }

    /// The C# GetBinaryOperatorPrecedence (SyntaxFacts.g.cs:73-127).
    fn precedence(self) -> i32 {
        match self {
            BinOpToken::Caret => 14,
            BinOpToken::Percent | BinOpToken::Slash | BinOpToken::Star => 11,
            BinOpToken::Minus | BinOpToken::Plus => 10,
            BinOpToken::TwoDots => 9,
            BinOpToken::DoubleLessThan | BinOpToken::DoubleGreaterThan => 7,
            BinOpToken::Ampersand => 6,
            BinOpToken::Tilde => 5,
            BinOpToken::Pipe => 4,
            BinOpToken::GreaterThan
            | BinOpToken::GreaterThanEqual
            | BinOpToken::LessThan
            | BinOpToken::LessThanEqual
            | BinOpToken::TildeEqual
            | BinOpToken::TwoEqual => 3,
            BinOpToken::And => 2,
            BinOpToken::Or => 1,
        }
    }

    /// The C# IsRightAssociative (SyntaxFacts.cs:23-24): the `^` and `..`.
    fn is_right_associative(self) -> bool {
        matches!(self, BinOpToken::Caret | BinOpToken::TwoDots)
    }
}

/// The C# LeftBindsStrongerThanRight (BinaryOperatorPrecedenceTests.cs:13-20).
fn left_binds_stronger_than_right(left: BinOpToken, right: BinOpToken) -> bool {
    let left_precedence = left.precedence();
    let right_precedence = right.precedence();
    if left_precedence > right_precedence {
        return true;
    }
    left_precedence == right_precedence && !left.is_right_associative()
}

/// The operator's text (the C# SyntaxFacts.GetText of the operator token).
fn op_text(op: BinOpToken) -> &'static str {
    match op {
        BinOpToken::Caret => "^",
        BinOpToken::Percent => "%",
        BinOpToken::Slash => "/",
        BinOpToken::Star => "*",
        BinOpToken::Minus => "-",
        BinOpToken::Plus => "+",
        BinOpToken::TwoDots => "..",
        BinOpToken::DoubleLessThan => "<<",
        BinOpToken::DoubleGreaterThan => ">>",
        BinOpToken::Ampersand => "&",
        BinOpToken::Tilde => "~",
        BinOpToken::Pipe => "|",
        BinOpToken::GreaterThan => ">",
        BinOpToken::GreaterThanEqual => ">=",
        BinOpToken::LessThan => "<",
        BinOpToken::LessThanEqual => "<=",
        BinOpToken::TildeEqual => "~=",
        BinOpToken::TwoEqual => "==",
        BinOpToken::And => "and",
        BinOpToken::Or => "or",
    }
}

/// Whether the expression is the identifier with the given name (the
/// full_moon parses the identifiers in the binary operands as the Var
/// primary expressions).
fn is_identifier(expr: &full_moon::ast::Expression, name: &str) -> bool {
    match expr {
        full_moon::ast::Expression::Symbol(t) => t.token().to_string() == name,
        full_moon::ast::Expression::Var(full_moon::ast::Var::Name(t)) => {
            t.token().to_string() == name
        }
        _ => false,
    }
}

/// The parsed root expression's operator symbol (None for non-binary roots).
fn root_symbol(expr: &full_moon::ast::Expression) -> Option<Symbol> {
    match expr {
        full_moon::ast::Expression::BinaryOperator { binop, .. } => {
            match binop.token().token().token_type() {
                full_moon::tokenizer::TokenType::Symbol { symbol } => Some(*symbol),
                _ => None,
            }
        }
        _ => None,
    }
}

#[test]
fn parser_does_binary_operator_precedences_correctly() {
    for left in BINARY_OPS {
        for right in BINARY_OPS {
            let text = format!("a {} b {} c", op_text(*left), op_text(*right));
            // The expression is parsed through the local-assignment wrapper
            // (the bare expression is not a valid chunk statement).
            let wrapped = format!("local _ = {text}");
            let ast =
                full_moon::parse_fallible(&wrapped, full_moon::LuaVersion::new().with_cfxlua())
                    .into_result()
                    .expect("the wrapper must parse");
            let stmt = ast.nodes().stmts().next().expect("the wrapper statement");
            let expr = match stmt {
                full_moon::ast::Stmt::LocalAssignment(la) => la
                    .expressions()
                    .iter()
                    .next()
                    .expect("the wrapper expression"),
                _ => panic!("unexpected statement: {stmt}"),
            };
            let root = root_symbol(expr).expect("a binary expression root");
            if left_binds_stronger_than_right(*left, *right) {
                // (a OP1 b) OP2 c — the root is the right operator.
                assert_eq!(root, right.symbol(), "root for {text:?}");
                match expr {
                    full_moon::ast::Expression::BinaryOperator { lhs, rhs, .. } => {
                        assert_eq!(
                            root_symbol(lhs).expect("the left child"),
                            left.symbol(),
                            "left child for {text:?}"
                        );
                        let rhs_expr: &full_moon::ast::Expression = rhs;
                        assert!(is_identifier(rhs_expr, "c"), "right child for {text:?}");
                    }
                    _ => unreachable!(),
                }
            } else {
                // a OP1 (b OP2 c) — the root is the left operator.
                assert_eq!(root, left.symbol(), "root for {text:?}");
                match expr {
                    full_moon::ast::Expression::BinaryOperator { lhs, rhs, .. } => {
                        assert!(is_identifier(lhs, "a"), "left child for {text:?}");
                        assert_eq!(
                            root_symbol(rhs).expect("the right child"),
                            right.symbol(),
                            "right child for {text:?}"
                        );
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
