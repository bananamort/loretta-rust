// Ported from Loretta.CodeAnalysis.Lua.Test.Utilities.RandomSpaceInserter (b767b4e): RandomSpaceInserter
// C# source: src/Compilers/Lua/Test/Utilities/RandomSpaceInserterDataAttribute.cs

/// C# RandomSpaceInserter (RandomSpaceInserterDataAttribute.cs:5-26): yields
/// every combination of having a single space inserted between adjacent token
/// parts (the data source behind the RandomSpaceInserterDataAttribute test
/// generator).
pub struct RandomSpaceInserter;

impl RandomSpaceInserter {
    /// C# GetTokenPairs (RandomSpaceInserterDataAttribute.cs:7-25): for each
    /// mask in 0..=2^(parts.Length - 1) - 1, joins the parts inserting a
    /// space where the mask bit is set. Added per TRANSLATION.md Numbers:
    /// C# `1UL << n` masks the shift count (& 63); Rust panics on oversize
    /// shifts, so the count is masked explicitly.
    pub fn get_token_pairs(parts: &[&str]) -> Vec<String> {
        let space_locations = parts.len().saturating_sub(1);
        // The C# lastCase = (1UL << spaceLocations) - 1 (masked & 63).
        let last_case = (1u64 << (space_locations & 63)) - 1;
        let mut result = Vec::new();
        let mut builder = String::new();
        for spaces in 0..=last_case {
            builder.clear();
            for (part_idx, part) in parts.iter().take(space_locations).enumerate() {
                builder.push_str(part);
                if ((1u64 << (part_idx & 63)) & spaces) != 0 {
                    builder.push(' ');
                }
            }
            // The C# trailing parts[^1].
            builder.push_str(parts[parts.len() - 1]);
            result.push(builder.clone());
        }
        result
    }
}
