//! [DOC: docs/system/startup.md]
//! Main entry point and runtime execution

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio_util::sync::CancellationToken;

use crate::application::context::{self, GameServiceContext};
use crate::cli::{Args, list_available_worlds, resolve_engine_data_path};
use crate::model::character::{NpcCard, PlayerCard};
use crate::model::game::generate_game_name;
use crate::model::map::MapDef;
use crate::model::prompt_preset::PromptPreset;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::model::template::TemplateVars;
use crate::model::world::WorldCard;
use crate::narrative::prompt::{PromptAssembler, PromptContext};
use crate::server::ServerConfig;

use super::inject_scenario_logs;

const PRESET_STORAGE_GAME_ID: u64 = 1;

fn with_settings<T>(settings: &Arc<RwLock<AppSettings>>, f: impl FnOnce(&AppSettings) -> T) -> T {
    let guard = settings.read().unwrap_or_else(|e| e.into_inner());
    f(&guard)
}

struct ArrivalTaskContext {
    storage: Arc<crate::storage::Storage>,
    world: Arc<WorldCard>,
    map: Arc<MapDef>,
    player: Arc<PlayerCard>,
    npcs: Arc<HashMap<String, NpcCard>>,
    room_id: String,
    arrival_preset: Option<PromptPreset>,
    response_length: String,
    max_context_tokens: u32,
    max_tokens: Option<u32>,
    nearby_npcs: Vec<NpcCard>,
    all_npcs: Vec<NpcCard>,
    db_pool: crate::storage::db::DbPool,
}

impl ArrivalTaskContext {
    fn run(self) {
        let preset_storage = Arc::new(crate::storage::Storage::new_sqlite(
            self.db_pool.clone(),
            PRESET_STORAGE_GAME_ID,
        ));

        let mut state = match context::load_expecting_valid_state(&GameServiceContext {
            storage: Arc::clone(&self.storage),
            world: Arc::clone(&self.world),
            map: Arc::clone(&self.map),
            player: Arc::clone(&self.player),
            npcs: Arc::clone(&self.npcs),
            cancel_token: CancellationToken::new(),
            is_generating: std::sync::atomic::AtomicBool::new(false).into(),
            settings: Arc::new(RwLock::new(AppSettings::default())),
            preset_storage,
        }) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to load snapshot in spawn ({e}), starting fresh");
                GameState::new(
                    Arc::clone(&self.world),
                    Arc::clone(&self.map),
                    Arc::clone(&self.player),
                    (*self.npcs).values().cloned().collect(),
                    self.world.starting_room_id.clone(),
                )
            }
        };

        if let Ok(msgs) = context::load_messages_with_swipes(&self.storage) {
            state.narrative.history.replace(msgs);
        }
        state.narrative.input_buffer.status = crate::model::state::GenerationStatus::Generating;

        let room = self
            .map
            .overworld
            .regions
            .iter()
            .flat_map(|r| r.rooms.iter())
            .find(|r| r.id == self.room_id);

        if let Some(room) = room {
            let backend = crate::narrative::llm::get_llm_backend_for(
                &AppSettings::default().narration_connection(),
                Some(Arc::clone(&self.storage)),
            );

            let context = PromptContext {
                world: &self.world,
                room,
                all_npcs: &self.all_npcs,
                npcs_in_area: &self.nearby_npcs,
                player: &self.player,
                user_message: "",
                history: &Vec::new(),
                template_vars: TemplateVars::new(&self.player.sheet.name),
            };

            let narration = if let Some(ref preset) = self.arrival_preset {
                let mut assembler =
                    crate::narrative::prompt::LayeredPromptAssembler::new(self.max_context_tokens);
                if let Some(max) = self.max_tokens {
                    assembler = assembler.with_max_tokens(max);
                }
                match assembler.assemble(
                    &context,
                    preset,
                    &self.world.global_rules,
                    Some(&self.response_length),
                ) {
                    Ok(assembled) => backend.complete(
                        crate::narrative::llm::backend::AGENT_NARRATOR,
                        &assembled.system_prompt,
                        &assembled.user_prompt,
                        Some(assembled.max_tokens),
                    ),
                    Err(e) => Err(e),
                }
            } else {
                Err(crate::error::EngineError::Config(
                    "No active preset found for arrival narration".into(),
                ))
            };

            match narration {
                Ok(result) => {
                    state.add_message(
                        result.text,
                        None,
                        crate::model::state::MessageType::Narration,
                    );
                    state.narrative.input_buffer.status =
                        crate::model::state::GenerationStatus::Idle;
                }
                Err(e) => {
                    state.narrative.input_buffer.status =
                        crate::model::state::GenerationStatus::Error(format!("LLM Error: {e}"));
                }
            }

            if let Err(e) = self.storage.save_snapshot(
                &crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state),
            ) {
                tracing::error!("Failed to save arrival snapshot: {e}");
            }
        }
    }
}

