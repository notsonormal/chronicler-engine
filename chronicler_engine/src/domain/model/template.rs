//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Template placeholder substitution for author-controlled text fields.

/// Known template variables available for substitution.
#[derive(Debug, Clone)]
pub struct TemplateVars {
    /// `{{user}}` — the player character's name.
    pub user: String,
}

impl TemplateVars {
    /// Creates a new `TemplateVars` with the given user name.
    pub fn new(user: impl Into<String>) -> Self {
        Self { user: user.into() }
    }
}

/// Replaces known template placeholders in `text` with values from `vars`.
/// Placeholder syntax: `{{key}}` — double curly braces, alphanumeric keys.
/// Unknown placeholders are left as-is (not stripped, not filtered).
pub fn render_template(text: &str, vars: &TemplateVars) -> String {
    text.replace("{{user}}", &vars.user)
}
