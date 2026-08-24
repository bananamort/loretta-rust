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

#[test]
fn slot_release_follows_the_last_use_like_the_csharp() {
    // Finding 39: the slot is released when the visited node equals OR
    // descends from the last use (RenameTable.cs:78-80 — the port's
    // self-identity-only check and its equivalence comment were wrong;
    // the port now uses the byte-span containment as the descendant
    // relation, since the node model has no parent links). After the
    // last use of `a`, `b` reuses its slot and takes the same name (the
    // SortedSlotAllocator — the minify default; the differential's
    // sequential allocator does not reuse).
    let cases: &[(&str, &str)] = &[
        (
            "local a = 1 print(a) local b = 2 print(b)",
            "local a=1 print(a)local a=2 print(a)",
        ),
        (
            "local a = 1 print(a) local b = 2",
            "local a=1 print(a)local a=2",
        ),
        (
            "local a = 1 print(a) a = 2 local b = 3 print(b)",
            "local a=1 print(a)a=2 local a=3 print(a)",
        ),
    ];
    for (code, expected) in cases {
        let minified = minify_with_strategy(code, Box::new(NamingStrategies::alphabetical));
        assert_eq!(minified, *expected, "minify({code:?})");
    }
}
