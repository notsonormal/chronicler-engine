//! [DOC: docs/system/game_flow.md]
//! Shared spawn helper for pipeline tasks

use std::sync::Arc;

use crate::application::application_service::DefaultApplicationService;

pub(crate) fn spawn_pipeline_task<F>(app: Arc<DefaultApplicationService>, f: F)
where
    F: FnOnce(&DefaultApplicationService) + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        f(&app);
    });
}