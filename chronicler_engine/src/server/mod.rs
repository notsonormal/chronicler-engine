pub mod debug;
pub mod fragments;
pub mod settings_fragment;
pub mod templates;

#[cfg(test)]
mod fragments_tests;
#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod settings_fragment_tests;
#[cfg(test)]
mod templates_tests;

fn build_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/fragment/header", get(fragments::header_fragment))
        .route("/fragment/story-log", get(fragments::story_log_fragment))
        .route(
            "/fragment/visual-sidebar",
            get(fragments::visual_sidebar_fragment),
        )
        .route(
            "/fragment/action-area",
            get(fragments::action_area_fragment),
        )
        .route(
            "/fragment/character-headshots",
            get(fragments::character_headshots_fragment),
        )
        .route("/action", post(fragments::action_handler))
        .route("/action/check", post(fragments::action_check_handler))
        .route("/action/confirm", post(fragments::action_confirm_handler))
        .route("/check-text", post(fragments::check_text_handler))
        .route("/hints", get(fragments::hints_handler))
        .route("/status/ready", get(fragments::status_ready_handler))
        .route(
            "/status/generating",
            get(fragments::generating_status_handler),
        )
        .route(
            "/status/reset-generating",
            post(fragments::reset_generating_handler),
        )
        // History edit, delete & retry endpoints
        .route("/history/:id", post(fragments::edit_history_handler))
        .route("/history/delete", post(fragments::delete_history_handler))
        .route("/retry", post(fragments::retry_handler))
        .route("/reset", post(fragments::reset_handler))
        // Settings endpoints
        .route(
            "/fragment/settings",
            get(crate::server::settings_fragment::settings_panel),
        )
        .route(
            "/settings",
            post(crate::server::settings_fragment::save_settings_handler),
        )
        .route(
            "/connections/add",
            post(crate::server::settings_fragment::add_connection_handler),
        )
        .route(
            "/fragment/connections/:id",
            get(crate::server::settings_fragment::connection_card_fragment),
        )
        .route(
            "/fragment/connections/:id/edit",
            get(crate::server::settings_fragment::edit_connection_form),
        )
        .route(
            "/connections/:id/edit",
            post(crate::server::settings_fragment::edit_connection_handler),
        )
        .route(
            "/connections/:id/delete",
            post(crate::server::settings_fragment::delete_connection_handler),
        )
        .route(
            "/connections/:id/set-narrator",
            post(crate::server::settings_fragment::set_narrator_handler),
        )
        .route(
            "/connections/:id/set-quantifier",
            post(crate::server::settings_fragment::set_quantifier_handler),
        )
        .route(
            "/settings/text-check",
            post(crate::server::settings_fragment::save_text_check_handler),
        )
        // NOTE: dev-only diagnostic endpoint
        .route("/debug/state", get(debug::debug_state_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
}

pub fn create_app_for_testing(state: GameState) -> Router {
    create_app_for_testing_with_settings(state, AppSettings::default())
}

pub fn create_app_for_testing_with_settings(state: GameState, settings: AppSettings) -> Router {
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        "test".to_string(),
        0,
    );
    let storage = Arc::new(crate::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn crate::storage::snapshot_storage::SnapshotStorage>;
    let _ = storage.save(&snapshot);
    create_app_with_storage(state, storage, settings)
}

pub fn create_app_with_storage(
    state: GameState,
    storage: Arc<dyn crate::storage::snapshot_storage::SnapshotStorage>,
    settings: AppSettings,
) -> Router {
    let app_state = AppState {
        snapshot_storage: storage,
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        starting_room_id: state.movement.current_room_id.clone(),
        game_service: Arc::new(crate::engine::game_service::DefaultGameService::new())
            as Arc<dyn crate::engine::game_service::GameService>,
        settings: Arc::new(RwLock::new(settings)),
        cancel_token: CancellationToken::new(),
        is_generating: Arc::new(AtomicBool::new(false)),
        scenario_text: None,
    };
    build_router(app_state)
}

use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicBool;
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;

use crate::engine::game_service::{DefaultGameService, GameService};
use crate::error::{EngineError, Result};
use crate::model::character::NpcCard;
use crate::model::map::MapDef;
use crate::model::settings::AppSettings;
use crate::model::state::GameState;
use crate::model::world::WorldCard;
use crate::storage::snapshot_storage::SnapshotStorage;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig { port: 3000 }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub snapshot_storage: Arc<dyn SnapshotStorage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
    pub starting_room_id: String,
    pub game_service: Arc<dyn GameService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancel_token: CancellationToken,
    pub is_generating: Arc<AtomicBool>,
    /// Scenario text injected on first load; used by reset to re-create initial state.
    pub scenario_text: Option<String>,
}

