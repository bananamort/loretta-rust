// Ported from Loretta.CodeAnalysis.Lua.LuaParseOptions (b767b4e): LuaParseOptions
// C# source: src/Compilers/Lua/Portable/LuaParseOptions.cs

use crate::luasyntaxoptions::LuaSyntaxOptions;

/// Stores source parsing related options and offers access to their values.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaParseOptions {
    /// The LuaSyntaxOptions to use when parsing.
    pub syntax_options: LuaSyntaxOptions,
    /// C# `_features`: ImmutableDictionary<string, string>, always empty by
    /// default ("the features flag doesn't do anything currently").
    pub features: Vec<(String, String)>,
    /// C# DocumentationMode (Core enum); the base ParseOptions ctor fixes it
    /// to Parse, and WithDocumentationMode can replace it.
    documentation_mode: String,
}

impl LuaParseOptions {
    /// The default parse options.
    pub fn default_options() -> Self {
        Self {
            syntax_options: LuaSyntaxOptions::ALL,
            features: Vec::new(),
            documentation_mode: "Parse".to_string(),
        }
    }

    /// Creates a new set of parse options.
    pub fn new(syntax_options: LuaSyntaxOptions) -> Self {
        Self {
            syntax_options,
            features: Vec::new(),
            documentation_mode: "Parse".to_string(),
        }
    }

    /// The documentation mode. C# `DocumentationMode` "does nothing
    /// currently"; the base ParseOptions ctor fixes it to Parse.
    pub fn documentation_mode(&self) -> &str {
        &self.documentation_mode
    }

    /// Creates a new instance with the documentation mode replaced, or
    /// returns self when it already matches. C# `WithDocumentationMode`
    /// (LuaParseOptions.cs:65-71).
    pub fn with_documentation_mode(&self, documentation_mode: &str) -> Self {
        if self.documentation_mode != documentation_mode {
            Self {
                syntax_options: self.syntax_options.clone(),
                features: self.features.clone(),
                documentation_mode: documentation_mode.to_string(),
            }
        } else {
            self.clone()
        }
    }

    /// The language name (C# `LanguageNames.Lua`).
    pub fn language(&self) -> &'static str {
        "Lua"
    }

    /// Validates the options, appending diagnostics to the builder.
    /// C# delegates to the base ParseOptions.ValidateOptions
    /// (ParseOptions.cs:49-55), which reports ERR_BadDocumentationMode
    /// for an invalid documentation mode (the valid modes are None,
    /// Parse and Diagnose — DocumentationMode.cs:12-32). Finding 64
    /// restored the validation path (the old empty body dropped it).
    pub fn validate_options(&self, builder: &mut Vec<String>) {
        if !matches!(
            self.documentation_mode.as_str(),
            "None" | "Parse" | "Diagnose"
        ) {
            builder.push(format!(
                "LUA{}: Provided documentation mode is unsupported or invalid: '{}'",
                crate::errors::errorcode::ErrorCode::ErrBadDocumentationMode as i32,
                self.documentation_mode
            ));
        }
    }

    /// The C# Features indexer — the IReadOnlyDictionary with the
    /// OrdinalIgnoreCase keys (LuaParseOptions.cs:56-57) — the
    /// case-insensitive lookup (Finding 65).
    pub fn get_feature(&self, key: &str) -> Option<&str> {
        self.features
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    /// Creates a new instance with the features replaced by the provided ones.
    /// C# `WithFeatures` (LuaParseOptions.cs:80-81) builds an
    /// ImmutableDictionary with StringComparer.OrdinalIgnoreCase — the
    /// keys are case-insensitive and a duplicate key (ignoring case)
    /// keeps the LAST value (the ToImmutableDictionary semantics;
    /// Finding 65 restored them).
    pub fn with_features(&self, features: Vec<(String, String)>) -> Self {
        let mut deduped: Vec<(String, String)> = Vec::new();
        for (key, value) in features {
            if let Some(slot) = deduped
                .iter_mut()
                .find(|(existing, _)| existing.eq_ignore_ascii_case(&key))
            {
                slot.1 = value;
            } else {
                deduped.push((key, value));
            }
        }
        Self {
            syntax_options: self.syntax_options.clone(),
            features: deduped,
            documentation_mode: self.documentation_mode.clone(),
        }
    }

    /// Creates a new instance with the syntax options replaced by the provided ones.
    pub fn with_syntax_options(&self, syntax_options: LuaSyntaxOptions) -> Self {
        if self.syntax_options != syntax_options {
            Self {
                syntax_options,
                features: self.features.clone(),
                documentation_mode: self.documentation_mode.clone(),
            }
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn features_are_case_insensitive_like_the_csharp_dictionary() {
        // Finding 65: the C# WithFeatures builds an ImmutableDictionary
        // with StringComparer.OrdinalIgnoreCase (LuaParseOptions.cs:80-81)
        // — the keys are case-insensitive and a duplicate key (ignoring
        // case) keeps the LAST value; the equality/hash include the
        // features (the C# EqualsHelper SequenceEqual — order- and
        // case-sensitive pairs, LuaParseOptions.cs:107-115).
        let options = LuaParseOptions::default_options();
        // The lookup is case-insensitive.
        let with = options.with_features(vec![("Foo".to_string(), "1".to_string())]);
        assert_eq!(with.get_feature("foo"), Some("1"));
        assert_eq!(with.get_feature("FOO"), Some("1"));
        assert_eq!(options.get_feature("anything"), None);
        // A duplicate key (ignoring case) keeps the last value.
        let with = options.with_features(vec![
            ("Foo".to_string(), "1".to_string()),
            ("foo".to_string(), "2".to_string()),
        ]);
        assert_eq!(with.get_feature("FOO"), Some("2"));
        // The equality includes the features (the SequenceEqual
        // semantics — order- and case-sensitive pairs).
        assert_ne!(
            options,
            options.with_features(vec![("foo".to_string(), "bar".to_string())])
        );
        assert_ne!(
            options.with_features(vec![("foo".to_string(), "1".to_string())]),
            options.with_features(vec![("FOO".to_string(), "1".to_string())])
        );
        assert_eq!(
            options.with_features(vec![("foo".to_string(), "1".to_string())]),
            options.with_features(vec![("foo".to_string(), "1".to_string())])
        );
    }

    use super::*;

    #[test]
    fn validate_options_reports_an_invalid_documentation_mode() {
        // Finding 64: the empty validate_options dropped the C# base
        // ParseOptions.ValidateOptions (ParseOptions.cs:49-55), which
        // reports ERR_BadDocumentationMode for an invalid documentation
        // mode (the valid modes are None, Parse and Diagnose).
        let mut builder = Vec::new();
        LuaParseOptions::default_options().validate_options(&mut builder);
        assert!(builder.is_empty(), "the default Parse mode is valid");
        let options = LuaParseOptions::default_options().with_documentation_mode("None");
        let mut builder = Vec::new();
        options.validate_options(&mut builder);
        assert!(builder.is_empty(), "None is a valid mode");
        let options = LuaParseOptions::default_options().with_documentation_mode("Bogus");
        let mut builder = Vec::new();
        options.validate_options(&mut builder);
        assert_eq!(
            builder,
            vec![
                "LUA2000: Provided documentation mode is unsupported or invalid: 'Bogus'"
                    .to_string()
            ]
        );
    }
}