pub fn run(args: Args) -> crate::error::Result<()> {
    if args.list_worlds {
        list_available_worlds()?;
        return Ok(());
    }

    let data_dir = resolve_engine_data_path();
    let db_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::PathBuf::from))
        .unwrap_or_else(|| data_dir.clone());
    let db_path = db_dir.join(format!("chronicler_{}.db", args.port));
    let db_pool = crate::storage::db::DbPool::new(db_path.to_str().unwrap_or("chronicler.db"))?;

    if let Err(e) = ensure_presets(&db_pool, &data_dir) {
        tracing::warn!("Failed to seed prompt presets: {e}");
    }

    let seed_storage = crate::storage::Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);
    if let Err(e) = super::load::seed_game_data(&seed_storage, &data_dir) {
        tracing::warn!("Failed to seed game data: {e}");
    }

    let lookup_storage =
        crate::storage::Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);
    let world_with_map = lookup_storage
        .get_world(&args.world)?
        .ok_or_else(|| crate::error::EngineError::WorldNotFound(args.world.clone()))?;
    let world_id = world_with_map.world_id;
    let world_card = world_with_map.world_card;
    let map = world_with_map.map;

    let player_key = world_card.player_key.clone();
    let player = lookup_storage.get_persona(&player_key)?.ok_or_else(|| {
        crate::error::EngineError::Config(format!("Persona '{player_key}' not found"))
    })?;

    let npcs = lookup_storage.list_characters(world_id)?;

    let world_arc = Arc::new(world_card.clone());
    let map_arc = Arc::new(map);
    let player_arc = Arc::new(player.clone());
    let npcs_map: HashMap<_, _> = npcs.into_iter().map(|n| (n.id.clone(), n)).collect();

    let active_game_id = match find_latest_game_for_world(&db_pool, &world_card.name)? {
        Some((id, name)) => {
            tracing::info!("Loaded existing game '{name}' (id={id})");
            id
        }
        None => {
            let existing_names = list_game_names_for_world(&db_pool, &world_card.name)?;
            let name = generate_game_name(&world_card.name, &existing_names);
            let conn = db_pool.conn();
            let now = chrono::Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO games (world_name, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
                rusqlite::params![&world_card.name, &name, &now],
            )
            .map_err(|e| crate::error::EngineError::Config(format!("Failed to create game: {e}")))?;
            let id = conn.last_insert_rowid() as u64;
            tracing::info!("Created new game '{name}' (id={id})");
            id
        }
    };

    let storage = Arc::new(crate::storage::Storage::new_sqlite(
        db_pool.clone(),
        active_game_id,
    ));

    let state = match storage.load_latest_snapshot() {
        Ok(Some(snap)) => {
            let mut new_state = GameState::from_snapshot(
                &snap,
                Arc::clone(&world_arc),
                Arc::clone(&map_arc),
                Arc::clone(&player_arc),
                npcs_map.clone(),
            );
            if let Ok(msgs) = context::load_messages_with_swipes(&storage) {
                new_state.narrative.history.replace(msgs);
            }
            new_state
        }
        _ => {
            let mut new_state = GameState::new(
                Arc::clone(&world_arc),
                Arc::clone(&map_arc),
                Arc::clone(&player_arc),
                npcs_map.values().cloned().collect(),
                world_arc.starting_room_id.clone(),
            );
            inject_scenario_logs(&mut new_state, &world_card, &player);
            if let Some(scenario) = world_card.default_scenario() {
                new_state.init_scenario_npcs(scenario);
            }
            let initial_snapshot =
                crate::model::state_snapshot::GameStateSnapshot::from_game_state(&new_state);
            let snapshot_id = storage.save_snapshot(&initial_snapshot)?;
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
            new_state
        }
    };

    let nearby_npcs: Vec<NpcCard> = state.scene.npcs_in_area.clone();
    let all_npcs: Vec<NpcCard> = state.npcs.values().cloned().collect();
    let room_id = state.movement.current_room_id.clone();
    let npcs_arc = Arc::new(state.npcs.clone());

    let settings = if let Some(ref path) = args.settings_path {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::error::EngineError::Config(format!(
                "Failed to read settings file {}: {e}",
                path.display()
            ))
        })?;
        let imported: AppSettings = serde_json::from_str(&content).map_err(|e| {
            crate::error::EngineError::Config(format!(
                "Failed to parse settings file {}: {e}",
                path.display()
            ))
        })?;
        storage.save_settings(&imported).map_err(|e| {
            crate::error::EngineError::Config(format!("Failed to save imported settings: {e}"))
        })?;
        tracing::info!("Imported settings from {}", path.display());
        imported
    } else {
        crate::settings::load_settings(&storage).unwrap_or_else(|_| AppSettings::default())
    };
    let settings = Arc::new(RwLock::new(settings));

    let config = ServerConfig { port: args.port };

    let runtime = tokio::runtime::Runtime::new().map_err(|e| {
        crate::error::EngineError::Io(format!("runtime_new {}: {e}", "tokio_runtime"))
    })?;

    let has_scenario = world_arc
        .default_scenario()
        .is_some_and(|s| !s.text.is_empty());

    if !has_scenario {
        let preset_storage =
            crate::storage::Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);
        let (arrival_preset, response_length, max_context_tokens, max_tokens) =
            with_settings(&settings, |guard| {
                let preset_id = &guard.active_system_prompt_preset_id;
                let preset = preset_storage.get_preset(preset_id).ok().flatten();
                let conn = guard.narration_connection();
                let max_context_tokens = conn.resolve_max_context_tokens();
                let max_tokens = conn.max_tokens;
                let response_length = guard.response_length.clone();
                (preset, response_length, max_context_tokens, max_tokens)
            });

        let task_ctx = ArrivalTaskContext {
            storage: Arc::clone(&storage),
            world: Arc::clone(&world_arc),
            map: Arc::clone(&map_arc),
            player: Arc::clone(&player_arc),
            npcs: Arc::clone(&npcs_arc),
            room_id,
            arrival_preset,
            response_length,
            max_context_tokens,
            max_tokens,
            nearby_npcs,
            all_npcs,
            db_pool: db_pool.clone(),
        };

        runtime.spawn_blocking(move || {
            task_ctx.run();
        });
    }

    let preset_storage = crate::storage::Storage::new_sqlite(db_pool, PRESET_STORAGE_GAME_ID);

    let resources = crate::server::ServerResources {
        world: world_arc,
        map: map_arc,
        player: player_arc,
        npcs: npcs_arc,
        storage,
        preset_storage: Arc::new(preset_storage),
        settings,
    };

    runtime.block_on(crate::server::run_server_with_config(resources, config))?;

    Ok(())
}

