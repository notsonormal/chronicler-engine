//! [DOC: docs/system/game_flow.md]
//! Shared spawn helper for pipeline tasks

use std::sync::Arc;

use crate::application::context::GameServiceContext;
use crate::application::game_service::GameService;

pub(crate) fn spawn_pipeline_task<F>(game_service: &Arc<GameService>, ctx: GameServiceContext, f: F)
where
    F: FnOnce(&GameService, GameServiceContext) + Send + 'static,
{
    let game_service = Arc::clone(game_service);
    tokio::task::spawn_blocking(move || {
        f(&game_service, ctx);
    });
}
