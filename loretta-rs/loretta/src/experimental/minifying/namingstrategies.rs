// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.NamingStrategies (b767b4e): NamingStrategies
// C# source: src/Compilers/Lua/Experimental/Minifying/NamingStrategies.cs

use crate::experimental::minifying::namingstrategy::NamingStrategy;

/// The default naming strategies.
pub struct NamingStrategies;

impl NamingStrategies {
    /// C# `MaxPrefixCount` — "We'll add up to 5 prefixes otherwise we'll call quits."
    const MAX_PREFIX_COUNT: usize = 5;

    /// C# `StringSequentialCore(int, IEnumerable<IScope>, char, string, int)`.
    /// The unavailable-names set is empty until IScope lands (scoping SCC);
    /// the keyword check mirrors `SyntaxFacts.GetKeywordKind`.
    fn string_sequential_core(
        slot: i32,
        prefix: char,
        alphabet: &str,
        min_prefix_count: usize,
    ) -> String {
        let len = get_max_digits(slot, alphabet.chars().count()) + Self::MAX_PREFIX_COUNT;
        let mut full_name: Vec<char> = vec![prefix; len];
        let mut pos = len - 1;
        let mut slot = slot;
        // C# do-while: the position is decremented on every iteration,
        // including the last (the max-digit count guarantees no underflow).
        loop {
            let num = slot % alphabet.chars().count() as i32;
            slot /= alphabet.chars().count() as i32;
            full_name[pos] = alphabet.chars().nth(num as usize).expect("index in range");
            pos -= 1;
            if slot == 0 {
                break;
            }
        }

        let first_name_char = pos + 1;
        let mut prefixes = first_name_char as i64 - min_prefix_count as i64;
        while prefixes > 0 {
            let name: String = full_name[prefixes as usize..].iter().collect();
            if !is_keyword_name(&name) {
                return name;
            }
            prefixes -= 1;
        }
        let base: String = full_name[first_name_char..].iter().collect();
        panic!("Code has too many variables named {base} with '{prefix}'s at the start.");
    }

    /// C# `Sequential(char, ImmutableArray<string>)` — a factory producing a
    /// capture-capable strategy; the C# ImmutableArray validation maps to the
    /// slice length check.
    pub fn sequential(prefix: char, alphabet: &'static [&'static str]) -> NamingStrategy {
        if alphabet.len() < 2 {
            panic!("Alphabet must have at least 2 elements.");
        }
        Box::new(move |slot: i32| {
            let mut name = String::new();
            let mut slot = slot;
            while slot > 0 {
                let num = slot % alphabet.len() as i32;
                slot /= alphabet.len() as i32;
                name.insert_str(0, alphabet[num as usize]);
            }

            let mut prefixes = 0usize;
            while prefixes <= Self::MAX_PREFIX_COUNT {
                if !is_keyword_name(&name) {
                    return name;
                }
                name.insert(0, prefix);
                prefixes += 1;
            }
            let base = name[prefixes..].to_string();
            panic!("Code has too many variables named {base} with '{prefix}'s at the start.");
        })
    }

    /// C# `Alphabetical(int, IEnumerable<IScope>)` — `_` prefix over
    /// `abcdefghijklmnopqrstuvwxyz`.
    pub fn alphabetical(slot: i32) -> String {
        Self::string_sequential_core(slot, '_', "abcdefghijklmnopqrstuvwxyz", 0)
    }

    /// C# `Numerical(int, IEnumerable<IScope>)` — `_` prefix over `0123456789`.
    pub fn numerical(slot: i32) -> String {
        Self::string_sequential_core(slot, '_', "0123456789", 1)
    }

    /// C# `ZeroWidth(int, IEnumerable<IScope>)` — ZERO WIDTH SPACE prefix over
    /// the zero-width non-joiner/joiner/no-break-space alphabet. ONLY WORKS
    /// WHEN TARGETTING LUAJIT.
    pub fn zero_width(slot: i32) -> String {
        Self::string_sequential_core(
            slot,
            '\u{200B}', // ZERO WIDTH SPACE
            "\u{200C}\u{200D}\u{FEFF}",
            0,
        )
    }
}

/// C# local `getMaxDigits` — `slot <= 1 ? 1 : (int) Math.Ceiling(Math.Log(slot, @base) + 1)`.
fn get_max_digits(slot: i32, base: usize) -> usize {
    if slot <= 1 {
        1
    } else {
        (slot as f64).log(base as f64).ceil() as usize + 1
    }
}

/// C# `SyntaxFacts.GetKeywordKind(name) == SyntaxKind.IdentifierToken` — the
/// name is not a Lua keyword (the 26-word set from SyntaxFacts.g.cs).
fn is_keyword_name(name: &str) -> bool {
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
    )
}
