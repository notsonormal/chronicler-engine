//! HTTP E2E tests for narrator mode: world-to-game posture inheritance, mode switching, and steering availability.

use std::sync::Arc;

use chronicler_engine::adapters::driven::llm::providers::MockBackend;
use chronicler_engine::adapters::driven::storage::Storage;
use chronicler_engine::adapters::driving::http::AppState;
use chronicler_engine::application::agents::registry::AgentRegistry;
use chronicler_engine::application::ports::llm_provider::AGENT_NARRATOR;
use chronicler_engine::domain::model::settings::AppSettings;
use chronicler_engine::domain::model::state::message_types::MessageType;
use chronicler_engine::test_support::TestMap;

use crate::test_helpers::{app_with_narrator_and_registry, if_world, post_action, post_form, wait_idle};

/// App whose narrator recorder persists every assembled prompt into the
/// shared storage, so tests can assert on what the narrator received.
/// Built on the default test data (world "test", persona "test_player",
/// one Novel game set current).
fn narrated_app() -> (axum::Router, AppState, Arc<Storage>) {
    app_with_narrator_and_registry(
        Arc::new(MockBackend::default()),
        AgentRegistry::default(),
        AppSettings::default(),
    )
}

fn narrator_prompts(storage: &Storage) -> Vec<(String, String)> {
    storage
        .list_latest_llm_messages(50)
        .expect("list_latest_llm_messages should succeed")
        .into_iter()
        .filter(|m| m.agent_name == AGENT_NARRATOR)
        .map(|m| (m.system_prompt, m.user_prompt))
        .collect()
}

fn novel_prompt_marker() -> &'static str {
    "You are a test narrator."
}

fn if_prompt_marker() -> &'static str {
    "test interactive fiction narrator"
}

// [docs/specs/narrator_mode.md] SCENARIO: 23.1
#[tokio::test]
async fn test_if_world_game_inherits_posture_and_bundle_http() {
    let (app, state, storage) = narrated_app();
    storage
        .seed_world(&if_world(), &TestMap::single_room("start"))
        .expect("seed IF world");

    let resp = post_form(&app, "/games", "world_key=if_world&persona_key=test_player").await;
    assert!(resp.status().is_success(), "game creation should succeed");

    let resp = post_action(&app, "open the door").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await, "action should complete");

    let prompts = narrator_prompts(&storage);
    assert!(
        !prompts.is_empty(),
        "the narration must have recorded a prompt"
    );
    for (system_prompt, _) in &prompts {
        assert!(
            system_prompt.contains(if_prompt_marker()),
            "narrator system prompt must carry the IF system preset: {system_prompt}"
        );
    }
    assert!(
        prompts
            .iter()
            .any(|(_, user)| user.contains("second-person limited perspective")),
        "perspective macro must resolve to the inherited second person"
    );
    assert!(
        prompts
            .iter()
            .any(|(_, user)| user.contains("present tense narrative prose")),
        "tense macro must resolve to the inherited present tense"
    );

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 1, "one Input entry");
    assert_eq!(inputs[0].text(), "open the door");
    assert!(
        messages
            .iter()
            .any(|m| m.message_type == MessageType::Narration),
        "a Narration entry must follow"
    );
}

// [docs/specs/narrator_mode.md] SCENARIO: 23.2
#[tokio::test]
async fn test_mode_switch_retargets_system_preset_http() {
    let (app, state, storage) = narrated_app();
    let game_id = storage.current_game_id();

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    let prompts = narrator_prompts(&storage);
    assert!(
        prompts
            .iter()
            .all(|(system, _)| system.contains(novel_prompt_marker())),
        "the Novel game must narrate with the Novel preset before the switch"
    );

    let resp = post_form(
        &app,
        &format!("/games/{game_id}/mode"),
        "narrator_mode=interactive_fiction",
    )
    .await;
    assert!(resp.status().is_success(), "mode switch should succeed");

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);

    let (last_system, _) = narrator_prompts(&storage)
        .pop()
        .expect("the switched-mode narration must record a prompt");
    assert!(
        last_system.contains(if_prompt_marker()),
        "the switched game must narrate with the IF preset: {last_system}"
    );
    assert!(
        !last_system.contains(novel_prompt_marker()),
        "the Novel preset must be gone after the switch: {last_system}"
    );
}

