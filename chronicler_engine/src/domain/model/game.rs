//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Game state and session management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Game {
    pub id: u64,
    pub world_name: String,
    pub world_key: String,
    pub persona_key: String,
    pub persona_name: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub fn generate_game_name(world_name: &str, existing_names: &[String]) -> String {
    let date = Utc::now().format("%Y-%m-%d");
    let base = format!("{world_name}_{date}");
    let max_n = existing_names
        .iter()
        .filter_map(|name| name.strip_prefix(&base))
        .filter_map(|stem| stem.trim_start_matches('_').parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    format!("{base}_{}", max_n + 1)
}
