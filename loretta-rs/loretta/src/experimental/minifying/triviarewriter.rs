// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.TriviaRewriter (b767b4e): TriviaRewriter
// C# source: src/Compilers/Lua/Experimental/Minifying/TriviaRewriter.cs
// RequiresSeparator logic ported from src/Compilers/Lua/Portable/Syntax/SyntaxFacts.cs:125
// (IsKeyword set from the generated SyntaxFacts.g.cs).

use full_moon::ast::Ast;
use full_moon::tokenizer::{Symbol, Token, TokenReference, TokenType};
use full_moon::visitors::{VisitMut, VisitorMut};
use full_moon::ShortString;

/// Rewrites tokens to strip trivia (whitespace and comments),
/// keeping only a single space separator where required.
///
/// Port note: C# `VisitToken` obtains the next token via the syntax tree
/// (`token.GetNextToken()`). full_moon's visitor has no next-token access, so
/// the port keeps a snapshot of the token stream (built in `visit_ast`) and a
/// cursor, mirroring `GetNextToken`'s view exactly: non-trivia tokens in
/// source order with the EOF token last.
pub struct TriviaRewriter {
    /// All non-trivia tokens in source order, with the EOF token last.
    tokens: Vec<TokenReference>,
    /// Index of the token currently being visited.
    index: usize,
}

impl TriviaRewriter {
    /// C# `public static readonly TriviaRewriter Instance = new();`
    pub const INSTANCE: Self = Self {
        tokens: Vec::new(),
        index: 0,
    };

