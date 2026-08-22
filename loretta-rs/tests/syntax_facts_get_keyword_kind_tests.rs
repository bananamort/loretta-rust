// Ported from Loretta.CodeAnalysis.Lua.UnitTests.SyntaxFactsGetKeywordKindTests (b767b4e):
// SyntaxFacts_GetKeywordKindString_ReturnsTheCorrectKindForEachKeyword
// C# source: src/Compilers/Lua/Test/Portable/Syntax/SyntaxFactsGetKeywordKindTests.cs
//
// The C# SyntaxFacts.GetKeywordKind maps a keyword's text to its kind (the
// red-tree SyntaxFacts class is dropped); the port docks on the full_moon
// tokenizer's Symbol::from_str text -> symbol mapping.

use full_moon::tokenizer::Symbol;
use full_moon::LuaVersion;

/// The C# SyntaxFacts.IsKeyword keyword texts (the 22 Lua keywords).
const KEYWORDS: [&str; 22] = [
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "if", "in", "local",
    "nil", "not", "or", "repeat", "return", "then", "true", "until", "while", "goto",
];

/// The C# Data() negative cases: texts that must map to IdentifierToken.
const NON_KEYWORDS: [&str; 4] = ["alseif", "doif", "andor", "and or"];

fn keyword_symbol(text: &str) -> Symbol {
    Symbol::from_str(text, LuaVersion::new())
        .unwrap_or_else(|| panic!("{text:?} must map to a keyword symbol"))
}

/// C# SyntaxFacts_GetKeywordKindString_ReturnsTheCorrectKindForEachKeyword
/// (SyntaxFactsGetKeywordKindTests.cs:8-16): GetKeywordKind(text) returns
/// the keyword kind for every keyword text; the port asserts the full_moon
/// text -> symbol mapping round-trips the keyword set and rejects the
/// identifier texts.
#[test]
fn syntaxfacts_getkeywordkindstring_returnsthecorrectkindforeachkeyword() {
    for text in KEYWORDS {
        let symbol = keyword_symbol(text);
        assert_eq!(symbol.to_string(), text, "the keyword round-trips its text");
    }
    for text in NON_KEYWORDS {
        assert!(
            Symbol::from_str(text, LuaVersion::new()).is_none(),
            "{text:?} is not a keyword — it must map to IdentifierToken"
        );
    }
}
