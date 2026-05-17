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
        .route("/checkpoint", post(fragments::create_checkpoint_handler))
        .route(
            "/checkpoint/:id/restore",
            post(fragments::restore_checkpoint_handler),
        )
        .route(
            "/checkpoint/:id/delete",
            post(fragments::delete_checkpoint_handler),
        )
        .route(
            "/fragment/checkpoints",
            get(fragments::list_checkpoints_fragment),
        )
        .route(
            "/fragment/llm-messages",
            get(fragments::llm_messages_fragment),
        )
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
    // [DOC: docs/architecture/system.md]
    let snapshot = crate::model::state_snapshot::GameStateSnapshot::from_game_state(&state);
    let storage = Arc::new(crate::test_support::InMemoryGameStorage::new());
    let snapshot_storage: Arc<dyn crate::storage::snapshot_storage::SnapshotStorage> =
        storage.clone();
    let message_storage: Arc<dyn crate::storage::message_storage::MessageStorage> = storage.clone();
    let llm_storage =
        Arc::new(crate::storage::llm_message_storage::InMemoryLlmMessageStorage::new())
            as Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage>;
    let _ = storage.save(&snapshot);
    for mut msg in state.narrative.messages.clone() {
        let _ = storage.insert_message(&mut msg);
    }
    create_app_with_storage(
        state,
        snapshot_storage,
        message_storage,
        llm_storage,
        settings,
    )
}

pub fn create_app_with_storage(
    state: GameState,
    snapshot_storage: Arc<dyn crate::storage::snapshot_storage::SnapshotStorage>,
    message_storage: Arc<dyn crate::storage::message_storage::MessageStorage>,
    llm_storage: Arc<dyn crate::storage::llm_message_storage::LlmMessageStorage>,
    settings: AppSettings,
) -> Router {
    let app_state = AppState {
        snapshot_storage,
        message_storage,
        llm_message_storage: Arc::clone(&llm_storage),
        world: state.world.clone(),
        map: state.map.clone(),
        player: state.player.clone(),
        npcs: Arc::new(state.npcs.clone()),
        game_service: Arc::new(
            crate::engine::game_service::DefaultGameService::with_storage(Some(llm_storage)),
        ) as Arc<dyn crate::engine::game_service::GameService>,
        settings: Arc::new(RwLock::new(settings)),
        cancel_token: Arc::new(std::sync::RwLock::new(CancellationToken::new())),
        is_generating: Arc::new(AtomicBool::new(false)),
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
use crate::storage::llm_message_storage::LlmMessageStorage;
use crate::storage::message_storage::MessageStorage;
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
    pub message_storage: Arc<dyn crate::storage::message_storage::MessageStorage>,
    pub llm_message_storage: Arc<dyn LlmMessageStorage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
    pub game_service: Arc<dyn GameService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancel_token: Arc<std::sync::RwLock<CancellationToken>>,
    pub is_generating: Arc<AtomicBool>,
}

impl AppState {
    pub fn load_state(&self) -> crate::error::Result<GameState> {
        let snapshot = self.snapshot_storage.load_latest()?;
        let mut game_state = match snapshot {
            Some(snap) => GameState::from_snapshot(
                &snap,
                Arc::clone(&self.world),
                Arc::clone(&self.map),
                Arc::clone(&self.player),
                (*self.npcs).clone(),
            ),
            None => GameState::new(
                Arc::clone(&self.world),
                Arc::clone(&self.map),
                Arc::clone(&self.player),
                (*self.npcs).values().cloned().collect(),
                self.world.starting_room_id.clone(),
            ),
        };
        if let Ok(messages) = self.message_storage.load_messages() {
            game_state.narrative.messages = messages;
        }
        Ok(game_state)
    }

    pub fn as_game_service_context(&self) -> crate::engine::game_service::GameServiceContext {
        crate::engine::game_service::GameServiceContext {
            snapshot_storage: Arc::clone(&self.snapshot_storage),
            message_storage: Arc::clone(&self.message_storage),
            llm_message_storage: Arc::clone(&self.llm_message_storage),
            world: Arc::clone(&self.world),
            map: Arc::clone(&self.map),
            player: Arc::clone(&self.player),
            npcs: Arc::clone(&self.npcs),
            cancel_token: self.current_cancel_token(),
            action_lock: Arc::new(std::sync::Mutex::new(())),
            is_generating: Arc::clone(&self.is_generating),
        }
    }

    /// Returns a clone of the current cancellation token.
    /// If the lock is poisoned, recovers the inner value.
    pub fn current_cancel_token(&self) -> CancellationToken {
        match self.cancel_token.read() {
            Ok(g) => g.clone(),
            Err(p) => p.into_inner().clone(),
        }
    }

    /// Replaces the current cancellation token with a fresh one.
    /// If the lock is poisoned, recovers the inner value before replacing.
    pub fn replace_cancel_token(&self) {
        let mut token = match self.cancel_token.write() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        *token = CancellationToken::new();
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
    snapshot_storage: Arc<dyn SnapshotStorage>,
    message_storage: Arc<dyn crate::storage::message_storage::MessageStorage>,
    llm_message_storage: Arc<dyn LlmMessageStorage>,
) -> Result<()> {
    run_server_with_config(
        world,
        map,
        player,
        npcs,
        snapshot_storage,
        message_storage,
        llm_message_storage,
        ServerConfig::default(),
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
    snapshot_storage: Arc<dyn SnapshotStorage>,
    message_storage: Arc<dyn crate::storage::message_storage::MessageStorage>,
    llm_message_storage: Arc<dyn LlmMessageStorage>,
    config: ServerConfig,
) -> Result<()> {
    let app_state = AppState {
        snapshot_storage,
        message_storage,
        llm_message_storage: Arc::clone(&llm_message_storage),
        world,
        map,
        player,
        npcs,
        is_generating: Arc::new(AtomicBool::new(false)),
        game_service: Arc::new(DefaultGameService::with_storage(Some(llm_message_storage)))
            as Arc<dyn GameService>,
        settings: Arc::new(RwLock::new(
            crate::settings::load_settings().unwrap_or_else(|_| AppSettings::default()),
        )),
        cancel_token: Arc::new(std::sync::RwLock::new(CancellationToken::new())),
    };
    let cancel_token_arc = Arc::clone(&app_state.cancel_token);

    let app = build_router(app_state);

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = bind_with_retry(&addr).await.map_err(|e| {
        EngineError::Config(format!("Failed to bind to port {}: {}", config.port, e))
    })?;

    log::info!("HTMX Dashboard running at http://127.0.0.1:{}", config.port);

    let shutdown_signal = async move {
        let _ = tokio::signal::ctrl_c().await;
        log::info!("Shutdown signal received, cancelling in-flight tasks...");
        let token = match cancel_token_arc.read() {
            Ok(g) => g.clone(),
            Err(p) => p.into_inner().clone(),
        };
        token.cancel();
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
