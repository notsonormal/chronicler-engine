use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use tower::util::ServiceExt;

use chronicler_engine::create_app_for_testing;
use chronicler_engine::error::{EngineError, internal_error};
use chronicler_engine::model::checkpoint::Checkpoint;
use chronicler_engine::model::state_snapshot::GameStateSnapshot;
use chronicler_engine::storage::snapshot_storage::SnapshotStorage;

use crate::create_test_state;

async fn fetch_body(app: Router, uri: &str) -> String {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert!(response.status().is_success(), "Expected success for {uri}");
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    String::from_utf8_lossy(&body).to_string()
}

#[tokio::test]
async fn test_header_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/header",
    )
    .await;
    assert!(body.contains("class=\"header\""));
    assert!(body.contains("Chronicler Engine"));
}

#[tokio::test]
async fn test_story_log_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/story-log",
    )
    .await;
    assert!(body.contains("id=\"story-log\""));
}

#[tokio::test]
async fn test_visual_sidebar_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/visual-sidebar",
    )
    .await;
    assert!(body.contains("id=\"visual-sidebar\""));
}

#[tokio::test]
async fn test_visual_sidebar_renders_room_image() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/visual-sidebar",
    )
    .await;
    // Should contain the image, not "No Location Image"
    assert!(
        body.contains("data/images/test_room.png"),
        "Expected room image in sidebar: {body}"
    );
    assert!(
        !body.contains("No Location Image"),
        "Should not show placeholder when image exists: {body}"
    );
}

#[tokio::test]
async fn test_action_area_fragment_returns_html() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/action-area",
    )
    .await;
    assert!(
        body.contains("id=\"action-area\""),
        "Expected action-area id: {body}"
    );
}

#[tokio::test]
async fn test_action_handler_accepts_command() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=look"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(body_str.contains("status"));
}

#[tokio::test]
async fn test_action_handler_empty_command() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Enter a command"),
        "Expected empty command error: {body_str}"
    );
}

#[tokio::test]
async fn test_hints_handler() {
    let body = fetch_body(create_app_for_testing(create_test_state()), "/hints").await;
    assert!(body.contains("Look"));
}

#[tokio::test]
async fn test_status_ready_handler() {
    let body = fetch_body(create_app_for_testing(create_test_state()), "/status/ready").await;
    assert!(body.contains("Ready"));
}

#[tokio::test]
async fn test_character_headshots_fragment() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/fragment/character-headshots",
    )
    .await;
    // The test state has npc_1 with a profile_image, so headshots should render
    assert!(
        body.contains("headshot"),
        "Expected headshot in fragment: {body}"
    );
}

#[tokio::test]
async fn test_generating_status_handler_idle() {
    let body = fetch_body(
        create_app_for_testing(create_test_state()),
        "/status/generating",
    )
    .await;
    // Should return "idle" when not generating
    assert!(body.contains("idle"));
}

#[tokio::test]
async fn test_generating_status_handler_narrating() {
    let mut state = create_test_state();
    state.narrative.generation.status =
        chronicler_engine::model::state::GenerationStatus::Generating;
    state.narrative.generation.phase = chronicler_engine::model::state::GenerationPhase::Narrating;

    let body = fetch_body(create_app_for_testing(state), "/status/generating").await;
    assert!(body.contains("narrating"));
}

#[tokio::test]
async fn test_generating_status_handler_quantifying() {
    let mut state = create_test_state();
    state.narrative.generation.status =
        chronicler_engine::model::state::GenerationStatus::Generating;
    state.narrative.generation.phase =
        chronicler_engine::model::state::GenerationPhase::Quantifying;

    let body = fetch_body(create_app_for_testing(state), "/status/generating").await;
    assert!(body.contains("quantifying"));
}

#[tokio::test]
async fn test_reset_generating_handler() {
    let app = create_app_for_testing(create_test_state());

    // reset-generating is POST, not GET
    let req = Request::builder()
        .uri("/status/reset-generating")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    // Should return "reset" on success
    assert!(body_str.contains("reset"));
}

