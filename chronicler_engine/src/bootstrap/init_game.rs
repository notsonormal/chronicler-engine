//! [DOC: docs/system/startup.md]
//! Game state initialization and arrival narration spawning

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::application::context;
use crate::domain::model::character::{NpcCard, PlayerCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::prompt_preset::PromptPreset;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::state::game_state::GameState;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::domain::model::state::message_types::MessageType;
use crate::domain::model::state_snapshot::GameStateSnapshot;
use crate::domain::model::world::WorldCard;
use crate::application::narrative_prompt::{NpcContext, make_prompt_context};

use super::run::{PRESET_STORAGE_GAME_ID, find_latest_game_for_world, list_game_names_for_world};
use super::inject_scenario_logs;

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

#[doc(hidden)]
pub struct ArrivalTaskContext {
    ctx: crate::application::context::GameServiceContext,
    room_id: String,
    arrival_preset: Option<PromptPreset>,
    response_length: String,
    max_context_tokens: u32,
    max_tokens: Option<u32>,
    nearby_npcs: Vec<NpcCard>,
    all_npcs: Vec<NpcCard>,
    connection: crate::domain::model::settings::Connection,
}

impl ArrivalTaskContext {
    #[doc(hidden)]
    #[allow(clippy::too_many_arguments)]
    pub fn new_for_test(
        ctx: crate::application::context::GameServiceContext,
        room_id: String,
        nearby_npcs: Vec<NpcCard>,
        all_npcs: Vec<NpcCard>,
        arrival_preset: Option<PromptPreset>,
        response_length: String,
        max_context_tokens: u32,
        max_tokens: Option<u32>,
        connection: crate::domain::model::settings::Connection,
    ) -> Self {
        Self {
            ctx,
            room_id,
            arrival_preset,
            response_length,
            max_context_tokens,
            max_tokens,
            nearby_npcs,
            all_npcs,
            connection,
        }
    }

    #[doc(hidden)]
    pub fn run_sync(self) {
        self.run();
    }

    fn run(self) {
        let mut state = match self.ctx.storage.load_latest_snapshot() {
            Ok(Some(snap)) => GameState::from_snapshot(
                &snap,
                Arc::clone(&self.ctx.world),
                Arc::clone(&self.ctx.map),
                Arc::clone(&self.ctx.player),
                (*self.ctx.npcs).clone(),
            ),
            _ => {
                tracing::warn!("No snapshot found in spawn, starting fresh");
                let starting_room_id = self.ctx.world.starting_room_id();
                let mut s = GameState::new(
                    Arc::clone(&self.ctx.world),
                    Arc::clone(&self.ctx.map),
                    Arc::clone(&self.ctx.player),
                    (*self.ctx.npcs).values().cloned().collect(),
                    starting_room_id,
                );
                inject_scenario_logs(&mut s, &self.ctx.world, &self.ctx.player);
                s
            }
        };
        if let Ok(msgs) = context::load_messages_with_swipes(&self.ctx.storage) {
            state.narrative.history.replace(msgs);
        }
        state.narrative.input_buffer.status = GenerationStatus::Generating;

        let room = match self
            .ctx
            .map
            .overworld
            .regions
            .iter()
            .flat_map(|r| r.rooms.iter())
            .find(|r| r.id == self.room_id)
        {
            Some(r) => r,
            None => return,
        };

        let backend = crate::application::ports::llm_provider::get_llm_backend_for(
            &self.connection,
            Some(Arc::clone(&self.ctx.storage)),
        );

        let prompt_context = make_prompt_context(
            &self.ctx.world,
            room,
            NpcContext {
                all_npcs: &self.all_npcs,
                npcs_in_area: &self.nearby_npcs,
            },
            &self.ctx.player,
            "",
            &[],
        );

        let narration = match self.arrival_preset.as_ref() {
            Some(preset) => {
                let mut assembler =
                    crate::application::narrative_prompt::LayeredPromptAssembler::new(
                        self.max_context_tokens,
                    );
                if let Some(max) = self.max_tokens {
                    assembler = assembler.with_max_tokens(max);
                }
                assembler
                    .assemble(
                        &prompt_context,
                        preset,
                        &self.ctx.world.global_rules,
                        Some(&self.response_length),
                    )
                    .and_then(|assembled| {
                        backend.complete(
                            crate::application::ports::llm_provider::AGENT_NARRATOR,
                            &assembled.system_prompt,
                            &assembled.user_prompt,
                            Some(assembled.max_tokens),
                        )
                    })
            }
            None => Err(crate::error::EngineError::Config(
                "No active preset found for arrival narration".into(),
            )),
        };

        match narration {
            Ok(result) => {
                state.add_message(result.text, None, MessageType::Narration);
                state.narrative.input_buffer.status = GenerationStatus::Idle;
            }
            Err(e) => {
                state.narrative.input_buffer.status =
                    GenerationStatus::Error(format!("LLM Error: {e}"));
            }
        }

        if let Err(e) =
            crate::application::context::save_message_and_snapshot(&self.ctx, &mut state)
        {
            tracing::error!("Failed to save arrival message and snapshot: {e}");
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
    let game_ctx = crate::application::context::GameServiceContext {
        storage: Arc::clone(storage),
        world: Arc::clone(world),
        map: Arc::clone(map),
        player: Arc::clone(player),
        npcs: Arc::clone(npcs),
        cancel_token: tokio_util::sync::CancellationToken::new(),
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::clone(settings),
        preset_storage: Arc::clone(&preset_storage_arc),
    };

    let task_ctx = ArrivalTaskContext {
        ctx: game_ctx,
        room_id: room_id.to_string(),
        arrival_preset,
        response_length,
        max_context_tokens,
        max_tokens,
        nearby_npcs,
        all_npcs,
        connection,
    };

    runtime.spawn_blocking(move || {
        task_ctx.run();
    });
}

/// Test-only API for integration tests.
/// DO NOT USE outside of tests.
#[doc(hidden)]
pub mod test_api {
    pub use super::ArrivalTaskContext;
}
