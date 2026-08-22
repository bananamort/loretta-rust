// Ported from Loretta.CodeAnalysis.Lua.UnitTests.SyntaxNormalizerTests (b767b4e):
// SyntaxNormalizer_CorrectlyInsertsExpressionSpaces
// C# source: src/Compilers/Lua/Test/Portable/Syntax/SyntaxNormalizerTests.cs
//
// The C# SyntaxNormalizer (the red-tree NormalizeWhitespace utility) is a
// dropped node-class utility (the LuaExtensions.cs precedent, AGENTS.md);
// the port asserts the parse structure the normalizer relies on.

use full_moon::ast::{Call, FunctionArgs, Stmt, Suffix};
use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;

use loretta_tests::luatestbase::options_to_version;

/// C# SyntaxNormalizer_CorrectlyInsertsExpressionSpaces
/// (SyntaxNormalizerTests.cs:1697-1705, WorkItem 108): parses "print(1,2)"
/// and asserts the normalized form "print(1, 2)". The SyntaxNormalizer is a
/// dropped red-tree utility; the port asserts the clean parse + the
/// argument structure the space insertion operates on: two parenthesized
/// arguments.
#[test]
fn syntaxnormalizer_correctlyinsertsexpressionspaces() {
    let text = "print(1,2)";
    let result = full_moon::parse_fallible(
        text,
        options_to_version(&LuaParseOptions::new(LuaSyntaxOptions::ALL)),
    );
    assert!(
        result.errors().is_empty(),
        "no parse errors: {:?}",
        result.errors()
    );
    assert_eq!(result.ast().to_string(), text, "the text must round-trip");
    let stmts: Vec<&Stmt> = result.ast().nodes().stmts().collect();
    assert_eq!(stmts.len(), 1, "exactly one statement");
    match stmts[0] {
        Stmt::FunctionCall(call) => {
            let mut args: Option<&FunctionArgs> = None;
            for suffix in call.suffixes() {
                if let Suffix::Call(Call::AnonymousCall(function_args)) = suffix {
                    args = Some(function_args);
                }
            }
            match args {
                Some(FunctionArgs::Parentheses { arguments, .. }) => {
                    let texts: Vec<String> = arguments.iter().map(|e| e.to_string()).collect();
                    assert_eq!(texts, ["1", "2"], "the arguments the normalizer spaces");
                }
                _ => panic!("the call must have parenthesized arguments"),
            }
        }
        _ => panic!("the statement must be a function call"),
    }
}
