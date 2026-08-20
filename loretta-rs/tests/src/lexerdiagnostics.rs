// The C#-mirror lexer diagnostics scanner lives in the loretta crate
// (loretta/src/errors/lexerdiagnostics.rs — the audit finding-B placement);
// the tests re-export it so the row-773+ fixtures keep their imports.

pub use loretta::errors::lexerdiagnostics::*;
