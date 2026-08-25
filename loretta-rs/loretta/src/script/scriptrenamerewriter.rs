// Ported from Loretta.CodeAnalysis.Lua.Script.RenameRewriter (b767b4e): RenameRewriter
// C# source: src/Compilers/Lua/Portable/Script/Script.RenameRewriter.cs

use std::collections::HashSet;
use std::rc::Rc;

use full_moon::tokenizer::{Token, TokenReference, TokenType};
use full_moon::visitors::{VisitMut, VisitorMut};
use full_moon::ShortString;

use crate::scoping::ivariable::SharedVariable;
use crate::script::scopeandvariablemanager::scopeandvariablewalker::ScopeAndVariableWalker;

/// C# RenameRewriter (RenameRewriter.cs:7-62): renames the variable's
/// identifier tokens across a tree.
pub struct RenameRewriter {
    /// C# _newName (RenameRewriter.cs:11).
    new_name: String,
    /// The target token positions (byte offsets).
    target_positions: HashSet<usize>,
}

impl RenameRewriter {
    /// C# RenameRewriter(Script, IVariable, string) (RenameRewriter.cs:13-19).
    pub fn new(target_positions: HashSet<usize>, new_name: String) -> Self {
        RenameRewriter {
            new_name,
            target_positions,
        }
    }

    /// C# CreateIdentifier (RenameRewriter.cs:21-27): the identifier token
    /// with the new name and the original trivia.
    fn create_identifier(&self, original: &TokenReference) -> TokenReference {
        let leading: Vec<Token> = original.leading_trivia().cloned().collect();
        let trailing: Vec<Token> = original.trailing_trivia().cloned().collect();
        TokenReference::new(
            leading,
            Token::new(TokenType::Identifier {
                identifier: ShortString::new(self.new_name.clone()),
            }),
            trailing,
        )
    }
}

impl VisitorMut for RenameRewriter {
    /// C# the rewriter's token replacement (the C# node visits replace the
    /// identifier tokens; the port matches by the recorded positions).
    fn visit_token_reference(&mut self, token_ref: TokenReference) -> TokenReference {
        if self
            .target_positions
            .contains(&token_ref.start_position().bytes())
        {
            self.create_identifier(&token_ref)
        } else {
            token_ref
        }
    }
}

/// Runs the rename over a tree (the C# visitor.Visit(root) +
/// WithRootAndOptions). The port re-runs the scope walk to obtain the
/// variable's node identity (the C# GetVariable map), collects the token
/// positions of the nodes mapping to the target variable, and replaces the
/// tokens via the visitor.
pub fn rename_in_tree(
    tree_idx: usize,
    tree: &str,
    variable: &SharedVariable,
    new_name: &str,
    script: &mut crate::script::script::Script,
) -> String {
    // The C# RenameRewriter runs over the error-recovery tree root — the
    // C# tree never fails to parse (Candidate E). full_moon's AstResult
    // carries the reconstructed AST alongside the errors, so the recovered
    // tree is renamed like the C# visits its error nodes.
    let full_ast = full_moon::parse_fallible(tree, full_moon::LuaVersion::new()).into_ast();
    // The target node ids come from the script's memoized state (the C#
    // GetVariable map identity); the walk below reproduces the node ids to
    // collect the token positions. The state's node ids continue across
    // trees (Finding 5), so the re-walk must seed its counter with this
    // tree's id base.
    let state = script.scope_and_variable_manager_state();
    let base = state.tree_id_bases.get(tree_idx).copied().unwrap_or(0);
    let target_nodes: HashSet<u64> = state
        .variables
        .iter()
        .filter(|(_, v)| Rc::ptr_eq(v, variable))
        .map(|(node, _)| node.id)
        .collect();

    let mut walker = ScopeAndVariableWalker::new(
        crate::scoping::iscope::Scope::new(
            crate::scoping::scopekind::ScopeKind::Global,
            None,
            None,
        ),
        std::collections::HashMap::new(),
        std::collections::HashMap::new(),
        std::rc::Rc::new(std::cell::Cell::new(base)),
    );
    walker.visit_ast(&full_ast);
    let positions = walker.identifier_positions;

    let target_positions: HashSet<usize> = positions
        .iter()
        .filter(|(node, _, _)| target_nodes.contains(&node.id))
        .map(|(_, pos, _)| *pos)
        .collect();

    let mut rewriter = RenameRewriter::new(target_positions, new_name.to_string());
    let nodes = full_ast.nodes().clone().visit_mut(&mut rewriter);
    let rewritten = full_ast.with_nodes(nodes);
    rewritten.to_string()
}