pub(crate) fn find_latest_game_for_world(
    db_pool: &crate::storage::db::DbPool,
    world_name: &str,
) -> Result<Option<(u64, String)>, crate::error::EngineError> {
    let conn = db_pool.conn();
    let mut stmt = conn
        .prepare(
            "SELECT g.id, g.name
             FROM games g
             LEFT JOIN (
                 SELECT game_id, MAX(timestamp) as last_message
                 FROM messages
                 GROUP BY game_id
             ) m ON g.id = m.game_id
             WHERE g.world_name = ?1
             ORDER BY COALESCE(m.last_message, g.updated_at) DESC
             LIMIT 1",
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to prepare query: {e}")))?;
    let result = stmt.query_row(rusqlite::params![world_name], |row| {
        Ok((row.get::<_, i64>(0)? as u64, row.get::<_, String>(1)?))
    });
    match result {
        Ok(pair) => Ok(Some(pair)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(crate::error::EngineError::Config(format!(
            "Failed to query games: {e}"
        ))),
    }
}

pub(crate) fn list_game_names_for_world(
    db_pool: &crate::storage::db::DbPool,
    world_name: &str,
) -> Result<Vec<String>, crate::error::EngineError> {
    let conn = db_pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM games WHERE world_name = ?1")
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to prepare query: {e}")))?;
    let rows = stmt
        .query_map(rusqlite::params![world_name], |row| row.get::<_, String>(0))
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to query games: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to read game names: {e}")))
}

pub(crate) fn ensure_presets(
    db_pool: &crate::storage::db::DbPool,
    data_dir: &std::path::Path,
) -> crate::error::Result<()> {
    use crate::model::prompt_preset::{PresetType, PromptPreset};

    let storage = crate::storage::Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID);

    for preset_type in [PresetType::System, PresetType::Quantifier] {
        let dir = data_dir.join("prompt_presets").join(preset_type.as_str());
        if !dir.exists() {
            tracing::info!("Prompt preset seed directory not found: {}", dir.display());
            continue;
        }

        let existing_ids: std::collections::HashSet<String> = storage
            .list_presets(preset_type)?
            .into_iter()
            .map(|p| p.id)
            .collect();

        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let content = std::fs::read_to_string(&path)?;
            let seed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                crate::error::EngineError::Parse(format!(
                    "Invalid preset seed {}: {e}",
                    path.display()
                ))
            })?;

            let id = seed["id"].as_str().unwrap_or("default").to_string();
            let preset = PromptPreset {
                id: id.clone(),
                name: seed["name"].as_str().unwrap_or("Default").to_string(),
                role: seed["role"].as_str().map(|s| s.to_string()),
                instructions: seed["instructions"].as_str().map(|s| s.to_string()),
                writing_style: seed["writing_style"].as_str().map(|s| s.to_string()),
                output_format: seed["output_format"].as_str().map(|s| s.to_string()),
                is_default: true,
                preset_type,
            };

            if existing_ids.contains(&id) {
                if let Ok(Some(existing)) = storage.get_preset(&id) {
                    let has_content = existing.role.is_some()
                        || existing.instructions.is_some()
                        || existing.writing_style.is_some()
                        || existing.output_format.is_some();
                    if !has_content {
                        storage.save_preset(&preset)?;
                        tracing::info!("Updated {} prompt preset: {}", preset_type.as_str(), id);
                    }
                }
                continue;
            }
            storage.save_preset(&preset)?;
            tracing::info!("Seeded {} prompt preset: {}", preset_type.as_str(), id);
        }
    }

    Ok(())
}
