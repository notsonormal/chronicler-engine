//! [DOC: docs/system/game_flow.md]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Game {
    pub id: u64,
    pub name: String,
    pub world_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Generate a unique game name following the pattern `{WorldName}_{Date}_N`.
/// Finds the highest existing `N` for today's prefix and returns `N+1`.
pub fn generate_game_name(world_name: &str, existing_names: &[String]) -> String {
    let date = Utc::now().format("%Y-%m-%d");
    let base = format!("{world_name}_{date}");
    let mut max_n = 0;
    for name in existing_names {
        if let Some(stem) = name.strip_prefix(&base) {
            if let Ok(n) = stem.trim_start_matches('_').parse::<u32>() {
                if n > max_n {
                    max_n = n;
                }
            }
        }
    }
    format!("{base}_{}", max_n + 1)
}