#[tokio::test]
async fn test_edit_history_handler_success() {
    let mut state = create_test_state();
    let entry_id = {
        state.add_log(
            "Original text".to_string(),
            Some("Test".to_string()),
            chronicler_engine::model::state::LogType::Narration,
        );
        state.narrative.history().last().unwrap().id
    };

    let app = create_app_for_testing(state);

    let req = Request::builder()
        .uri(format!("/history/{entry_id}"))
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited+text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Edited"),
        "Expected success message: {body_str}"
    );
}

#[tokio::test]
async fn test_edit_history_handler_not_found() {
    let app = create_app_for_testing(create_test_state());

    // Try to edit a non-existent log entry (ID 9999) - correct path is /history/:id
    let req = Request::builder()
        .uri("/history/9999")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("text=Edited text"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    // Should return NOT_FOUND for non-existent entry
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_history_handler_success() {
    let mut state = create_test_state();

    // Add a log entry first
    state.add_log(
        "Test message".to_string(),
        Some("Test".to_string()),
        chronicler_engine::model::state::LogType::Narration,
    );

    let app = create_app_for_testing(state.clone());

    let req = Request::builder()
        .uri("/history/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Verify the entry was deleted by fetching the story log fragment
    let body = fetch_body(app, "/fragment/story-log").await;
    assert!(
        !body.contains("Test message"),
        "Log entry should be deleted from rendered story log"
    );
}

#[tokio::test]
async fn test_delete_history_handler_empty() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/history/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_action_confirm_empty_command() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action/confirm")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command="))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Enter a command"),
        "Expected empty command error: {body_str}"
    );
}

#[tokio::test]
async fn test_action_concurrent_rejection() {
    let app = create_app_for_testing(create_test_state());

    // First async action sets is_generating = true
    let req1 = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go north"))
        .unwrap();
    let response1 = app.clone().oneshot(req1).await.unwrap();
    assert!(response1.status().is_success());

    // Second async action while first is in flight
    let req2 = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=go south"))
        .unwrap();
    let response2 = app.oneshot(req2).await.unwrap();
    assert!(response2.status().is_success());
    let body = axum::body::to_bytes(response2.into_body(), 1024)
        .await
        .unwrap();
    let body_str = String::from_utf8_lossy(&body);
    assert!(
        body_str.contains("Still thinking..."),
        "Expected concurrent rejection: {body_str}"
    );
}

#[tokio::test]
async fn test_action_sync_inventory() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=inventory"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_trigger = response.headers().get("HX-Trigger");
    assert!(
        hx_trigger.is_some(),
        "Expected HX-Trigger header for sync action"
    );
}

#[tokio::test]
async fn test_action_sync_quit() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/action")
        .method(http::Method::POST)
        .header(
            http::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(Body::from("command=quit"))
        .unwrap();
    let response = app.oneshot(req).await.unwrap();

    assert!(response.status().is_success());
    let hx_trigger = response.headers().get("HX-Trigger");
    assert!(
        hx_trigger.is_some(),
        "Expected HX-Trigger header for sync action"
    );
}

#[tokio::test]
async fn test_checkpoint_create_and_list() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/checkpoint")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let req = Request::builder()
        .uri("/fragment/checkpoints")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.contains("checkpoint-item"),
        "Should render checkpoint item"
    );
    assert!(html.contains("Restore"), "Should have restore button");
}

