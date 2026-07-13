//! [DOC: docs/system/dashboard.md]
//! Server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing;

use crate::error::{EngineError, Result};

use super::app_state::{AppState, ServerResources};
use super::router::build_router;
use super::port_utils::bind_with_retry;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub bind_attempts: Option<u32>,
}

#[cfg(test)]
impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            port: 3000,
            bind_attempts: None,
        }
    }
}

pub async fn run_server_with_config(
    resources: ServerResources,
    config: ServerConfig,
) -> Result<(SocketAddr, JoinHandle<std::io::Result<()>>)> {
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let app_state = AppState {
        storage: Arc::clone(&resources.storage),
        preset_storage: Arc::clone(&resources.preset_storage),
        settings: Arc::clone(&resources.settings),
        game_service: Arc::clone(&resources.game_service),
        application_service: Arc::new(
            crate::application::application_service::DefaultApplicationService::new(
                Arc::clone(&resources.storage),
                Arc::clone(&resources.preset_storage),
                Arc::clone(&resources.settings),
                shutdown_token.clone(),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
                Arc::clone(&resources.game_service),
            ),
        ),
        text_check_service: Arc::clone(&resources.text_check_service),
        shutdown_token: Arc::new(std::sync::RwLock::new(shutdown_token)),
    };
    let shutdown_token_arc = Arc::clone(&app_state.shutdown_token);

    let app = build_router(app_state);

    let bind_addr = format!("127.0.0.1:{}", config.port);
    let listener = bind_with_retry(&bind_addr, config.bind_attempts)
        .await
        .map_err(|e| {
            EngineError::Config(format!("Failed to bind to port {}: {}", config.port, e))
        })?;

    let addr = listener
        .local_addr()
        .map_err(|e| EngineError::Config(format!("local_addr: {e}")))?;

    tracing::info!("HTMX Dashboard running at http://{addr}");

    let shutdown_signal = async move {
        let _ = tokio::signal::ctrl_c().await;
        tracing::info!("Shutdown signal received, cancelling in-flight tasks...");
        let token = shutdown_token_arc
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|p| {
                tracing::warn!("Poisoned shutdown_token read lock recovered during shutdown");
                p.into_inner().clone()
            });
        token.cancel();
    };

    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
    });
    Ok((addr, handle))
}
