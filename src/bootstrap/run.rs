//! [DOC: docs/diataxis/reference/startup.md]
//! Main entry point and runtime execution

use std::collections::HashMap;
use std::sync::Arc;

use crate::utils::cli::{Args, list_available_worlds, resolve_engine_data_path};
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::storage::db::DbPool;
use crate::adapters::driving::http::ServerConfig;
use crate::application::errors::ApplicationError;
use crate::application::games::active::ActiveGame;
use crate::application::games::catalogue::GameCatalogue;
use crate::application::message_service::MessageService;
use crate::bootstrap::load::{seed_game_data, seed_settings};
use crate::bootstrap::wiring::{WiredApp, build_app_graph};
use crate::domain::model::character::{NpcCard, PersonaCard};
use crate::domain::model::game::GameId;
use crate::domain::model::map::MapDef;
use crate::domain::model::world::WorldCard;
use crate::error::EngineError;

struct PreparedData {
    storage: Arc<Storage>,
    active_game_id: GameId,
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
        host: args.host,
        port: args.port,
    };
    let state = prepare_state(&data)?;
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
    let storage = Arc::new(Storage::new_sqlite(db_pool));

    if let Err(e) = ensure_presets(&storage, &data_dir) {
        tracing::warn!("Failed to seed prompt presets: {e}");
    }

    if let Err(e) = seed_game_data(&storage, &data_dir) {
        tracing::warn!("Failed to seed game data: {e}");
    }

    // Fatal on a malformed seed file. Runs before any reader of the settings
    // row, so the boot reads the seeded values.
    seed_settings(&storage, &data_dir)?;

    let world_with_map = match storage.get_world(&args.world)? {
        Some(w) => w,
        None => {
            let all_worlds = storage.list_worlds()?;
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
            storage.require_world(&all_worlds[0].key)?
        }
    };

    let npcs: Vec<NpcCard> = storage.list_characters(world_with_map.world_id)?;
    let npcs_map: HashMap<String, NpcCard> = npcs.into_iter().map(|n| (n.id.clone(), n)).collect();
    let world_arc = Arc::new(world_with_map.world_card.clone());
    let map_arc = Arc::new(world_with_map.map);

    let player = storage.require_persona(&args.persona)?;

    let active_game_id = resolve_boot_game(&storage, &world_arc.key, &args.persona)?;
    Ok(PreparedData {
        storage,
        active_game_id,
        world_card: world_with_map.world_card,
        map: map_arc,
        player,
        npcs_map,
    })
}

/// Boot resolves its Game before the wired graph exists, so this builds its own
/// catalogue; the Game is the newest of the World, or a fresh one through
/// `GameCatalogue::create_game`.
pub(super) fn resolve_boot_game(
    storage: &Arc<Storage>,
    world_key: &str,
    persona_key: &str,
) -> crate::error::Result<GameId> {
    let catalogue = GameCatalogue::new(
        Arc::clone(storage),
        Arc::new(MessageService::new(Arc::clone(storage))),
        Arc::new(ActiveGame::new(Arc::clone(storage))),
        None,
    );
    let game_id = super::init_game::resolve_game_id(&catalogue, world_key, persona_key)
        .map_err(boot_error)?;
    match storage.get_game(game_id.0) {
        Ok(Some(game)) => tracing::info!(
            "Starting on game '{}' (id={game_id}) with persona '{persona_key}'",
            game.name
        ),
        Ok(None) => tracing::warn!("Starting on game {game_id}, which has no row"),
        Err(e) => tracing::warn!("Failed to read the Game {game_id} at boot: {e}"),
    }
    Ok(game_id)
}

/// A refusal at boot keeps its kind, so the caller sees the same error a Games
/// page request would.
fn boot_error(error: ApplicationError) -> EngineError {
    match error {
        ApplicationError::Validation(message) => EngineError::Validation(message),
        ApplicationError::Engine(error) => error,
        ApplicationError::ShuttingDown => {
            EngineError::Config("Server is shutting down".to_string())
        }
        ApplicationError::ConcurrentGeneration => {
            EngineError::Config("Generation in progress".to_string())
        }
    }
}

fn prepare_state(data: &PreparedData) -> crate::error::Result<StateResources> {
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| EngineError::Io(format!("runtime_new {}: {e}", "tokio_runtime")))?;

    let world_arc = Arc::new(data.world_card.clone());
    let player_arc = Arc::new(data.player.clone());

    // Its write is the point: a game with no Snapshot gets the initial one and
    // the scenario Message.
    super::init_game::load_game_state(
        &data.storage,
        data.active_game_id,
        &world_arc,
        &data.map,
        &player_arc,
        &data.npcs_map,
    )?;

    let wired = build_app_graph(Arc::clone(&data.storage), Some(data.active_game_id))?;

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

pub(crate) fn ensure_presets(
    storage: &crate::adapters::driven::storage::Storage,
    data_dir: &std::path::Path,
) -> crate::error::Result<()> {
    use crate::domain::model::prompt_preset::PresetType;

    for preset_type in [
        PresetType::System,
        PresetType::Quantifier,
        PresetType::Impersonate,
        PresetType::Options,
    ] {
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
            process_preset_file(storage, &path, &existing_ids, preset_type)?;
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
    use crate::domain::model::settings::NarratorMode;
    use crate::domain::model::utils::settings_defaults;

    if path.extension().and_then(|s| s.to_str()) != Some("json") {
        return Ok(());
    }
    let content = std::fs::read_to_string(path)?;
    let seed: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        crate::error::EngineError::Parse(format!("Invalid preset seed {}: {e}", path.display()))
    })?;

    // `allowed_modes` are user-owned from the first write, so a seed only sets
    // them at insert.
    let allowed_modes = match seed.get("allowed_modes") {
        None | Some(serde_json::Value::Null) => settings_defaults::default_allowed_modes(),
        Some(value) => {
            serde_json::from_value::<Vec<NarratorMode>>(value.clone()).unwrap_or_else(|e| {
                tracing::warn!(
                    "Invalid allowed_modes in {}: {e}; allowing all modes",
                    path.display()
                );
                settings_defaults::default_allowed_modes()
            })
        }
    };

    let id = seed["id"].as_str().unwrap_or("default").to_string();
    let preset = PromptPreset {
        id: id.clone(),
        name: seed["name"].as_str().unwrap_or("Default").to_string(),
        role: seed["role"].as_str().map(|s| s.to_string()),
        instructions: seed["instructions"].as_str().map(|s| s.to_string()),
        writing_style: seed["writing_style"].as_str().map(|s| s.to_string()),
        output_format: seed["output_format"].as_str().map(|s| s.to_string()),
        allowed_modes,
        is_default: seed
            .get("is_default")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        preset_type,
    };

    if existing_ids.contains(&id) {
        if let Ok(Some(existing)) = storage.get_preset(&id) {
            let has_content = existing.role.is_some()
                || existing.instructions.is_some()
                || existing.writing_style.is_some()
                || existing.output_format.is_some();
            if !has_content {
                let mut preset = preset;
                preset.allowed_modes = existing.allowed_modes;
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
