//! [DOC: docs/system/dashboard.md]
//! Server implementation

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing;

use crate::error::{EngineError, Result};

use super::app_state::{AppState, ServerConfig, ServerResources};
use super::router::build_router;
use super::port_utils::bind_with_retry;

pub async fn run_server_with_config(
    resources: ServerResources,
    config: ServerConfig,
) -> Result<(SocketAddr, JoinHandle<std::io::Result<()>>)> {
    let is_generating = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel_token = tokio_util::sync::CancellationToken::new();
    let app_state = AppState {
        storage: Arc::clone(&resources.storage),
        preset_storage: Arc::clone(&resources.preset_storage),
        is_generating: Arc::clone(&is_generating),
        settings: Arc::clone(&resources.settings),
        game_service: Arc::clone(&resources.game_service),
        application_service: Arc::new(
            crate::application::application_service::DefaultApplicationService::new(
                Arc::clone(&resources.storage),
                Arc::clone(&resources.preset_storage),
                Arc::clone(&resources.settings),
                cancel_token.clone(),
                Arc::clone(&is_generating),
                Arc::clone(&resources.game_service),
            ),
        ),
        text_check_service: Arc::clone(&resources.text_check_service),
        cancel_token: Arc::new(std::sync::RwLock::new(cancel_token)),
    };
    let cancel_token_arc = Arc::clone(&app_state.cancel_token);

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
        let token = cancel_token_arc
            .read()
            .map(|g| g.clone())
            .unwrap_or_else(|p| {
                tracing::warn!("Poisoned cancel_token read lock recovered during shutdown");
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
