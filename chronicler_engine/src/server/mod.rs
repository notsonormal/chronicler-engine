pub mod debug;
pub mod fragments;
pub mod prompt_presets_fragment;
pub mod settings_fragment;
pub mod templates;
pub mod view_models;

#[cfg(test)]
mod fragments_tests;
#[cfg(test)]
mod mod_tests;
#[cfg(test)]
mod prompt_presets_fragment_tests;
#[cfg(test)]
mod settings_fragment_tests;
#[cfg(test)]
mod templates_tests;

pub(crate) fn build_router(app_state: AppState) -> Router {
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
        .route("/swipe/new", post(fragments::retry_handler))
        .route(
            "/message/:id/swipe/:index",
            post(fragments::switch_swipe_handler),
        )
        .route("/retrigger", post(fragments::retrigger_handler))
        .route("/reset", post(fragments::reset_handler))
        .route("/games", post(fragments::create_game_handler))
        .route("/games/:id/switch", post(fragments::switch_game_handler))
        .route("/games/:id/delete", post(fragments::delete_game_handler))
        .route("/fragment/games", get(fragments::list_games_fragment))
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
        // Prompt Presets endpoints
        .route(
            "/fragment/prompt-presets",
            get(crate::server::prompt_presets_fragment::panel_handler),
        )
        .route(
            "/prompt-presets",
            post(crate::server::prompt_presets_fragment::save_preset_handler),
        )
        .route(
            "/fragment/prompt-presets/:id",
            get(crate::server::prompt_presets_fragment::preset_card_handler),
        )
        .route(
            "/fragment/prompt-presets/:id/edit",
            get(crate::server::prompt_presets_fragment::edit_preset_form_handler),
        )
        .route(
            "/fragment/prompt-presets/:id/view",
            get(crate::server::prompt_presets_fragment::view_preset_form_handler),
        )
        .route(
            "/prompt-presets/:id",
            post(crate::server::prompt_presets_fragment::update_preset_handler),
        )
        .route(
            "/prompt-presets/:id/delete",
            post(crate::server::prompt_presets_fragment::delete_preset_handler),
        )
        .route(
            "/prompt-presets/:id/duplicate",
            post(crate::server::prompt_presets_fragment::duplicate_preset_handler),
        )
        .route(
            "/prompt-presets/:id/activate",
            post(crate::server::prompt_presets_fragment::activate_preset_handler),
        )
        // NOTE: dev-only diagnostic endpoint
        .route("/debug/state", get(debug::debug_state_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
}

pub fn create_app_with_state(app_state: AppState) -> Router {
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

use crate::application::application_service::DefaultApplicationService;
use crate::application::game_service::DefaultGameService;
use crate::error::{EngineError, Result};
use crate::model::character::NpcCard;
use crate::model::map::MapDef;
use crate::model::settings::AppSettings;
use crate::model::world::WorldCard;

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
pub struct ServerResources {
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
    pub storage: Arc<crate::storage::Storage>,
    pub preset_storage: Arc<crate::storage::Storage>,
    pub settings: Arc<RwLock<AppSettings>>,
}

#[derive(Clone)]
pub struct AppState {
    pub storage: Arc<crate::storage::Storage>,
    pub preset_storage: Arc<crate::storage::Storage>,
    pub world: Arc<WorldCard>,
    pub map: Arc<MapDef>,
    pub player: Arc<crate::model::character::PlayerCard>,
    pub npcs: Arc<HashMap<String, NpcCard>>,
    pub game_service: Arc<DefaultGameService>,
    pub application_service: Arc<DefaultApplicationService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancel_token: Arc<std::sync::RwLock<CancellationToken>>,
    pub is_generating: Arc<AtomicBool>,
}

impl AppState {
    pub fn as_game_service_context(&self) -> crate::application::game_service::GameServiceContext {
        crate::application::game_service::GameServiceContext {
            storage: Arc::clone(&self.storage),
            world: Arc::clone(&self.world),
            map: Arc::clone(&self.map),
            player: Arc::clone(&self.player),
            npcs: Arc::clone(&self.npcs),
            cancel_token: self.current_cancel_token(),
            is_generating: Arc::clone(&self.is_generating),
            settings: Arc::clone(&self.settings),
            preset_storage: Arc::clone(&self.preset_storage),
        }
    }

    /// If the lock is poisoned, recovers the inner value.
    pub fn current_cancel_token(&self) -> CancellationToken {
        self.cancel_token
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|p| {
                tracing::warn!("Poisoned cancel_token read lock recovered");
                p.into_inner().clone()
            })
    }

    /// If the lock is poisoned, recovers the inner value before replacing.
    pub fn replace_cancel_token(&self) {
        let mut token = self.cancel_token.write().unwrap_or_else(|p| {
            tracing::warn!("Poisoned cancel_token write lock recovered");
            p.into_inner()
        });
        *token = CancellationToken::new();
    }

    /// If the lock is poisoned, recovers the inner value.
    pub fn settings(&self) -> AppSettings {
        self.settings.read().map(|g| g.clone()).unwrap_or_else(|p| {
            tracing::warn!("Poisoned settings read lock recovered");
            p.into_inner().clone()
        })
    }
}

/// [DOC: docs/architecture/system.md]
pub async fn run_server_with_config(
    resources: ServerResources,
    config: ServerConfig,
) -> Result<()> {
    let app_state = AppState {
        storage: Arc::clone(&resources.storage),
        preset_storage: Arc::clone(&resources.preset_storage),
        world: resources.world,
        map: resources.map,
        player: resources.player,
        npcs: resources.npcs,
        is_generating: Arc::new(AtomicBool::new(false)),
        settings: Arc::clone(&resources.settings),
        game_service: Arc::new(DefaultGameService::with_storage(
            Some(Arc::clone(&resources.storage)),
            Some(Arc::clone(&resources.preset_storage)),
            Arc::clone(&resources.settings),
        )),
        application_service: Arc::new(DefaultApplicationService::new(Arc::new(
            DefaultGameService::with_storage(
                Some(resources.storage),
                Some(resources.preset_storage),
                Arc::clone(&resources.settings),
            ),
        ))),
        cancel_token: Arc::new(std::sync::RwLock::new(CancellationToken::new())),
    };
    let cancel_token_arc = Arc::clone(&app_state.cancel_token);

    let app = build_router(app_state);

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = bind_with_retry(&addr).await.map_err(|e| {
        EngineError::Config(format!("Failed to bind to port {}: {}", config.port, e))
    })?;

    tracing::info!("HTMX Dashboard running at http://127.0.0.1:{}", config.port);

    let shutdown_signal = async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Shutdown signal received, cancelling in-flight tasks...");
        let token = cancel_token_arc
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|p| {
                tracing::warn!("Poisoned cancel_token read lock recovered during shutdown");
                p.into_inner().clone()
            });
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
                tracing::error!("Port in use, attempting to free it...");
                if let Some(pid) = find_process_on_port(addr) {
                    tracing::error!("Found process on port, attempting to kill PID {pid}...");
                    let _ = kill_process(pid);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
                // Fall through to try again anyway
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            Err(e) => {
                tracing::error!("Bind error: {e:?}");
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
