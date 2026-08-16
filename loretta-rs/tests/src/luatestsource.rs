// Ported from Loretta.CodeAnalysis.Lua.Test.Utilities.LuaTestSource (b767b4e): LuaTestSource
// C# source: src/Compilers/Lua/Test/Utilities/LuaTestSource.cs

use full_moon::ast::Ast;

use crate::luatestbase::LuaTestBase;
use loretta::luaparseoptions::LuaParseOptions;

/// C# LuaTestSource (LuaTestSource.cs:14-72): a readonly struct that holds the
/// source code used for a Lua test in whichever shape the test provided it
/// (string, string[], a pre-parsed tree, ...) and turns it into the parsed
/// tree list. The dropped `SyntaxTree` maps to the full_moon AST; the C#
/// `object`-typed `Value` (a boxed reference) maps to the boxed
/// [`LuaTestSourceValue`] below (the C# `default: throw` arm of
/// GetSyntaxTreesAsync is unreachable because the union is closed here); the
/// C# implicit operators map to the `From` impls below. The C# `filename`
/// parameter of ParseAsync is dropped — it only set the dropped tree's
/// FilePath metadata.
#[derive(Debug, Clone)]
pub enum LuaTestSource {
    /// C# `None` (LuaTestSource.cs:16) — `new(null)`.
    None,
    /// C# `Value` (LuaTestSource.cs:18) — the boxed carried value.
    Value(Box<LuaTestSourceValue>),
}

/// The carried value union (the C# `Value`/`object`, LuaTestSource.cs:18-23).
#[derive(Debug, Clone)]
pub enum LuaTestSourceValue {
    /// C# `string` value.
    Text(String),
    /// C# `string[]` value.
    Texts(Vec<String>),
    /// C# `SyntaxTree` value (maps to the dropped tree's AST).
    Tree(Box<Ast>),
    /// C# `SyntaxTree[]` / `List<SyntaxTree>` / `ImmutableArray<SyntaxTree>`.
    Trees(Vec<Ast>),
    /// C# `LuaTestSource[]` value.
    Sources(Vec<LuaTestSource>),
}

impl LuaTestSource {
    /// C# `public static LuaTestSource None => new(null);` (LuaTestSource.cs:16).
    pub fn none() -> Self {
        Self::None
    }

    /// C# `Value` (LuaTestSource.cs:18) — the carried value, if any.
    pub fn value(&self) -> Option<&LuaTestSourceValue> {
        match self {
            Self::None => None,
            Self::Value(value) => Some(value),
        }
    }

    /// C# GetSyntaxTreesAsync (LuaTestSource.cs:25-57): parses the carried
    /// source(s) into the AST list, or returns the carried tree(s) directly.
    pub fn get_syntax_trees_async(
        &self,
        parse_options: Option<&LuaParseOptions>,
        source_file_name: &str,
    ) -> Vec<Ast> {
        match self {
            Self::None => Vec::new(),
            Self::Value(value) => match value.as_ref() {
                LuaTestSourceValue::Text(source) => {
                    vec![LuaTestBase::parse_async(source, parse_options)]
                }
                LuaTestSourceValue::Texts(sources) => {
                    assert!(
                        source_file_name.is_empty(),
                        "the source file name must be empty when parsing multiple sources"
                    );
                    let source_refs: Vec<&str> = sources.iter().map(String::as_str).collect();
                    LuaTestBase::parse_async_many(&source_refs, parse_options)
                }
                LuaTestSourceValue::Tree(tree) => {
                    assert!(
                        parse_options.is_none(),
                        "a pre-parsed tree cannot take parse options"
                    );
                    assert!(
                        source_file_name.is_empty(),
                        "the source file name must be empty for a pre-parsed tree"
                    );
                    vec![tree.as_ref().clone()]
                }
                LuaTestSourceValue::Trees(trees) => {
                    assert!(
                        parse_options.is_none(),
                        "pre-parsed trees cannot take parse options"
                    );
                    assert!(
                        source_file_name.is_empty(),
                        "the source file name must be empty for pre-parsed trees"
                    );
                    trees.clone()
                }
                LuaTestSourceValue::Sources(test_sources) => {
                    let mut list = Vec::new();
                    for source in test_sources {
                        list.extend(source.get_syntax_trees_async(parse_options, source_file_name));
                    }
                    list
                }
            },
        }
    }
}

/// C# `implicit operator LuaTestSource(string)` (LuaTestSource.cs:59).
impl From<&str> for LuaTestSource {
    fn from(source: &str) -> Self {
        Self::Value(Box::new(LuaTestSourceValue::Text(source.to_string())))
    }
}

/// C# `implicit operator LuaTestSource(string)` (LuaTestSource.cs:59).
impl From<String> for LuaTestSource {
    fn from(source: String) -> Self {
        Self::Value(Box::new(LuaTestSourceValue::Text(source)))
    }
}

/// C# `implicit operator LuaTestSource(string[])` (LuaTestSource.cs:61).
impl From<Vec<String>> for LuaTestSource {
    fn from(source: Vec<String>) -> Self {
        Self::Value(Box::new(LuaTestSourceValue::Texts(source)))
    }
}

/// C# `implicit operator LuaTestSource(SyntaxTree)` (LuaTestSource.cs:63).
impl From<Ast> for LuaTestSource {
    fn from(source: Ast) -> Self {
        Self::Value(Box::new(LuaTestSourceValue::Tree(Box::new(source))))
    }
}

/// C# `implicit operator LuaTestSource(SyntaxTree[])` (LuaTestSource.cs:65),
/// `List<SyntaxTree>` (LuaTestSource.cs:67) and
/// `ImmutableArray<SyntaxTree>` (LuaTestSource.cs:69).
impl From<Vec<Ast>> for LuaTestSource {
    fn from(source: Vec<Ast>) -> Self {
        Self::Value(Box::new(LuaTestSourceValue::Trees(source)))
    }
}

/// C# `implicit operator LuaTestSource(LuaTestSource[])` (LuaTestSource.cs:71).
impl From<Vec<LuaTestSource>> for LuaTestSource {
    fn from(source: Vec<LuaTestSource>) -> Self {
        Self::Value(Box::new(LuaTestSourceValue::Sources(source)))
    }
}
