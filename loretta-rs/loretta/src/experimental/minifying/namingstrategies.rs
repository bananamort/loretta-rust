// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.NamingStrategies (b767b4e): NamingStrategies
// C# source: src/Compilers/Lua/Experimental/Minifying/NamingStrategies.cs
// The SyntaxFacts.GetKeywordKind check maps to the Lua keyword set (the C#
// generated IsKeyword covers the Lua keywords + the Luau words continue/type/
// typeof/export, which the port treats as keywords too).

use std::cell::RefCell;
use std::rc::Rc;

use crate::experimental::minifying::minifyingutils::MinifyingUtils;
use crate::experimental::minifying::namingstrategy::NamingStrategy;
use crate::scoping::iscope::Scope;

/// The default naming strategies (C# NamingStrategies.cs:8).
pub struct NamingStrategies;

impl NamingStrategies {
    /// C# MaxPrefixCount (NamingStrategies.cs:11) — "We'll add up to 5
    /// prefixes otherwise we'll call quits."
    pub const MAX_PREFIX_COUNT: usize = 5;
}

/// C# SyntaxFacts.GetKeywordKind(name) == SyntaxKind.IdentifierToken — the
/// generated name must not be a keyword.
pub fn is_keyword(name: &str) -> bool {
    matches!(
        name,
        "and"
            | "break"
            | "do"
            | "else"
            | "elseif"
            | "end"
            | "false"
            | "for"
            | "function"
            | "goto"
            | "if"
            | "in"
            | "local"
            | "nil"
            | "not"
            | "or"
            | "repeat"
            | "return"
            | "then"
            | "true"
            | "until"
            | "while"
            | "continue"
            | "type"
            | "typeof"
            | "export"
    )
}

/// C# StringSequentialCore (NamingStrategies.cs:18-49).
fn string_sequential_core(
    mut slot: i32,
    scopes: &[Rc<RefCell<Scope>>],
    prefix: char,
    alphabet: &str,
    min_prefix_count: usize,
) -> String {
    // The base-conversion digits (the C# fills the buffer from the least
    // significant digit; the digit count is the conversion length).
    let mut digits: Vec<char> = Vec::new();
    loop {
        let num = (slot % alphabet.len() as i32) as usize;
        slot /= alphabet.len() as i32;
        digits.push(alphabet.chars().nth(num).expect("alphabet index"));
        if slot <= 0 {
            break;
        }
    }
    digits.reverse();
    let base_name: String = digits.into_iter().collect();
    let unavailable_names = MinifyingUtils::get_unavailable_names(scopes);
    let mut prefixes = min_prefix_count;
    while prefixes <= NamingStrategies::MAX_PREFIX_COUNT {
        let name = format!("{}{}", prefix.to_string().repeat(prefixes), base_name);
        if !is_keyword(&name) && !unavailable_names.contains(&name) {
            return name;
        }
        prefixes += 1;
    }
    panic!(
        "Code has too many variables named {} with '{}'s at the start.",
        base_name, prefix
    );
}

impl NamingStrategies {
    /// C# Sequential (NamingStrategies.cs:66-101).
    pub fn sequential(prefix: char, alphabet: &[String]) -> NamingStrategy {
        if alphabet.len() < 2 {
            panic!("Alphabet must have at least 2 elements.");
        }
        let alphabet = alphabet.to_vec();
        Box::new(move |mut slot: i32, scopes: &[Rc<RefCell<Scope>>]| {
            let mut name_parts: Vec<&str> = Vec::new();
            while slot > 0 {
                let num = (slot % alphabet.len() as i32) as usize;
                slot /= alphabet.len() as i32;
                name_parts.insert(0, &alphabet[num]);
            }
            let mut name = name_parts.concat();
            let mut prefixes = 0;
            let unavailable_names = MinifyingUtils::get_unavailable_names(scopes);
            while prefixes <= NamingStrategies::MAX_PREFIX_COUNT {
                if !is_keyword(&name) && !unavailable_names.contains(&name) {
                    return name;
                }
                name = format!("{prefix}{name}");
                prefixes += 1;
            }
            panic!(
                "Code has too many variables named {} with '{}'s at the start.",
                name.trim_start_matches(prefix),
                prefix
            );
        })
    }

    /// C# Alphabetical (NamingStrategies.cs:113-120).
    pub fn alphabetical(slot: i32, scopes: &[Rc<RefCell<Scope>>]) -> String {
        string_sequential_core(slot, scopes, '_', "abcdefghijklmnopqrstuvwxyz", 0)
    }

    /// C# Numerical (NamingStrategies.cs:134-141).
    pub fn numerical(slot: i32, scopes: &[Rc<RefCell<Scope>>]) -> String {
        string_sequential_core(slot, scopes, '_', "0123456789", 1)
    }

    /// C# ZeroWidth (NamingStrategies.cs:155-162) — ONLY WORKS WHEN
    /// TARGETTING LUAJIT.
    pub fn zero_width(slot: i32, scopes: &[Rc<RefCell<Scope>>]) -> String {
        string_sequential_core(
            slot,
            scopes,
            '\u{200B}',                 // ZERO WIDTH SPACE
            "\u{200C}\u{200D}\u{FEFF}", // ZWNJ, ZWJ, ZWNO-BREAK SPACE
            0,
        )
    }
}
