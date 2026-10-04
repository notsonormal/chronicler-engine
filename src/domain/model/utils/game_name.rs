//! [DOC: docs/diataxis/reference/game_flow.md]
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

/// `Redmist Estate_2026-09-29_1` becomes `Redmist Estate — 29 Sep 2026 (1)`.
///
/// A value that does not match the `{world}_{YYYY-MM-DD}_{N}` shape is
/// returned unchanged, so a hand-written stable name passes through untouched.
pub fn default_display_name(stable_name: &str) -> String {
    let Some((stem, ordinal)) = stable_name.rsplit_once('_') else {
        return stable_name.to_string();
    };
    let Ok(ordinal) = ordinal.parse::<u32>() else {
        return stable_name.to_string();
    };
    let Some((world, date)) = stem.rsplit_once('_') else {
        return stable_name.to_string();
    };
    let Ok(date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") else {
        return stable_name.to_string();
    };
    format!("{} — {} ({})", world, date.format("%d %b %Y"), ordinal)
}
