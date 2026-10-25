//! HTTP E2E tests for the per-game config endpoints (posture, presets, mode): storage failures surface as 500 error fragments instead of panics.

use std::sync::Arc;

use axum::http::StatusCode;

use chronicler_engine::adapters::driven::storage::{Storage, TestOverride, TxWrite};
use chronicler_engine::test_support::{
    active_game_of, TestAppBuilder, TestPromptPreset, empty_sqlite_storage,
};

use crate::support::http_assertions::assert_option_selected;
use crate::support::http_requests::{assert_answers_nothing, fetch_body, post_form, response_body};

fn failing_storage() -> Arc<Storage> {
    let (storage, failures) = empty_sqlite_storage().with_test_failures();
    failures.set_write(
        TxWrite::SetGameConfig,
        TestOverride::internal("update_game_config failure"),
    );
    Arc::new(storage)
}

// [docs/specs/games.md] SCENARIO: 20.4
#[tokio::test]
async fn test_posture_save_failure_surfaces_500_error_span_http() {
    let storage = failing_storage();
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = active_game_of(&state).0;

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
        "the error fragment must name the storage failure: {body_str}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.5
#[tokio::test]
async fn test_presets_save_failure_surfaces_500_error_span_http() {
    let storage = failing_storage();
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = active_game_of(&state).0;

    let resp = post_form(
        &app,
        &format!("/games/{id}/presets"),
        "system_preset_id=system_default&quantifier_preset_id=quantifier_default&impersonate_preset_id=impersonate_default&options_preset_id=options_default",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(resp.into_body(), 8192).await.unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("update_game_config failure"),
        "the error fragment must name the storage failure: {body_str}"
    );
}

// [docs/specs/games.md] SCENARIO: 20.6
#[tokio::test]
async fn test_mode_switch_failure_surfaces_500_error_span_http() {
    let storage = failing_storage();
    let (app, state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = active_game_of(&state).0;

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
        "the error fragment must name the storage failure: {body_str}"
    );
}

// The select's `hx-post="/games/{id}/posture"` with `hx-include="closest .posture-row"`
// sends exactly this form body. The save answers nothing, so the new value is read
// back from the game and from the Games fragment, which the panel loads.
// [docs/specs/games.md] SCENARIO: 20.7
#[tokio::test]
async fn test_posture_autosave_answers_nothing_and_the_fragment_renders_the_new_tense_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let id = active_game_of(&state);

    let resp = post_form(
        &app,
        &format!("/games/{id}/posture"),
        "narrative_perspective=third&narrative_tense=present",
    )
    .await;
    assert_answers_nothing(resp).await;

    let game = state
        .game_catalogue
        .active_game()
        .expect("active_game should succeed")
        .expect("the active game must still exist");
    assert_eq!(game.narrative_tense.as_str(), "present");
    assert_eq!(game.narrative_perspective.as_str(), "third");

    let html = fetch_body(&app, "/fragment/games").await;
    assert_option_selected(
        &html,
        "present",
        "the fragment's tense select must show 'present' selected",
    );
    assert_option_selected(
        &html,
        "third",
        "the fragment's perspective select must keep 'third' selected",
    );
}

// [docs/specs/games.md] SCENARIO: 20.11
#[tokio::test]
async fn test_options_preset_selection_persists_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let id = active_game_of(&state);

    state
        .prompt_preset_service
        .save_preset(&TestPromptPreset::options(
            "options_custom",
            "Custom Options",
        ))
        .expect("save an options preset");

    let current = state
        .game_catalogue
        .active_game()
        .expect("active_game should succeed")
        .expect("the active game must exist");
    let body = format!(
        "system_preset_id={}&quantifier_preset_id={}&impersonate_preset_id={}&options_preset_id=options_custom",
        current.active_system_prompt_preset_id,
        current.active_quantifier_prompt_preset_id,
        current.active_impersonate_prompt_preset_id,
    );

    let resp = post_form(&app, &format!("/games/{id}/presets"), &body).await;
    assert_answers_nothing(resp).await;

    let game = state
        .game_catalogue
        .active_game()
        .expect("active_game should succeed")
        .expect("the active game must still exist");
    assert_eq!(game.active_options_prompt_preset_id, "options_custom");

    let html = fetch_body(&app, "/fragment/games").await;
    assert_option_selected(
        &html,
        "options_custom",
        "the fragment's Options select must select the chosen preset",
    );
}

// [docs/specs/games.md] SCENARIO: 20.12
#[tokio::test]
async fn test_a_config_save_with_a_rejected_value_answers_400_http() {
    let (app, state) = TestAppBuilder::default_test().build_with_state();
    let id = active_game_of(&state);
    let before = state
        .game_catalogue
        .active_game()
        .expect("active_game should succeed")
        .expect("the active game must exist")
        .narrator_mode;

    let resp = post_form(
        &app,
        &format!("/games/{id}/mode"),
        "narrator_mode=no_such_mode",
    )
    .await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body = response_body(resp).await;
    assert!(
        body.contains("no_such_mode"),
        "the error fragment must name the rejected value: {body}"
    );

    let after = state
        .game_catalogue
        .active_game()
        .expect("active_game should succeed")
        .expect("the active game must still exist")
        .narrator_mode;
    assert_eq!(after, before, "a rejected value must not change the mode");
}
