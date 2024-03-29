use crate::key::Key;

/// An action that can be performed in the application
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// The name of the action
    pub name: String,

    /// The short name of the action
    pub short_name: String,

    /// The codepoint (if any) in the flexpad-icon font that represents this action
    pub icon_codepoint: Option<char>,

    /// The keyboard shortcut (if any) that can trigger this action
    pub shortcut: Option<Key>,
}

impl Action {
    /// Creates a new [`Action`]
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            short_name: name.to_string(),
            icon_codepoint: None,
            shortcut: None,
        }
    }

    /// Sets the short_name of this [`Action`]
    pub fn short_name(mut self, short_name: impl ToString) -> Self {
        self.short_name = short_name.to_string();
        self
    }

    /// Adds a codepoint in the flexpad-icon font to this [`Action`]
    pub fn icon_codepoint(mut self, icon_codepoint: char) -> Self {
        self.icon_codepoint = Some(icon_codepoint);
        self
    }

    /// Adds a shortcut to this [`Action`]
    pub fn shortcut(mut self, shortcut: Key) -> Self {
        self.shortcut = Some(shortcut);
        self
    }
}
