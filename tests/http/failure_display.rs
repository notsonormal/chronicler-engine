//! HTTP E2E tests for the failure display, tagged against `docs/specs/failure_display.md`.

use std::sync::Arc;

use chronicler_engine::domain::model::agent::Role;
use chronicler_engine::test_support::active_game_of;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::util::ServiceExt;
use chronicler_engine::adapters::driven::storage::{StorageCall, TestOverride, TxWrite};
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::domain::model::llm_backend::LlmBackendType;
use chronicler_engine::domain::model::settings::{AppSettings, LlmProviderConfig};
use chronicler_engine::domain::model::state::generation_status::{
    GenerationFailure, GenerationFailureKind, Record,
};
use chronicler_engine::error::EngineError;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::fixtures::llm_message;
use chronicler_engine::test_support::{
    configured_runner, test_connection, ScriptedModel, TestAppBuilder,
};
use chronicler_engine::test_support::{empty_sqlite_storage, set_game_config_row};

use crate::SettingsTestGuard;
use crate::support::app_wiring::{app_with_model, app_with_model_and_storage};
use crate::support::http_assertions::status_error_message;
use crate::support::http_fixtures::{break_start_room, corrupt_start_state, story_of};
use crate::support::http_requests::{fetch_body, post_action, post_form, wait_for_jobs, wait_settled};
// [docs/specs/failure_display.md] SCENARIO: 38.1
#[tokio::test]
async fn test_degraded_role_raises_header_banner_and_clears_on_recovery() {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let app = build_router(state);

    storage
        .save_llm_message(&llm_message(
            "quantifier",
            "mock",
            Some("fallback NPC IDs used"),
            -10,
        ))
        .unwrap();
    storage
        .save_llm_message(&llm_message("narrator", "mock", None, -20))
        .unwrap();

    let body = fetch_body(&app, "/fragment/header").await;
    assert!(
        body.contains("failure-banner"),
        "a degraded Quantifier must raise the banner: {body}"
    );
    assert!(
        body.contains("Quantifier failed") && body.contains("using fallback NPC IDs"),
        "the banner must name the role and what the engine did instead: {body}"
    );
    assert!(
        body.contains("(engine-wide role health)"),
        "the banner must state that role health covers every game: {body}"
    );
    assert!(
        body.contains("error-details-toggle"),
        "the banner must offer the Details disclosure: {body}"
    );
    assert!(
        body.contains("Mock mock"),
        "the disclosure must list per-role backend/model: {body}"
    );
    assert!(
        body.contains(r#"<span class="error-role-ok">last call succeeded</span>"#),
        "the disclosure must report the Narrator's successful newest attempt as such: {body}"
    );
    assert_eq!(
        body.matches("last call succeeded").count(),
        1,
        "only the role with a successful attempt may read as a success: {body}"
    );
    assert_eq!(
        body.matches("No calls yet").count(),
        2,
        "the two roles with no recorded attempt must read as no calls: {body}"
    );
    assert!(
        body.contains(r#"<pre class="error-detail-raw">fallback NPC IDs used</pre>"#),
        "the raw failure text must be reachable inside the disclosure: {body}"
    );

    storage
        .save_llm_message(&llm_message("quantifier", "mock", None, 0))
        .unwrap();
    let body = fetch_body(&app, "/fragment/header").await;
    assert!(
        !body.contains("error-disclosure"),
        "a later success must clear the banner: {body}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.8
#[tokio::test]
async fn test_degraded_trigger_role_raises_the_banner() {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let app = build_router(state);

    storage
        .save_llm_message(&llm_message(
            "trigger",
            "mock",
            Some("trigger call timed out"),
            -10,
        ))
        .unwrap();

    let body = fetch_body(&app, "/fragment/header").await;
    assert!(
        body.contains("failure-banner"),
        "a degraded Trigger must raise the banner: {body}"
    );
    assert!(
        body.contains("Trigger failed") && body.contains("skipping the trigger"),
        "the banner must name the Trigger role and what the engine did instead: {body}"
    );
    assert!(
        body.contains(r#"<span class="error-role-name">Trigger</span>"#),
        "the disclosure must list the Trigger role: {body}"
    );
    assert!(
        body.contains(r#"<pre class="error-detail-raw">trigger call timed out</pre>"#),
        "the raw failure text must be reachable inside the disclosure: {body}"
    );
    assert!(
        body.contains(
            r#"<span class="subtab-marker-slot" id="subtab-connections-marker" hx-swap-oob="true"></span>"#
        ),
        "a Trigger failure has no Settings row to mark: {body}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.6
#[tokio::test]
async fn test_header_poll_keeps_the_settings_role_health_cells_current() {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let app = build_router(state);

    let settings = fetch_body(&app, "/fragment/settings").await;
    for cell_id in [
        "role-health-narrator",
        "role-health-quantifier",
        "subtab-connections-marker",
    ] {
        assert!(
            settings.contains(&format!(r#"id="{cell_id}""#)),
            "the Settings panel must render the cell '{cell_id}' the header poll refreshes: {settings}"
        );
    }
    assert!(
        !settings.contains("hx-swap-oob"),
        "the panel's own cells are not out-of-band swaps: {settings}"
    );

    let header = fetch_body(&app, "/fragment/header").await;
    assert!(
        header.contains(
            r#"<span class="role-health-slot" id="role-health-narrator" hx-swap-oob="true"><span class="role-health unknown">No calls yet</span>"#
        ),
        "a role with no recorded attempt must swap in as no calls: {header}"
    );

    storage
        .save_llm_message(&llm_message(
            "quantifier",
            "mock",
            Some("fallback NPC IDs used"),
            -10,
        ))
        .unwrap();
    storage
        .save_llm_message(&llm_message("narrator", "mock", None, -20))
        .unwrap();

    let header = fetch_body(&app, "/fragment/header").await;
    assert!(
        header.contains(
            r#"<span class="role-health-slot" id="role-health-quantifier" hx-swap-oob="true"><span class="role-health degraded">Degraded</span>"#
        ),
        "the degraded role's cell must swap in as Degraded: {header}"
    );
    assert!(
        header.contains(
            r#"<span class="role-health-slot" id="role-health-narrator" hx-swap-oob="true"><span class="role-health healthy">Healthy</span>"#
        ),
        "a role whose newest attempt succeeded must swap in as Healthy: {header}"
    );
    assert!(
        header.contains(
            r#"<span class="subtab-marker-slot" id="subtab-connections-marker" hx-swap-oob="true"><span class="subtab-degraded-marker""#
        ),
        "the Connections sub-tab's warning marker must ride along: {header}"
    );

    storage
        .save_llm_message(&llm_message("quantifier", "mock", None, 0))
        .unwrap();
    let header = fetch_body(&app, "/fragment/header").await;
    assert!(
        header.contains(
            r#"<span class="role-health-slot" id="role-health-quantifier" hx-swap-oob="true"><span class="role-health healthy">Healthy</span>"#
        ),
        "a later success must swap the cell back to Healthy: {header}"
    );
    assert!(
        header.contains(
            r#"<span class="subtab-marker-slot" id="subtab-connections-marker" hx-swap-oob="true"></span>"#
        ),
        "a recovered role must clear the marker: {header}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.2
#[tokio::test]
async fn test_failed_poll_answers_non_2xx_and_reswap_none() {
    let storage = Arc::new(empty_sqlite_storage());
    let state = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_service();
    break_start_room(&storage, active_game_of(&state), "non_existent_room");
    let app = build_router(state);

    let request = Request::builder()
        .uri("/fragment/visual-sidebar")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "a failed poll must answer non-2xx, not 200 with an error body"
    );
    assert_eq!(
        response
            .headers()
            .get("HX-Reswap")
            .and_then(|value| value.to_str().ok()),
        Some("none"),
        "a failed poll must ask the client to swap nothing"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.3
#[tokio::test]
async fn test_generation_error_clamps_with_raw_text_in_disclosure() {
    let state = TestAppBuilder::default_test()
        .failure(GenerationFailure::new(
            GenerationFailureKind::Unreachable,
            "LLM Error: connection refused",
        ))
        .build_service();
    let app = build_router(state);

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The language model could not be reached."),
        "the clamped line carries a short user-facing message: {body}"
    );
    assert!(
        body.contains(r#"<pre class="error-detail-raw">LLM Error: connection refused</pre>"#),
        "the raw transport message must be reachable in the disclosure: {body}"
    );
    assert!(
        !body.contains(r#"<span class="status error">Error: LLM Error"#),
        "the raw transport message must never be rendered inline: {body}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.5
#[tokio::test]
async fn test_missing_preset_failure_clamps_with_the_missing_data_line() {
    let state = TestAppBuilder::default_test()
        .failure(GenerationFailure::unreadable(
            Record::Preset,
            &EngineError::PresetNotFound("system_default".to_string()),
        ))
        .build_service();
    let app = build_router(state);

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The data for this request is missing: the preset."),
        "a missing preset keeps its own line, not the general one: {body}"
    );
    assert!(
        !body.contains("The engine failed with an internal error."),
        "the general line belongs to failures no kind names: {body}"
    );
    assert!(
        body.contains(
            r#"<pre class="error-detail-raw">Prompt preset not found: system_default</pre>"#
        ),
        "the missing preset id must be reachable in the disclosure: {body}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.7
#[tokio::test]
async fn test_spent_token_budget_clamps_with_its_own_line() {
    let raw = "LLM returned no answer after spending its whole token budget (2048 completion tokens, 2041 of them reasoning)";
    let state = TestAppBuilder::default_test()
        .failure(GenerationFailure::new(
            GenerationFailureKind::TokenBudgetSpent,
            raw,
        ))
        .build_service();
    let app = build_router(state);

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("Reasoning used the budget. Lower the reasoning effort."),
        "a spent token budget must keep its own line: {body}"
    );
    assert!(
        !body.contains("Max Tokens"),
        "the line must name only an action the dashboard offers: {body}"
    );
    assert!(
        !body.contains("The engine failed with an internal error."),
        "the general line belongs to failures no kind names: {body}"
    );
    assert!(
        !body.contains("The language model's answer could not be read."),
        "a spent budget is not an unreadable answer: {body}"
    );
    assert_eq!(
        body.matches("spending its whole token budget").count(),
        1,
        "the raw text with the token counts belongs in the disclosure only: {body}"
    );
    assert!(
        body.contains(&format!(r#"<pre class="error-detail-raw">{raw}</pre>"#)),
        "the token counts must be reachable in the disclosure: {body}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.4
#[tokio::test]
async fn test_failed_user_action_is_not_a_failed_poll() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    failures.set_write(
        TxWrite::SetGameConfig,
        TestOverride::internal("update_game_config failure"),
    );
    let storage = Arc::new(storage);
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = active_game_of(&state);

    let response = post_form(
        &app,
        &format!("/games/{id}/posture"),
        "narrative_perspective=second&narrative_tense=present",
    )
    .await;
    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "a failed user action answers non-2xx"
    );
    assert!(
        response.headers().get("HX-Reswap").is_none(),
        "a failed user action must not ask the client to swap nothing"
    );
    let body = axum::body::to_bytes(response.into_body(), 8192)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("update_game_config failure"),
        "the failure text must reach the client: {body_str}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.9
#[tokio::test]
async fn test_empty_answer_keeps_its_own_line_clamps() {
    let (app, state) = app_with_model(Arc::new(ScriptedModel::new().empty(Role::Narrator)));

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the failing Job should end"
    );
    assert!(
        wait_for_jobs(&state, 1000).await,
        "the Job should end before the status reads"
    );

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The language model gave an empty answer."),
        "an empty answer keeps its own line: {body}"
    );
    assert!(
        !body.contains("answer could not be read"),
        "an empty answer is not an unreadable answer: {body}"
    );
    let narrations = story_of(&state)
        .into_iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 1,
        "the empty answer saves no Narration, so the opening is the only one"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.10
#[tokio::test]
async fn test_contradiction_clamps_with_its_own_line_and_keeps_the_text() {
    // The start state's encounter log names a Character the World does not hold,
    // so the game rules refuse to apply the analysis. The narration itself answers
    // and is saved first.
    let storage = Arc::new(empty_sqlite_storage());
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .model(Arc::new(
            ScriptedModel::new()
                .narration("The room waits.")
                .after_narration("{\"npcs_in_room\": [], \"movement\": null}"),
        ))
        .build_with_state();
    corrupt_start_state(&storage, active_game_of(&state), |broken| {
        broken
            .npc_encounter_log
            .set_currently_meeting("a_ghost_the_world_lacks", true);
    });

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 1000).await,
        "the failing Job should end"
    );
    assert!(
        wait_for_jobs(&state, 1000).await,
        "the Job should end before the status reads"
    );

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The text was saved, but the game state could not be updated."),
        "a contradiction keeps its own line: {body}"
    );
    let raw = "npc_encounter_log references unknown NPC";
    assert!(
        status_error_message(&body).contains(raw),
        "the disclosure holds the raw internal message: {body}"
    );
    assert_eq!(
        body.matches(raw).count(),
        1,
        "the raw internal message appears once, inside the disclosure: {body}"
    );
    let narrations: Vec<_> = story_of(&state)
        .into_iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.len(),
        2,
        "the opening Narration and the Action's"
    );
    assert_eq!(
        narrations.last().map(|message| message.text().to_string()),
        Some("The room waits.".to_string()),
        "the text the Job saved stays when the state update is refused"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.11
#[tokio::test]
async fn test_save_failure_clamps_with_the_save_line_and_saves_no_narration() {
    let _guard = SettingsTestGuard::new();
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(storage),
        Arc::clone(&model),
        AppSettings::default(),
        configured_runner,
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held narration call should reach the model"
    );
    // The Input is saved before the call, so the refused write is the Swipe of
    // the Narration.
    failures.set_write(
        TxWrite::InsertSwipe,
        TestOverride::internal("the swipe write failed"),
    );
    hold.release();
    assert!(
        wait_settled(&state, 2000).await,
        "the failing Job should end"
    );
    assert!(
        wait_for_jobs(&state, 2000).await,
        "the Job should end before the status reads"
    );

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The new text could not be saved."),
        "a refused save keeps the save-failed line: {body}"
    );
    assert_eq!(
        body.matches("The new text could not be saved.").count(),
        1,
        "the line appears once: {body}"
    );
    assert!(
        status_error_message(&body).contains("the swipe write failed"),
        "the storage error behind the line is reachable in the disclosure: {body}"
    );

    let messages = story_of(&state);
    let narrations = messages
        .iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 1,
        "the opening is the only Narration: the refused save landed no text"
    );
    let inputs = messages
        .iter()
        .filter(|message| message.message_type == MessageType::Input)
        .count();
    assert_eq!(
        inputs, 1,
        "the Input saved before the failure stays in the story"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.12
#[tokio::test]
async fn test_context_overflow_clamps_with_the_prompt_too_long_line() {
    let _guard = SettingsTestGuard::new();
    // The fixed safety margin and minimum input budget together exceed this
    // window, so the prompt can never fit beside the system prompt.
    let connection = LlmProviderConfig {
        max_context_tokens: Some(512),
        ..test_connection("narration-conn", LlmBackendType::Mock)
    };
    let settings = AppSettings {
        connections: vec![connection],
        narration_connection_id: "narration-conn".into(),
        quantifier_connection_id: "narration-conn".into(),
        ..AppSettings::default()
    };
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(empty_sqlite_storage()),
        Arc::new(ScriptedModel::new().turn("You look around.")),
        settings,
        configured_runner,
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        wait_settled(&state, 2000).await,
        "the failing Job should end"
    );
    assert!(
        wait_for_jobs(&state, 2000).await,
        "the Job should end before the status reads"
    );

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The story was too long for the language model."),
        "a prompt that cannot fit keeps the prompt-too-long line: {body}"
    );
    assert!(
        !body.contains("The engine failed with an internal error."),
        "the general line belongs to failures no kind names: {body}"
    );
    assert!(
        status_error_message(&body).contains("Context overflow"),
        "the budget the prompt missed is reachable in the disclosure: {body}"
    );

    let narrations = story_of(&state)
        .iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 1,
        "the model was never asked, so the opening is the only Narration"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.13
#[tokio::test]
async fn preset_game_read_fails() {
    let _guard = SettingsTestGuard::new();
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(Role::Narrator);
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(storage),
        Arc::clone(&model),
        AppSettings::default(),
        configured_runner,
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held narration call should reach the model"
    );
    // Every read the Job made before the Quantifier preset is behind it, so the
    // refused read is the one that chooses that preset.
    failures.set(
        StorageCall::GetGame,
        TestOverride::internal("the game row is unreadable"),
    );
    hold.release();
    assert!(
        wait_settled(&state, 2000).await,
        "the failing Job should end"
    );
    assert!(
        wait_for_jobs(&state, 2000).await,
        "the Job should end before the status reads"
    );

    // The status reads the Game row, so the read works again before it asks.
    failures.clear(StorageCall::GetGame);
    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The data for this request is missing: the game."),
        "an unreadable Game row is a failure line that names the Game, not a registry preset: {body}"
    );
    assert!(
        status_error_message(&body).contains("the game row is unreadable"),
        "the storage error behind the line is reachable in the disclosure: {body}"
    );
    assert!(
        model
            .calls()
            .iter()
            .all(|call| call.role != Role::Quantifier),
        "the Quantifier is never asked with a preset the Job could not choose"
    );
    let narrations = story_of(&state)
        .iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    assert_eq!(
        narrations, 2,
        "the Narration saved before the failure stays beside the opening"
    );
}

/// A Job whose call of `held` is held while `call` starts to fail: the Job reads
/// again after the call, so the refused read is one of the steps behind it.
async fn status_after_a_read_fails_behind_a_call(call: StorageCall, held: Role) -> (String, usize) {
    let _guard = SettingsTestGuard::new();
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let model = Arc::new(ScriptedModel::new().turn("You look around."));
    let hold = model.hold(held);
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(storage),
        Arc::clone(&model),
        AppSettings::default(),
        configured_runner,
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(
        hold.wait_started(1000),
        "the held call should reach the model"
    );
    failures.set(call, TestOverride::internal("the read is refused"));
    hold.release();
    assert!(wait_settled(&state, 2000).await, "the Job should end");
    assert!(
        wait_for_jobs(&state, 2000).await,
        "the Job should end before the status reads"
    );

    // The status reads the Game row, so every read works again before it asks.
    failures.clear_all();
    let body = fetch_body(&app, "/status/generating").await;
    let narrations = story_of(&state)
        .iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .count();
    (body, narrations)
}

// [docs/specs/failure_display.md] SCENARIO: 38.14
#[tokio::test]
async fn job_read_names_record_story_and_saved_state() {
    let rows = [
        (StorageCall::StoriesEntries, Role::Narrator, "the story", 2),
        (
            StorageCall::StoriesSwipeState,
            Role::Narrator,
            "the start state",
            1,
        ),
        (
            StorageCall::StoriesSwipeState,
            Role::Quantifier,
            "the saved state",
            2,
        ),
    ];
    for (call, held, record, expected_narrations) in rows {
        let (body, narrations) = status_after_a_read_fails_behind_a_call(call, held).await;
        let line = format!("The data for this request is missing: {record}.");
        assert!(
            body.contains(&line),
            "{call:?}: the line names {record}, not the save line: {body}"
        );
        assert!(
            !body.contains("The new text could not be saved."),
            "{call:?}: a failed read is not a failed write: {body}"
        );
        assert!(
            status_error_message(&body).contains("the read is refused"),
            "{call:?}: the storage error is reachable in the disclosure: {body}"
        );
        assert_eq!(
            narrations, expected_narrations,
            "{call:?} behind the {held:?} call: the Narrations in the story log"
        );
    }
}

// [docs/specs/failure_display.md] SCENARIO: 38.14
#[tokio::test]
async fn job_read_names_record_world() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(storage),
        Arc::new(ScriptedModel::new().turn("You look around.")),
        AppSettings::default(),
        configured_runner,
    );
    // The accept reads the Game, the Persona and the start state; only the Job
    // reads the World.
    failures.set(
        StorageCall::GetWorld,
        TestOverride::internal("the world is unreadable"),
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Job should end");
    assert!(
        wait_for_jobs(&state, 2000).await,
        "the Job should end before the status reads"
    );

    failures.clear_all();
    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The data for this request is missing: the world."),
        "the line names the World: {body}"
    );
    assert!(
        status_error_message(&body).contains("the world is unreadable"),
        "the storage error is reachable in the disclosure: {body}"
    );
    let inputs = story_of(&state)
        .iter()
        .filter(|message| message.message_type == MessageType::Input)
        .count();
    assert_eq!(
        inputs, 0,
        "the room check is the first step, so no Input is saved"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.15
#[tokio::test]
async fn job_read_names_record_accept_start_state() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(storage),
        Arc::new(ScriptedModel::new().turn("You look around.")),
        AppSettings::default(),
        configured_runner,
    );
    failures.set(
        StorageCall::StoriesSwipeState,
        TestOverride::internal("the start state is unreadable"),
    );
    let before = story_of(&state).len();

    let response = post_action(&app, "look").await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let answer = axum::body::to_bytes(response.into_body(), 65536)
        .await
        .unwrap();
    let answer = String::from_utf8_lossy(&answer);
    let line = "The data for this request is missing: the start state.";
    assert!(
        answer.contains(line),
        "the answer names the start state: {answer}"
    );
    assert!(
        !answer.contains("the start state is unreadable"),
        "the storage error stays out of the answer: {answer}"
    );

    failures.clear_all();
    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains(line),
        "the status carries the line of the answer: {body}"
    );
    assert!(
        status_error_message(&body).contains("the start state is unreadable"),
        "the storage error is reachable in the disclosure: {body}"
    );
    assert_eq!(
        story_of(&state).len(),
        before,
        "a request that cannot run saves nothing"
    );
}

/// The Game picks a preset that the library does not hold. A delete through the
/// service is refused while a Game uses the preset, so the row is written directly.
fn game_picks_a_missing_preset(
    storage: &chronicler_engine::adapters::driven::storage::Storage,
    state: &chronicler_engine::adapters::driving::http::AppState,
    preset_type: chronicler_engine::domain::model::prompt_preset::PresetType,
) {
    let mut game = storage
        .get_game(active_game_of(state).0)
        .expect("the Game row reads back")
        .expect("the Game exists");
    game.set_active_preset_id(preset_type, "preset_that_is_gone".to_string());
    set_game_config_row(storage, &game);
}

// [docs/specs/failure_display.md] SCENARIO: 38.16
#[tokio::test]
async fn job_read_names_record_preset() {
    use chronicler_engine::domain::model::prompt_preset::PresetType;

    // The narrator preset is needed at the first model call; the Quantifier preset
    // behind the Narration.
    for (preset_type, narrations, quantifier_asked) in [
        (PresetType::System, 1, false),
        (PresetType::Quantifier, 2, false),
    ] {
        let storage = Arc::new(empty_sqlite_storage());
        let model = Arc::new(ScriptedModel::new().turn("You look around."));
        let (app, state, storage) = app_with_model_and_storage(
            storage,
            Arc::clone(&model),
            AppSettings::default(),
            configured_runner,
        );
        game_picks_a_missing_preset(&storage, &state, preset_type);

        assert!(post_action(&app, "look").await.status().is_success());
        assert!(wait_settled(&state, 2000).await, "the Job should end");
        assert!(
            wait_for_jobs(&state, 2000).await,
            "the Job should end before the status reads"
        );

        let body = fetch_body(&app, "/status/generating").await;
        assert!(
            body.contains("The data for this request is missing: the preset."),
            "{preset_type:?}: the line names the preset: {body}"
        );
        assert!(
            status_error_message(&body).contains("preset_that_is_gone"),
            "{preset_type:?}: the missing preset id is reachable in the disclosure: {body}"
        );
        let saved = story_of(&state)
            .iter()
            .filter(|message| message.message_type == MessageType::Narration)
            .count();
        assert_eq!(
            saved, narrations,
            "{preset_type:?}: the Narrations of the story"
        );
        assert_eq!(
            model
                .calls()
                .iter()
                .any(|call| call.role == Role::Quantifier),
            quantifier_asked,
            "{preset_type:?}: the Quantifier is never asked with a preset the Job could not choose"
        );
    }
}

// [docs/specs/failure_display.md] SCENARIO: 38.17
#[tokio::test]
async fn state_update_refused() {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    let (app, state, _storage) = app_with_model_and_storage(
        Arc::new(storage),
        Arc::new(ScriptedModel::new().turn("You look around.")),
        AppSettings::default(),
        configured_runner,
    );
    failures.set_write(
        TxWrite::SetSwipeState,
        TestOverride::internal("the state write failed"),
    );

    assert!(post_action(&app, "look").await.status().is_success());
    assert!(wait_settled(&state, 2000).await, "the Job should end");
    assert!(
        wait_for_jobs(&state, 2000).await,
        "the Job should end before the status reads"
    );

    let body = fetch_body(&app, "/status/generating").await;
    assert!(
        body.contains("The text was saved, but the game state could not be updated."),
        "a refused state update shows the contradiction line: {body}"
    );
    assert!(
        !body.contains("The new text could not be saved."),
        "the text is saved, so the save line would be wrong: {body}"
    );
    assert!(
        status_error_message(&body).contains("the state write failed"),
        "the storage error is reachable in the disclosure: {body}"
    );
    let narrations: Vec<_> = story_of(&state)
        .into_iter()
        .filter(|message| message.message_type == MessageType::Narration)
        .collect();
    assert_eq!(
        narrations.last().map(|message| message.text().to_string()),
        Some("You look around.".to_string()),
        "the Narration stays beside the opening"
    );
    assert_eq!(narrations.len(), 2);
}
