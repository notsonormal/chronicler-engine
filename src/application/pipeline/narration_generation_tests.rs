//! Unit tests for NarrationGeneration — the narrate-and-persist prefix.

use std::sync::Arc;

use super::narration_generation::{GenerationInputs, ImpersonateInputs, NarrationGeneration};
use super::pipeline_run::PipelineRun;

use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::application::pipeline::PhaseError;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::state::generation_status::GenerationStatus;
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::{
    make_test_pipeline_with_mock_quantifier, make_test_recorder, TestAppBuilder, TestDataBuilder,
};

fn mock_quantifier() -> Arc<dyn LlmProvider> {
    Arc::new(MockBackend::default())
}

fn make_app_with_narrations(
    narrations: Vec<String>,
) -> (crate::adapters::driving::http::AppState, Arc<Storage>) {
    let narrator = Arc::new(MockBackend::default().with_narrations(narrations));
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(narrator as Arc<dyn LlmProvider>),
        mock_quantifier(),
    );
    TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage()
}

fn free_inputs(text: &str) -> GenerationInputs {
    GenerationInputs {
        input: text.to_string(),
        guide: None,
        impersonate: None,
    }
}

#[test]
fn test_run_free_action_adds_narration_message() {
    let (app, _storage) = make_app_with_narrations(vec!["The door creaks.".to_string()]);
    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();

    let outcome = NarrationGeneration::new(&pipeline_run, free_inputs("look around"))
        .run(&mut state)
        .expect("free-action prefix should succeed");

    assert_eq!(outcome.narration_text, "The door creaks.");
    assert_eq!(outcome.backend_name, "Mock");
    assert_eq!(outcome.model_name, "mock");

    let last = state.narrative.history.last().expect("narration added");
    assert_eq!(last.message_type, MessageType::Narration);
    assert_eq!(last.text(), "The door creaks.");
}

#[test]
fn test_run_guided_generation_consumes_guide() {
    let (app, _storage) = make_app_with_narrations(vec!["Ominous silence.".to_string()]);
    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();

    let inputs = GenerationInputs {
        input: String::new(),
        guide: Some("make it ominous".to_string()),
        impersonate: None,
    };
    let outcome = NarrationGeneration::new(&pipeline_run, inputs)
        .run(&mut state)
        .expect("guided prefix should succeed");

    assert_eq!(outcome.narration_text, "Ominous silence.");
    let last = state.narrative.history.last().expect("narration added");
    assert_eq!(last.message_type, MessageType::Narration);
}

#[test]
fn test_run_impersonate_adds_input_message() {
    let (app, _storage) = make_app_with_narrations(vec!["I act wary.".to_string()]);
    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();

    let inputs = GenerationInputs {
        input: String::new(),
        guide: None,
        impersonate: Some(ImpersonateInputs {
            direction: Some("act wary".to_string()),
            preset_id: None,
        }),
    };
    let outcome = NarrationGeneration::new(&pipeline_run, inputs)
        .run(&mut state)
        .expect("impersonate prefix should succeed");

    assert_eq!(outcome.narration_text, "I act wary.");
    let last = state.narrative.history.last().expect("input added");
    assert_eq!(
        last.message_type,
        MessageType::Input,
        "impersonate output is saved as a player-voiced Input"
    );
    assert_eq!(last.text(), "I act wary.");
}

#[test]
fn test_run_bundle_load_failure_returns_fetch_failed() {
    let data = TestDataBuilder::default_test().build();
    let (base_storage, handle) = {
        let base = Storage::new_in_memory();
        data.seed_into(&base);
        base.with_test_failures()
    };
    handle.set(
        "get_world",
        TestOverride::internal("simulated get_world failure"),
    );
    let narrator = Arc::new(MockBackend::default());
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(narrator as Arc<dyn LlmProvider>),
        mock_quantifier(),
    );
    let (app, _storage) = TestAppBuilder::with_data(data)
        .storage(Arc::new(base_storage))
        .skip_seeding(true)
        .pipeline(pipeline)
        .build_service_with_storage();

    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();

    let outcome = NarrationGeneration::new(&pipeline_run, free_inputs("look")).run(&mut state);
    match outcome {
        Err(PhaseError::FetchFailed(msg)) => {
            assert!(
                msg.contains("simulated get_world failure"),
                "unexpected FetchFailed message: {msg}"
            );
        }
        other => panic!("expected FetchFailed, got {other:?}"),
    }
}

#[test]
fn test_run_room_not_found_sets_error_status() {
    let (app, _storage) = make_app_with_narrations(vec!["ignored".to_string()]);
    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();
    state.movement.current_room_id = "non_existent_room".to_string();

    let outcome = NarrationGeneration::new(&pipeline_run, free_inputs("look")).run(&mut state);
    match outcome {
        Err(PhaseError::NarratorFailed(msg)) => {
            assert_eq!(msg, "Room not found");
        }
        other => panic!("expected NarratorFailed, got {other:?}"),
    }
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Error("Room not found".to_string()),
        "set_error must record the failure on the in-flight state"
    );
}

