//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Template placeholder substitution for author-controlled text fields.

use crate::domain::model::character::PersonaCard;

/// Known template variables available for substitution.
#[derive(Debug, Clone)]
pub struct TemplateVars {
    /// `{{user}}` — the player character's name.
    pub user: String,
    /// `{{persona_description}}` — the player character's description.
    pub persona_description: String,
    /// `{{persona_personality}}` — the player character's personality.
    pub persona_personality: String,
    /// `{{persona_background}}` — the player character's background/scenario.
    pub persona_background: String,
}

impl TemplateVars {
    /// Creates a new `TemplateVars` with the given user name and empty persona
    /// fields. Use [`TemplateVars::from_persona`] to populate persona macros.
    pub fn new(user: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            persona_description: String::new(),
            persona_personality: String::new(),
            persona_background: String::new(),
        }
    }

    /// Creates a `TemplateVars` with `{{user}}` and the persona macros
    /// (`{{persona_description}}`, `{{persona_personality}}`,
    /// `{{persona_background}}`) populated from a persona card's sheet.
    pub fn from_persona(persona: &PersonaCard) -> Self {
        Self {
            user: persona.sheet.name.clone(),
            persona_description: persona.sheet.description.clone(),
            persona_personality: persona.sheet.personality.clone(),
            persona_background: persona.sheet.scenario.clone(),
        }
    }
}
