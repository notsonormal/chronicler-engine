//! [DOC: chronicler_engine/docs/diataxis/reference/startup.md]
//! Game state initialization and arrival narration spawning

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::adapters::driven::storage::Storage;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::message::Message;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

use super::run::{PRESET_STORAGE_GAME_ID, find_latest_game_for_world, list_game_names_for_world};

fn with_settings<T>(settings: &Arc<RwLock<AppSettings>>, f: impl FnOnce(&AppSettings) -> T) -> T {
    let guard = settings.read().unwrap_or_else(|e| e.into_inner());
    f(&guard)
}

pub(crate) fn resolve_game_id(
    db_pool: &crate::adapters::driven::storage::db::DbPool,
    world: &WorldCard,
    persona_key: &str,
    persona_name: &str,
) -> crate::error::Result<u64> {
    match find_latest_game_for_world(db_pool, &world.key)? {
        Some((id, name)) => {
            tracing::info!("Loaded existing game '{name}' (id={id})");
            Ok(id)
        }
        None => {
            let existing_names = list_game_names_for_world(db_pool, &world.key)?;
            let name = crate::domain::model::utils::game_name::generate_game_name(
                &world.name,
                &existing_names,
            );
            let id =
                db_pool.insert_game(&world.name, &world.key, persona_key, persona_name, &name)?;
            tracing::info!("Created new game '{name}' (id={id}) with persona '{persona_key}'");
            Ok(id)
        }
    }
}

pub(crate) fn load_game_state(
    storage: &crate::adapters::driven::storage::Storage,
    world_arc: &Arc<WorldCard>,
    map_arc: &Arc<MapDef>,
    player_arc: &Arc<PersonaCard>,
    npcs_map: &HashMap<String, NpcCard>,
) -> crate::error::Result<GameState> {
    match storage.load_latest_snapshot() {
        Ok(Some(snap)) => {
            let mut new_state = GameState::from_snapshot(&snap);
            if let Ok(msgs) = storage.load_messages_with_swipes() {
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
            let snapshot_id = storage.save_snapshot(&initial_snapshot)?;

            if let Some(msg) = new_state.narrative.history.last_mut() {
                try_persist_initial_message(msg, storage, snapshot_id)?;
            }
            Ok(new_state)
        }
    }
}

fn try_persist_initial_message(
    msg: &mut Message,
    storage: &Storage,
    snapshot_id: u64,
) -> Result<(), EngineError> {
    if !msg.is_unpersisted() {
        return Ok(());
    }
    msg.set_snapshot_id(Some(snapshot_id));
    if let Some(swipe) = msg.swipes.first_mut() {
        swipe.snapshot_id = Some(snapshot_id);
    }
    let id = storage.insert_message(&*msg)?;
    if let Some(swipe) = msg.swipes.first() {
        storage.insert_swipe(id, swipe, 0)?;
    }
    msg.id = id;
    Ok(())
}

pub struct ArrivalSpawnRequest {
    pub world: Arc<WorldCard>,
    pub room_id: String,
    pub nearby_npcs: Vec<NpcCard>,
    pub all_npcs: Vec<NpcCard>,
}

pub fn spawn_arrival_task_if_needed(
    runtime: &tokio::runtime::Runtime,
    settings: &Arc<RwLock<AppSettings>>,
    app: &Arc<crate::application::application_service::DefaultApplicationService>,
    _storage: &Arc<crate::adapters::driven::storage::Storage>,
    db_pool: &crate::adapters::driven::storage::db::DbPool,
    request: ArrivalSpawnRequest,
) {
    let ArrivalSpawnRequest {
        world,
        room_id,
        nearby_npcs,
        all_npcs,
    } = request;

    let has_scenario = world.default_scenario().is_some_and(|s| !s.text.is_empty());

    if has_scenario {
        return;
    }

    let preset_storage = crate::adapters::driven::storage::Storage::new_sqlite(
        db_pool.clone(),
        PRESET_STORAGE_GAME_ID,
    );
    let (arrival_preset, response_length, max_context_tokens, max_tokens) =
        with_settings(settings, |guard| {
            let preset_id = &guard.active_system_prompt_preset_id;
            let preset = preset_storage.get_preset(preset_id).ok().flatten();
            let conn = guard.narration_connection();
            let max_context_tokens = conn.resolve_max_context_tokens();
            let max_tokens = conn.max_tokens;
            let response_length = guard.response_length.clone();
            (preset, response_length, max_context_tokens, max_tokens)
        });

    let recorder = Arc::clone(&app.game_service().llm_recorder);

    let task_ctx = crate::application::arrival_service::ArrivalTaskContext {
        app: Arc::clone(app),
        room_id,
        arrival_preset,
        response_length,
        max_context_tokens,
        max_tokens,
        nearby_npcs,
        all_npcs,
        recorder,
    };

    runtime.spawn_blocking(move || {
        let _ = task_ctx.run();
    });
}
