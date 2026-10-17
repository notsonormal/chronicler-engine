//! Unit tests for retrigger entry path.

use std::sync::Arc;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::{Storage, TestOverride};
use super::action_tests::app_with_second_game;
use crate::application::agents::registry::AgentRegistry;
use crate::application::errors::ProcessActionResult;
use crate::application::pipeline::action_pipeline::retry_tests::{
    add_input_and_save, insert_message_with_swipe, save_pre_main, setup_event_flow,
};
use crate::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::{
    active_game_of, make_test_pipeline_app_with_storage as make_test_app_with_storage,
    make_test_pipeline_with_backends, make_test_recorder, TestAppBuilder, TestDataBuilder,
};

#[tokio::test]
async fn test_retrigger_event_cancels_cleanly() {
    let (app, storage) = make_test_app_with_storage();

    let _input_id = add_input_and_save(&app, &storage, "test input");
    let _pre_main_id = save_pre_main(&app, &storage);

    let mut pre_event_state = app.message_service.load_or_fresh(active_game_of(&app));
    pre_event_state.narrative.last_trigger =
        Some(crate::test_support::TestStoredTriggerContext::standard());
    pre_event_state.add_message("Main narration".to_string(), MessageType::Narration);
    let snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &pre_event_state,
        );
    let pre_event_id = storage
        .save_snapshot(active_game_of(&app), &snapshot)
        .unwrap();
    if let Some(last) = pre_event_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(pre_event_id));
        insert_message_with_swipe(&app, &storage, last);
    }

    let mut final_state = pre_event_state;
    final_state.add_message("Event narration".to_string(), MessageType::Narration);
    final_state
        .narrative
        .history
        .last_mut()
        .unwrap()
        .set_event_header(Some("Event".to_string()));
    let final_snapshot =
        crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
            &final_state,
        );
    let final_id = storage
        .save_snapshot(active_game_of(&app), &final_snapshot)
        .unwrap();
    if let Some(last) = final_state.narrative.history.last_mut() {
        last.set_snapshot_id(Some(final_id));
        insert_message_with_swipe(&app, &storage, last);
    }

    app.shutdown_token.cancel();

    app.pipeline.retrigger_event(active_game_of(&app));

    let state = app.message_service.load_or_fresh(active_game_of(&app));
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancelled retrigger should reset status to Idle, got {:?}",
        state.narrative.input_buffer.status
    );
    assert_eq!(
        state.narrative.input_buffer.phase,
        GenerationPhase::default(),
        "Cancelled retrigger should reset phase to default"
    );
}

#[tokio::test]
async fn test_retrigger_event_cancelled_at_phase_boundary() {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::thread;

    let mock_backend = Arc::new(MockBackend::default().with_trigger_delay(200));
    let narrator_recorder = make_test_recorder(Arc::clone(&mock_backend) as Arc<_>);
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::default_test()
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    // The flipper switches the app to a real second Game while the work runs.
    let (initial_game_id, other_game_id) = app_with_second_game(&app);
    let catalogue_for_thread = app.game_catalogue.clone();
    let backend_for_thread = Arc::clone(&mock_backend);

    let flipper = thread::spawn(move || {
        while !backend_for_thread.trigger_started.load(Ordering::SeqCst) {
            std::hint::spin_loop();
        }
        catalogue_for_thread
            .switch_game(other_game_id.0)
            .expect("the other Game row exists");
    });

    app.pipeline.retrigger_event(initial_game_id);

    flipper
        .join()
        .expect("game-id flipper thread should complete");

    let state = app.message_service.load_or_fresh(active_game_of(&app));
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "Cancelled retrigger must reset status to Idle, got {:?}",
        state.narrative.input_buffer.status
    );
}

#[tokio::test]
async fn test_retrigger_event_emits_error_on_world_fetch_failure() {
    let data = TestDataBuilder::default_test().build();
    let (storage, handle, game_id) = {
        let base = Storage::new_in_memory();
        let game_id = data.seed_into(&base);
        let (storage, handle) = base.with_test_failures();
        (storage, handle, game_id)
    };
    handle.set(
        "get_world",
        TestOverride::internal("simulated get_world failure"),
    );
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let agent_registry = AgentRegistry::default();
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        agent_registry,
    );
    let (app, storage) = TestAppBuilder::with_data(data)
        .storage(Arc::new(storage))
        .skip_seeding(true)
        .active_game(game_id)
        .pipeline(service)
        .build_service_with_storage();

    setup_event_flow(&app, &storage);

    app.pipeline.retrigger_event(active_game_of(&app));

    let state = app.message_service.load_or_fresh(active_game_of(&app));
    let msg = match &state.narrative.input_buffer.status {
        GenerationStatus::Error(failure) => failure.raw.clone(),
        other => {
            panic!("expected GenerationStatus::Error on retrigger fetch failure, got {other:?}")
        }
    };
    assert!(
        msg.contains("simulated get_world failure"),
        "expected FetchFailed(msg) to flow through finalize_phase_error, got: {msg}"
    );
}

#[tokio::test]
async fn test_retrigger_returns_concurrent_generation_when_gate_busy() {
    let (app, _storage) = TestAppBuilder::default_test()
        .last_trigger(crate::test_support::TestStoredTriggerContext::standard())
        .log("Main narration", MessageType::Narration)
        .build_service_with_storage();

    let game_id = active_game_of(&app);
    let mut state = app.message_service.load_or_fresh(active_game_of(&app));
    let (_, _, claim) = app
        .generation_gate
        .try_claim(game_id, &mut state, app.message_service.as_ref())
        .expect("pre-claim should succeed");
    assert!(matches!(claim, ProcessActionResult::Started));

    let result = app
        .pipeline
        .retrigger(&app.generation_gate, active_game_of(&app));

    assert!(
        matches!(result, Ok(ProcessActionResult::ConcurrentGeneration)),
        "retrigger() should return Ok(ConcurrentGeneration) when gate is busy, got {result:?}"
    );
}

#[tokio::test]
async fn test_retrigger_returns_shutting_down_when_token_cancelled() {
    let (app, _storage) = make_test_app_with_storage();
    app.shutdown_token.cancel();

    let result = app
        .pipeline
        .retrigger(&app.generation_gate, active_game_of(&app));
    assert!(
        matches!(result, Ok(ProcessActionResult::ShuttingDown)),
        "retrigger() should return Ok(ShuttingDown) when token is cancelled, got {result:?}"
    );
}

#[test]
fn test_retrigger_heals_stale_status_before_validation_error() {
    let (app, storage) = make_test_app_with_storage();

    let mut state = app.message_service.load_or_fresh(active_game_of(&app));
    state.narrative.input_buffer.status = GenerationStatus::Generating;
    state.narrative.input_buffer.phase = GenerationPhase::Narrating;
    storage
        .save_snapshot(
            active_game_of(&app),
            &crate::domain::model::state::game_state_snapshot::GameStateSnapshot::from_game_state(
                &state,
            ),
        )
        .expect("save stale snapshot should succeed");

    let result = app
        .pipeline
        .retrigger(&app.generation_gate, active_game_of(&app));

    assert!(
        result.is_err(),
        "retrigger should fail with no trigger context, got {result:?}"
    );
    let persisted = app
        .message_service
        .load_expecting_valid_state(active_game_of(&app))
        .expect("the healed snapshot should load");
    assert_eq!(
        persisted.narrative.input_buffer.status,
        GenerationStatus::Idle,
        "stale Generating should be healed before the validation error returns"
    );
}
