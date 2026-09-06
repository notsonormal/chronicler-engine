//! Unit tests for the action HTTP handler.
use std::sync::Arc;

use axum::{Form, http::StatusCode};

use crate::adapters::driven::storage::Storage;
use crate::domain::model::settings::TextCheckMode;
use crate::adapters::driving::http::action::handlers::actions::{
    action_check_handler, action_confirm_handler, action_handler,
};
use crate::adapters::driving::http::action::handlers::ActionForm;
use crate::test_support::{TestAppBuilder, TestWorld, TestMap, TestPersona};

#[test]
fn test_action_form_deserialization() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "look"}"#).unwrap();
    assert_eq!(form.command, "look");
}

#[test]
fn test_action_form_empty_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": ""}"#).unwrap();
    assert!(form.command.is_empty());
}

#[test]
fn test_action_form_with_whitespace_command() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "  look  "}"#).unwrap();
    assert_eq!(form.command, "  look  ");
}

#[test]
fn test_action_form_with_special_characters() {
    let form: ActionForm =
        serde_json::from_str(r#"{"command": "go north & talk to guard"}"#).unwrap();
    assert_eq!(form.command, "go north & talk to guard");
}

#[test]
fn test_action_form_deserialize_unicode() {
    let form: ActionForm = serde_json::from_str(r#"{"command": "こんにちは"}"#).unwrap();
    assert_eq!(form.command, "こんにちは");
}

#[test]
fn test_action_form_roundtrip() {
    let original = ActionForm {
        command: "test command".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: ActionForm = serde_json::from_str(&json).unwrap();
    assert_eq!(original.command, parsed.command);
}

#[tokio::test]
async fn test_action_handler_empty_command() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: String::new(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Empty command should trigger continuation"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Thinking"),
        "Expected Thinking status: {body_str}"
    );
}

#[tokio::test]
async fn test_action_handler_whitespace_command() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: "   ".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("Thinking"));
}

#[tokio::test]
async fn test_action_handler_started() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: "look".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_handler_guide_command_dispatches() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: "/guide make it tense".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Thinking"),
        "guide command should dispatch to the continue seam: {body_str}"
    );
}

#[tokio::test]
async fn test_action_handler_narrator_command_dispatches() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: "/narrator The lights flicker".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Thinking"),
        "narrator command should dispatch to the continue seam: {body_str}"
    );
}

#[tokio::test]
async fn test_action_handler_impersonate_command_dispatches() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: "/impersonate act confident".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Thinking"),
        "impersonate command should dispatch to the continue seam: {body_str}"
    );
}

#[tokio::test]
async fn test_action_confirm_handler_empty_command() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: String::new(),
    };

    let response = action_confirm_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "Empty command should trigger continuation"
    );
}

#[tokio::test]
async fn test_action_confirm_handler_started() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: "look".to_string(),
    };

    let response = action_confirm_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_empty_command() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: String::new(),
    };

    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("Thinking"));
}

#[tokio::test]
async fn test_action_check_handler_disabled_mode() {
    let state = TestAppBuilder::default_test().build_service();
    state.settings.write().unwrap().text_check.mode = TextCheckMode::Disabled;
    let form = ActionForm {
        command: "test".to_string(),
    };

    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_auto_check_disabled() {
    let state = TestAppBuilder::default_test().build_service();
    state.settings.write().unwrap().text_check.enable_auto_check = false;
    let form = ActionForm {
        command: "test".to_string(),
    };

    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_check_result_none() {
    let state = TestAppBuilder::default_test().build_service();
    let form = ActionForm {
        command: "go north".to_string(),
    };

    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

// dispatch_action Err arm: process_action errors at require_persona before
// try_claim/spawn. Missing-persona storage → 500.
#[tokio::test]
async fn test_action_handler_returns_500_on_pipeline_error() {
    let storage = Arc::new(Storage::new_in_memory());
    let world = TestWorld::minimal();
    let map = TestMap::single_room("start");
    storage.seed_world(&world, &map).unwrap();
    let player = TestPersona::standard();
    // Persona intentionally NOT seeded → require_persona errors.
    let game_id = storage
        .create_game(
            &world.name,
            &world.key,
            "__missing_persona__",
            &player.sheet.name,
            "Test Game",
        )
        .unwrap();
    storage.set_game_id(game_id);

    let state = TestAppBuilder::default_test()
        .storage(storage)
        .skip_seeding(true)
        .build_service();
    let form = ActionForm {
        command: "look".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(
        response.status(),
        StatusCode::INTERNAL_SERVER_ERROR,
        "missing-persona should produce 500 via dispatch_action Err arm"
    );
}

// dispatch_action Ok(ConcurrentGeneration) arm: pre-claim the generation slot
// so process_action's try_claim sees it busy → 200 "Still thinking...".
#[tokio::test]
async fn test_action_handler_returns_200_still_thinking_on_concurrent_generation() {
    let state = TestAppBuilder::default_test().build_service();

    // Pre-claim the slot so the handler's process_action sees it busy.
    let game_id = state.game_catalogue.current_game_id();
    let mut gs = state.message_service.load_or_fresh();
    let (_gid, _gen_id, claim) = state
        .generation_gate
        .try_claim(game_id, &mut gs, &state.message_service)
        .unwrap();
    assert!(matches!(
        claim,
        crate::application::errors::ProcessActionResult::Started
    ));

    let form = ActionForm {
        command: "look".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(
        response.status(),
        StatusCode::OK,
        "concurrent generation should produce 200"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Still thinking"),
        "expected 'Still thinking' in body: {body_str}"
    );
}

// dispatch_action Ok(ShuttingDown) arm: cancelled shutdown_token → 503.
// Relies on process_action's top-of-fn is_shutting_down() check.
#[tokio::test]
async fn test_action_handler_returns_503_on_shutdown() {
    let state = TestAppBuilder::default_test().build_service();
    state.shutdown_token.cancel();

    let form = ActionForm {
        command: "look".to_string(),
    };

    let response = action_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(
        response.status(),
        StatusCode::SERVICE_UNAVAILABLE,
        "cancelled shutdown should produce 503 via Ok(ShuttingDown) arm"
    );
}