    /// Port of `SyntaxFacts.RequiresSeparator(kindA, kindAText, kindB, kindBText)`
    /// (SyntaxFacts.cs:125). `kindAText` is only null-checked in C# and never
    /// read, so it is carried as `_text_a`.
    ///
    /// C# arms whose kinds full_moon can never produce are omitted with the
    /// reason in a comment at that spot:
    ///   - SingleLineCommentTrivia / MultiLineCommentTrivia kindB: comments are
    ///     trivia in both Loretta and full_moon; `GetNextToken` never yields
    ///     trivia, so those arms are unreachable in C# as well.
    ///   - BangToken / BangEqualsToken / AmpersandAmpersandToken /
    ///     PipePipeToken (GLua-only kinds): full_moon never produces them.
    ///
    /// C# `IsKeyword` additionally contains Continue/Type/Typeof/Export
    /// (Luau-only words); full_moon tokenizes those as identifiers, and every
    /// RequiresSeparator outcome is preserved (each pairing that would fire
    /// with keyword status also fires with identifier status).
    fn requires_separator(kind_a: Kind, _text_a: &str, kind_b: Kind, text_b: &str) -> bool {
        let kind_a_is_keyword = kind_a == Kind::Keyword;
        let kind_b_is_keyword = kind_b == Kind::Keyword;

        if kind_a == Kind::Identifier && kind_b == Kind::Identifier {
            return true;
        }
        if kind_a_is_keyword && kind_b_is_keyword {
            return true;
        }
        if kind_a_is_keyword && kind_b == Kind::Identifier {
            return true;
        }
        if kind_a == Kind::Identifier && kind_b_is_keyword {
            return true;
        }
        if kind_a == Kind::Identifier && kind_b == Kind::NumericLiteral {
            return true;
        }
        if kind_a == Kind::NumericLiteral && kind_b == Kind::Identifier {
            return true;
        }
        if kind_a == Kind::NumericLiteral && kind_b_is_keyword {
            return true;
        }
        if kind_a == Kind::NumericLiteral
            && matches!(
                kind_b,
                Kind::Operator(
                    Symbol::Dot | Symbol::TwoDots | Symbol::Ellipsis | Symbol::TwoDotsEqual
                )
            )
        {
            return true;
        }
        if kind_a_is_keyword && kind_b == Kind::NumericLiteral {
            return true;
        }
        if kind_a == Kind::NumericLiteral && kind_b == Kind::NumericLiteral {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::LeftBracket)
            && kind_b == Kind::Operator(Symbol::LeftBracket)
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::LeftBracket)
            && kind_b == Kind::StringLiteral
            && text_b.starts_with('[')
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Colon)
            && matches!(kind_b, Kind::Operator(Symbol::Colon | Symbol::TwoColons))
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Plus)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Minus)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        // C# also checks MinusToken && comment trivia with a "-" prefix —
        // unreachable (comment trivia never appears as a token), see above.
        if kind_a == Kind::Operator(Symbol::Minus)
            && matches!(kind_b, Kind::Operator(Symbol::Minus | Symbol::MinusEqual))
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Star)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Slash)
            && matches!(
                kind_b,
                Kind::Operator(Symbol::Equal | Symbol::SlashEqual | Symbol::TwoEqual)
            )
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Slash)
            && matches!(
                kind_b,
                Kind::Operator(Symbol::Slash | Symbol::Star | Symbol::StarEqual)
            )
        {
            return true;
        }
        // C# also checks SlashToken && comment trivia with a "/" prefix —
        // unreachable, see above.
        if kind_a == Kind::Operator(Symbol::Caret)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Percent)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::TwoDots)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        if matches!(kind_a, Kind::Operator(Symbol::Dot | Symbol::TwoDots))
            && matches!(
                kind_b,
                Kind::Operator(
                    Symbol::Dot | Symbol::TwoDots | Symbol::Ellipsis | Symbol::TwoDotsEqual
                )
            )
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Equal)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        // C# also checks BangToken && (Equals|EqualsEquals) — GLua-only kind,
        // unreachable, see above.
        if kind_a == Kind::Operator(Symbol::LessThan)
            && matches!(
                kind_b,
                Kind::Operator(
                    Symbol::LessThan
                        | Symbol::LessThanEqual
                        | Symbol::Equal
                        | Symbol::TwoEqual
                        | Symbol::DoubleLessThan
                )
            )
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::GreaterThan)
            && matches!(
                kind_b,
                Kind::Operator(
                    Symbol::GreaterThan
                        | Symbol::GreaterThanEqual
                        | Symbol::Equal
                        | Symbol::TwoEqual
                        | Symbol::DoubleGreaterThan
                )
            )
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Ampersand)
            && matches!(kind_b, Kind::Operator(Symbol::Ampersand))
        {
            return true;
        }
        // C# also checks AmpersandAmpersand — GLua-only, unreachable.
        if kind_a == Kind::Operator(Symbol::Pipe) && matches!(kind_b, Kind::Operator(Symbol::Pipe))
        {
            return true;
        }
        // C# also checks PipePipe — GLua-only, unreachable.
        // Dot can be the start of a number
        if matches!(
            kind_a,
            Kind::Operator(Symbol::Dot | Symbol::TwoDots | Symbol::Ellipsis)
        ) && kind_b == Kind::NumericLiteral
        {
            return true;
        }
        // C# shebang arm (HashToken && BangToken|BangEqualsToken): the shebang
        // is trivia and Bang is GLua-only — unreachable, see above.
        if kind_a == Kind::Operator(Symbol::Tilde)
            && matches!(kind_b, Kind::Operator(Symbol::Equal | Symbol::TwoEqual))
        {
            return true;
        }
        if kind_a == Kind::Operator(Symbol::Minus)
            && matches!(
                kind_b,
                Kind::Operator(Symbol::ThinArrow | Symbol::GreaterThan | Symbol::GreaterThanEqual)
            )
        {
            return true;
        }
        if matches!(kind_a, Kind::Operator(Symbol::Slash | Symbol::DoubleSlash))
            && matches!(
                kind_b,
                Kind::Operator(
                    Symbol::Slash
                        | Symbol::DoubleSlash
                        | Symbol::SlashEqual
                        | Symbol::DoubleSlashEqual
                        | Symbol::Equal
                        | Symbol::TwoEqual
                )
            )
        {
            return true;
        }
        false
    }
}