#[tokio::test]
async fn test_checkpoint_restore_not_found() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/checkpoint/nonexistent/restore")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_checkpoint_delete_not_found() {
    let app = create_app_for_testing(create_test_state());

    let req = Request::builder()
        .uri("/checkpoint/nonexistent/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_switch_swipe_success() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        chronicler_engine::model::state::LogType::Input,
    );
    let turn_id = state.narrative.messages.last().unwrap().turn_id.clone();
    {
        let msg = state.narrative.messages.last_mut().unwrap();
        msg.create_swipe("alternate text");
    }

    let storage = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let snapshot = chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        turn_id.clone(),
        0,
    );
    storage.save(&snapshot).unwrap();

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );

    let req = Request::builder()
        .uri(format!("/turn/{turn_id}/swipe/1"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    assert!(
        String::from_utf8_lossy(&body).contains("Switched"),
        "Expected success message"
    );
}

#[tokio::test]
async fn test_switch_swipe_invalid_index() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        chronicler_engine::model::state::LogType::Input,
    );
    let turn_id = state.narrative.messages.last().unwrap().turn_id.clone();

    let app = create_app_for_testing(state);
    let req = Request::builder()
        .uri(format!("/turn/{turn_id}/swipe/99"))
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_switch_swipe_turn_not_found() {
    let app = create_app_for_testing(create_test_state());
    let req = Request::builder()
        .uri("/turn/nonexistent/swipe/0")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_checkpoint_no_state() {
    let state = create_test_state();
    let storage = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );

    let req = Request::builder()
        .uri("/checkpoint")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_restore_checkpoint_success() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        chronicler_engine::model::state::LogType::Input,
    );
    let turn_id = state.narrative.messages.last().unwrap().turn_id.clone();

    let storage = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;
    let snapshot = chronicler_engine::model::state_snapshot::GameStateSnapshot::from_game_state(
        &state,
        turn_id.clone(),
        0,
    );
    storage.save(&snapshot).unwrap();

    let checkpoint = chronicler_engine::model::checkpoint::Checkpoint {
        id: "cp1".to_string(),
        turn_id: turn_id.clone(),
        swipe_index: 0,
        name: "Test".to_string(),
        created_at: chrono::Utc::now(),
    };
    storage.save_checkpoint(&checkpoint).unwrap();

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );

    let req = Request::builder()
        .uri("/checkpoint/cp1/restore")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    assert!(
        String::from_utf8_lossy(&body).contains("Restored"),
        "Expected restored message"
    );
}

#[tokio::test]
async fn test_restore_checkpoint_snapshot_missing() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        chronicler_engine::model::state::LogType::Input,
    );
    let turn_id = state.narrative.messages.last().unwrap().turn_id.clone();

    let storage = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn chronicler_engine::storage::snapshot_storage::SnapshotStorage>;

    let checkpoint = chronicler_engine::model::checkpoint::Checkpoint {
        id: "cp1".to_string(),
        turn_id: turn_id.clone(),
        swipe_index: 0,
        name: "Test".to_string(),
        created_at: chrono::Utc::now(),
    };
    storage.save_checkpoint(&checkpoint).unwrap();

    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );

    let req = Request::builder()
        .uri("/checkpoint/cp1/restore")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_checkpoints_empty() {
    let app = create_app_for_testing(create_test_state());
    let req = Request::builder()
        .uri("/fragment/checkpoints")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), 1024)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&body);
    assert!(
        html.is_empty(),
        "Empty checkpoints should return empty body"
    );
}

// Failing storage wrapper for testing error paths
struct FailingStorage {
    inner: Arc<dyn SnapshotStorage>,
    fail_load_latest: bool,
    fail_save_checkpoint: bool,
    fail_load_checkpoint: bool,
    fail_load_by_turn: bool,
    fail_save: bool,
    fail_delete_checkpoint: bool,
    fail_list_checkpoints: bool,
}

impl SnapshotStorage for FailingStorage {
    fn save(&self, snapshot: &GameStateSnapshot) -> Result<(), EngineError> {
        if self.fail_save {
            return Err(EngineError::Internal(internal_error("save fail")));
        }
        self.inner.save(snapshot)
    }

