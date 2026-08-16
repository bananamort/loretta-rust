// Ported from Loretta.CodeAnalysis.Lua.Syntax.SyntaxNode (b767b4e) — the
// dropped Syntax model maps to a minimal value type carrying the oracle data
// the scoping/script clusters need (the C# SyntaxKind name + source text).

/// A syntax node (C# SyntaxNode — the dropped Syntax infrastructure). The
/// scoping/script clusters only observe the node's kind name and text; the
/// `id` provides the C# node-reference identity for the maps.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node {
    /// The Loretta SyntaxKind name (e.g. "CompilationUnit", "DoStatement") —
    /// oracle data for the scope op.
    pub kind: String,
    /// The node's source text (C# ToString()).
    pub text: String,
    /// The node's identity (C# reference identity).
    pub id: u64,
}

impl Node {
    /// Creates a node from its kind name and source text.
    pub fn new(kind: impl Into<String>, text: impl Into<String>) -> Self {
        Node {
            kind: kind.into(),
            text: text.into(),
            id: 0,
        }
    }

    /// Creates a node with a unique identity id (the C# node-reference
    /// identity used by the scope/variable maps).
    pub fn new_with_id(kind: impl Into<String>, text: impl Into<String>, id: u64) -> Self {
        Node {
            kind: kind.into(),
            text: text.into(),
            id,
        }
    }

    /// C# SyntaxNode.Kind() — the kind name (oracle data).
    pub fn kind_name(&self) -> &str {
        &self.kind
    }
}
