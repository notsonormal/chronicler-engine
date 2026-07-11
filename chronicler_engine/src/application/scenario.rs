//! [DOC: docs/system/startup.md]
//! Scenario log injection at game initialization

use crate::domain::model::character::PersonaCard;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::template::{render_template, TemplateVars};
use crate::domain::model::world::WorldCard;

pub fn inject_scenario_logs(
    state: &mut GameState,
    world: &WorldCard,
    player: &PersonaCard,
    map: &crate::domain::model::map::MapDef,
) {
    let Some(scenario) = world.default_scenario() else {
        return;
    };
    if scenario.text.is_empty() {
        return;
    }

    let room_name =
        crate::domain::engine::logic::find_room_in_world_map(map, &scenario.starting_room_id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| scenario.starting_room_id.clone());

    state.narrative.pending_location = Some(room_name);
    let text = render_template(&scenario.text, &TemplateVars::new(&player.sheet.name));
    state.add_message(text, None, MessageType::Narration);
}