    fn load_latest(&self, turn_id: Option<&str>) -> Result<Option<GameStateSnapshot>, EngineError> {
        if self.fail_load_latest {
            return Err(EngineError::Internal(internal_error("load_latest fail")));
        }
        self.inner.load_latest(turn_id)
    }

    fn load_by_turn(
        &self,
        turn_id: &str,
        swipe_index: u32,
    ) -> Result<Option<GameStateSnapshot>, EngineError> {
        if self.fail_load_by_turn {
            return Err(EngineError::Internal(internal_error("load_by_turn fail")));
        }
        self.inner.load_by_turn(turn_id, swipe_index)
    }

    fn delete_turn_snapshots(&self, turn_id: &str) -> Result<(), EngineError> {
        self.inner.delete_turn_snapshots(turn_id)
    }

    fn commit(&self, snapshot_id: &str) -> Result<(), EngineError> {
        self.inner.commit(snapshot_id)
    }

    fn reset(&self) -> Result<(), EngineError> {
        self.inner.reset()
    }

    fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), EngineError> {
        if self.fail_save_checkpoint {
            return Err(EngineError::Internal(internal_error(
                "save_checkpoint fail",
            )));
        }
        self.inner.save_checkpoint(checkpoint)
    }

    fn load_checkpoint(&self, id: &str) -> Result<Option<Checkpoint>, EngineError> {
        if self.fail_load_checkpoint {
            return Err(EngineError::Internal(internal_error(
                "load_checkpoint fail",
            )));
        }
        self.inner.load_checkpoint(id)
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, EngineError> {
        if self.fail_list_checkpoints {
            return Err(EngineError::Internal(internal_error(
                "list_checkpoints fail",
            )));
        }
        self.inner.list_checkpoints()
    }

    fn delete_checkpoint(&self, id: &str) -> Result<(), EngineError> {
        if self.fail_delete_checkpoint {
            return Err(EngineError::Internal(internal_error(
                "delete_checkpoint fail",
            )));
        }
        self.inner.delete_checkpoint(id)
    }
}

