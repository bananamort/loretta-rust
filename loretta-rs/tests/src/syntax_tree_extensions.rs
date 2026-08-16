// Ported from Loretta.CodeAnalysis.Lua.UnitTests.SyntaxTreeExtensions (b767b4e): SyntaxTreeExtensions
// C# source: src/Compilers/Lua/Test/Utilities/SyntaxTreeExtensions.cs

/// C# SyntaxTreeExtensions (SyntaxTreeExtensions.cs:13-96): test helpers over
/// the dropped SyntaxTree. The tree docks on the source text (the parse
/// boundary — the helpers apply the C# TextChange to the byte text, re-parsed
/// at the test boundary). Offsets are byte offsets per TRANSLATION.md — the
/// C# UTF-16 offsets are only meaningful on ASCII test sources.
///
/// The C# Dump helpers (lines 54-61) and the LuaSyntaxPrinter walker (lines
/// 63-94) are intentionally dropped: they walk the dropped SyntaxNode via the
/// dropped LuaSyntaxWalker infra (Locked Decision 1), and no Ported test calls
/// them (verified: only WithReplace is used, by Parsing/RegressionTests.cs:64).
pub struct SyntaxTreeExtensions;

impl SyntaxTreeExtensions {
    /// C# WithReplace (SyntaxTreeExtensions.cs:15-20): the text after
    /// replacing [offset, offset+length) with newText.
    pub fn with_replace(text: &str, offset: usize, length: usize, new_text: &str) -> String {
        let mut result = String::with_capacity(text.len() + new_text.len() - length);
        result.push_str(&text[..offset]);
        result.push_str(new_text);
        result.push_str(&text[offset + length..]);
        result
    }

    /// C# WithReplaceFirst (SyntaxTreeExtensions.cs:22-28): replaces the first
    /// occurrence of oldText. A missing occurrence is a test-author error (the
    /// C# IndexOf returns -1 and the downstream TextSpan throws).
    pub fn with_replace_first(text: &str, old_text: &str, new_text: &str) -> String {
        let offset = text
            .find(old_text)
            .expect("the text to replace must occur in the source");
        let length = old_text.len();
        Self::with_replace(text, offset, length, new_text)
    }

    /// C# WithReplace (SyntaxTreeExtensions.cs:30-36): replaces the first
    /// occurrence of oldText at or after startIndex.
    pub fn with_replace_from(
        text: &str,
        start_index: usize,
        old_text: &str,
        new_text: &str,
    ) -> String {
        let relative = text[start_index..]
            .find(old_text)
            .expect("the text to replace must occur at or after startIndex");
        let offset = start_index + relative;
        let length = old_text.len();
        Self::with_replace(text, offset, length, new_text)
    }

    /// C# WithInsertAt (SyntaxTreeExtensions.cs:38-39): inserts newText at the
    /// given offset.
    pub fn with_insert_at(text: &str, offset: usize, new_text: &str) -> String {
        Self::with_replace(text, offset, 0, new_text)
    }

    /// C# WithInsertBefore (SyntaxTreeExtensions.cs:41-46): inserts newText
    /// before the first occurrence of existingText.
    pub fn with_insert_before(text: &str, existing_text: &str, new_text: &str) -> String {
        let offset = text
            .find(existing_text)
            .expect("the text to insert before must occur in the source");
        Self::with_replace(text, offset, 0, new_text)
    }

    /// C# WithRemoveAt (SyntaxTreeExtensions.cs:48-49): removes the
    /// [offset, offset+length) text.
    pub fn with_remove_at(text: &str, offset: usize, length: usize) -> String {
        Self::with_replace(text, offset, length, "")
    }

    /// C# WithRemoveFirst (SyntaxTreeExtensions.cs:51-52): removes the first
    /// occurrence of oldText.
    pub fn with_remove_first(text: &str, old_text: &str) -> String {
        Self::with_replace_first(text, old_text, "")
    }
}
