//! [DOC: docs/system/startup.md]
//! Scenario loading and validation
use crate::model::character::PlayerCard;
use crate::model::state::GameState;
use crate::model::template::{render_template, TemplateVars};
use crate::model::world::WorldManifest;

pub fn inject_scenario_logs(state: &mut GameState, manifest: &WorldManifest, player: &PlayerCard) {
    let Some(scenario) = manifest.default_scenario() else {
        return;
    };
    if scenario.text.is_empty() {
        return;
    }

    let room_name = crate::engine::logic::find_room_in_world_map(state, &manifest.starting_room_id)
        .map(|r| r.name.clone())
        .unwrap_or_else(|| manifest.starting_room_id.clone());

    state.narrative.pending_location = Some(room_name);
    let text = render_template(&scenario.text, &TemplateVars::new(&player.sheet.name));
    state.add_message(text, None, crate::model::state::MessageType::Narration);
}
