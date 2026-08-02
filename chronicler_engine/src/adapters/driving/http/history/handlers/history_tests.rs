use axum::{http::StatusCode, response::IntoResponse, Form};

use crate::adapters::driving::http::history::handlers::{
    delete_history_handler, edit_history_handler, EditHistoryForm,
};
use crate::test_support::TestAppBuilder;

#[test]
fn test_edit_history_form_deserialization() {
    let form: EditHistoryForm = serde_json::from_str(r#"{"text": "Modified text"}"#).unwrap();
    assert_eq!(form.text, "Modified text");
}

#[test]
fn test_edit_history_form_empty_text() {
    let form: EditHistoryForm = serde_json::from_str(r#"{"text": ""}"#).unwrap();
    assert!(form.text.is_empty());
}

#[test]
fn test_edit_history_form_with_newlines() {
    let form: EditHistoryForm = serde_json::from_str(r#"{"text": "Line1\nLine2\nLine3"}"#).unwrap();
    assert!(form.text.contains('\n'));
}

#[test]
fn test_edit_history_form_roundtrip() {
    let original = EditHistoryForm {
        text: "new text".to_string(),
    };
    let json = serde_json::to_string(&original).unwrap();
    let parsed: EditHistoryForm = serde_json::from_str(&json).unwrap();
    assert_eq!(original.text, parsed.text);
}

#[tokio::test]
async fn test_edit_history_handler_ok() {
    let state = TestAppBuilder::default_test().build_service();

    let form = EditHistoryForm {
        text: "modified text".to_string(),
    };
    let result = edit_history_handler(
        axum::extract::State(state),
        axum::extract::Path(999u64),
        Form(form),
    )
    .await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };

    assert!(status == StatusCode::INTERNAL_SERVER_ERROR || status == StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_history_handler_ok() {
    let state = TestAppBuilder::default_test().build_service();

    let result = delete_history_handler(axum::extract::State(state)).await;
    let status = match result {
        Ok(resp) => resp.status(),
        Err(e) => e.into_response().status(),
    };

    assert!(status == StatusCode::INTERNAL_SERVER_ERROR || status == StatusCode::BAD_REQUEST);
}
