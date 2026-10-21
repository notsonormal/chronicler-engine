//! [DOC: docs/diataxis/reference/game_flow.md]
//! Current scene NPCs

use serde::{Deserialize, Serialize};
use crate::domain::model::character::NpcCard;

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneState {
    pub npcs_in_area: Vec<NpcCard>,
}
