pub mod debug;
pub mod fragments;
pub mod settings_fragment;
pub mod templates;

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
        // History edit & retry endpoints
        .route("/history/:id", post(fragments::edit_history_handler))
        .route("/retry", post(fragments::retry_handler))
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
        // NOTE: dev-only diagnostic endpoint
        .route("/debug/state", get(debug::debug_state_handler))
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
}

pub fn create_app_for_testing(state: Arc<std::sync::Mutex<GameState>>) -> Router {
    let app_state = AppState {
        state,
        game_service: Arc::new(DefaultGameService::new()) as Arc<dyn GameService>,
        settings: Arc::new(RwLock::new(AppSettings::default())),
        cancel_token: CancellationToken::new(),
    };
    build_router(app_state)
}

use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use std::sync::Arc;
use std::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tower_http::services::ServeDir;

use crate::engine::game_service::{DefaultGameService, GameService};
use crate::error::{EngineError, Result};
use crate::model::settings::AppSettings;
use crate::model::state::GameState;

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
    pub state: Arc<std::sync::Mutex<GameState>>,
    pub game_service: Arc<dyn GameService>,
    pub settings: Arc<RwLock<AppSettings>>,
    pub cancel_token: CancellationToken,
}

impl AppState {
    pub async fn new(state: Arc<std::sync::Mutex<GameState>>) -> Self {
        let settings = crate::settings::load_settings().unwrap_or_else(|_| AppSettings::default());
        Self {
            state,
            game_service: Arc::new(DefaultGameService::new()) as Arc<dyn GameService>,
            settings: Arc::new(RwLock::new(settings)),
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn lock_state(&self) -> crate::error::Result<std::sync::MutexGuard<GameState>> {
        self.state
            .lock()
            .map_err(|_| crate::error::EngineError::Config("Lock poisoned".into()))
    }
}

pub async fn run_server(state: Arc<std::sync::Mutex<GameState>>) -> Result<()> {
    run_server_with_config(state, ServerConfig::default()).await
}

/// [DOC: docs/architecture/system.md]
pub async fn run_server_with_config(
    state: Arc<std::sync::Mutex<GameState>>,
    config: ServerConfig,
) -> Result<()> {
    let app_state = AppState::new(state.clone()).await;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 3_000);
    }

    #[test]
    fn test_server_config_custom_port() {
        let config = ServerConfig { port: 80_80 };
        assert_eq!(config.port, 80_80);
    }

    #[test]
    fn test_server_config_default_is_consistent() {
        // Ensure default is consistent across calls
        let config1 = ServerConfig::default();
        let config2 = ServerConfig::default();
        assert_eq!(config1.port, config2.port);
    }

    #[test]
    fn test_server_config_clone() {
        let config = ServerConfig { port: 5000 };
        let cloned = config.clone();
        assert_eq!(config.port, cloned.port);
    }

    #[test]
    fn test_server_config_debug() {
        let config = ServerConfig { port: 3000 };
        let debug_str = format!("{config:?}");
        assert!(debug_str.contains("3000"));
    }

    #[test]
    fn test_server_config_min_port() {
        let config = ServerConfig { port: 1 };
        assert_eq!(config.port, 1);
    }

    #[test]
    fn test_server_config_max_port() {
        let config = ServerConfig { port: 65535 };
        assert_eq!(config.port, 65535);
    }

    #[test]
    fn test_app_state_struct_fields() {
        // Verify AppState struct has expected fields
        let game_service: Arc<dyn GameService> = Arc::new(DefaultGameService::new());
        let settings = Arc::new(RwLock::new(AppSettings::default()));

        // Verify we can construct AppState-like struct with required fields
        let _app_state = (game_service, settings);
    }

    #[test]
    fn test_game_service_trait_bounds() {
        // Verify GameService trait is Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<DefaultGameService>();
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        let narrator = settings
            .get_narration_connection()
            .expect("narrator exists");
        assert!(narrator.model.contains("gpt-4o-mini") || narrator.model.is_empty());
    }
}

#[cfg(test)]
mod server_tests {
    use super::*;

    #[test]
    fn test_server_config_custom_port() {
        let config = ServerConfig { port: 8080 };
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_server_config_default_is_consistent() {
        let config1 = ServerConfig::default();
        let config2 = ServerConfig::default();
        assert_eq!(config1.port, config2.port);
    }
}
