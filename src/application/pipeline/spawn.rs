//! [DOC: docs/diataxis/reference/game_flow.md]
//! Shared spawn helper for pipeline tasks

use std::sync::Arc;

use crate::application::pipeline::pipeline::ActionPipeline;

pub(crate) fn spawn_pipeline_task<F>(pipeline: Arc<ActionPipeline>, f: F)
where
    F: FnOnce(&ActionPipeline) + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        f(&pipeline);
    });
}
