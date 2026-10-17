//! [DOC: docs/diataxis/reference/startup.md]
//! Game state initialization and game resolution

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::game::GameId;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

use super::run::{find_latest_game_for_world, list_game_names_for_world};

pub(crate) fn resolve_game_id(
    db_pool: &crate::adapters::driven::storage::db::DbPool,
    world: &WorldCard,
    persona_key: &str,
    persona_name: &str,
    settings: &AppSettings,
) -> crate::error::Result<GameId> {
    match find_latest_game_for_world(db_pool, &world.key)? {
        Some((id, name)) => {
            tracing::info!("Loaded existing game '{name}' (id={id})");
            Ok(GameId(id))
        }
        None => {
            let existing_names = list_game_names_for_world(db_pool, &world.key)?;
            let name = crate::domain::model::utils::game_name::generate_game_name(
                &world.name,
                &existing_names,
            );
            let bundle = settings
                .mode_preset_registry
                .bundle_for(world.narrator_mode);
            let request = crate::domain::model::game::NewGame {
                world_name: world.name.clone(),
                world_key: world.key.clone(),
                persona_key: persona_key.to_string(),
                persona_name: persona_name.to_string(),
                name: name.clone(),
                narrator_mode: world.narrator_mode,
                narrative_perspective: world.narrative_perspective,
                narrative_tense: world.narrative_tense,
                system_prompt_preset_id: bundle.system_prompt_preset_id.clone(),
                quantifier_prompt_preset_id: bundle.quantifier_prompt_preset_id.clone(),
                impersonate_prompt_preset_id: bundle.impersonate_prompt_preset_id.clone(),
                options_prompt_preset_id: settings.active_options_prompt_preset_id.clone(),
                options_always_on: world.options_always_on,
            };
            let id = db_pool.insert_game_from_request(&request)?;
            tracing::info!("Created new game '{name}' (id={id}) with persona '{persona_key}'");
            Ok(GameId(id))
        }
    }
}

pub(crate) fn load_game_state(
    storage: &crate::adapters::driven::storage::Storage,
    game_id: GameId,
    world_arc: &Arc<WorldCard>,
    map_arc: &Arc<MapDef>,
    player_arc: &Arc<PersonaCard>,
    npcs_map: &HashMap<String, NpcCard>,
) -> crate::error::Result<GameState> {
    match storage.load_latest_snapshot(game_id) {
        Ok(Some(snap)) => {
            let mut new_state = GameState::from_snapshot(&snap);
            if let Ok(msgs) = storage.load_messages_with_swipes(game_id) {
                new_state.narrative.history.replace(msgs);
            }
            Ok(new_state)
        }
        _ => {
            let starting_room_id = world_arc.starting_room_id();
            let mut new_state = GameState::new(starting_room_id);
            new_state.inject_scenario_logs(world_arc, player_arc, map_arc);
            if let Some(scenario) = world_arc.default_scenario() {
                new_state.init_scenario_npcs(scenario, npcs_map);
            }
            let initial_snapshot = GameStateSnapshot::from_game_state(&new_state);
            let snapshot_id = storage.save_snapshot(game_id, &initial_snapshot)?;

            if let Some(msg) = new_state.narrative.history.last_mut() {
                try_persist_initial_message(msg, storage, game_id, snapshot_id)?;
            }
            Ok(new_state)
        }
    }
}

fn try_persist_initial_message(
    msg: &mut Message,
    storage: &Storage,
    game_id: GameId,
    snapshot_id: u64,
) -> Result<(), EngineError> {
    if !msg.is_unpersisted() {
        return Ok(());
    }
    msg.set_snapshot_id(Some(snapshot_id));
    if let Some(swipe) = msg.swipes.first_mut() {
        swipe.snapshot_id = Some(snapshot_id);
    }
    let id = storage.insert_message(game_id, &*msg)?;
    if let Some(swipe) = msg.swipes.first() {
        storage.insert_swipe(id, swipe, 0)?;
    }
    msg.id = id;
    Ok(())
}
