//! Pipeline-flow tests for options generation: on-demand `/options`,
//! the always-on turn-end hook, and their failure fallbacks.

use std::sync::Arc;

use super::action_tests::wait_for_gate_idle;
use crate::adapters::driven::llm::providers::MockBackend;
use crate::adapters::driven::storage::Storage;
use crate::application::agents::options::OptionsAgent;
use crate::application::agents::registry::AgentRegistry;
use crate::application::ports::llm_provider::LlmProvider;
use crate::domain::model::action::Action;
use crate::domain::model::state::message_types::MessageType;
use crate::test_support::{make_test_pipeline_with_backends, make_test_recorder, TestAppBuilder};

const TAGGED_RESPONSE: &str = "<suggestion>Search the desk</suggestion>\
<suggestion>Question the guard</suggestion><suggestion>Leave the hall</suggestion>";

pub(super) fn tagged_options_provider() -> Arc<dyn LlmProvider> {
    Arc::new(MockBackend::new().with_prompt_responses(vec![TAGGED_RESPONSE.to_string()]))
}

pub(super) fn make_app(
    options_provider: Option<Arc<dyn LlmProvider>>,
) -> (crate::adapters::driving::http::AppState, Arc<Storage>) {
    let narrator_recorder = make_test_recorder(Arc::new(MockBackend::default()));
    let mut registry = AgentRegistry::default();
    if let Some(provider) = options_provider {
        registry.add_agent(Box::new(OptionsAgent::with_provider(
            "options".to_string(),
            provider,
        )));
    }
    let service = make_test_pipeline_with_backends(
        Arc::new(Storage::new_in_memory()),
        narrator_recorder,
        registry,
    );
    TestAppBuilder::default_test()
        .pipeline(service)
        .build_service_with_storage()
}

pub(super) fn set_always_on(storage: &Storage, on: bool) {
    let game_id = storage.current_game_id();
    let mut game = storage
        .require_game(game_id)
        .expect("seeded game row must exist");
    game.options_always_on = on;
    storage
        .update_game_config(&game)
        .expect("update_game_config should succeed");
}

pub(super) fn set_prior_options(app: &crate::adapters::driving::http::AppState, options: &[&str]) {
    let mut state = app.message_service.load_or_fresh();
    state.narrative.current_options = options.iter().map(|s| s.to_string()).collect();
    app.message_service
        .save_state(&state)
        .expect("save_state should succeed");
}

pub(super) fn current_options(app: &crate::adapters::driving::http::AppState) -> Vec<String> {
    app.message_service
        .load_or_fresh()
        .narrative
        .current_options
}

fn system_messages_contain(app: &crate::adapters::driving::http::AppState, needle: &str) -> bool {
    app.message_service
        .load_or_fresh()
        .narrative
        .history()
        .iter()
        .any(|entry| entry.message_type == MessageType::System && entry.text.contains(needle))
}

/// The seeded test app reloads with an empty messages table (the snapshot
/// excludes messages), so on-demand tests run one narration turn first to
/// create real scene history.

#[test]
fn test_always_on_generates_options_after_narration_turn() {
    let (app, storage) = make_app(Some(tagged_options_provider()));
    set_always_on(&storage, true);

    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);

    let options = current_options(&app);
    assert_eq!(options.len(), 3, "always-on turn must install a fresh set");
    assert!(options.contains(&"Search the desk".to_string()));
}

#[test]
fn test_always_on_no_agent_clears_set_and_surfaces_message() {
    // Registry has no options agent; the always-on rewrite must still run —
    // clear the stale set and surface the unavailable-agent system message.
    let (app, storage) = make_app(None);
    set_always_on(&storage, true);
    set_prior_options(&app, &["Stale option"]);

    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);

    assert!(
        current_options(&app).is_empty(),
        "the set must clear even when no agent is registered"
    );
    assert!(system_messages_contain(
        &app,
        "Options agent is not available"
    ));
}

#[test]
fn test_always_on_skipped_after_impersonate_turn() {
    let (app, storage) = make_app(Some(tagged_options_provider()));
    set_always_on(&storage, true);
    set_prior_options(&app, &["Stale option"]);

    app.pipeline
        .execute_action_with_inputs(String::new(), true, None);

    assert_eq!(
        current_options(&app),
        vec!["Stale option".to_string()],
        "impersonate output IS the player acting — the set must be untouched"
    );
}

