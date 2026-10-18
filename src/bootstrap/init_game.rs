//! [DOC: docs/diataxis/reference/startup.md]
//! Game state initialization and game resolution

use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::driven::storage::Storage;
use crate::application::errors::ApplicationError;
use crate::application::games::catalogue::GameCatalogue;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::game::GameId;
use crate::domain::model::map::MapDef;
use crate::domain::model::message::Message;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

pub(crate) fn resolve_game_id(
    catalogue: &GameCatalogue,
    world_key: &str,
    persona_key: &str,
) -> Result<GameId, ApplicationError> {
    match catalogue.newest_game_of_world(world_key)? {
        Some(game_id) => {
            tracing::info!("Loaded existing game (id={game_id})");
            Ok(game_id)
        }
        None => {
            let id = catalogue.create_game(world_key, persona_key)?;
            tracing::info!("Created new game (id={id}) with persona '{persona_key}'");
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
