//! [DOC: docs/system/worlds.md]
//! Worlds fragment renderers

use askama::Template;

use crate::model::character::PlayerCard;
use crate::model::map::MapDef;
use crate::model::scenario::StartingScenario;
use crate::model::world::WorldCard;

use super::template::{WorldFormTemplate, WorldRowView, WorldsPanelTemplate};

/// Render the worlds panel listing all worlds with their game counts.
pub fn render_worlds_panel(
    worlds: &[WorldCard],
    games_per_world: &std::collections::HashMap<String, usize>,
) -> String {
    let rows: Vec<WorldRowView> = worlds
        .iter()
        .map(|w| {
            let game_count = games_per_world.get(&w.key).copied().unwrap_or(0);
            WorldRowView {
                key: w.key.clone(),
                name: w.name.clone(),
                description: w.description.clone(),
                game_count,
            }
        })
        .collect();
    WorldsPanelTemplate { worlds: rows }
        .render()
        .unwrap_or_default()
}

/// Render the world edit/create form.
pub fn render_world_edit_form(
    world: Option<&WorldCard>,
    map: Option<&MapDef>,
    scenarios: &[StartingScenario],
    personas: &[PlayerCard],
) -> String {
    WorldFormTemplate::from_world_data(world, map, scenarios, personas)
        .render()
        .unwrap_or_default()
}
