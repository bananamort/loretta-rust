// Ported from Loretta.CodeAnalysis.Lua.UnitTests.Experimental.RegressionTests (b767b4e): RegressionTests
// C# source: src/Compilers/Lua/Test/Portable/Experimental/RegressionTests.cs

use loretta::experimental::luaextensions::minify_with_strategy;
use loretta::experimental::minifying::namingstrategies::NamingStrategies;

#[test]
fn naming_strategies_alphabetic_does_not_fall_into_an_infinite_loop() {
    // C# RegressionTests.cs:10-22 — renaming 'c' to 'b' collides with the
    // existing global 'b', so the strategy prefixes '_' instead of looping.
    let code = "local a, c = 1, 2\r\nprint(a, b)";
    let minified = minify_with_strategy(code, Box::new(NamingStrategies::alphabetical));
    assert_eq!(minified, "local a,_b=1,2 print(a,b)");
}

#[test]
fn minifier_does_not_double_free_on_read_and_write_ending_in_the_same_place() {
    // C# RegressionTests.cs:24-44 (WorkItem 55): a read and a write ending at
    // the same position must not free the variable twice.
    let cases: &[(&str, &str)] = &[
        ("local x = 0\nx = x + 1", "local a=0 a=a+1"),
        ("local x = 0\nx += x + 1", "local a=0 a+=a+1"),
    ];
    for (code, expected) in cases {
        let minified = minify_with_strategy(code, Box::new(NamingStrategies::alphabetical));
        assert_eq!(minified, *expected, "minify({code:?})");
    }
}
