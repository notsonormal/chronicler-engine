//! [DOC: docs/system/startup.md]
//! Game state initialization and arrival narration spawning

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::application::context;
use crate::application::scenario::inject_scenario_logs;
use crate::domain::model::character::{NpcCard, PlayerCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::game_state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;

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
            let name = crate::domain::model::game::generate_game_name(&world.name, &existing_names);
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
    player_arc: &Arc<PlayerCard>,
    npcs_map: &HashMap<String, NpcCard>,
) -> crate::error::Result<GameState> {
    match storage.load_latest_snapshot() {
        Ok(Some(snap)) => {
            let mut new_state = GameState::from_snapshot(
                &snap,
                Arc::clone(world_arc),
                Arc::clone(map_arc),
                Arc::clone(player_arc),
                npcs_map.clone(),
            );
            if let Ok(msgs) = context::load_messages_with_swipes(storage) {
                new_state.narrative.history.replace(msgs);
            }
            Ok(new_state)
        }
        _ => {
            let starting_room_id = world_arc.starting_room_id();
            let mut new_state = GameState::new(
                Arc::clone(world_arc),
                Arc::clone(map_arc),
                Arc::clone(player_arc),
                npcs_map.values().cloned().collect(),
                starting_room_id,
            );
            inject_scenario_logs(&mut new_state, world_arc, player_arc);
            if let Some(scenario) = world_arc.default_scenario() {
                new_state.init_scenario_npcs(scenario);
            }
            let initial_snapshot = GameStateSnapshot::from_game_state(&new_state);
            let snapshot_id = storage.save_snapshot(&initial_snapshot)?;
            // Snapshot carries history for debugging/audit; `messages` table is source of truth on load
            // per `context::load_messages_with_swipes` replace pattern (see ADR-023).
            if let Some(msg) = new_state.narrative.history.last_mut() {
                if msg.is_unpersisted() {
                    msg.set_snapshot_id(Some(snapshot_id));
                    if let Some(swipe) = msg.swipes.first_mut() {
                        swipe.snapshot_id = Some(snapshot_id);
                    }
                    let id = storage.insert_message(&*msg)?;
                    if let Some(swipe) = msg.swipes.first() {
                        storage.insert_swipe(id, swipe, 0)?;
                    }
                    msg.id = id;
                }
            }
            Ok(new_state)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_arrival_task_if_needed(
    runtime: &tokio::runtime::Runtime,
    settings: &Arc<RwLock<AppSettings>>,
    storage: &Arc<crate::adapters::driven::storage::Storage>,
    world: &Arc<WorldCard>,
    map: &Arc<MapDef>,
    player: &Arc<PlayerCard>,
    npcs: &Arc<HashMap<String, NpcCard>>,
    room_id: &str,
    nearby_npcs: Vec<NpcCard>,
    all_npcs: Vec<NpcCard>,
    db_pool: &crate::adapters::driven::storage::db::DbPool,
) {
    let has_scenario = world.default_scenario().is_some_and(|s| !s.text.is_empty());

    if has_scenario {
        return;
    }

    let preset_storage = crate::adapters::driven::storage::Storage::new_sqlite(
        db_pool.clone(),
        PRESET_STORAGE_GAME_ID,
    );
    let (arrival_preset, response_length, max_context_tokens, max_tokens, connection) =
        with_settings(settings, |guard| {
            let preset_id = &guard.active_system_prompt_preset_id;
            let preset = preset_storage.get_preset(preset_id).ok().flatten();
            let conn = guard.narration_connection();
            let max_context_tokens = conn.resolve_max_context_tokens();
            let max_tokens = conn.max_tokens;
            let response_length = guard.response_length.clone();
            (
                preset,
                response_length,
                max_context_tokens,
                max_tokens,
                conn,
            )
        });

    let preset_storage_arc = Arc::new(preset_storage);
    let game_ctx = crate::application::context::OpContext {
        storage: Arc::clone(storage),
        world_snapshot: crate::application::context::WorldSnapshot {
            world: Arc::clone(world),
            map: Arc::clone(map),
            player: Arc::clone(player),
            npcs: Arc::clone(npcs),
        },
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::clone(settings),
        preset_storage: Arc::clone(&preset_storage_arc),
    };

    let recorder =
        match crate::bootstrap::llm_factory::get_llm_recorder_for(&connection, Arc::clone(storage))
        {
            Ok(recorder) => recorder,
            Err(e) => {
                tracing::error!("Failed to create LLM recorder for arrival task: {e}");
                return;
            }
        };

    let task_ctx = crate::application::arrival_service::ArrivalTaskContext {
        ctx: game_ctx,
        room_id: room_id.to_string(),
        arrival_preset,
        response_length,
        max_context_tokens,
        max_tokens,
        nearby_npcs,
        all_npcs,
        recorder,
    };

    runtime.spawn_blocking(move || {
        task_ctx.run();
    });
}