#[test]
fn test_run_missing_preset_sets_error_status() {
    let (app, storage) = make_app_with_narrations(vec!["ignored".to_string()]);
    let active_preset_id = {
        let settings = app
            .pipeline
            .settings
            .read()
            .unwrap_or_else(|e| e.into_inner());
        app.pipeline.storage.active_system_preset_id(&settings)
    };
    storage
        .delete_preset(&active_preset_id)
        .expect("setup: delete active system preset");

    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();

    let outcome = NarrationGeneration::new(&pipeline_run, free_inputs("look")).run(&mut state);
    match outcome {
        Err(PhaseError::NarratorFailed(msg)) => {
            assert_eq!(msg, "Active system preset not found");
        }
        other => panic!("expected NarratorFailed, got {other:?}"),
    }
    assert_eq!(
        state.narrative.input_buffer.status,
        GenerationStatus::Error("Active system preset not found".to_string()),
    );
}

#[test]
fn test_run_narrator_error_sets_error_status() {
    let narrator = Arc::new(MockBackend::default().with_fail());
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(narrator as Arc<dyn LlmProvider>),
        mock_quantifier(),
    );
    let (app, _storage) = TestAppBuilder::default_test()
        .pipeline(pipeline)
        .build_service_with_storage();

    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();

    let outcome = NarrationGeneration::new(&pipeline_run, free_inputs("look")).run(&mut state);
    match outcome {
        Err(PhaseError::NarratorFailed(msg)) => {
            assert!(
                msg.contains("configured_failure"),
                "unexpected narrator error: {msg}"
            );
        }
        other => panic!("expected NarratorFailed, got {other:?}"),
    }
}

#[test]
fn test_run_game_changed_returns_cancelled() {
    let (app, _storage) = make_app_with_narrations(vec!["ignored".to_string()]);
    // A game switch before run construction: the phase-boundary check reads
    // the new current game and cancels.
    let game1 = app.pipeline.storage.current_game_id();
    let game2 = app
        .game_catalogue
        .create_game("test", "test_player")
        .expect("create_game should succeed");
    assert_ne!(game2, game1, "create_game must produce a distinct game id");
    let pipeline_run = PipelineRun::new(&app.pipeline, game1);
    let mut state = app.message_service.load_or_fresh();
    let history_len_before = state.narrative.history.len();

    let outcome = NarrationGeneration::new(&pipeline_run, free_inputs("look")).run(&mut state);
    assert!(
        matches!(outcome, Err(PhaseError::Cancelled)),
        "expected Cancelled, got {outcome:?}"
    );
    assert_eq!(
        state.narrative.history.len(),
        history_len_before,
        "cancelled generation must not add a message"
    );
}

#[test]
fn test_run_save_failure_returns_persist_failed() {
    let data = TestDataBuilder::default_test().build();
    let (base_storage, handle) = {
        let base = Storage::new_in_memory();
        data.seed_into(&base);
        base.with_test_failures()
    };
    let base_storage = Arc::new(base_storage);
    let narrator =
        Arc::new(MockBackend::default().with_narrations(vec!["Fresh text.".to_string()]));
    let pipeline = make_test_pipeline_with_mock_quantifier(
        Arc::new(Storage::new_in_memory()),
        make_test_recorder(narrator as Arc<dyn LlmProvider>),
        mock_quantifier(),
    );
    let (app, _storage) = TestAppBuilder::with_data(data)
        .storage(Arc::clone(&base_storage))
        .skip_seeding(true)
        .pipeline(pipeline)
        .build_service_with_storage();

    let started_for = app.pipeline.storage.current_game_id();
    let pipeline_run = PipelineRun::new(&app.pipeline, started_for);
    let mut state = app.message_service.load_or_fresh();

    handle.set(
        "insert_swipe",
        TestOverride::internal("simulated insert_swipe failure"),
    );
    let outcome = NarrationGeneration::new(&pipeline_run, free_inputs("look")).run(&mut state);
    handle.clear("insert_swipe");

    match outcome {
        Err(PhaseError::PersistFailed { label, source }) => {
            assert_eq!(label, "pre-quantifier narration");
            assert!(
                source
                    .to_string()
                    .contains("simulated insert_swipe failure"),
                "unexpected persist source: {source}"
            );
        }
        other => panic!("expected PersistFailed, got {other:?}"),
    }
    let reloaded = app.message_service.load_or_fresh();
    assert!(
        matches!(
            reloaded.narrative.input_buffer.status,
            GenerationStatus::Error(_)
        ),
        "persist_snapshot_or_err must record the save failure, got {:?}",
        reloaded.narrative.input_buffer.status
    );
}
