// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Scoping.ScriptTestsBase (b767b4e): ScriptTestsBase
// C# source: src/Compilers/Lua/Test/Portable/Scoping/ScriptTestsBase.cs
//
// The two ParseScriptAsync helpers parse each code with the round-trip check,
// verify the tree has no diagnostics, and build a Script. The port's Script
// (loretta/src/script/script.rs) takes the code strings (the C# takes the
// dropped SyntaxTree objects — the port's Script re-parses the codes
// internally); the tree diagnostics verify maps to the lexer-diagnostics
// scanner (the C# lexer is DROP — see the row-773 port).

use full_moon::ast::Ast;

use loretta::luaparseoptions::LuaParseOptions;
use loretta::luasyntaxoptions::LuaSyntaxOptions;
use loretta::script::script::Script;

use crate::lexerdiagnostics::lexer_diagnostics;
use crate::luatestbase::LuaTestBase;

/// C# ScriptTestsBase (ScriptTestsBase.cs:5-30): the scoping test base.
pub struct ScriptTestsBase;

impl ScriptTestsBase {
    /// C# ParseScriptAsync(code, options) (ScriptTestsBase.cs:7-13): parse
    /// with the round-trip check, verify no diagnostics, then build the
    /// script from the single tree.
    pub fn parse_script_async(code: &str, options: Option<&LuaSyntaxOptions>) -> (Ast, Script) {
        let options = options.unwrap_or(&LuaSyntaxOptions::ALL);
        let parse_options = LuaParseOptions::new(options.clone());
        let ast = LuaTestBase::parse_with_round_trip_check_async(code, Some(&parse_options));
        let produced = lexer_diagnostics(code, options);
        assert!(
            produced.is_empty(),
            "unexpected diagnostics for {code:?}: {produced:?}"
        );
        let script = Script::new_with_options(vec![code.to_string()], options.clone());
        (ast, script)
    }

    /// C# ParseScriptAsync(params codes) (ScriptTestsBase.cs:15-16): the
    /// all-features syntax options.
    pub fn parse_script_async_codes(codes: &[&str]) -> Script {
        Self::parse_script_async_many(&LuaSyntaxOptions::ALL, codes)
    }

    /// C# ParseScriptAsync(options, params codes) (ScriptTestsBase.cs:18-29):
    /// parse each code with the round-trip check and the zero-diagnostics
    /// verify, then build the script from all the trees.
    pub fn parse_script_async_many(options: &LuaSyntaxOptions, codes: &[&str]) -> Script {
        let parse_options = LuaParseOptions::new(options.clone());
        for code in codes {
            let ast = LuaTestBase::parse_with_round_trip_check_async(code, Some(&parse_options));
            let _ = ast;
            let produced = lexer_diagnostics(code, options);
            assert!(
                produced.is_empty(),
                "unexpected diagnostics for {code:?}: {produced:?}"
            );
        }
        Script::new_with_options(
            codes.iter().map(|c| c.to_string()).collect(),
            options.clone(),
        )
    }
}
