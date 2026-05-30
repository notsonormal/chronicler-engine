//! [DOC: docs/reference/testing.md]

use std::sync::Arc;

use chronicler_engine::application::action_pipeline::{
    execute_action_impl, retry_last_response_impl,
};
use chronicler_engine::application::game_service::DefaultGameService;
use chronicler_engine::model::state::{
    GenerationPhase, GenerationStatus, MessageType, StoredTriggerContext,
};
use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::narrative::agents::registry::AgentRegistry;
use chronicler_engine::narrative::llm::MockBackend;
use chronicler_engine::test_support::{make_test_context, make_test_context_without_snapshot};
use pipeline_helpers::{
    create_test_state_with_trigger_npc, latest_state, wait_for_generation_complete,
};

#[path = "helpers/pipeline_helpers.rs"]
mod pipeline_helpers;
mod test_data;

use test_data::create_test_state;

fn working_backend() -> DefaultGameService {
    DefaultGameService::with_backends(Arc::new(MockBackend::default()), AgentRegistry::default())
}

fn failing_backend() -> DefaultGameService {
    DefaultGameService::with_backends(Arc::new(MockBackend::failing()), AgentRegistry::default())
}

#[path = "action_pipeline/actions.rs"]
mod actions;
#[path = "action_pipeline/pipeline.rs"]
mod pipeline;
#[path = "action_pipeline/retry.rs"]
mod retry;
