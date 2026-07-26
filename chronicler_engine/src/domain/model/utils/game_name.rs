//! [DOC: chronicler_engine/docs/diataxis/reference/game_flow.md]
//! Game name generation with date-based disambiguation.

use chrono::Utc;

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
