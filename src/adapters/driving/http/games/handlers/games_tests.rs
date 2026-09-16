use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

use crate::adapters::driven::storage::{Storage, TestOverride};
use crate::adapters::driving::http::games::handlers::{
    list_games_fragment, switch_game_handler, switch_game_mode_handler,
    update_game_posture_handler, update_game_presets_handler, GameModeForm, GamePostureForm,
    GamePresetsForm,
};
use crate::domain::model::settings::{NarratorMode, NarrativePerspective, NarrativeTense};
use crate::test_support::TestAppBuilder;

#[tokio::test]
async fn test_list_games_empty() {
    let state = TestAppBuilder::default_test().build_service();
    let response = list_games_fragment(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_switch_game_ok() {
    let state = TestAppBuilder::default_test().build_service();
    let _ = state.game_catalogue.create_game("test", "test_player");
    let games = state.game_catalogue.list_games().unwrap();
    if let Some(game) = games.first() {
        let result =
            switch_game_handler(axum::extract::State(state), axum::extract::Path(game.id)).await;
        let status = match result {
            Ok(resp) => resp.status(),
            Err(e) => e.into_response().status(),
        };
        assert_eq!(status, StatusCode::OK);
    }
}

#[tokio::test]
async fn test_switch_game_mode_handler_retargets_and_nudges() {
    let state = TestAppBuilder::default_test().build_service();
    state
        .game_catalogue
        .create_game("test", "test_player")
        .unwrap();
    let game_id = state.game_catalogue.current_game_id();

    let response = switch_game_mode_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path(game_id),
        axum::extract::Form(GameModeForm {
            narrator_mode: "interactive_fiction".into(),
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body);
    assert!(
        body.contains("posture-override"),
        "mode switch must re-render the override fragment"
    );

    let game = state.game_catalogue.current_game().unwrap().unwrap();
    assert_eq!(game.narrator_mode, NarratorMode::InteractiveFiction);
    assert_eq!(game.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(game.active_system_prompt_preset_id, "system_if_default");
}

#[tokio::test]
async fn test_switch_game_mode_handler_invalid_mode_returns_error_span() {
    let state = TestAppBuilder::default_test().build_service();
    state
        .game_catalogue
        .create_game("test", "test_player")
        .unwrap();
    let game_id = state.game_catalogue.current_game_id();

    let response = switch_game_mode_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path(game_id),
        axum::extract::Form(GameModeForm {
            narrator_mode: "bogus".into(),
        }),
    )
    .await;
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body);
    assert!(body.contains("Unknown narrator mode"));

    let game = state.game_catalogue.current_game().unwrap().unwrap();
    assert_eq!(game.narrator_mode, NarratorMode::Novel);
}

#[tokio::test]
async fn test_update_game_posture_handler_saves_perspective_and_tense() {
    let state = TestAppBuilder::default_test().build_service();
    state
        .game_catalogue
        .create_game("test", "test_player")
        .unwrap();
    let game_id = state.game_catalogue.current_game_id();

    let response = update_game_posture_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path(game_id),
        axum::extract::Form(GamePostureForm {
            narrative_perspective: "second".into(),
            narrative_tense: "present".into(),
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let game = state.game_catalogue.current_game().unwrap().unwrap();
    assert_eq!(game.narrative_perspective, NarrativePerspective::Second);
    assert_eq!(game.narrative_tense, NarrativeTense::Present);
    assert_eq!(game.narrator_mode, NarratorMode::Novel);
}

#[tokio::test]
async fn test_update_game_posture_handler_invalid_value_returns_error_span() {
    let state = TestAppBuilder::default_test().build_service();
    state
        .game_catalogue
        .create_game("test", "test_player")
        .unwrap();
    let game_id = state.game_catalogue.current_game_id();

    let response = update_game_posture_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path(game_id),
        axum::extract::Form(GamePostureForm {
            narrative_perspective: "bogus".into(),
            narrative_tense: "past".into(),
        }),
    )
    .await;
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body);
    assert!(body.contains("Unknown narrative perspective"));
}

#[tokio::test]
async fn test_update_game_presets_handler_saves_selection() {
    let state = TestAppBuilder::default_test().build_service();
    state
        .game_catalogue
        .create_game("test", "test_player")
        .unwrap();
    let game_id = state.game_catalogue.current_game_id();

    // An all-modes custom system preset the game can switch to.
    state
        .prompt_preset_service
        .save_preset(&crate::test_support::TestPromptPreset::system(
            "custom_sys",
            "Custom",
        ))
        .unwrap();

    let response = update_game_presets_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path(game_id),
        axum::extract::Form(GamePresetsForm {
            system_preset_id: "custom_sys".into(),
            quantifier_preset_id: "quantifier_default".into(),
            impersonate_preset_id: "impersonate_default".into(),
        }),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);

    let game = state.game_catalogue.current_game().unwrap().unwrap();
    assert_eq!(game.active_system_prompt_preset_id, "custom_sys");
}

#[tokio::test]
async fn test_update_game_presets_handler_unknown_preset_returns_error_span() {
    let state = TestAppBuilder::default_test().build_service();
    state
        .game_catalogue
        .create_game("test", "test_player")
        .unwrap();
    let game_id = state.game_catalogue.current_game_id();

    let response = update_game_presets_handler(
        axum::extract::State(state.clone()),
        axum::extract::Path(game_id),
        axum::extract::Form(GamePresetsForm {
            system_preset_id: "no_such".into(),
            quantifier_preset_id: "quantifier_default".into(),
            impersonate_preset_id: "impersonate_default".into(),
        }),
    )
    .await;
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body);
    assert!(body.contains("Preset not found"));
}

#[tokio::test]
async fn test_posture_fragment_renders_preset_load_error_span() {
    let (storage, handle) = Storage::new_in_memory().with_test_failures();
    let state = TestAppBuilder::default_test()
        .storage(std::sync::Arc::new(storage))
        .build_service();
    state
        .game_catalogue
        .create_game("test", "test_player")
        .unwrap();

    handle.set(
        "list_presets",
        TestOverride::internal("simulated preset library failure"),
    );

    let response = list_games_fragment(axum::extract::State(state)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body = String::from_utf8_lossy(&body);
    assert!(
        body.contains(r#"class="error-message""#),
        "preset load failure should render the error span: {body}"
    );
    assert!(
        body.contains("Presets unavailable"),
        "error span should name the failure: {body}"
    );
    assert!(
        body.contains(r#"name="narrator_mode""#),
        "posture controls must still render: {body}"
    );
    assert!(
        !body.contains(r#"name="system_preset_id""#),
        "preset selects must be skipped on load failure: {body}"
    );

    handle.clear("list_presets");
}
