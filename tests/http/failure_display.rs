//! HTTP E2E tests for the failure display, tagged against `docs/specs/failure_display.md`.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use chrono::Utc;
use tower::util::ServiceExt;

use chronicler_engine::adapters::driven::storage::{Storage, TestOverride};
use chronicler_engine::adapters::driving::http::builders::router::build_router;
use chronicler_engine::domain::model::llm_message::LlmMessage;
use chronicler_engine::domain::model::state::generation_status::{GenerationPhase, GenerationStatus};
use chronicler_engine::test_support::TestAppBuilder;

use crate::support::http_requests::{fetch_body, post_form};

fn llm_message(agent: &str, error: Option<&str>, created_offset_secs: i64) -> LlmMessage {
    LlmMessage {
        id: 0,
        agent_name: agent.to_string(),
        backend_name: "Mock".to_string(),
        model_name: "mock".to_string(),
        system_prompt: String::new(),
        user_prompt: String::new(),
        raw_request_json: String::new(),
        raw_response_json: String::new(),
        parsed_response: String::new(),
        error_message: error.map(str::to_string),
        created_at: Utc::now() + chrono::Duration::seconds(created_offset_secs),
    }
}

// [docs/specs/failure_display.md] SCENARIO: 38.1
#[tokio::test]
async fn test_degraded_role_raises_header_banner_and_clears_on_recovery() {
    let (state, storage) = TestAppBuilder::default_test().build_service_with_storage();
    let app = build_router(state);

    storage
        .save_llm_message(&llm_message(
            "quantifier",
            Some("fallback NPC IDs used"),
            -10,
        ))
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
        body.contains("error-details-toggle"),
        "the banner must offer the Details disclosure: {body}"
    );
    assert!(
        body.contains("Mock mock") && body.contains("last call succeeded"),
        "the disclosure must list per-role backend/model and health: {body}"
    );
    assert!(
        body.contains(r#"<pre class="error-detail-raw">fallback NPC IDs used</pre>"#),
        "the raw failure text must be reachable inside the disclosure: {body}"
    );

    storage
        .save_llm_message(&llm_message("quantifier", None, 0))
        .unwrap();
    let body = fetch_body(&app, "/fragment/header").await;
    assert!(
        !body.contains("error-disclosure"),
        "a later success must clear the banner: {body}"
    );
}

// [docs/specs/failure_display.md] SCENARIO: 38.2
#[tokio::test]
async fn test_failed_poll_answers_non_2xx_and_reswap_none() {
    let state = TestAppBuilder::default_test().build_service();
    {
        let mut game_state = state.message_service.load_or_fresh();
        game_state.movement.current_room_id = "non_existent_room".to_string();
        let _ = state.message_service.save_state(&game_state);
    }
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
        .generation_status(
            GenerationStatus::Error("LLM Error: connection refused".to_string()),
            GenerationPhase::Narrating,
        )
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

// [docs/specs/failure_display.md] SCENARIO: 38.4
#[tokio::test]
async fn test_failed_user_action_is_not_a_failed_poll() {
    let storage = Arc::new(Storage::new_in_memory().with_failure(
        "update_game_config",
        TestOverride::internal("update_game_config failure"),
    ));
    let (app, _state) = TestAppBuilder::default_test()
        .storage(Arc::clone(&storage))
        .build_with_state();
    let id = storage.current_game_id();

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