impl AppState {
    pub fn load_state(&self) -> crate::error::Result<GameState> {
        let snapshot = self.snapshot_storage.load_latest(None)?;
        match snapshot {
            Some(snap) => Ok(GameState::from_snapshot(
                &snap,
                Arc::clone(&self.world),
                Arc::clone(&self.map),
                Arc::clone(&self.player),
                (*self.npcs).clone(),
            )),
            None => Ok(GameState::new(
                Arc::clone(&self.world),
                Arc::clone(&self.map),
                Arc::clone(&self.player),
                (*self.npcs).values().cloned().collect(),
                self.starting_room_id.clone(),
            )),
        }
    }

    pub fn as_game_service_context(&self) -> crate::engine::game_service::GameServiceContext {
        crate::engine::game_service::GameServiceContext {
            snapshot_storage: Arc::clone(&self.snapshot_storage),
            world: Arc::clone(&self.world),
            map: Arc::clone(&self.map),
            player: Arc::clone(&self.player),
            npcs: Arc::clone(&self.npcs),
            starting_room_id: self.starting_room_id.clone(),
            cancel_token: self.cancel_token.clone(),
            action_lock: Arc::new(std::sync::Mutex::new(())),
            is_generating: Arc::clone(&self.is_generating),
        }
    }

    /// Read a snapshot of the current settings.
    /// Returns default settings if the lock is poisoned.
    pub fn settings(&self) -> AppSettings {
        self.settings.read().map(|g| g.clone()).unwrap_or_default()
    }
}

pub async fn run_server(
    world: Arc<WorldCard>,
    map: Arc<MapDef>,
    player: Arc<crate::model::character::PlayerCard>,
    npcs: Arc<HashMap<String, NpcCard>>,
    starting_room_id: String,
    snapshot_storage: Arc<dyn SnapshotStorage>,
) -> Result<()> {
    run_server_with_config(
        world,
        map,
        player,
        npcs,
        starting_room_id,
        snapshot_storage,
        ServerConfig::default(),
        None,
    )
    .await
}

/// [DOC: docs/architecture/system.md]
#[allow(clippy::too_many_arguments)]
pub async fn run_server_with_config(
    world: Arc<WorldCard>,
    map: Arc<MapDef>,
    player: Arc<crate::model::character::PlayerCard>,
    npcs: Arc<HashMap<String, NpcCard>>,
    starting_room_id: String,
    snapshot_storage: Arc<dyn SnapshotStorage>,
    config: ServerConfig,
    scenario_text: Option<String>,
) -> Result<()> {
    let app_state = AppState {
        snapshot_storage,
        world,
        map,
        player,
        npcs,
        is_generating: Arc::new(AtomicBool::new(false)),
        starting_room_id,
        game_service: Arc::new(DefaultGameService::new()) as Arc<dyn GameService>,
        settings: Arc::new(RwLock::new(
            crate::settings::load_settings().unwrap_or_else(|_| AppSettings::default()),
        )),
        cancel_token: CancellationToken::new(),
        scenario_text,
    };
    let cancel_token = app_state.cancel_token.clone();

    let app = build_router(app_state);

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = bind_with_retry(&addr).await.map_err(|e| {
        EngineError::Config(format!("Failed to bind to port {}: {}", config.port, e))
    })?;

    log::info!("HTMX Dashboard running at http://127.0.0.1:{}", config.port);

    let shutdown_signal = async move {
        let _ = tokio::signal::ctrl_c().await;
        log::info!("Shutdown signal received, cancelling in-flight tasks...");
        cancel_token.cancel();
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(|e| EngineError::Config(e.to_string()))
}

async fn bind_with_retry(addr: &str) -> std::io::Result<tokio::net::TcpListener> {
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                log::error!("Port in use, attempting to free it...");
                if let Some(pid) = find_process_on_port(addr) {
                    log::error!("Found process on port, attempting to kill PID {pid}...");
                    let _ = kill_process(pid);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
                // Fall through to try again anyway
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            Err(e) => {
                log::error!("Bind error: {e:?}");
                return Err(e);
            }
        }
    }
}

fn find_process_on_port(addr: &str) -> Option<u32> {
    let port = addr.split(':').next_back()?.parse::<u16>().ok()?;
    let output = std::process::Command::new("netstat")
        .args(["-ano"])
        .output()
        .ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.contains(&format!(":{port}")) && line.contains("LISTENING") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if let Some(pid_str) = parts.last() {
                return pid_str.parse().ok();
            }
        }
    }
    None
}

fn kill_process(pid: u32) -> std::io::Result<std::process::Output> {
    std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .output()
}

async fn index_handler() -> Html<String> {
    Html(include_str!("../../assets/index.html").to_string())
}
