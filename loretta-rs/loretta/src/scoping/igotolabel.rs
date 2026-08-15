// Ported from Loretta.CodeAnalysis.Lua.IGotoLabel (b767b4e): IGotoLabel, IGotoLabelInternal, GotoLabel
// C# source: src/Compilers/Lua/Portable/Scoping/IGotoLabel.cs

use full_moon::ast::lua52;

/// The interface for a goto label.
pub trait IGotoLabel {
    /// The label's name.
    fn name(&self) -> &str;

    /// The label's syntax node (::label::), if it exists.
    fn label_syntax(&self) -> Option<&lua52::Label>;

    /// The goto statements that jump to this label.
    fn jump_syntaxes(&self) -> &[lua52::Goto];
}

/// Internal interface for goto labels that can have jumps added.
pub trait IGotoLabelInternal: IGotoLabel {
    /// Adds a jump to this label.
    fn add_jump(&mut self, jump: lua52::Goto);
}

/// Concrete implementation of a goto label.
pub struct GotoLabel {
    name: String,
    label_syntax: Option<lua52::Label>,
    jumps: Vec<lua52::Goto>,
}

impl GotoLabel {
    /// Creates a new GotoLabel.
    pub fn new(name: String, label: Option<lua52::Label>) -> Self {
        assert!(!name.is_empty(), "GotoLabel name must not be empty");
        Self {
            name,
            label_syntax: label,
            jumps: Vec::new(),
        }
    }
}

impl IGotoLabel for GotoLabel {
    fn name(&self) -> &str {
        &self.name
    }

    fn label_syntax(&self) -> Option<&lua52::Label> {
        self.label_syntax.as_ref()
    }

    fn jump_syntaxes(&self) -> &[lua52::Goto] {
        &self.jumps
    }
}

impl IGotoLabelInternal for GotoLabel {
    fn add_jump(&mut self, jump: lua52::Goto) {
        self.jumps.push(jump);
    }
}
