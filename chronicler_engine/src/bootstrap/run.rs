//! [DOC: chronicler_engine/docs/diataxis/reference/startup.md]
//! Main entry point and runtime execution

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::utils::cli::{Args, list_available_worlds, resolve_engine_data_path};
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::storage::db::DbPool;
use crate::adapters::driving::http::ServerConfig;
use crate::bootstrap::wiring::{WiredApp, build_app_graph};
use crate::utils::settings::load_settings;
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::map::MapDef;
use crate::domain::model::settings::AppSettings;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

pub(crate) const PRESET_STORAGE_GAME_ID: u64 = 1;

struct PreparedData {
    db_pool: DbPool,
    storage: Arc<Storage>,
    preset_storage: Arc<Storage>,
    world_card: WorldCard,
    map: Arc<MapDef>,
    player: PersonaCard,
    npcs_map: HashMap<String, NpcCard>,
}

struct StateResources {
    runtime: tokio::runtime::Runtime,
    wired: WiredApp,
}

pub fn run(args: Args) -> crate::error::Result<()> {
    if args.list_worlds {
        list_available_worlds()?;
        return Ok(());
    }

    let data = prepare_data(&args)?;
    let config = ServerConfig {
        port: args.port,
        bind_attempts: None,
    };
    let state = prepare_state(&args, &data)?;
    start_server(state, config)?;
    Ok(())
}

fn prepare_data(args: &Args) -> crate::error::Result<PreparedData> {
    let data_dir = resolve_engine_data_path();
    let db_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(std::path::PathBuf::from))
        .unwrap_or_else(|| data_dir.clone());
    let db_path = db_dir.join(format!("chronicler_{}.db", args.port));
    let db_pool = DbPool::new(db_path.to_str().unwrap_or("chronicler.db"))?;

    if let Err(e) = ensure_presets(&db_pool, &data_dir) {
        tracing::warn!("Failed to seed prompt presets: {e}");
    }

    let preset_storage = Arc::new(Storage::new_sqlite(db_pool.clone(), PRESET_STORAGE_GAME_ID));
    if let Err(e) = preset_storage.seed_game_data(&data_dir) {
        tracing::warn!("Failed to seed game data: {e}");
    }

    let world_with_map = match preset_storage.get_world(&args.world)? {
        Some(w) => w,
        None => {
            let all_worlds = preset_storage.list_worlds()?;
            if all_worlds.is_empty() {
                return Err(EngineError::Config(
                    "No worlds available in database".to_string(),
                ));
            }
            tracing::warn!(
                "World '{}' not found, falling back to '{}'",
                args.world,
                all_worlds[0].key
            );
            preset_storage.require_world(&all_worlds[0].key)?
        }
    };

    let npcs: Vec<NpcCard> = preset_storage.list_characters(world_with_map.world_id)?;
    let npcs_map: HashMap<String, NpcCard> = npcs.into_iter().map(|n| (n.id.clone(), n)).collect();
    let world_arc = Arc::new(world_with_map.world_card.clone());
    let map_arc = Arc::new(world_with_map.map);

    let player = preset_storage.require_persona(&args.persona)?;

    let active_game_id =
        super::init_game::resolve_game_id(&db_pool, &world_arc, &args.persona, &player.sheet.name)?;
    let storage = Arc::new(Storage::new_sqlite(db_pool.clone(), active_game_id));

    Ok(PreparedData {
        db_pool,
        storage,
        preset_storage,
        world_card: world_with_map.world_card,
        map: map_arc,
        player,
        npcs_map,
    })
}

