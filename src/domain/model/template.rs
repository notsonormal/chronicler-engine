//! [DOC: docs/diataxis/reference/narrative/prompt_system.md]
//! Template placeholder substitution for author-controlled text fields.

use crate::domain::model::character::PersonaCard;

/// Default number of generated options substituted for `{{option_count}}`.
pub const DEFAULT_OPTION_COUNT: u32 = 3;

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
    /// `{{narrative_perspective}}` — the configured narrative point of view (`second` / `third`).
    pub narrative_perspective: String,
    /// `{{narrative_tense}}` — the configured narrative tense (`past` / `present`).
    pub narrative_tense: String,
    /// `{{option_count}}` — how many options the options prompts request.
    pub option_count: String,
}

impl TemplateVars {
    pub fn new(user: impl Into<String>) -> Self {
        Self {
            user: user.into(),
            persona_description: String::new(),
            persona_personality: String::new(),
            persona_background: String::new(),
            narrative_perspective: "third".to_string(),
            narrative_tense: "past".to_string(),
            option_count: DEFAULT_OPTION_COUNT.to_string(),
        }
    }

    pub fn from_persona(persona: &PersonaCard) -> Self {
        Self {
            user: persona.sheet.name.clone(),
            persona_description: persona.sheet.description.clone(),
            persona_personality: persona.sheet.personality.clone(),
            persona_background: persona.sheet.scenario.clone(),
            narrative_perspective: "third".to_string(),
            narrative_tense: "past".to_string(),
            option_count: DEFAULT_OPTION_COUNT.to_string(),
        }
    }
}
