use std::sync::Arc;
use tracing;

use crate::error::{EngineError, Result};

use super::app_state::{AppState, ServerConfig, ServerResources};
use super::router::build_router;
use super::port_utils::bind_with_retry;

/// Starts the HTTP server with the given configuration and resources.
/// 
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
        is_generating: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        settings: Arc::clone(&resources.settings),
        game_service: Arc::new(crate::application::game_service::DefaultGameService::with_storage(
            Some(Arc::clone(&resources.storage)),
            Some(Arc::clone(&resources.preset_storage)),
            Arc::clone(&resources.settings),
        )),
        application_service: Arc::new(crate::application::application_service::DefaultApplicationService::new(Arc::new(
            crate::application::game_service::DefaultGameService::with_storage(
                Some(Arc::clone(&resources.storage)),
                Some(Arc::clone(&resources.preset_storage)),
                Arc::clone(&resources.settings),
            ),
        ))),
        cancel_token: Arc::new(std::sync::RwLock::new(tokio_util::sync::CancellationToken::new())),
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
