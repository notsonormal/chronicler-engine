pub mod fragments;
pub mod templates;

/// Test helper - create the Axum app without running a server.
/// This allows same-process testing with coverage.
pub fn create_app_for_testing(state: Arc<std::sync::Mutex<GameState>>) -> Router {
    let app_state = AppState { state };

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
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state)
}

use axum::{
    Router,
    response::Html,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::services::ServeDir;

use crate::error::{EngineError, Result};
use crate::model::state::GameState;

#[derive(Clone)]
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
}

pub async fn run_server(state: Arc<std::sync::Mutex<GameState>>) -> Result<()> {
    run_server_with_config(state, ServerConfig::default()).await
}

pub async fn run_server_with_config(
    state: Arc<std::sync::Mutex<GameState>>,
    config: ServerConfig,
) -> Result<()> {
    let app_state = AppState { state };

    let app = Router::new()
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
        .nest_service("/assets", ServeDir::new("assets"))
        .nest_service("/data", ServeDir::new("data"))
        .fallback_service(ServeDir::new("assets"))
        .with_state(app_state);

    let addr = format!("127.0.0.1:{}", config.port);
    let listener = bind_with_retry(&addr).await.map_err(|e| {
        EngineError::Config(format!("Failed to bind to port {}: {}", config.port, e))
    })?;

    println!("HTMX Dashboard running at http://127.0.0.1:{}", config.port);
    axum::serve(listener, app)
        .await
        .map_err(|e| EngineError::Config(e.to_string()))
}

async fn bind_with_retry(addr: &str) -> std::io::Result<tokio::net::TcpListener> {
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                eprintln!("Port in use, attempting to free it...");
                if let Some(pid) = find_process_on_port(addr) {
                    eprintln!("Found process on port, attempting to kill PID {pid}...");
                    let _ = kill_process(pid);
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    continue;
                }
                // Fall through to try again anyway
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            Err(e) => {
                eprintln!("Bind error: {e:?}");
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
