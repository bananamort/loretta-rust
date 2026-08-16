// Ported from Loretta.CodeAnalysis.Lua.Experimental.Minifying.RenamingRewriter (b767b4e)
// C# source: src/Compilers/Lua/Experimental/Minifying/RenamingRewriter.cs

use std::collections::HashMap;

use full_moon::ast;
use full_moon::tokenizer::{Token, TokenReference, TokenType};
use full_moon::visitors::{VisitMut, VisitorMut};
use full_moon::ShortString;

use crate::experimental::minifying::islotallocator::ISlotAllocator;
use crate::experimental::minifying::namingstrategy::NamingStrategy;
use crate::experimental::minifying::renametable::{IdentifierRecord, RenameTable};
use crate::scoping::iscope::Scope;
use crate::script::script::Script;

/// C# RenamingRewriter (RenamingRewriter.cs:7-...).
pub struct RenamingRewriter {
    /// C# _renameTable (RenamingRewriter.cs:9).
    rename_table: RenameTable,
}

impl RenamingRewriter {
    /// C# RenamingRewriter(Script, NamingStrategy, ISlotAllocator)
    /// (RenamingRewriter.cs:11-15).
    pub fn new(
        script: Script,
        naming_strategy: NamingStrategy,
        slot_allocator: Box<dyn ISlotAllocator>,
    ) -> Self {
        RenamingRewriter {
            rename_table: RenameTable::new(script, naming_strategy, slot_allocator),
        }
    }

    /// Runs the rename over the tree (the C# visitor.Visit(root)). The port
    /// re-runs the scope walk to obtain the identifier records (the C# node
    /// locations with their source spans), asks the rename table per
    /// identifier, and replaces the tokens via the visitor.
    pub fn rewrite(&mut self, full_ast: &ast::Ast) -> ast::Ast {
        let mut walker = crate::script::scopeandvariablemanager::scopeandvariablewalker::ScopeAndVariableWalker::new(
            Scope::new(crate::scoping::scopekind::ScopeKind::Global, None, None),
            std::collections::HashMap::new(),
            std::collections::HashMap::new(),
        );
        walker.visit_ast(full_ast);
        let records: Vec<IdentifierRecord> = walker.identifier_positions;
        self.rename_table.prepare(&records);

        // The token replacements: position -> new name.
        let mut replacements: HashMap<usize, String> = HashMap::new();
        for (node, pos, _) in &records {
            if let Some(new_name) = self.rename_table.get_new_variable_name(node) {
                replacements.insert(*pos, new_name);
            }
        }

        let mut replacer = TokenReplacer {
            replacements,
            current_position: None,
        };
        let nodes = full_ast.nodes().clone().visit_mut(&mut replacer);
        full_ast.clone().with_nodes(nodes)
    }
}

/// Replaces the identifier tokens at the recorded positions (the C# node
/// visits' `WithIdentifier`/`Update` — the port matches by position).
struct TokenReplacer {
    replacements: HashMap<usize, String>,
    current_position: Option<usize>,
}

impl VisitorMut for TokenReplacer {
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        let position = token_ref.start_position().bytes();
        self.current_position = Some(position);
        match self.replacements.get(&position) {
            Some(new_name) => {
                let leading: Vec<Token> = token_ref.leading_trivia().cloned().collect();
                let trailing: Vec<Token> = token_ref.trailing_trivia().cloned().collect();
                TokenReference::new(
                    leading,
                    Token::new(TokenType::Identifier {
                        identifier: ShortString::new(new_name.clone()),
                    }),
                    trailing,
                )
            }
            None => token_ref,
        }
    }
}
