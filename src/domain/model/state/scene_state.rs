//! [DOC: docs/diataxis/reference/game_flow.md]
//! Current scene NPCs and quantifier confidence

use serde::{Deserialize, Serialize};
use crate::domain::model::character::NpcCard;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneState {
    pub npcs_in_area: Vec<NpcCard>,
    #[serde(default)]
    pub quantifier_confidence: Option<String>,
}
