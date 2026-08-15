// Ported from Loretta.CodeAnalysis.Lua.IGotoLabel (b767b4e): IGotoLabel, IGotoLabelInternal, GotoLabel
// C# source: src/Compilers/Lua/Portable/Scoping/IGotoLabel.cs
// NOTE: GotoLabelStatementSyntax and GotoStatementSyntax are from dropped Syntax infrastructure.
// In full_moon, these are lua52::Label and lua52::Goto (feature-gated).

/// The interface for a goto label.
pub trait IGotoLabel {
    /// The label's name.
    fn name(&self) -> &str;
}

/// Internal interface for goto labels that can have jumps added.
pub trait IGotoLabelInternal: IGotoLabel {
    /// Adds a jump to this label.
    fn add_jump(&mut self, jump_index: usize);
}

/// Concrete implementation of a goto label.
pub struct GotoLabel {
    name: String,
    jump_indices: Vec<usize>,
}

impl GotoLabel {
    pub fn new(name: String) -> Self {
        assert!(!name.is_empty(), "GotoLabel name must not be empty");
        Self {
            name,
            jump_indices: Vec::new(),
        }
    }
}

impl IGotoLabel for GotoLabel {
    fn name(&self) -> &str {
        &self.name
    }
}

impl IGotoLabelInternal for GotoLabel {
    fn add_jump(&mut self, jump_index: usize) {
        self.jump_indices.push(jump_index);
    }
}
