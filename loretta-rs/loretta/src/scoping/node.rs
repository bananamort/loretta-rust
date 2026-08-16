// Ported from Loretta.CodeAnalysis.Lua.Syntax.SyntaxNode (b767b4e) — the
// dropped Syntax model maps to a minimal value type carrying the oracle data
// the scoping/script clusters need (the C# SyntaxKind name + source text).

/// A syntax node (C# SyntaxNode — the dropped Syntax infrastructure). The
/// scoping/script clusters only observe the node's kind name and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// The Loretta SyntaxKind name (e.g. "CompilationUnit", "DoStatement") —
    /// oracle data for the scope op.
    pub kind: String,
    /// The node's source text (C# ToString()).
    pub text: String,
}

impl Node {
    /// Creates a node from its kind name and source text.
    pub fn new(kind: impl Into<String>, text: impl Into<String>) -> Self {
        Node {
            kind: kind.into(),
            text: text.into(),
        }
    }

    /// C# SyntaxNode.Kind() — the kind name (oracle data).
    pub fn kind_name(&self) -> &str {
        &self.kind
    }
}
