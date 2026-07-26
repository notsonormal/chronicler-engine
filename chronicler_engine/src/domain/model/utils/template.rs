//! [DOC: chronicler_engine/docs/diataxis/reference/narrative/prompt_system.md]
//! Template placeholder substitution for author-controlled text fields.

use crate::domain::model::template::TemplateVars;

/// Replaces known template placeholders in `text` with values from `vars`.
/// Placeholder syntax: `{{key}}` — double curly braces, alphanumeric keys.
/// Unknown placeholders are left as-is (not stripped, not filtered).
pub fn render_template(text: &str, vars: &TemplateVars) -> String {
    text.replace("{{user}}", &vars.user)
}
