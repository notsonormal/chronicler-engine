//! Unit tests for the action entry path (process_action / execute_action).

use std::sync::Arc;
use std::sync::Barrier;

use crate::application::agents::quantifier::QuantifierAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::agents::Agent;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::agent::{AgentContext, AgentResult, BackendSelector, ExecutionPhase};
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::adapters::driven::storage::Storage;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::test_support::make_test_recorder;
use crate::test_support::{
    make_test_pipeline_with_backends, make_test_pipeline_with_mock_quantifier, TestAppBuilder,
    TestDataBuilder,
};

#[test]
fn test_execute_action_clears_last_trigger() {
    use crate::test_support::TestStoredTriggerContext;

    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service.clone())
        .build_service();

    let mut state = app.message_service.load_or_fresh();
    state.narrative.last_trigger = Some(TestStoredTriggerContext::for_npc(
        "npc_1",
        "Old",
        "The old trigger fires.",
    ));
    app.message_service
        .save_state(&state)
        .expect("save_state should succeed");

    app.pipeline.execute_action("look".to_string());

    let final_state = app.message_service.load_or_fresh();
    assert!(
        final_state.narrative.last_trigger.is_none(),
        "last_trigger should be cleared by execute_action"
    );
}

#[test]
fn test_streaming_narration_saved_before_quantifier_complete() {
    use std::thread;
    use std::time::Duration;

    let quantifier_provider: Arc<dyn crate::application::ports::llm_provider::LlmProvider> =
        Arc::new(
            MockBackend::default()
                .with_prompt_responses(vec![r#"{"npcs_in_room": []}"#.to_string()])
                .with_delay(500),
        );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let service = crate::test_support::make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let (app, _storage) = TestAppBuilder::with_data(TestDataBuilder::default_test().build())
        .pipeline(service)
        .build_service_with_storage();

    let app_clone = app.clone();
    let message_service = Arc::clone(&app.message_service);
    let handle = thread::spawn(move || {
        app_clone.pipeline.execute_action("look around".to_string());
    });

    let start = std::time::Instant::now();
    let mut narration_found = false;
    while start.elapsed() < Duration::from_millis(400) {
        if message_service
            .load_messages()
            .map(|msgs| {
                msgs.iter()
                    .any(|m| m.message_type == MessageType::Narration)
            })
            .unwrap_or(false)
        {
            narration_found = true;
            break;
        }
        thread::sleep(Duration::from_millis(20));
    }
    assert!(
        narration_found,
        "Narration should be saved before quantifier completes (quantifier takes 500ms)"
    );

    handle.join().expect("Action thread should complete");

    let final_state = app.message_service.load_or_fresh();
    assert!(
        !final_state.narrative.input_buffer.status.is_generating(),
        "Should complete after quantifier finishes"
    );
    let final_narration_count = message_service
        .load_messages()
        .unwrap()
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        final_narration_count, 1,
        "Should have exactly 1 narration (no duplicates), found {final_narration_count}"
    );
}

#[test]
fn test_execute_action_completes_and_persists_state() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.message_service.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle
    );
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default()
    );
    let has_narration = final_state
        .narrative
        .history()
        .iter()
        .any(|e| e.message_type == MessageType::Narration);
    assert!(has_narration, "execute_action should persist narration");
}

#[test]
fn test_execute_action_handles_narration_error() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default().with_fail()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.message_service.load_or_fresh();
    assert!(
        matches!(
            final_state.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "State should reflect error status after failed narration"
    );
}

#[test]
fn test_execute_action_handles_cancellation() {
    let data = TestDataBuilder::default_test().build();
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::with_data(data)
        .pipeline(service)
        .build_service();
    app.shutdown_token.cancel();
    app.pipeline.execute_action("look".to_string());
    let final_state = app.message_service.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancellation should reset status to Idle"
    );
}

#[test]
fn test_execute_action_preserves_existing_input_log() {
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let service = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        quantifier_provider,
    );
    let app = TestAppBuilder::default_test()
        .log("examine room", Some("Player"), MessageType::Input)
        .pipeline(service)
        .build_service();
    app.pipeline.execute_action("examine room".to_string());
    let final_state = app.message_service.load_or_fresh();
    let entries: Vec<_> = final_state.narrative.history().into_iter().collect();
    let input_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Input);
    let narration_idx = entries
        .iter()
        .position(|e| e.message_type == MessageType::Narration);
    assert!(input_idx.is_some(), "Existing input should be preserved");
    assert!(narration_idx.is_some(), "Narration should be added");
    assert!(
        input_idx.unwrap() < narration_idx.unwrap(),
        "Input should appear before narration"
    );
}

struct SyncQuantifierAgent {
    inner: QuantifierAgent,
    entered: Arc<Barrier>,
    release: Arc<Barrier>,
}

impl std::fmt::Debug for SyncQuantifierAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncQuantifierAgent")
            .finish_non_exhaustive()
    }
}

impl Agent for SyncQuantifierAgent {
    fn name(&self) -> &str {
        self.inner.name()
    }
    fn phase(&self) -> ExecutionPhase {
        self.inner.phase()
    }
    fn backend_selector(&self) -> BackendSelector {
        self.inner.backend_selector()
    }
    fn execute(&self, ctx: &AgentContext) -> crate::error::Result<AgentResult> {
        self.entered.wait();
        self.release.wait();
        self.inner.execute(ctx)
    }
}

#[test]
fn test_phase_transitions_to_quantifying_during_post_generation() {
    use std::thread;

    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let base_agent =
        QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider.clone());
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let sync_agent = Box::new(SyncQuantifierAgent {
        inner: base_agent,
        entered: entered.clone(),
        release: release.clone(),
    });
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        AgentRegistry::with_agent(sync_agent),
    );
    let app = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service();
    let app_for_thread = app.clone();

    let handle = thread::spawn(move || {
        app_for_thread.pipeline.execute_action("look".to_string());
    });

    entered.wait();
    let mid_state = app.message_service.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying during post-generation, not stuck on Narrating"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
    let final_state = app.message_service.load_or_fresh();
    assert_eq!(
        final_state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Phase should reset to default after completion"
    );
}

#[test]
fn test_narration_saved_before_quantifying_phase() {
    use std::thread;

    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let quantifier_provider = Arc::new(MockBackend::default()) as Arc<dyn LlmProvider>;
    let base_agent =
        QuantifierAgent::with_provider("quantifier".to_string(), quantifier_provider.clone());
    let entered = Arc::new(Barrier::new(2));
    let release = Arc::new(Barrier::new(2));
    let sync_agent = Box::new(SyncQuantifierAgent {
        inner: base_agent,
        entered: entered.clone(),
        release: release.clone(),
    });
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        AgentRegistry::with_agent(sync_agent),
    );
    let app = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service();
    let app_for_thread = app.clone();

    let handle = thread::spawn(move || {
        app_for_thread.pipeline.execute_action("test".to_string());
    });

    entered.wait();
    let messages = app.message_service.load_messages().unwrap();
    let narration_count = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Narration)
        .count();
    assert!(
        narration_count >= 1,
        "Narration should be saved before quantifier completes"
    );
    let mid_state = app.message_service.load_or_fresh();
    assert_eq!(
        mid_state.narrative.input_buffer.phase,
        GenerationPhase::Quantifying,
        "Phase should be Quantifying while quantifier runs"
    );

    release.wait();
    handle.join().expect("Action thread should complete");
}
