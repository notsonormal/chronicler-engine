//! [DOC: chronicler_engine/docs/diataxis/reference/frontend/dashboard.md]
//! Server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing;

use crate::error::{EngineError, Result};

use super::app_state::AppState;
use super::router::build_router;
use super::port_utils::bind_with_retry;
use crate::bootstrap::wiring::WiredApp;

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
    app: WiredApp,
    config: ServerConfig,
) -> Result<(SocketAddr, JoinHandle<std::io::Result<()>>)> {
    let shutdown_token = tokio_util::sync::CancellationToken::new();
    let app_state = AppState::from_wired(app, shutdown_token.clone());
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
