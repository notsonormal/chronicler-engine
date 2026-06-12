//! [DOC: docs/system/startup.md]
//! Bootstrap game state from saved snapshots
use std::sync::Arc;

use crate::application::context::GameServiceContext;
use crate::model::state::GameState;
use crate::model::template::{render_template, TemplateVars};

pub fn build_fresh_initial_state(ctx: &GameServiceContext) -> GameState {
    let mut initial_state = GameState::new(
        Arc::clone(&ctx.world),
        Arc::clone(&ctx.map),
        Arc::clone(&ctx.player),
        (*ctx.npcs).values().cloned().collect(),
        ctx.world.starting_room_id.clone(),
    );

    if let Some(scenario) = ctx.world.default_scenario() {
        let room_name = ctx
            .map
            .get_room_by_id(&ctx.world.starting_room_id)
            .map(|r| r.name.clone())
            .unwrap_or_else(|| ctx.world.starting_room_id.clone());

        initial_state.narrative.pending_location = Some(room_name);

        let text = render_template(&scenario.text, &TemplateVars::new(&ctx.player.sheet.name));
        if !text.is_empty() {
            initial_state.add_message(text, None, crate::model::state::MessageType::Narration);
        }

        initial_state.init_scenario_npcs(scenario);
    }

    initial_state
}
