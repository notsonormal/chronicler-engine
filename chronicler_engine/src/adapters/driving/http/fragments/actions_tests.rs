use axum::{Form, http::StatusCode};

use crate::domain::model::settings::TextCheckMode;
use crate::adapters::driving::http::fragments::actions::{
    action_check_handler, action_confirm_handler, action_handler,
};
use crate::adapters::driving::http::fragments::ActionForm;
use crate::adapters::driving::http::op_context_loader::load_op_context_for_active_game;
use crate::test_support::TestAppBuilder;

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
    let state = TestAppBuilder::default_test().build_app_state();
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
    let state = TestAppBuilder::default_test().build_app_state();
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
    let state = TestAppBuilder::default_test().build_app_state();
    let form = ActionForm {
        command: "look".to_string(),
    };
    
    let response = action_handler(axum::extract::State(state), Form(form)).await;

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_confirm_handler_empty_command() {
    let state = TestAppBuilder::default_test().build_app_state();
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
    let state = TestAppBuilder::default_test().build_app_state();
    let form = ActionForm {
        command: "look".to_string(),
    };
    
    let response = action_confirm_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_empty_command() {
    let state = TestAppBuilder::default_test().build_app_state();
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
    let state = TestAppBuilder::default_test().build_app_state();
    state.settings.write().unwrap().text_check.mode = TextCheckMode::Disabled;
    let form = ActionForm {
        command: "test".to_string(),
    };
    
    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_auto_check_disabled() {
    let state = TestAppBuilder::default_test().build_app_state();
    state.settings.write().unwrap().text_check.enable_auto_check = false;
    let form = ActionForm {
        command: "test".to_string(),
    };
    
    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_action_check_handler_check_result_none() {
    let state = TestAppBuilder::default_test().build_app_state();
    let form = ActionForm {
        command: "go north".to_string(),
    };
    
    let response = action_check_handler(axum::extract::State(state), Form(form)).await;
    assert_eq!(response.status(), StatusCode::OK);
}