#[test]
fn test_toggle_off_clears_set_after_turn() {
    let (app, _storage) = make_app(Some(tagged_options_provider()));
    set_prior_options(&app, &["Stale option"]);

    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);

    assert!(
        current_options(&app).is_empty(),
        "a non-impersonate turn with the toggle off clears the stale set"
    );
}

#[test]
fn test_always_on_failure_clears_set_and_surfaces_system_message() {
    let (app, storage) = make_app(Some(Arc::new(MockBackend::new().with_fail())));
    set_always_on(&storage, true);
    set_prior_options(&app, &["Stale option"]);

    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);

    assert!(current_options(&app).is_empty());
    assert!(system_messages_contain(&app, "Options generation failed"));
}

#[tokio::test]
async fn test_process_options_replaces_set() {
    let (app, _storage) = make_app(Some(tagged_options_provider()));
    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);
    set_prior_options(&app, &["Old option"]);
    let game_id = app.pipeline.storage.current_game_id();

    let result = app
        .pipeline
        .process_action(&app.generation_gate, Action::Options)
        .expect("on-demand options claim should succeed");
    assert!(matches!(
        result,
        crate::application::errors::ProcessActionResult::Started
    ));

    wait_for_gate_idle(&app.generation_gate, game_id).await;

    let options = current_options(&app);
    assert_eq!(options.len(), 3);
    assert!(!options.contains(&"Old option".to_string()));
}

#[tokio::test]
async fn test_process_options_during_shutdown_returns_shutting_down() {
    // The token is consulted at claim time (claim_and_spawn's first guard);
    // a cancelled token must refuse the claim without touching the set or
    // the gate. The mid-refresh Cancelled branch (game-id switch) has no
    // deterministic seam, so the claim-time guard is the covered path.
    let (app, _storage) = make_app(Some(tagged_options_provider()));
    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);
    set_prior_options(&app, &["Old option"]);

    app.shutdown_token.cancel();

    let result = app
        .pipeline
        .process_action(&app.generation_gate, Action::Options)
        .expect("process_action must not error on a cancelled token");
    assert!(
        matches!(
            result,
            crate::application::errors::ProcessActionResult::ShuttingDown
        ),
        "expected ShuttingDown, got {result:?}"
    );
    assert_eq!(
        current_options(&app),
        vec!["Old option".to_string()],
        "a refused claim must leave the set untouched"
    );
    assert!(
        !system_messages_contain(&app, "Options"),
        "a refused claim must not surface an options system message"
    );
}

#[test]
fn test_process_options_empty_history_rejected() {
    // A freshly seeded app has no persisted message rows, so scene history
    // does not exist yet — /options must be rejected before any claim.
    let (app, _storage) = make_app(Some(tagged_options_provider()));

    let state = app.message_service.load_or_fresh();
    assert!(
        state.narrative.history().is_empty(),
        "precondition: seeded app starts without scene history"
    );

    let result = app
        .pipeline
        .process_action(&app.generation_gate, Action::Options)
        .expect_err("options without scene history must be rejected");
    assert!(
        result.to_string().contains("No scene"),
        "expected a validation error, got {result}"
    );
}

#[tokio::test]
async fn test_process_options_failure_keeps_prior_set() {
    let (app, _storage) = make_app(Some(Arc::new(MockBackend::new().with_fail())));
    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);
    set_prior_options(&app, &["Old option"]);
    let game_id = app.pipeline.storage.current_game_id();

    app.pipeline
        .process_action(&app.generation_gate, Action::Options)
        .expect("claim should succeed before generation fails");

    wait_for_gate_idle(&app.generation_gate, game_id).await;

    assert_eq!(
        current_options(&app),
        vec!["Old option".to_string()],
        "on-demand failure keeps the prior set — the scene did not change"
    );
    assert!(system_messages_contain(&app, "Options generation failed"));
}

#[tokio::test]
async fn test_no_options_agent_surfaces_unavailable_message() {
    let (app, _storage) = make_app(None);
    app.pipeline
        .execute_action_with_inputs("look".to_string(), false, None);
    set_prior_options(&app, &["Old option"]);
    let game_id = app.pipeline.storage.current_game_id();

    app.pipeline
        .process_action(&app.generation_gate, Action::Options)
        .expect("claim should succeed");

    wait_for_gate_idle(&app.generation_gate, game_id).await;

    assert_eq!(current_options(&app), vec!["Old option".to_string()]);
    assert!(system_messages_contain(
        &app,
        "Options agent is not available"
    ));
}
