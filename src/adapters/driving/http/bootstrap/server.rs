//! [DOC: docs/diataxis/reference/frontend/dashboard.md]
//! Server implementation

use std::net::{IpAddr, SocketAddr};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use tracing;

use crate::error::{EngineError, Result};

use crate::adapters::driving::http::app_state::AppState;
use crate::adapters::driving::http::builders::router::build_router;
use crate::bootstrap::wiring::WiredApp;

#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: IpAddr,
    pub port: u16,
}

#[cfg(test)]
impl Default for ServerConfig {
    fn default() -> Self {
        ServerConfig {
            host: IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED),
            port: 3000,
        }
    }
}

pub async fn run_server_with_config(
    app: WiredApp,
    config: ServerConfig,
) -> Result<(SocketAddr, JoinHandle<std::io::Result<()>>)> {
    let app_state = AppState::from_wired(app);
    let shutdown_token = app_state.shutdown_token.clone();

    let app = build_router(app_state);

    let bind_addr = SocketAddr::new(config.host, config.port).to_string();
    let listener = TcpListener::bind(&bind_addr).await.map_err(|e| {
        EngineError::Config(format!("Failed to bind to port {}: {}", config.port, e))
    })?;

    let addr = listener
        .local_addr()
        .map_err(|e| EngineError::Config(format!("local_addr: {e}")))?;

    tracing::info!("HTMX Dashboard running at http://{addr}");

    let shutdown_signal = async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            tracing::warn!("ctrl_c handler unavailable ({e}); shutting down");
        } else {
            tracing::info!("Shutdown signal received, cancelling in-flight tasks...");
        }
        shutdown_token.cancel();
    };

    let handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal)
            .await
    });
    Ok((addr, handle))
}