// [docs/specs/narrator_mode.md] SCENARIO: 23.3
#[tokio::test]
async fn test_mode_switch_renders_if_bundle_and_nudges_perspective_http() {
    let (app, state, storage) = narrated_app();
    let game_id = storage.current_game_id();

    let resp = post_form(
        &app,
        &format!("/games/{game_id}/mode"),
        "narrator_mode=interactive_fiction",
    )
    .await;
    assert!(resp.status().is_success());
    let body = axum::body::to_bytes(resp.into_body(), 16384).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains(r#"value="second" selected"#),
        "the nudge must select second person: {html}"
    );
    assert!(
        html.contains(r#"value="system_if_default" selected"#),
        "the system preset picker must show the IF bundle preset: {html}"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    let (last_system, last_user) = narrator_prompts(&storage)
        .pop()
        .expect("the post-switch narration must record a prompt");
    assert!(
        last_system.contains(if_prompt_marker()),
        "the post-switch narration must use the IF preset: {last_system}"
    );
    assert!(
        last_user.contains("second-person limited perspective"),
        "the nudged perspective must reach the prompt: {last_user}"
    );
}

// [docs/specs/narrator_mode.md] SCENARIO: 23.4
#[tokio::test]
async fn test_deliberate_perspective_survives_mode_switch_http() {
    let (app, state, storage) = narrated_app();
    let game_id = storage.current_game_id();

    let resp = post_form(
        &app,
        &format!("/games/{game_id}/posture"),
        "narrative_perspective=second&narrative_tense=past",
    )
    .await;
    assert!(
        resp.status().is_success(),
        "posture auto-save should succeed"
    );

    let resp = post_form(
        &app,
        &format!("/games/{game_id}/mode"),
        "narrator_mode=interactive_fiction",
    )
    .await;
    assert!(resp.status().is_success());
    let body = axum::body::to_bytes(resp.into_body(), 16384).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains(r#"value="second" selected"#),
        "the deliberate perspective must survive the switch: {html}"
    );

    let resp = post_action(&app, "look").await;
    assert!(resp.status().is_success());
    assert!(wait_idle(&state, 1000).await);
    let (_, last_user) = narrator_prompts(&storage)
        .pop()
        .expect("the post-switch narration must record a prompt");
    assert!(
        last_user.contains("second-person limited perspective"),
        "the prompt must still resolve to the deliberate perspective: {last_user}"
    );
}

// [docs/specs/narrator_mode.md] SCENARIO: 23.5
#[tokio::test]
async fn test_impersonate_works_in_if_game_http() {
    let (app, state, storage) = narrated_app();
    storage
        .seed_world(&if_world(), &TestMap::single_room("start"))
        .expect("seed IF world");
    let resp = post_form(&app, "/games", "world_key=if_world&persona_key=test_player").await;
    assert!(resp.status().is_success());

    let resp = post_action(&app, "/impersonate hello").await;
    assert!(resp.status().is_success(), "/impersonate should accept");
    assert!(wait_idle(&state, 1000).await, "impersonate should complete");

    let messages = state.message_service.load_messages().unwrap();
    let inputs: Vec<_> = messages
        .iter()
        .filter(|m| m.message_type == MessageType::Input)
        .collect();
    assert_eq!(inputs.len(), 1, "exactly one Input entry");
    assert!(
        inputs[0].impersonated(),
        "the impersonated flag must be stored on the swipe"
    );
    assert_eq!(
        inputs[0].steering_instruction(),
        Some("hello"),
        "the steering instruction must be stored on the swipe"
    );
    assert!(
        !inputs[0].text().contains("/impersonate"),
        "the raw command must not be persisted"
    );
}
