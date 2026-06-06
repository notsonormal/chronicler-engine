//! [DOC: docs/system/character_state.md]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Relationship {
    pub with: String,
    #[serde(rename = "static")]
    pub static_text: String,
    #[serde(default)]
    pub dynamic: String,
}

impl Relationship {
    /// Returns the dynamic text if set, otherwise falls back to static text.
    pub fn display_text(&self) -> &str {
        if self.dynamic.is_empty() {
            &self.static_text
        } else {
            &self.dynamic
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CharacterSheet {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    #[serde(default)]
    pub example_dialogue: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub profile_image: Option<String>,
    #[serde(default)]
    pub headshot_image: Option<String>,
}

impl CharacterSheet {
    pub fn preferred_image(&self) -> Option<&str> {
        self.headshot_image
            .as_deref()
            .or(self.profile_image.as_deref())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerCard {
    #[serde(flatten)]
    pub sheet: CharacterSheet,
    #[serde(default)]
    pub inventory: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NpcCard {
    pub id: String,
    #[serde(flatten)]
    pub sheet: CharacterSheet,
    #[serde(default)]
    pub inventory: Vec<String>,
    #[serde(default)]
    pub triggers: Vec<crate::model::trigger::Trigger>,
    #[serde(default)]
    pub relationships: Vec<Relationship>,
}
