//! HTTP E2E tests for the per-game config endpoints (posture, presets, mode): storage failures surface as 500 error spans instead of panics.

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::storage::{Storage, TestOverride};
use chronicler_engine::test_support::TestAppBuilder;

use crate::support::http_assertions::assert_option_selected;
use crate::support::http_requests::post_form;

fn failing_storage() -> Arc<Storage> {
    Arc::new(Storage::new_in_memory().with_failure(
        "update_game_config",
        TestOverride::internal("update_game_config failure"),
    ))
}

// [docs/specs/games.md] SCENARIO: 20.4
#[tokio::test]
async fn test_posture_save_failure_surfaces_500_error_span_http() {
    let storage = failing_storage();
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = storage.current_game_id();

    let resp = post_form(
        &app,
        &format!("/games/{id}/posture"),
        "narrative_perspective=second&narrative_tense=past",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("update_game_config failure"),
        "the error span must name the storage failure: {body_str}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.5
#[tokio::test]
async fn test_presets_save_failure_surfaces_500_error_span_http() {
    let storage = failing_storage();
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = storage.current_game_id();

    let resp = post_form(
        &app,
        &format!("/games/{id}/presets"),
        "system_preset_id=system_default&quantifier_preset_id=quantifier_default&impersonate_preset_id=impersonate_default",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("update_game_config failure"),
        "the error span must name the storage failure: {body_str}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.6
#[tokio::test]
async fn test_mode_switch_failure_surfaces_500_error_span_http() {
    let storage = failing_storage();
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = storage.current_game_id();

    let resp = post_form(
        &app,
        &format!("/games/{id}/mode"),
        "narrator_mode=interactive_fiction",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("update_game_config failure"),
        "the error span must name the storage failure: {body_str}"
    );
}

// The select's `hx-post="/games/{id}/posture"` with
// `hx-include="closest .posture-row"` sends exactly this form body, and the
// handler returns the fragment's outerHTML — so the re-render is asserted from
// the response body rather than the live DOM.
// [docs/specs/games.md] SCENARIO: 20.7
#[tokio::test]
async fn test_posture_autosave_rerenders_fragment_with_new_tense_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let id = state.game_catalogue.current_game_id();

    let resp = post_form(
        &app,
        &format!("/games/{id}/posture"),
        "narrative_perspective=third&narrative_tense=present",
    )
    .await;
    assert!(
        resp.status().is_success(),
        "the posture auto-save should succeed"
    );
    let body = axum::body::to_bytes(resp.into_body(), 16384).await.unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains(r#"id="game-posture-controls""#),
        "the response must be the re-rendered posture fragment: {html}"
    );
    assert_option_selected(
        &html,
        "present",
        "the re-rendered tense select must show 'present' selected",
    );
    assert_option_selected(
        &html,
        "third",
        "the re-rendered perspective select must keep 'third' selected",
    );

    let game = state
        .game_catalogue
        .current_game()
        .expect("current_game should succeed")
        .expect("the active game must still exist");
    assert_eq!(game.narrative_tense.as_str(), "present");
    assert_eq!(game.narrative_perspective.as_str(), "third");
}