impl VisitorMut for TriviaRewriter {
    fn visit_ast(&mut self, ast: Ast) -> Ast {
        // Rebuild the token snapshot for this tree (the instance is reusable).
        self.tokens = collect_tokens(&ast);
        // The C# GetNextToken yields the EOF after the last token; the
        // collector only visits the node tokens, so the EOF is appended.
        self.tokens.push(ast.eof().to_owned());
        self.index = 0;
        let eof = ast.eof().to_owned();
        let nodes = ast.nodes().clone().visit_mut(self);
        ast.with_nodes(nodes).with_eof(self.visit_eof(eof))
    }

    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        // C#: `if (token.IsKind(SyntaxKind.None)) return token;` — the None-kind
        // guard maps to the EOF token (the only zero-width token in full_moon).
        if token_ref.token().token_kind() == full_moon::tokenizer::TokenKind::Eof {
            return token_ref;
        }

        let kind_a = kind_of(token_ref.token());
        let text_a = token_ref.token().to_string();
        let next = &self.tokens[self.index + 1];
        let kind_b = kind_of(next.token());
        let text_b = next.token().to_string();

        let rebuilt = if Self::requires_separator(kind_a, &text_a, kind_b, &text_b) {
            // C#: WithLeadingTrivia(none) + WithTrailingTrivia(SyntaxFactory.Space)
            TokenReference::new(
                Vec::new(),
                token_ref.token().to_owned(),
                vec![Token::new(TokenType::Whitespace {
                    characters: ShortString::new(" "),
                })],
            )
        } else {
            // C#: WithoutTrivia()
            TokenReference::new(Vec::new(), token_ref.token().to_owned(), Vec::new())
        };
        self.index += 1;
        rebuilt
    }
}

/// Collects every TokenReference in the AST in source order (non-trivia
/// tokens only, EOF last) — mirrors C# `DescendantTokens`/`GetNextToken`.
struct TokenCollector {
    tokens: Vec<TokenReference>,
}

impl VisitorMut for TokenCollector {
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        self.tokens.push(token_ref.clone());
        token_ref
    }
}

fn collect_tokens(ast: &Ast) -> Vec<TokenReference> {
    let mut collector = TokenCollector { tokens: Vec::new() };
    collector.visit_ast(ast.clone());
    collector.tokens
}

/// The token kind categories used by RequiresSeparator (full_moon projection
/// of the relevant C# SyntaxKinds).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Eof,
    Identifier,
    NumericLiteral,
    StringLiteral,
    /// A keyword word-symbol (and/break/do/.../while/goto).
    Keyword,
    /// An operator symbol (with its full_moon Symbol).
    Operator(Symbol),
    /// C# BacktickToken equivalent — no RequiresSeparator arm matches it.
    InterpolatedString,
}

fn is_keyword_symbol(symbol: Symbol) -> bool {
    matches!(
        symbol,
        Symbol::And
            | Symbol::Break
            | Symbol::Do
            | Symbol::Else
            | Symbol::ElseIf
            | Symbol::End
            | Symbol::False
            | Symbol::For
            | Symbol::Function
            | Symbol::Goto
            | Symbol::If
            | Symbol::In
            | Symbol::Local
            | Symbol::Nil
            | Symbol::Not
            | Symbol::Or
            | Symbol::Repeat
            | Symbol::Return
            | Symbol::Then
            | Symbol::True
            | Symbol::Until
            | Symbol::While
    )
}

fn kind_of(token: &Token) -> Kind {
    match token.token_type() {
        TokenType::Eof => Kind::Eof,
        TokenType::Identifier { .. } => Kind::Identifier,
        TokenType::Number { .. } => Kind::NumericLiteral,
        TokenType::StringLiteral { .. } => Kind::StringLiteral,
        TokenType::Symbol { symbol } if is_keyword_symbol(*symbol) => Kind::Keyword,
        TokenType::Symbol { symbol } => Kind::Operator(*symbol),
        TokenType::InterpolatedString { .. } => Kind::InterpolatedString,
        // Trivia tokens never appear as TokenReference tokens (they live in
        // leading/trailing trivia; C# GetNextToken likewise skips trivia).
        TokenType::Whitespace { .. }
        | TokenType::SingleLineComment { .. }
        | TokenType::MultiLineComment { .. }
        | TokenType::Shebang { .. }
        | TokenType::CStyleComment { .. } => unreachable!("trivia token reached kind_of"),
        // TokenType is #[non_exhaustive]; full_moon never produces other kinds.
        _ => unreachable!("unknown TokenType variant reached kind_of"),
    }
}
