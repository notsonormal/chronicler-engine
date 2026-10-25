//! [DOC: docs/diataxis/reference/game_flow.md]
//! The Characters present in one Swipe state

use serde::{Deserialize, Serialize};
use crate::domain::model::character::CharacterCard;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneState {
    /// The stored key is `npcs_in_area`.
    #[serde(rename = "npcs_in_area")]
    pub characters_present: Vec<CharacterCard>,
}