#[tokio::test]
async fn test_switch_swipe_load_state_error() {
    let state = create_test_state();
    let inner = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn SnapshotStorage>;
    let storage = Arc::new(FailingStorage {
        inner,
        fail_load_latest: true,
        fail_save_checkpoint: false,
        fail_load_checkpoint: false,
        fail_load_by_turn: false,
        fail_save: false,
        fail_delete_checkpoint: false,
        fail_list_checkpoints: false,
    });
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );
    let req = Request::builder()
        .uri("/turn/test/swipe/0")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_checkpoint_save_error() {
    let state = create_test_state();
    let inner = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn SnapshotStorage>;
    let snapshot = GameStateSnapshot::from_game_state(&state, "test".to_string(), 0);
    inner.save(&snapshot).unwrap();
    let storage = Arc::new(FailingStorage {
        inner,
        fail_load_latest: false,
        fail_save_checkpoint: true,
        fail_load_checkpoint: false,
        fail_load_by_turn: false,
        fail_save: false,
        fail_delete_checkpoint: false,
        fail_list_checkpoints: false,
    });
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );
    let req = Request::builder()
        .uri("/checkpoint")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_restore_checkpoint_load_checkpoint_error() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        chronicler_engine::model::state::LogType::Input,
    );
    let turn_id = state.narrative.messages.last().unwrap().turn_id.clone();
    let inner = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn SnapshotStorage>;
    let snapshot = GameStateSnapshot::from_game_state(&state, turn_id.clone(), 0);
    inner.save(&snapshot).unwrap();
    let checkpoint = Checkpoint {
        id: "cp1".to_string(),
        turn_id: turn_id.clone(),
        swipe_index: 0,
        name: "Test".to_string(),
        created_at: chrono::Utc::now(),
    };
    inner.save_checkpoint(&checkpoint).unwrap();
    let storage = Arc::new(FailingStorage {
        inner,
        fail_load_latest: false,
        fail_save_checkpoint: false,
        fail_load_checkpoint: true,
        fail_load_by_turn: false,
        fail_save: false,
        fail_delete_checkpoint: false,
        fail_list_checkpoints: false,
    });
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );
    let req = Request::builder()
        .uri("/checkpoint/cp1/restore")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_restore_checkpoint_load_by_turn_error() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        chronicler_engine::model::state::LogType::Input,
    );
    let turn_id = state.narrative.messages.last().unwrap().turn_id.clone();
    let inner = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn SnapshotStorage>;
    let snapshot = GameStateSnapshot::from_game_state(&state, turn_id.clone(), 0);
    inner.save(&snapshot).unwrap();
    let checkpoint = Checkpoint {
        id: "cp1".to_string(),
        turn_id: turn_id.clone(),
        swipe_index: 0,
        name: "Test".to_string(),
        created_at: chrono::Utc::now(),
    };
    inner.save_checkpoint(&checkpoint).unwrap();
    let storage = Arc::new(FailingStorage {
        inner,
        fail_load_latest: false,
        fail_save_checkpoint: false,
        fail_load_checkpoint: false,
        fail_load_by_turn: true,
        fail_save: false,
        fail_delete_checkpoint: false,
        fail_list_checkpoints: false,
    });
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );
    let req = Request::builder()
        .uri("/checkpoint/cp1/restore")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_restore_checkpoint_save_error() {
    let mut state = create_test_state();
    state.add_log(
        "look".to_string(),
        Some("Player".to_string()),
        chronicler_engine::model::state::LogType::Input,
    );
    let turn_id = state.narrative.messages.last().unwrap().turn_id.clone();
    let inner = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn SnapshotStorage>;
    let snapshot = GameStateSnapshot::from_game_state(&state, turn_id.clone(), 0);
    inner.save(&snapshot).unwrap();
    let checkpoint = Checkpoint {
        id: "cp1".to_string(),
        turn_id: turn_id.clone(),
        swipe_index: 0,
        name: "Test".to_string(),
        created_at: chrono::Utc::now(),
    };
    inner.save_checkpoint(&checkpoint).unwrap();
    let storage = Arc::new(FailingStorage {
        inner,
        fail_load_latest: false,
        fail_save_checkpoint: false,
        fail_load_checkpoint: false,
        fail_load_by_turn: false,
        fail_save: true,
        fail_delete_checkpoint: false,
        fail_list_checkpoints: false,
    });
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );
    let req = Request::builder()
        .uri("/checkpoint/cp1/restore")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_checkpoint_error() {
    let state = create_test_state();
    let inner = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn SnapshotStorage>;
    let storage = Arc::new(FailingStorage {
        inner,
        fail_load_latest: false,
        fail_save_checkpoint: false,
        fail_load_checkpoint: false,
        fail_load_by_turn: false,
        fail_save: false,
        fail_delete_checkpoint: true,
        fail_list_checkpoints: false,
    });
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );
    let req = Request::builder()
        .uri("/checkpoint/cp1/delete")
        .method(http::Method::POST)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_list_checkpoints_error() {
    let state = create_test_state();
    let inner = Arc::new(chronicler_engine::test_support::InMemorySnapshotStorage::new())
        as Arc<dyn SnapshotStorage>;
    let storage = Arc::new(FailingStorage {
        inner,
        fail_load_latest: false,
        fail_save_checkpoint: false,
        fail_load_checkpoint: false,
        fail_load_by_turn: false,
        fail_save: false,
        fail_delete_checkpoint: false,
        fail_list_checkpoints: true,
    });
    let app = chronicler_engine::server::create_app_with_storage(
        state,
        storage,
        Arc::new(chronicler_engine::storage::llm_message_storage::InMemoryLlmMessageStorage::new()),
        chronicler_engine::model::settings::AppSettings::default(),
    );
    let req = Request::builder()
        .uri("/fragment/checkpoints")
        .method(http::Method::GET)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
}