fn prepare_state(args: &Args, data: &PreparedData) -> crate::error::Result<StateResources> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| EngineError::Io(format!("runtime_new {}: {e}", "tokio_runtime")))?;

    let world_arc = Arc::new(data.world_card.clone());
    let player_arc = Arc::new(data.player.clone());

    let state = super::init_game::load_game_state(
        &data.storage,
        &world_arc,
        &data.map,
        &player_arc,
        &data.npcs_map,
    )?;

    let nearby_npcs: Vec<NpcCard> = state.scene.npcs_in_area.clone();
    let all_npcs: Vec<NpcCard> = data.npcs_map.values().cloned().collect();
    let room_id = state.movement.current_room_id.clone();

    let settings = if let Some(path) = &args.settings_path {
        let content = std::fs::read_to_string(path).map_err(|e| {
            EngineError::Config(format!(
                "Failed to read settings file {}: {e}",
                path.display()
            ))
        })?;
        let imported: AppSettings = serde_json::from_str(&content).map_err(|e| {
            EngineError::Config(format!(
                "Failed to parse settings file {}: {e}",
                path.display()
            ))
        })?;
        data.storage
            .save_settings(&imported)
            .map_err(|e| EngineError::Config(format!("Failed to save imported settings: {e}")))?;
        tracing::info!("Imported settings from {}", path.display());
        imported
    } else {
        load_settings(&data.storage).unwrap_or_else(|_| AppSettings::default())
    };
    let settings = Arc::new(RwLock::new(settings));
    let wired = build_app_graph(
        settings,
        Arc::clone(&data.storage),
        Arc::clone(&data.preset_storage),
    )?;

    super::init_game::spawn_arrival_task_if_needed(
        &runtime,
        &wired.settings,
        Arc::clone(&wired.message_service),
        Arc::new(wired.pipeline.clone()),
        &data.storage,
        &data.db_pool,
        super::init_game::ArrivalSpawnRequest {
            world: world_arc,
            room_id,
            nearby_npcs,
            all_npcs,
        },
    );

    Ok(StateResources { runtime, wired })
}

fn start_server(state: StateResources, config: ServerConfig) -> crate::error::Result<()> {
    let StateResources { runtime, wired } = state;
    let (_addr, server) = runtime.block_on(
        crate::adapters::driving::http::run_server_with_config(wired, config),
    )?;
    runtime
        .block_on(server)
        .map_err(|e| EngineError::Config(format!("Server stopped: {e}")))??;
    Ok(())
}

pub(crate) fn find_latest_game_for_world(
    db_pool: &crate::adapters::driven::storage::db::DbPool,
    world_key: &str,
) -> Result<Option<(u64, String)>, crate::error::EngineError> {
    let conn = db_pool.conn();
    let mut stmt = conn
        .prepare(
            "SELECT g.id, g.name
             FROM games g
             LEFT JOIN (
                 SELECT game_id, MAX(timestamp) AS last_message
                 FROM messages
                 GROUP BY game_id
             ) m ON g.id = m.game_id
             WHERE g.world_key = ?1
             ORDER BY COALESCE(m.last_message, g.updated_at) DESC
             LIMIT 1",
        )
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to prepare query: {e}")))?;
    let result = stmt.query_row(rusqlite::params![world_key], |row| {
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
    db_pool: &crate::adapters::driven::storage::db::DbPool,
    world_key: &str,
) -> Result<Vec<String>, crate::error::EngineError> {
    let conn = db_pool.conn();
    let mut stmt = conn
        .prepare("SELECT name FROM games WHERE world_key = ?1")
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to prepare query: {e}")))?;
    let rows = stmt
        .query_map(rusqlite::params![world_key], |row| row.get::<_, String>(0))
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to query games: {e}")))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|e| crate::error::EngineError::Config(format!("Failed to read game names: {e}")))
}

pub(crate) fn ensure_presets(
    db_pool: &crate::adapters::driven::storage::db::DbPool,
    data_dir: &std::path::Path,
) -> crate::error::Result<()> {
    use crate::domain::model::prompt_preset::PresetType;

    let storage = crate::adapters::driven::storage::Storage::new_sqlite(
        db_pool.clone(),
        PRESET_STORAGE_GAME_ID,
    );

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
            process_preset_file(&storage, &path, &existing_ids, preset_type)?;
        }
    }

    Ok(())
}

fn process_preset_file(
    storage: &Storage,
    path: &std::path::Path,
    existing_ids: &std::collections::HashSet<String>,
    preset_type: crate::domain::model::prompt_preset::PresetType,
) -> Result<(), crate::error::EngineError> {
    use crate::domain::model::prompt_preset::PromptPreset;

    if path.extension().and_then(|s| s.to_str()) != Some("json") {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)?;
    let seed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        crate::error::EngineError::Parse(format!("Invalid preset seed {}: {e}", path.display()))
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
        return Ok(());
    }
    storage.save_preset(&preset)?;
    tracing::info!("Seeded {} prompt preset: {}", preset_type.as_str(), id);
    Ok(())
}
